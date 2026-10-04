package mcp

import (
	"context"
	"sync"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

// MemorySession belongs to exactly one MCP transport session. Its caller must
// discard it when that SDK session closes; it must never be shared by clients.
// Mutations delegate atomic revision checks to the same service used by the UI.
type MemorySession struct {
	service memory.Service
	actor   actor.Actor
	mu      sync.Mutex
	reads   map[string]int64
}

func NewMemorySession(service memory.Service, clientName string) *MemorySession {
	return &MemorySession{service: service, actor: actor.MCP(clientName), reads: make(map[string]int64)}
}

func (s *MemorySession) observe(id string, rev int64) {
	s.mu.Lock()
	defer s.mu.Unlock()
	// Out-of-order read responses must not lower a revision already observed.
	if rev > s.reads[id] {
		s.reads[id] = rev
	}
}

func (s *MemorySession) expected(id string, explicit *int64) int64 {
	if explicit != nil {
		return *explicit
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.reads[id] // zero means first-write last-write-wins at the domain port
}

func (s *MemorySession) Get(ctx context.Context, id string, include memory.Include) (memory.GetResult, error) {
	result, err := s.service.Get(ctx, id, include)
	if err == nil {
		s.observe(result.Memory.ID, result.Memory.Rev)
	}
	return result, err
}

func (s *MemorySession) List(ctx context.Context, filter memory.ListFilter) (memory.ListPage, error) {
	page, err := s.service.List(ctx, filter)
	if err == nil {
		for _, item := range page.Memories {
			s.observe(item.ID, item.Rev)
		}
	}
	return page, err
}

func (s *MemorySession) Update(ctx context.Context, id string, explicitRev *int64, patch memory.Patch) (memory.Memory, error) {
	result, err := s.service.Update(ctx, s.actor, id, s.expected(id, explicitRev), patch)
	if err == nil {
		s.observe(result.ID, result.Rev)
	}
	return result, err
}

func (s *MemorySession) SetStatus(ctx context.Context, id string, explicitRev *int64, status string, occurrence *time.Time) (memory.Memory, error) {
	result, err := s.service.SetStatus(ctx, s.actor, id, s.expected(id, explicitRev), status, occurrence)
	if err == nil {
		s.observe(result.ID, result.Rev)
	}
	return result, err
}

func (s *MemorySession) Create(ctx context.Context, in memory.Create) (memory.Memory, error) {
	result, err := s.service.Create(ctx, s.actor, in)
	if err == nil {
		s.observe(result.ID, result.Rev)
	}
	return result, err
}

func (s *MemorySession) Actor() actor.Actor { return s.actor }

func (s *MemorySession) observeMind(id string, rev int64) { s.observe("mind:"+id, rev) }

func (s *MemorySession) expectedMind(id string, explicit *int64) int64 {
	return s.expected("mind:"+id, explicit)
}
