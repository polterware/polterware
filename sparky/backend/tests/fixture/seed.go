// Package fixture seeds synthetic performance data through domain use cases.
package fixture

import (
	"context"
	"fmt"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

func Seed(ctx context.Context, service memory.Service, count int) error {
	for i := 0; i < count; i++ {
		_, err := service.Create(ctx, actor.UI(), memory.Create{Title: fmt.Sprintf("Performance memory %05d", i), Note: fmt.Sprintf("Benchmark group %d", i%100)})
		if err != nil {
			return fmt.Errorf("seed memory %d: %w", i, err)
		}
	}
	return nil
}
