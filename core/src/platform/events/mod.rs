//! The change bus: table kinds, changes, and subscribers.
//!
//! The core's in-memory change bus (Decision 0007). SQLite commit hooks record
//! which tables a transaction wrote; the bus coalesces them into numbered change
//! events that clients watch to refresh what they show.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use time::OffsetDateTime;

/// Kinds a change event can carry. Clients map them to the slices they reload.
pub const KIND_PROJECTS: &str = "projects";
pub const KIND_SETTINGS: &str = "settings";
pub const KIND_WORK: &str = "work";
pub const KIND_KNOWLEDGE: &str = "knowledge";
pub const KIND_CANON: &str = "canon";
pub const KIND_AGENT_RUNS: &str = "agent_runs";
pub const KIND_CANVAS: &str = "canvas";

/// Returns the event kind for a table, or `""` for tables clients do not show
/// (for example store_health).
pub fn kind_for_table(table: &str) -> &'static str {
    match table {
        "projects" => KIND_PROJECTS,
        "project_settings" => KIND_SETTINGS,
        "project_repos" => KIND_SETTINGS,
        "work_cards" => KIND_WORK,
        "work_tasks" => KIND_WORK,
        "work_questions" => KIND_WORK,
        "work_invariants" => KIND_WORK,
        "work_handoffs" => KIND_WORK,
        "work_commit_links" => KIND_WORK,
        "work_claims" => KIND_WORK,
        "milestones" => KIND_WORK,
        "inbox_dispositions" => KIND_WORK,
        "structure_nodes" => KIND_WORK,
        "rules" => KIND_WORK,
        "change_ops" => KIND_WORK,
        "knowledge_items" => KIND_KNOWLEDGE,
        "knowledge_entity_links" => KIND_KNOWLEDGE,
        "canon_entities" => KIND_CANON,
        "canon_relationships" => KIND_CANON,
        "canon_facts" => KIND_CANON,
        "agent_runs" => KIND_AGENT_RUNS,
        "agent_run_steps" => KIND_AGENT_RUNS,
        "agent_tool_calls" => KIND_AGENT_RUNS,
        "canvases" => KIND_CANVAS,
        _ => "",
    }
}

/// One coalesced event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub seq: i64,
    pub project_ids: Vec<String>,
    pub kinds: Vec<String>,
    pub origins: Vec<String>,
    pub resync: bool,
    pub at: OffsetDateTime,
}

const DEFAULT_WINDOW: Duration = Duration::from_millis(100);
const HISTORY_SIZE: usize = 256;
const SUBSCRIBER_BUF: usize = 64;

fn guard<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Coalesces recorded writes for a short window (which also lets the commit
/// become visible) and fans the resulting change out to subscribers.
#[derive(Clone)]
pub struct Bus {
    inner: Arc<BusInner>,
}

struct BusInner {
    state: Mutex<State>,
}

struct State {
    window: Duration,
    seq: i64,
    history: Vec<Change>,
    subs: BTreeMap<u64, Arc<SubscriberState>>,
    next_sub: u64,
    next_timer: u64,
    kinds: BTreeSet<String>,
    projects: BTreeSet<String>,
    origins: BTreeSet<String>,
    timer: Option<u64>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            inner: Arc::new(BusInner {
                state: Mutex::new(State {
                    window: DEFAULT_WINDOW,
                    seq: 0,
                    history: Vec::new(),
                    subs: BTreeMap::new(),
                    next_sub: 0,
                    next_timer: 0,
                    kinds: BTreeSet::new(),
                    projects: BTreeSet::new(),
                    origins: BTreeSet::new(),
                    timer: None,
                }),
            }),
        }
    }

    /// Sets how long writes coalesce before they are flushed.
    pub fn set_window(&self, window: Duration) {
        guard(&self.inner.state).window = window;
    }

    /// Adds one committed transaction: the tables it wrote, the project ids
    /// found in its rows, and the clients whose calls were in flight.
    pub fn record(&self, tables: &[String], project_ids: &[String], origins: &[String]) {
        let mut state = guard(&self.inner.state);
        let mut added = false;
        for table in tables {
            let kind = kind_for_table(table);
            if !kind.is_empty() {
                state.kinds.insert(kind.to_string());
                added = true;
            }
        }
        if !added {
            return;
        }
        for id in project_ids {
            state.projects.insert(id.clone());
        }
        for origin in origins {
            state.origins.insert(origin.clone());
        }
        if state.timer.is_none() {
            state.next_timer += 1;
            let generation = state.next_timer;
            state.timer = Some(generation);
            let window = state.window;
            let inner = Arc::clone(&self.inner);
            std::thread::spawn(move || {
                std::thread::sleep(window);
                flush_generation(&inner, generation);
            });
        }
    }

    /// Flushes the recorded writes now, as the timer would.
    pub fn flush(&self) {
        let mut state = guard(&self.inner.state);
        state.timer = None;
        flush_locked(&mut state);
    }

    /// Returns the current seq, the events after `after_seq` still in history
    /// (or resync when they are gone), and a subscription of new events.
    pub fn subscribe(&self, after_seq: i64) -> Subscription {
        let mut state = guard(&self.inner.state);
        let current = state.seq;
        let mut backlog = Vec::new();
        let mut resync = false;
        if after_seq > state.seq {
            // The client saw a previous core's sequence.
            resync = true;
        } else if after_seq > 0 && after_seq < state.seq {
            if state.history.is_empty() || state.history[0].seq > after_seq + 1 {
                resync = true;
            } else {
                backlog.extend(
                    state
                        .history
                        .iter()
                        .filter(|change| change.seq > after_seq)
                        .cloned(),
                );
            }
        }
        let id = state.next_sub;
        state.next_sub += 1;
        let subscriber = Arc::new(SubscriberState {
            queue: Mutex::new(VecDeque::new()),
            ready: Condvar::new(),
        });
        state.subs.insert(id, Arc::clone(&subscriber));
        Subscription {
            bus: Arc::clone(&self.inner),
            id,
            subscriber,
            current,
            backlog,
            resync,
            cancelled: false,
        }
    }

    /// Reports how many clients are watching.
    pub fn subscribers(&self) -> usize {
        guard(&self.inner.state).subs.len()
    }
}

impl Default for Bus {
    fn default() -> Self {
        Bus::new()
    }
}

fn flush_generation(inner: &BusInner, generation: u64) {
    let mut state = guard(&inner.state);
    if state.timer != Some(generation) {
        return;
    }
    state.timer = None;
    flush_locked(&mut state);
}

fn flush_locked(state: &mut State) {
    if state.kinds.is_empty() {
        return;
    }
    state.seq += 1;
    let change = Change {
        seq: state.seq,
        project_ids: state.projects.iter().cloned().collect(),
        kinds: state.kinds.iter().cloned().collect(),
        origins: state.origins.iter().cloned().collect(),
        resync: false,
        at: OffsetDateTime::now_utc(),
    };
    state.kinds.clear();
    state.projects.clear();
    state.origins.clear();
    state.history.push(change.clone());
    if state.history.len() > HISTORY_SIZE {
        let excess = state.history.len() - HISTORY_SIZE;
        state.history.drain(..excess);
    }
    for subscriber in state.subs.values() {
        subscriber.send(change.clone());
    }
}

struct SubscriberState {
    queue: Mutex<VecDeque<Change>>,
    ready: Condvar,
}

impl SubscriberState {
    fn send(&self, change: Change) {
        let mut queue = guard(&self.queue);
        if queue.len() >= SUBSCRIBER_BUF {
            // A subscriber that cannot keep up is told to reload everything.
            queue.clear();
            queue.push_back(Change {
                seq: change.seq,
                project_ids: Vec::new(),
                kinds: Vec::new(),
                origins: Vec::new(),
                resync: true,
                at: change.at,
            });
        } else {
            queue.push_back(change);
        }
        drop(queue);
        self.ready.notify_all();
    }

    fn try_recv(&self) -> Option<Change> {
        guard(&self.queue).pop_front()
    }

    fn recv_until(&self, deadline: Option<Instant>) -> Option<Change> {
        let mut queue = guard(&self.queue);
        loop {
            if let Some(change) = queue.pop_front() {
                return Some(change);
            }
            queue = match deadline {
                Some(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        return None;
                    }
                    let wait = deadline - now;
                    self.ready
                        .wait_timeout(queue, wait)
                        .unwrap_or_else(|poison| poison.into_inner())
                        .0
                }
                None => self
                    .ready
                    .wait(queue)
                    .unwrap_or_else(|poison| poison.into_inner()),
            };
        }
    }
}

/// A live stream of coalesced change events.
#[derive(Clone)]
pub struct Subscriber {
    state: Arc<SubscriberState>,
}

impl Subscriber {
    pub fn try_recv(&self) -> Option<Change> {
        self.state.try_recv()
    }

    /// Blocks until the next event.
    pub fn recv(&self) -> Option<Change> {
        self.state.recv_until(None)
    }

    /// Blocks until the next event or the timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<Change> {
        self.state.recv_until(Some(Instant::now() + timeout))
    }
}

/// The result of `Bus::subscribe`: the current sequence, the backlog, whether
/// the client must resync, and the event stream. Dropping it cancels the
/// subscription.
pub struct Subscription {
    bus: Arc<BusInner>,
    id: u64,
    subscriber: Arc<SubscriberState>,
    pub current: i64,
    pub backlog: Vec<Change>,
    pub resync: bool,
    cancelled: bool,
}

impl Subscription {
    pub fn events(&self) -> Subscriber {
        Subscriber {
            state: Arc::clone(&self.subscriber),
        }
    }

    pub fn try_recv(&self) -> Option<Change> {
        self.subscriber.try_recv()
    }

    pub fn recv(&self) -> Option<Change> {
        self.subscriber.recv_until(None)
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<Change> {
        self.subscriber.recv_until(Some(Instant::now() + timeout))
    }

    pub fn cancel(&mut self) {
        if self.cancelled {
            return;
        }
        self.cancelled = true;
        guard(&self.bus.state).subs.remove(&self.id);
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests;
