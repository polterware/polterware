package sqlite

import (
	"context"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
)

func TestForeignKeyCheckAfterRandomOps(t *testing.T) {
	s := openStore(t)
	ctx := context.Background()
	now := time.Date(2026, 10, 3, 21, 30, 0, 0, time.UTC)
	rng := rand.New(rand.NewSource(4))
	var minds []string
	var memories []string
	var tags []string
	for i := 0; i < 200; i++ {
		switch rng.Intn(8) {
		case 0:
			id := fmt.Sprintf("mind%03d", i)
			var parent *string
			if len(minds) > 0 && rng.Intn(2) == 0 {
				parent = &minds[rng.Intn(len(minds))]
			}
			if err := s.Minds().Insert(ctx, mind.Mind{ID: id, ParentID: parent, Name: id, CreatedAt: now, UpdatedAt: now, Rev: 1}); err == nil {
				minds = append(minds, id)
			}
		case 1:
			if len(minds) == 0 {
				continue
			}
			id := minds[rng.Intn(len(minds))]
			_ = s.Minds().Delete(ctx, id)
			next := minds[:0]
			for _, m := range minds {
				if _, err := s.Minds().Get(ctx, m); err == nil {
					next = append(next, m)
				}
			}
			minds = next
		case 2:
			id := fmt.Sprintf("mem%03d", i)
			var mindID *string
			if len(minds) > 0 && rng.Intn(3) != 0 {
				mindID = &minds[rng.Intn(len(minds))]
			}
			if err := s.Memories().Insert(ctx, memory.Memory{ID: id, MindID: mindID, Title: id, Status: memory.StatusActive, CreatedAt: now, UpdatedAt: now, Rev: 1}); err == nil {
				memories = append(memories, id)
			}
		case 3:
			if len(memories) == 0 {
				continue
			}
			id := memories[rng.Intn(len(memories))]
			_ = s.Memories().Delete(ctx, id)
			next := memories[:0]
			for _, m := range memories {
				if _, err := s.Memories().Get(ctx, m); err == nil {
					next = append(next, m)
				}
			}
			memories = next
		case 4:
			id := fmt.Sprintf("tag%03d", i)
			if err := s.Tags().Insert(ctx, tag.Tag{ID: id, Name: id, CreatedAt: now, UpdatedAt: now, Rev: 1}); err == nil {
				tags = append(tags, id)
			}
		case 5:
			if len(tags) == 0 {
				continue
			}
			id := tags[rng.Intn(len(tags))]
			_ = s.Tags().Delete(ctx, id)
			next := tags[:0]
			for _, x := range tags {
				if _, err := s.Tags().Get(ctx, x); err == nil {
					next = append(next, x)
				}
			}
			tags = next
		case 6:
			if len(memories) == 0 || len(tags) == 0 {
				continue
			}
			_ = s.Memories().SetTagIDs(ctx, memories[rng.Intn(len(memories))], []string{tags[rng.Intn(len(tags))]})
		default:
			if len(memories) == 0 {
				continue
			}
			id := memories[rng.Intn(len(memories))]
			_ = s.Memories().ReplaceCheckItems(ctx, id, []memory.CheckItem{{
				ID: fmt.Sprintf("chk%03d", i), MemoryID: id, Title: "item", CreatedAt: now, UpdatedAt: now,
			}})
		}
	}
	rows, err := s.Handle().Writer().QueryContext(ctx, `PRAGMA foreign_key_check`)
	if err != nil {
		t.Fatal(err)
	}
	defer rows.Close()
	var violations int
	for rows.Next() {
		violations++
	}
	if err := rows.Err(); err != nil {
		t.Fatal(err)
	}
	if violations != 0 {
		t.Fatalf("foreign_key_check rows=%d", violations)
	}
}
