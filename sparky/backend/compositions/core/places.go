package core

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/clock"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/place"
)

type placeService struct {
	repo     place.Repository
	memories memory.Repository
	clock    clock.Clock
	audit    activity.Audit
}

func (s placeService) Set(ctx context.Context, p place.Place) (place.Place, error) {
	if p.MemoryID == "" {
		return place.Place{}, invalid("place.memory_id", "required")
	}
	if p.Lat < -90 || p.Lat > 90 || p.Lng < -180 || p.Lng > 180 {
		return place.Place{}, invalid("place", "coordinates must be valid WGS84")
	}
	if p.RadiusM == 0 {
		p.RadiusM = place.DefaultRadiusM
	}
	if p.RadiusM < place.MinRadiusM || p.RadiusM > place.MaxRadiusM {
		return place.Place{}, invalid("place.radius_m", "must be between 25 and 5000")
	}
	if p.Event == "" {
		p.Event = place.EventEnter
	}
	if p.Event != place.EventEnter && p.Event != place.EventExit {
		return place.Place{}, invalid("place.event", "must be enter or exit")
	}
	p.UpdatedAt = s.clock.Now().UTC()
	err := s.audit.Write(ctx, caller(ctx), "place", p.MemoryID, activity.OpUpdate, "Set place", func(ctx context.Context) error {
		if err := bumpMemoryRevision(ctx, s.memories, p.MemoryID, p.UpdatedAt); err != nil {
			return err
		}
		return s.repo.Upsert(ctx, p)
	})
	if err != nil {
		return place.Place{}, err
	}
	return p, nil
}

func (s placeService) Clear(ctx context.Context, memoryID string) error {
	if memoryID == "" {
		return invalid("memory_id", "required")
	}
	err := s.audit.Write(ctx, caller(ctx), "place", memoryID, activity.OpDelete, "Clear place", func(ctx context.Context) error {
		if err := s.repo.Delete(ctx, memoryID); err != nil {
			return err
		}
		return bumpMemoryRevision(ctx, s.memories, memoryID, s.clock.Now().UTC())
	})
	if err != nil && !fault.IsCode(err, fault.CodeNotFound) {
		return err
	}
	return err
}

func (s placeService) ListInBounds(ctx context.Context, b place.Bounds) ([]place.Place, error) {
	if b.South > b.North {
		return nil, invalid("bounds", "south must be at most north")
	}
	return s.repo.ListInBounds(ctx, b)
}
