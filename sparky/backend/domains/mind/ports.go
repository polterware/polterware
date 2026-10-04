package mind

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/actor"
)

// Repository persists minds.
// Update succeeds only when the stored rev equals expectedRev, then stores m.Rev.
// expectedRev 0 skips the check.
type Repository interface {
	List(ctx context.Context) ([]Mind, error)
	Get(ctx context.Context, id string) (Mind, error)
	Insert(ctx context.Context, m Mind) error
	Update(ctx context.Context, expectedRev int64, m Mind) error
	Delete(ctx context.Context, id string) error
}

// Service is the mind use-case port. W1-DOMAIN implements it.
type Service interface {
	List(ctx context.Context) ([]Mind, error)
	Get(ctx context.Context, id string) (Mind, error)
	Create(ctx context.Context, actor actor.Actor, in Create) (Mind, error)
	Update(ctx context.Context, actor actor.Actor, id string, expectedRev int64, patch Patch) (Mind, error)
	Delete(ctx context.Context, actor actor.Actor, id string) (DeleteResult, error)
	Reorder(ctx context.Context, actor actor.Actor, parentID *string, orderedIDs []string) ([]Mind, error)
}
