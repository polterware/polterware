package mcp

import (
	"context"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/attachments"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/place"
	"github.com/polterware/polterware/sparky/backend/domains/review"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/search"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
)

const (
	ServerName    = "sparky"
	MaxLimit      = 100
	DefaultLimit  = 20
	MaxBatch      = 50
	MaxBodyBytes  = 4 << 20
	snippetRunes  = 120
	defaultMCPVer = "0.0.0-dev"
)

// Schedules is the MCP-facing schedule port. Core adapts ScheduleUseCases to it.
type Schedules interface {
	Set(context.Context, actor.Actor, schedule.Schedule) (schedule.Schedule, error)
	Clear(context.Context, actor.Actor, string) error
	Preview(context.Context, schedule.Schedule, int) ([]time.Time, string, error)
	ListOccurrences(ctx context.Context, from, to time.Time, mindID *string, includeCompleted bool, limit int) ([]schedule.Occurrence, error)
}

// Deps is the same domain surface the UI uses. Optional ports may be nil.
type Deps struct {
	Within      func(context.Context, func(context.Context) error) error
	Version     string
	Memories    memory.Service
	Minds       mind.Service
	Tags        tag.Service
	Activity    activity.Service
	Review      review.Service
	Search      search.Service
	Attachments attachments.Service
	Places      place.Service
	Settings    settings.Service
	Schedules   Schedules
	GetSchedule func(context.Context, string) (schedule.Schedule, error)
	GetPlace    func(context.Context, string) (place.Place, error)
	Now         func() time.Time
}

func (d Deps) now() time.Time {
	if d.Now != nil {
		return d.Now()
	}
	return time.Now()
}
