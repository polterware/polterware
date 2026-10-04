package core

import (
	"testing"

	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/search"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
)

func TestMindDeleteMovesAndReindexesMemoriesAtomically(t *testing.T) {
	for _, rejectAudit := range []bool{false, true} {
		t.Run(map[bool]string{false: "commit", true: "rollback"}[rejectAudit], func(t *testing.T) {
			s, audit, mem := mutationFixture(t)
			svc := mind.NewService(s.Minds(), s.Memories(), audit)
			parent, err := svc.Create(t.Context(), actor.UI(), mind.Create{Name: "Obsolete"})
			if err != nil {
				t.Fatal(err)
			}
			child, err := svc.Create(t.Context(), actor.UI(), mind.Create{Name: "Child", ParentID: &parent.ID})
			if err != nil {
				t.Fatal(err)
			}
			var before []memory.Memory
			for _, id := range []string{parent.ID, child.ID} {
				m, err := mem.Create(t.Context(), actor.UI(), memory.Create{Title: "Retained", MindID: &id})
				if err != nil {
					t.Fatal(err)
				}
				before = append(before, m)
			}
			unrelated, err := mem.Create(t.Context(), actor.UI(), memory.Create{Title: "Inbox"})
			if err != nil {
				t.Fatal(err)
			}
			searchSvc := WireSearch(s)
			oldHits, err := searchSvc.Search(t.Context(), "Obsolete", search.Filter{})
			if err != nil || len(oldHits.Hits) != 2 {
				t.Fatalf("seed index: %+v %v", oldHits, err)
			}
			if rejectAudit {
				audit.Repository = rejectedActivity{audit.Repository}
				svc = mind.NewService(s.Minds(), s.Memories(), audit)
			}
			deleted, err := svc.Delete(t.Context(), actor.UI(), parent.ID, parent.Rev)
			if rejectAudit && err == nil || !rejectAudit && err != nil {
				t.Fatalf("delete: %+v %v", deleted, err)
			}
			if !rejectAudit && (len(deleted.DeletedMindIDs) != 2 || deleted.MovedMemoryCount != 2) {
				t.Fatal(deleted)
			}
			for _, old := range before {
				got, err := s.Memories().Get(t.Context(), old.ID)
				if err != nil {
					t.Fatal(err)
				}
				if rejectAudit {
					if got.MindID == nil || *got.MindID != *old.MindID || got.Rev != old.Rev {
						t.Fatalf("rollback: %+v", got)
					}
				} else {
					if got.MindID != nil || got.Rev != old.Rev+1 || !got.UpdatedAt.After(old.UpdatedAt) {
						t.Fatalf("moved memory: %+v", got)
					}
					title := "stale"
					if _, err := mem.Update(t.Context(), actor.UI(), got.ID, old.Rev, memory.Patch{Title: &title}); !fault.IsCode(err, fault.CodeConflict) {
						t.Fatal(err)
					}
				}
			}
			got, err := s.Memories().Get(t.Context(), unrelated.ID)
			if err != nil || got.Rev != unrelated.Rev || !got.UpdatedAt.Equal(unrelated.UpdatedAt) {
				t.Fatalf("unrelated memory changed: %+v %v", got, err)
			}
			hits, err := searchSvc.Search(t.Context(), "Obsolete", search.Filter{})
			expected := 0
			if rejectAudit {
				expected = 2
			}
			if err != nil || len(hits.Hits) != expected {
				t.Fatalf("index after delete: %+v %v", hits, err)
			}
			page, err := s.Activity().List(t.Context(), "", "", 50)
			if err != nil {
				t.Fatal(err)
			}
			deletes := 0
			for _, event := range page.Events {
				if event.Entity == "mind" && event.EntityID == parent.ID && event.Op == "delete" {
					deletes++
				}
			}
			if rejectAudit && deletes != 0 || !rejectAudit && deletes != 1 {
				t.Fatalf("delete audit rows: %d", deletes)
			}
		})
	}
}

func TestTagMergeInvalidatesAffectedMemoryRevisions(t *testing.T) {
	for _, rejectAudit := range []bool{false, true} {
		t.Run(map[bool]string{false: "commit", true: "rollback"}[rejectAudit], func(t *testing.T) {
			s, audit, mem := mutationFixture(t)
			svc := tag.NewService(s.Tags(), audit)
			from, err := svc.Create(t.Context(), actor.UI(), "Source", nil)
			if err != nil {
				t.Fatal(err)
			}
			to, err := svc.Create(t.Context(), actor.UI(), "Target", nil)
			if err != nil {
				t.Fatal(err)
			}
			var before []memory.Memory
			for _, tags := range [][]string{{from.ID}, {from.ID, to.ID}, {to.ID}} {
				m, err := mem.Create(t.Context(), actor.UI(), memory.Create{Title: "Tagged", TagIDs: tags})
				if err != nil {
					t.Fatal(err)
				}
				before = append(before, m)
			}
			if rejectAudit {
				audit.Repository = rejectedActivity{audit.Repository}
				svc = tag.NewService(s.Tags(), audit)
			}
			merged, err := svc.Merge(t.Context(), actor.UI(), from.ID, to.ID)
			if rejectAudit && err == nil || !rejectAudit && err != nil {
				t.Fatalf("merge: %+v %v", merged, err)
			}
			if !rejectAudit && merged.MovedMemoryCount != 2 {
				t.Fatal(merged)
			}
			for i, old := range before {
				got, err := s.Memories().Get(t.Context(), old.ID)
				if err != nil {
					t.Fatal(err)
				}
				expected := old.Rev
				if !rejectAudit && i < 2 {
					expected++
				}
				if got.Rev != expected {
					t.Fatalf("memory %d rev=%d want %d", i, got.Rev, expected)
				}
				if !rejectAudit && i < 2 {
					if !got.UpdatedAt.After(old.UpdatedAt) {
						t.Fatal("timestamp not advanced")
					}
					if err := mem.SetTags(t.Context(), actor.UI(), old.ID, old.Rev, nil); !fault.IsCode(err, fault.CodeConflict) {
						t.Fatalf("stale SetTags: %v", err)
					}
				}
				ids, err := s.Memories().ListTagIDs(t.Context(), old.ID)
				if err != nil {
					t.Fatal(err)
				}
				if !rejectAudit && (len(ids) != 1 || ids[0] != to.ID) {
					t.Fatalf("tags: %v", ids)
				}
			}
			idx := WireSearch(s)
			hits, err := idx.Search(t.Context(), "Source", search.Filter{})
			expected := 0
			if rejectAudit {
				expected = 2
			}
			if err != nil || len(hits.Hits) != expected {
				t.Fatalf("merge index: %+v %v", hits, err)
			}
		})
	}
}
