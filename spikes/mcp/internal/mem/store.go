// Package mem is the in-memory fake behind both spike MCP servers.
package mem

import (
	"fmt"
	"strconv"
	"sync"
	"unicode/utf8"
)

const snippetLimit = 120

// Summary is the compact list row from plans/sparky-next/04-mcp.md.
type Summary struct {
	ID      string   `json:"id"`
	Title   string   `json:"title"`
	Status  string   `json:"status"`
	Mind    string   `json:"mind,omitempty"`
	Due     string   `json:"due,omitempty"`
	Pinned  bool     `json:"pinned,omitempty"`
	Tags    []string `json:"tags,omitempty"`
	Snippet string   `json:"snippet,omitempty"`
	Rev     int      `json:"rev"`
}

// Draft is one create_memories item.
type Draft struct {
	Title string
	Note  string
}

// ItemResult is one batch entry.
type ItemResult struct {
	OK    bool   `json:"ok"`
	ID    string `json:"id,omitempty"`
	Title string `json:"title,omitempty"`
	Rev   int    `json:"rev,omitempty"`
	Error string `json:"error,omitempty"`
}

// Page is one list_memories page.
type Page struct {
	Memories   []Summary
	Total      int
	NextCursor string
}

// Store is safe for concurrent tool calls.
type Store struct {
	mu    sync.Mutex
	next  int
	items []memory
}

type memory struct {
	id, title, note string
	rev             int
}

// New returns an empty store.
func New() *Store { return &Store{} }

// List returns summaries, newest first.
func (s *Store) List(limit int, cursor string) (Page, error) {
	offset := 0
	if cursor != "" {
		n, err := strconv.Atoi(cursor)
		if err != nil || n < 0 {
			return Page{}, fmt.Errorf("cursor is not valid. Omit it to start from the first page")
		}
		offset = n
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	total := len(s.items)
	if offset > total {
		offset = total
	}
	page := Page{Total: total, Memories: []Summary{}}
	end := offset + limit
	if end > total {
		end = total
	}
	for i := offset; i < end; i++ {
		page.Memories = append(page.Memories, s.items[total-1-i].summary())
	}
	if end < total {
		page.NextCursor = strconv.Itoa(end)
	}
	return page, nil
}

// Create inserts each draft. An empty title is a per-item error.
func (s *Store) Create(items []Draft) []ItemResult {
	out := make([]ItemResult, len(items))
	s.mu.Lock()
	defer s.mu.Unlock()
	for i, item := range items {
		if item.Title == "" {
			out[i] = ItemResult{Error: "Memory title is empty."}
			continue
		}
		s.next++
		mem := memory{
			id:    fmt.Sprintf("mem_%04d", s.next),
			title: item.Title,
			note:  item.Note,
			rev:   1,
		}
		s.items = append(s.items, mem)
		out[i] = ItemResult{OK: true, ID: mem.id, Title: mem.title, Rev: mem.rev}
	}
	return out
}

// Today is the sparky://today resource body.
func (s *Store) Today() string {
	s.mu.Lock()
	defer s.mu.Unlock()
	return fmt.Sprintf("%d memories", len(s.items))
}

func (m memory) summary() Summary {
	row := Summary{ID: m.id, Title: m.title, Status: "open", Rev: m.rev}
	if m.note != "" {
		row.Snippet = snippet(m.note)
	}
	return row
}

func snippet(note string) string {
	if utf8.RuneCountInString(note) <= snippetLimit {
		return note
	}
	runes := []rune(note)
	return string(runes[:snippetLimit])
}
