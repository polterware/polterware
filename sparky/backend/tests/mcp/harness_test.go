package mcp_test

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/sparky/backend/domains/activity"
	af "github.com/polterware/polterware/sparky/backend/domains/activity/fake"
	"github.com/polterware/polterware/sparky/backend/domains/activity/fake/testkit"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/attachments"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/place"
	"github.com/polterware/polterware/sparky/backend/domains/review"
	rf "github.com/polterware/polterware/sparky/backend/domains/review/fake"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/search"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
	provider "github.com/polterware/polterware/sparky/backend/providers/mcp"
)

type harness struct {
	fix  *testkit.Fixture
	srv  *provider.Server
	sess *mcp.ClientSession
	att  *attachStub
}

func setupMCP(t *testing.T, configure ...func(*provider.Deps)) *harness {
	t.Helper()
	f := testkit.Setup()
	att := &attachStub{items: map[string][]attachments.Attachment{}}
	att.revs = func(id string) (int64, error) {
		got, err := f.MS.Get(context.Background(), id, memory.Include{})
		if err != nil {
			return 0, err
		}
		return got.Memory.Rev, nil
	}
	deps := provider.Deps{
		Within: f.Store.Within, Memories: f.MS, Minds: f.NS, Tags: f.TS, Activity: activity.NewService(af.New(f.Store)),
		Review: review.NewService(rf.New(f.Store)), Search: searchStub{mem: f.MS}, Attachments: att,
		Places: placeStub{rows: map[string]place.Place{}}, Settings: f.SS, Schedules: scheduleStub{},
		Now: f.Audit.Clock.Now,
	}
	for _, fn := range configure {
		fn(&deps)
	}
	srv, err := provider.New(deps)
	if err != nil {
		t.Fatal(err)
	}
	client := mcp.NewClient(&mcp.Implementation{Name: "test-client", Version: "0"}, nil)
	ct, st := mcp.NewInMemoryTransports()
	go func() { _ = srv.SDK().Run(t.Context(), st) }()
	sess, err := client.Connect(t.Context(), ct, nil)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = sess.Close() })
	return &harness{fix: f, srv: srv, sess: sess, att: att}
}

func (h *harness) call(t *testing.T, name string, args any) *mcp.CallToolResult {
	t.Helper()
	res, err := h.sess.CallTool(t.Context(), &mcp.CallToolParams{Name: name, Arguments: args})
	if err != nil {
		t.Fatal(err)
	}
	return res
}

func (h *harness) allowFiles(t *testing.T) {
	t.Helper()
	if _, err := h.fix.SS.Set(h.fix.Ctx, actor.UI(), []string{settings.KeyAgentsAllowFileAttach}, settings.Settings{AgentsAllowFileAttach: true}); err != nil {
		t.Fatal(err)
	}
}

func structured(t *testing.T, res *mcp.CallToolResult) map[string]any {
	t.Helper()
	raw, err := json.Marshal(res.StructuredContent)
	if err != nil {
		t.Fatal(err)
	}
	var out map[string]any
	if err := json.Unmarshal(raw, &out); err != nil {
		t.Fatal(err)
	}
	return out
}

func textOf(res *mcp.CallToolResult) string {
	if len(res.Content) == 0 {
		return ""
	}
	if t, ok := res.Content[0].(*mcp.TextContent); ok {
		return t.Text
	}
	return ""
}

type searchStub struct{ mem memory.Service }

func (s searchStub) Search(ctx context.Context, q string, f search.Filter) (search.Result, error) {
	page, err := s.mem.List(ctx, memory.ListFilter{Query: q, Limit: f.Limit, MindID: f.MindID, TagIDs: f.TagIDs, Status: f.Status})
	out := search.Result{}
	for _, item := range page.Memories {
		out.Hits = append(out.Hits, search.Hit{MemoryID: item.ID, Title: item.Title, Score: 1, Snippet: item.Snippet, MatchedIn: []string{"title"}})
	}
	return out, err
}
func (searchStub) Status(context.Context) (search.Status, error) { return search.Status{}, nil }
func (searchStub) Reindex(context.Context) (int, error)          { return 0, nil }

type attachStub struct {
	items map[string][]attachments.Attachment
	revs  func(id string) (int64, error)
}

func (a *attachStub) checkRev(id string, expectedRev int64) error {
	if expectedRev == 0 || a.revs == nil {
		return nil
	}
	rev, err := a.revs(id)
	if err != nil {
		return err
	}
	if rev != expectedRev {
		return fault.Conflict("memory", id, rev, memory.Memory{ID: id, Rev: rev})
	}
	return nil
}

func (a *attachStub) ImportFromPath(_ context.Context, memoryID, sourcePath, _ string, expectedRev int64) (attachments.Attachment, error) {
	if err := a.checkRev(memoryID, expectedRev); err != nil {
		return attachments.Attachment{}, err
	}
	item := attachments.Attachment{ID: "file-1", MemoryID: memoryID, Kind: attachments.KindFile}
	a.items[memoryID] = append(a.items[memoryID], item)
	return item, nil
}
func (a *attachStub) AddLink(_ context.Context, memoryID, url, title string, expectedRev int64) (attachments.Attachment, error) {
	if err := a.checkRev(memoryID, expectedRev); err != nil {
		return attachments.Attachment{}, err
	}
	titlePtr, u := title, url
	item := attachments.Attachment{ID: "link-1", MemoryID: memoryID, Kind: attachments.KindLink, URL: &u, Title: &titlePtr}
	a.items[memoryID] = append(a.items[memoryID], item)
	return item, nil
}
func (a *attachStub) List(_ context.Context, memoryID string) ([]attachments.Attachment, error) {
	return a.items[memoryID], nil
}
func (*attachStub) Delete(context.Context, string) error { return nil }
func (*attachStub) Reorder(context.Context, string, []string) ([]attachments.Attachment, error) {
	return nil, nil
}
func (*attachStub) OpenInfo(context.Context, string) (attachments.OpenInfo, error) {
	return attachments.OpenInfo{}, nil
}

type placeStub struct{ rows map[string]place.Place }

func (p placeStub) Set(_ context.Context, _ actor.Actor, in place.Place) (place.Place, error) {
	p.rows[in.MemoryID] = in
	return in, nil
}
func (p placeStub) Clear(_ context.Context, _ actor.Actor, id string) error {
	delete(p.rows, id)
	return nil
}
func (placeStub) ListInBounds(context.Context, place.Bounds) ([]place.Place, error) {
	return nil, nil
}

type scheduleStub struct{}

func (scheduleStub) Set(_ context.Context, _ actor.Actor, in schedule.Schedule) (schedule.Schedule, error) {
	return in, nil
}
func (scheduleStub) Clear(context.Context, actor.Actor, string) error { return nil }
func (scheduleStub) Preview(_ context.Context, in schedule.Schedule, limit int) ([]time.Time, string, error) {
	loc, err := time.LoadLocation(in.TZ)
	if err != nil {
		loc = time.UTC
	}
	rule, err := in.Rule(loc)
	if err != nil {
		return nil, "", err
	}
	return schedule.Preview(rule, limit)
}
func (scheduleStub) ListOccurrences(context.Context, time.Time, time.Time, *string, bool, int) ([]schedule.Occurrence, error) {
	return nil, nil
}
