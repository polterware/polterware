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
	Set(ctx context.Context, p Place) (Place, error)
	Clear(ctx context.Context, memoryID string) error
	ListInBounds(ctx context.Context, b Bounds) ([]Place, error)
}
