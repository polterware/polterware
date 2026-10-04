const { createHash } = require('node:crypto');
const path = require('node:path');
const owned = id => typeof id === 'string' && /^(rem:|one:).+/.test(id);
function normalize(items) {
  if (!Array.isArray(items) || items.length > 60) throw Error('plan must contain at most 60 items');
  const seen = new Set();
  return items.map(item => {
    if (!owned(item.id) || seen.has(item.id)) throw Error('duplicate or foreign identifier');
    seen.add(item.id);
    for (const field of ['memoryId', 'title', 'body', 'fireAt']) if (typeof item[field] !== 'string') throw Error(`invalid ${field}`);
    if (!['reminder', 'reminder-focus', 'focus-phase'].includes(item.category)) throw Error('invalid category');
    const date = new Date(item.fireAt);
    if (!Number.isFinite(date.getTime())) throw Error('invalid fireAt');
    const rule = item.repeat;
    let repeat = null;
    if (rule != null) {
      if (typeof rule !== 'object' || Array.isArray(rule) || typeof rule.tz !== 'string') throw Error('repeat requires a timezone');
      new Intl.DateTimeFormat('en', { timeZone: rule.tz });
      repeat = { tz: rule.tz };
      const ranges = { month:[1,12], day:[1,31], weekday:[1,7], hour:[0,23], minute:[0,59], second:[0,59] };
      for (const field of Object.keys(rule)) {
        if (field === 'tz') continue;
        const range = ranges[field], value = rule[field];
        if (!range || !Number.isInteger(value) || value < range[0] || value > range[1]) throw Error(`invalid repeat ${field}`);
      }
      for (const field of Object.keys(ranges)) if (rule[field] != null) repeat[field] = rule[field];
      if (Object.keys(repeat).length < 2) throw Error('empty repeat');
    } else if (date.getTime() <= Date.now()) throw Error('one-shot must be in the future');
    const value = { id:item.id, memoryId:item.memoryId, title:item.title, body:item.body, fireAt:date.toISOString(), category:item.category, repeat,
      occurrenceAt:item.occurrenceAt ?? date.toISOString() };
    value.hash = createHash('sha256').update(JSON.stringify(value)).digest('hex');
    return value;
  });
}
function loadNotify(form, directory) {
  const waiting = new Map(), buffered = [];
  let actionCallback, eventCallback, nextID = 0, send, timer;
  function receive(message) {
    if (message.rid) {
      const pending = waiting.get(message.rid);
      if (!pending) return;
      waiting.delete(message.rid); clearTimeout(pending.timeout);
      message.error ? pending.reject(Error(message.error)) : pending.resolve(message.value);
    } else if (message.event === 'action') {
      actionCallback ? actionCallback(message) : buffered.push(message);
    } else if (eventCallback) eventCallback(message);
  }
  if (form === 'napi') {
    const native = require(path.join(directory, 'notify.node'));
    native.start(raw => receive(JSON.parse(raw)));
    send = (rid, method, value) => native.send(rid, method, JSON.stringify(value));
  } else if (form === 'swift') {
    const koffi = require('koffi');
    const lib = koffi.load(path.join(directory, 'libNotify.dylib'));
    const start = lib.func('void notify_start()');
    const poll = lib.func('void *notify_poll()');
    const release = lib.func('void notify_free(void *ptr)');
    const command = lib.func('void notify_send(const char *rid, const char *method, const char *input)');
    start();
    timer = setInterval(() => {
      const ptr = poll();
      if (!ptr) return;
      try { for (const message of JSON.parse(koffi.decode(ptr, 'char', -1))) receive(message); }
      finally { release(ptr); }
    }, 20);
    send = (rid, method, value) => command(rid, method, JSON.stringify(value));
  } else throw Error('unknown module form');
  function command(method, value = null) {
    return new Promise((resolve, reject) => {
      const rid = String(++nextID);
      const timeout = setTimeout(() => { waiting.delete(rid); reject(Error(`${method} timeout`)); }, 60000);
      waiting.set(rid, { resolve, reject, timeout });
      try { send(rid, method, value); } catch (error) { clearTimeout(timeout); waiting.delete(rid); reject(error); }
    });
  }
  // Serialize mutations; validation happens before either native engine sees data.
  let mutation = Promise.resolve();
  function mutate(method, value) {
    const result = mutation.then(() => command(method, value));
    mutation = result.catch(() => {});
    return result;
  }
  return {
    requestAuthorization: () => command('requestAuthorization'),
    apply: items => mutate('apply', normalize(items)),
    pending: () => command('pending'),
    removeAll: prefix => {
      if (!['rem:', 'one:'].some(p => typeof prefix === 'string' && prefix.startsWith(p))) throw Error('foreign prefix');
      return mutate('removeAll', prefix);
    },
    onAction: cb => { actionCallback = cb; for (const action of buffered.splice(0)) cb(action); },
    status: () => command('status'),
    onEvent: cb => { eventCallback = cb; },
    finishAction: token => command('finishAction', token),
    // Spike-only observation and synthetic event injection, never renderer APIs.
    delivered: () => command('delivered'), testHooks: () => command('testHooks'),
    close: () => { if (timer) clearInterval(timer); }
  };
}
module.exports = { loadNotify, normalize };
