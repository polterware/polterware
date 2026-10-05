package place

import "context"

// Repository persists places.
type Repository interface {
	Get(ctx context.Context, memoryID string) (Place, error)
	Upsert(ctx context.Context, p Place) error
	Delete(ctx context.Context, memoryID string) error
	ListInBounds(ctx context.Context, b Bounds) ([]Place, error)
}

// Service is the place use-case port.
type Service interface {
	// expectedRev 0 means unchecked; otherwise it must equal the memory revision (checked atomically).
	Set(ctx context.Context, p Place, expectedRev int64) (Place, error)
	Clear(ctx context.Context, memoryID string, expectedRev int64) error
	ListInBounds(ctx context.Context, b Bounds) ([]Place, error)
}
