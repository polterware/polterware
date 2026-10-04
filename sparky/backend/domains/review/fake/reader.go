package fake

import (
	"context"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/review"
)

// Reader is a repository-port skeleton. Methods panic until W1 replaces them.
type Reader struct{}

var _ review.Reader = (*Reader)(nil)

func (*Reader) Completions(context.Context, time.Time, time.Time) ([]review.Completion, error) {
	panic("review/fake: not implemented")
}

func (*Reader) FocusSpans(context.Context, time.Time, time.Time) ([]review.FocusSpan, error) {
	panic("review/fake: not implemented")
}
