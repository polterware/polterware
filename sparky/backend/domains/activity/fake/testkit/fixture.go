// Package testkit wires the owned domain services around one transactional fake store.
package testkit

import (
	"context"

	"fmt"
	"github.com/polterware/polterware/sparky/backend/domains/activity"
	af "github.com/polterware/polterware/sparky/backend/domains/activity/fake"

	"github.com/polterware/polterware/sparky/backend/domains/focus"
	ff "github.com/polterware/polterware/sparky/backend/domains/focus/fake"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	mf "github.com/polterware/polterware/sparky/backend/domains/memory/fake"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	nf "github.com/polterware/polterware/sparky/backend/domains/mind/fake"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
	sf "github.com/polterware/polterware/sparky/backend/domains/settings/fake"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	tf "github.com/polterware/polterware/sparky/backend/domains/tag/fake"
	"sync"
	"time"
)

type fixedClock struct{ at time.Time }

func (c fixedClock) Now() time.Time { return c.at }

type generator struct {
	mu sync.Mutex
	n  int
}

func (g *generator) New() string {
	g.mu.Lock()
	defer g.mu.Unlock()
	g.n++
	return fmt.Sprintf("01K00000000000000000%06d", g.n)
}

type Fixture struct {
	Ctx      context.Context
	Store    *af.Store
	Audit    activity.Audit
	Mem      *mf.Repository
	Minds    *nf.Repository
	Tags     *tf.Repository
	Settings *sf.Repository
	Focus    *ff.Repository
	MS       memory.Service
	NS       mind.Service
	TS       tag.Service
	SS       settings.Service
	FS       focus.Service
}

func Setup() *Fixture {
	st := af.NewStore()
	f := &Fixture{Ctx: context.Background(), Store: st, Mem: mf.New(st), Minds: nf.New(st), Tags: tf.New(st), Settings: sf.New(st), Focus: ff.New(st)}
	f.Audit = activity.Audit{Repository: af.New(st), Transactions: st, Clock: fixedClock{time.Date(2026, 10, 4, 18, 0, 0, 0, time.UTC)}, IDs: &generator{}}
	f.MS = memory.NewService(f.Mem, f.Minds, f.Tags, nil, nil, f.Audit)
	f.NS = mind.NewService(f.Minds, f.Mem, f.Audit)
	f.TS = tag.NewService(f.Tags, f.Audit)
	f.SS = settings.NewService(f.Settings, f.Audit)
	f.FS = focus.NewService(f.Focus, f.Settings, f.Mem, f.Audit)
	return f
}
