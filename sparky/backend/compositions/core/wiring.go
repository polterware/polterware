package core

import (
	"context"
	"fmt"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/focus"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/review"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"github.com/polterware/polterware/sparky/backend/platform/clock"
	"github.com/polterware/polterware/sparky/backend/platform/ids"
	"github.com/polterware/polterware/sparky/backend/platform/runstate"
	"github.com/polterware/polterware/sparky/backend/providers/sqlite"
)

// Runtime owns provider resources. Close is called after HTTP handlers have stopped.
type Runtime struct {
	Services Services
	MCP      MCP
	Close    func() error
}

// OpenRuntime binds SQLite repositories to domain services. Search query
// execution stays nil until W1-SEARCH; attachment bytes stay unavailable until W1-ATTACH.
func OpenRuntime(_ context.Context, dbPath string) (Runtime, error) {
	store, err := sqlite.Open(dbPath)
	if err != nil {
		return Runtime{}, fmt.Errorf("open sqlite: %w", err)
	}
	clk := clock.Clock{}
	idgen := ids.Generator{}
	audit := activity.Audit{Repository: store.Activity(), Transactions: store, Clock: clk, IDs: idgen}
	planner := schedule.Engine{Clock: clk}
	settingsRepo := publicSettings{store.Settings()}
	settingsSvc := settings.NewService(settingsRepo, audit)
	memories := memory.NewService(store.Memories(), store.Minds(), store.Tags(), store.Schedules(), planner, audit)
	sched := scheduleRuntime{
		clock: clk, audit: audit, memories: store.Memories(), minds: store.Minds(),
		schedules: store.Schedules(), planner: planner, settings: settingsSvc, complete: memories,
	}
	remind := reminderService{
		repo: store.Reminders(), clock: clk, ids: idgen, audit: audit, planner: planner,
		schedules: sched, settings: settingsSvc, complete: memories,
	}
	doc, err := settingsSvc.Get(context.Background())
	if err != nil {
		_ = store.Close()
		return Runtime{}, fmt.Errorf("load settings: %w", err)
	}
	return Runtime{
		Services: Services{
			Memories: memories,
			Minds:    mind.NewService(store.Minds(), store.Memories(), audit),
			Tags:     tag.NewService(store.Tags(), audit),
			Focus:    focus.NewService(store.Focus(), store.Settings(), store.Memories(), audit),
			Review:   review.NewService(store.Review()),
			Activity: activity.NewService(store.Activity()),
			Settings: settingsSvc,
			Places: placeService{
				repo: store.Places(), memories: store.Memories(), clock: clk, audit: audit,
			},
			Attachments: attachmentMeta{store: store.Attachments()},
			Reminders:   remind,
			Agents:      agentService{settings: settingsSvc, tokenPath: runstate.TokenPath(dbPath), audit: audit},
			Maintenance: maintenanceService{
				store: store, minds: store.Minds(), memories: store.Memories(), tags: store.Tags(),
				schedules: store.Schedules(), places: store.Places(), attachments: store.Attachments(),
				activity: store.Activity(), settings: settingsSvc, clock: clk,
			},
			Schedules:      sched,
			ScheduleReader: store.Schedules(),
			PlaceReader:    store.Places(),
			Changes:        liveFeed{bus: store.Changes()},
		},
		MCP:   MCP{Port: doc.AgentsMCPPort, Enabled: doc.AgentsMCPEnabled},
		Close: store.Close,
	}, nil
}
