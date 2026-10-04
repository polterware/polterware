package repotest

import (
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/attachments"
	"github.com/polterware/polterware/sparky/backend/domains/focus"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/place"
	"github.com/polterware/polterware/sparky/backend/domains/reminder"
	"github.com/polterware/polterware/sparky/backend/domains/review"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"github.com/polterware/polterware/sparky/backend/domains/tx"
)

// Repos is one backend under the shared repository contract.
type Repos struct {
	Tx          tx.Manager
	Minds       mind.Repository
	Memories    memory.Repository
	Tags        tag.Repository
	Schedules   schedule.Repository
	Places      place.Repository
	Attachments attachments.Store
	Focus       focus.Repository
	Activity    activity.Repository
	Settings    settings.Repository
	Reminders   reminder.Repository
	Review      review.Reader
}

var epoch = time.Date(2026, 10, 3, 21, 30, 0, 0, time.UTC)

func at(n int) time.Time { return epoch.Add(time.Duration(n) * time.Millisecond) }

func ptr[T any](v T) *T { return &v }

// Run executes the shared suite against setup. setup must return a fresh empty store.
func Run(t *testing.T, name string, setup func(*testing.T) (Repos, func())) {
	t.Helper()
	t.Run(name, func(t *testing.T) {
		t.Run("mindCascadeAndRev", func(t *testing.T) { testMindCascadeAndRev(t, setup) })
		t.Run("memoryChecklistTagsCompletions", func(t *testing.T) { testMemoryGraph(t, setup) })
		t.Run("memoryKeyset", func(t *testing.T) { testMemoryKeyset(t, setup) })
		t.Run("tagRetarget", func(t *testing.T) { testTagRetarget(t, setup) })
		t.Run("schedulePlaceSettingsFocus", func(t *testing.T) { testRelated(t, setup) })
		t.Run("activityCursor", func(t *testing.T) { testActivity(t, setup) })
		t.Run("txRollback", func(t *testing.T) { testTxRollback(t, setup) })
	})
}
