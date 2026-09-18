package localcore

import (
	"context"
	"time"

	"github.com/polterware/polterware/backend/domains/operations"
)

func (core *Core) StartScheduleRunner() {
	core.scheduleMu.Lock()
	defer core.scheduleMu.Unlock()
	if core.scheduleCancel != nil {
		return
	}
	ctx, cancel := context.WithCancel(context.Background())
	core.scheduleCancel = cancel
	if core.scheduleFiring == nil {
		core.scheduleFiring = map[string]struct{}{}
	}
	go core.runScheduleLoop(ctx)
}

func (core *Core) StopScheduleRunner() {
	core.scheduleMu.Lock()
	cancel := core.scheduleCancel
	core.scheduleCancel = nil
	core.scheduleMu.Unlock()
	if cancel != nil {
		cancel()
	}
}

func (core *Core) runScheduleLoop(ctx context.Context) {
	ticker := time.NewTicker(schedulePollInterval)
	defer ticker.Stop()
	core.tickSchedules(time.Now().UTC())
	for {
		select {
		case <-ctx.Done():
			return
		case now := <-ticker.C:
			core.tickSchedules(now.UTC())
		}
	}
}

func (core *Core) tickSchedules(now time.Time) {
	items, err := core.Store.ListActiveSchedules()
	if err != nil {
		return
	}
	for _, item := range items {
		core.handleScheduleTick(item, now)
	}
}

func (core *Core) handleScheduleTick(item operations.Schedule, now time.Time) {
	action, newNext, err := DecideScheduleTick(item.Expression, item.Timezone, item.NextRunAt, now)
	if err != nil {
		return
	}

	switch action {
	case scheduleTickWait:
		return
	case scheduleTickAdvance:
		if newNext == item.NextRunAt {
			return
		}
		item.NextRunAt = newNext
		_ = core.Store.SaveSchedule(item)
		return
	case scheduleTickFire:
		core.scheduleMu.Lock()
		if _, busy := core.scheduleFiring[item.ID]; busy {
			core.scheduleMu.Unlock()
			return
		}
		core.scheduleFiring[item.ID] = struct{}{}
		core.scheduleMu.Unlock()

		defer func() {
			core.scheduleMu.Lock()
			delete(core.scheduleFiring, item.ID)
			core.scheduleMu.Unlock()
		}()

		_, _, runErr := core.startWorkflowRun(item.WorkflowID, item.EnvironmentID)
		item.LastRunAt = FormatScheduleInstant(now)
		item.NextRunAt = newNext
		_ = core.Store.SaveSchedule(item)
		_ = runErr
		return
	}
}
