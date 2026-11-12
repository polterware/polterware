//! Bounded log buffers and the non-blocking stream handoff.
//!
//! A bounded log line buffer and the bounded, non-blocking stream handoff
//! between a process and its readers.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub const DEFAULT_CHUNK_BYTES: usize = 64 * 1024;
pub const DEFAULT_QUEUE_DEPTH: usize = 256;
pub const DEFAULT_MAX_LINE_BYTES: usize = 1024 * 1024;

/// Retains a bounded, drop-oldest sequence of log lines.
pub struct Buffer {
    capacity: usize,
    inner: Mutex<BufferInner>,
}

struct BufferInner {
    items: VecDeque<String>,
    dropped: usize,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            capacity,
            inner: Mutex::new(BufferInner {
                items: VecDeque::with_capacity(capacity),
                dropped: 0,
            }),
        }
    }

    /// Stores a line and reports whether it was truncated.
    pub fn append(&self, line: &str) -> bool {
        let truncated = line.len() > DEFAULT_MAX_LINE_BYTES;
        let line = if truncated {
            &line[..floor_char_boundary(line, DEFAULT_MAX_LINE_BYTES)]
        } else {
            line
        };
        let mut inner = self.inner.lock().expect("log buffer lock");
        if inner.items.len() == self.capacity {
            inner.items.pop_front();
            inner.items.push_back(line.to_string());
            inner.dropped += 1;
            return truncated;
        }
        inner.items.push_back(line.to_string());
        truncated
    }

    pub fn len(&self) -> usize {
        self.inner.lock().expect("log buffer lock").items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn dropped(&self) -> usize {
        self.inner.lock().expect("log buffer lock").dropped
    }
}

/// The largest char boundary at or below `index`, so a cut keeps the line
/// valid UTF-8 (Go cuts at the byte index and tolerates the invalid tail).
fn floor_char_boundary(text: &str, mut index: usize) -> usize {
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Provides bounded, non-blocking stream handoff, the Rust form of Go's
/// buffered channel: [`EventQueue::try_send`] never blocks, [`EventQueue::send`]
/// waits for room, and [`EventQueue::recv`] waits for the next event.
pub struct EventQueue {
    capacity: usize,
    inner: Mutex<EventQueueInner>,
    not_empty: Condvar,
    not_full: Condvar,
}

struct EventQueueInner {
    events: VecDeque<String>,
    closed: bool,
}

impl EventQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            inner: Mutex::new(EventQueueInner {
                events: VecDeque::new(),
                closed: false,
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    pub fn try_send(&self, event: String) -> bool {
        let mut inner = self.inner.lock().expect("event queue lock");
        if inner.closed || inner.events.len() >= self.capacity {
            return false;
        }
        inner.events.push_back(event);
        self.not_empty.notify_one();
        true
    }

    /// Blocks while the queue is full. The event is dropped when the queue is
    /// closed, as a send on Go's closed channel would panic instead.
    pub fn send(&self, event: String) {
        let mut inner = self.inner.lock().expect("event queue lock");
        while !inner.closed && inner.events.len() >= self.capacity {
            inner = self.not_full.wait(inner).expect("event queue lock");
        }
        if inner.closed {
            return;
        }
        inner.events.push_back(event);
        self.not_empty.notify_one();
    }

    /// Blocks until the next event, or returns `None` once the queue is closed
    /// and drained.
    pub fn recv(&self) -> Option<String> {
        let mut inner = self.inner.lock().expect("event queue lock");
        loop {
            if let Some(event) = inner.events.pop_front() {
                self.not_full.notify_one();
                return Some(event);
            }
            if inner.closed {
                return None;
            }
            inner = self.not_empty.wait(inner).expect("event queue lock");
        }
    }

    pub fn try_recv(&self) -> Option<String> {
        let mut inner = self.inner.lock().expect("event queue lock");
        let event = inner.events.pop_front();
        if event.is_some() {
            self.not_full.notify_one();
        }
        event
    }

    pub fn len(&self) -> usize {
        self.inner.lock().expect("event queue lock").events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn close(&self) {
        let mut inner = self.inner.lock().expect("event queue lock");
        inner.closed = true;
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_drops_the_oldest_line() {
        let buffer = Buffer::new(2);
        assert!(buffer.is_empty());
        assert!(!buffer.append("one"));
        assert!(!buffer.append("two"));
        assert!(!buffer.append("three"));
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.dropped(), 1);
        assert!(!buffer.append("four"));
        assert_eq!(buffer.dropped(), 2);
    }

    #[test]
    fn buffer_capacity_has_a_floor() {
        let buffer = Buffer::new(0);
        buffer.append("one");
        buffer.append("two");
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.dropped(), 1);
    }

    #[test]
    fn buffer_truncates_long_lines() {
        let buffer = Buffer::new(1);
        let long = "x".repeat(DEFAULT_MAX_LINE_BYTES + 5);
        assert!(buffer.append(&long));
        assert_eq!(buffer.len(), 1);
        let short = "y".repeat(DEFAULT_MAX_LINE_BYTES);
        assert!(!buffer.append(&short));
    }

    #[test]
    fn queue_is_bounded_and_try_send_never_blocks() {
        let queue = EventQueue::new(1);
        assert!(queue.try_send("one".to_string()));
        assert!(!queue.try_send("two".to_string()));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.recv(), Some("one".to_string()));
        assert!(queue.is_empty());
        assert_eq!(queue.try_recv(), None);
        queue.close();
        assert_eq!(queue.recv(), None);
    }

    #[test]
    fn queue_delivers_in_order() {
        let queue = EventQueue::new(2);
        assert!(queue.try_send("one".to_string()));
        assert!(queue.try_send("two".to_string()));
        assert_eq!(queue.try_recv(), Some("one".to_string()));
        assert_eq!(queue.try_recv(), Some("two".to_string()));
    }
}
