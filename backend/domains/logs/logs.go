package logs

import "sync"

const (
	DefaultChunkBytes   = 64 * 1024
	DefaultQueueDepth   = 256
	DefaultMaxLineBytes = 1024 * 1024
)

// Buffer retains a bounded, drop-oldest sequence of log lines.
type Buffer struct {
	mu       sync.RWMutex
	capacity int
	items    []string
	dropped  int
}

func NewBuffer(capacity int) *Buffer {
	if capacity < 1 {
		capacity = 1
	}
	return &Buffer{capacity: capacity, items: make([]string, 0, capacity)}
}

// Append stores a line and reports whether it was truncated.
func (buffer *Buffer) Append(line string) bool {
	buffer.mu.Lock()
	defer buffer.mu.Unlock()

	truncated := len(line) > DefaultMaxLineBytes
	if truncated {
		line = line[:DefaultMaxLineBytes]
	}
	if len(buffer.items) == buffer.capacity {
		copy(buffer.items, buffer.items[1:])
		buffer.items[len(buffer.items)-1] = line
		buffer.dropped++
		return truncated
	}
	buffer.items = append(buffer.items, line)
	return truncated
}

func (buffer *Buffer) Len() int {
	buffer.mu.RLock()
	defer buffer.mu.RUnlock()
	return len(buffer.items)
}

func (buffer *Buffer) Dropped() int {
	buffer.mu.RLock()
	defer buffer.mu.RUnlock()
	return buffer.dropped
}

// EventQueue provides bounded, non-blocking stream handoff.
type EventQueue struct {
	events chan string
}

func NewEventQueue(capacity int) *EventQueue {
	if capacity < 1 {
		capacity = 1
	}
	return &EventQueue{events: make(chan string, capacity)}
}

func (queue *EventQueue) TrySend(event string) bool {
	select {
	case queue.events <- event:
		return true
	default:
		return false
	}
}

func (queue *EventQueue) Events() <-chan string {
	return queue.events
}
