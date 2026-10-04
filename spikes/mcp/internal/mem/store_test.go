package mem

import "testing"

func TestCreateAndList(t *testing.T) {
	s := New()
	got := s.Create([]Draft{{Title: ""}, {Title: "Buy milk", Note: "oat"}})
	if got[0].OK || got[0].Error == "" {
		t.Fatalf("empty title = %+v", got[0])
	}
	if !got[1].OK || got[1].ID != "mem_0001" || got[1].Rev != 1 {
		t.Fatalf("created = %+v", got[1])
	}
	page, err := s.List(20, "")
	if err != nil {
		t.Fatal(err)
	}
	if page.Total != 1 || len(page.Memories) != 1 || page.Memories[0].Snippet != "oat" {
		t.Fatalf("page = %+v", page)
	}
	if page.NextCursor != "" {
		t.Fatalf("next = %q", page.NextCursor)
	}
	if _, err := s.List(20, "nope"); err == nil {
		t.Fatal("expected a bad cursor error")
	}
}

func TestSnippetBudget(t *testing.T) {
	note := make([]rune, 200)
	for i := range note {
		note[i] = 'a'
	}
	s := New()
	s.Create([]Draft{{Title: "long", Note: string(note)}})
	page, err := s.List(20, "")
	if err != nil {
		t.Fatal(err)
	}
	if got := len([]rune(page.Memories[0].Snippet)); got != 120 {
		t.Fatalf("snippet runes = %d", got)
	}
}
