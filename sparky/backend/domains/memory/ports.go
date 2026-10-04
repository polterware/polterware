package memory

import (
	"context"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/actor"
)

// Repository persists memories, checklist rows, tag links, and completions.
// Update succeeds only when the stored rev equals expectedRev, then stores m.Rev.
// expectedRev 0 skips the check.
type Repository interface {
	List(ctx context.Context, filter ListFilter) (ListPage, error)
	Get(ctx context.Context, id string) (Memory, error)
	Insert(ctx context.Context, m Memory) error
	Update(ctx context.Context, expectedRev int64, m Memory) error
	Delete(ctx context.Context, id string) error
	ListCheckItems(ctx context.Context, memoryID string) ([]CheckItem, error)
	ReplaceCheckItems(ctx context.Context, memoryID string, items []CheckItem) error
	GetCheckItem(ctx context.Context, id string) (CheckItem, error)
	UpdateCheckItem(ctx context.Context, item CheckItem) error
	ListTagIDs(ctx context.Context, memoryID string) ([]string, error)
	SetTagIDs(ctx context.Context, memoryID string, tagIDs []string) error
	ListCompletions(ctx context.Context, memoryID string) ([]Completion, error)
	InsertCompletion(ctx context.Context, c Completion) error
	DeleteCompletion(ctx context.Context, memoryID string, occurrenceAt time.Time) error
	CountByMinds(ctx context.Context, mindIDs []string) (int, error)
}

// Service is the memory use-case port. W1-DOMAIN implements it.
// Schedule, place, and attachments are composed by the core on Get.
type Service interface {
	List(ctx context.Context, filter ListFilter) (ListPage, error)
	Get(ctx context.Context, id string, include Include) (GetResult, error)
	Create(ctx context.Context, actor actor.Actor, in Create) (Memory, error)
	Update(ctx context.Context, actor actor.Actor, id string, expectedRev int64, patch Patch) (Memory, error)
	Delete(ctx context.Context, actor actor.Actor, id string) error
	SetStatus(ctx context.Context, actor actor.Actor, id string, expectedRev int64, status string, occurrenceAt *time.Time) (Memory, error)
	SetPinned(ctx context.Context, actor actor.Actor, id string, expectedRev int64, pinned bool) (Memory, error)
	Reorder(ctx context.Context, actor actor.Actor, mindID *string, orderedIDs []string) error
	ToggleCheckItem(ctx context.Context, actor actor.Actor, memoryID, itemID string, done *bool) (CheckItem, error)
	ReplaceChecklist(ctx context.Context, actor actor.Actor, memoryID string, expectedRev int64, items []CheckItemInput) ([]CheckItem, error)
	SetTags(ctx context.Context, actor actor.Actor, memoryID string, expectedRev int64, tagIDs []string) error
	BulkAction(ctx context.Context, actor actor.Actor, action BulkAction) ([]BulkItemResult, error)
}
