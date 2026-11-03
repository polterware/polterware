package polter

import (
	"testing"

	"github.com/polterware/polterware/backend/domains/work"
)

func TestParseCommitRefs(t *testing.T) {
	got := ParseCommitRefs("fix git-metadata-import T2 and other-card T1")
	if len(got) != 2 || got[0] != (CommitRef{Slug: "git-metadata-import", Number: 2}) || got[1] != (CommitRef{Slug: "other-card", Number: 1}) {
		t.Fatalf("plain refs: %+v", got)
	}
	hashed := ParseCommitRefs("polter: git-metadata-import#T2")
	if len(hashed) != 1 || hashed[0] != (CommitRef{Slug: "git-metadata-import", Number: 2}) {
		t.Fatalf("polter ref: %+v", hashed)
	}
	if again := ParseCommitRefs("git-metadata-import T2 git-metadata-import T2"); len(again) != 1 {
		t.Fatalf("duplicate: %+v", again)
	}
	for _, subject := range []string{
		"Git-Metadata-Import T2",
		"git-metadata-importT2",
		"git-metadata-import T0",
		"not a task",
	} {
		if refs := ParseCommitRefs(subject); len(refs) != 0 {
			t.Fatalf("%q: %+v", subject, refs)
		}
	}

	cards := []work.Card{
		{
			Slug: "git-metadata-import", Status: work.CardActive,
			Tasks: []work.Task{
				{ID: "t1", Number: 1, State: work.TaskOpen},
				{ID: "t2", Number: 2, State: work.TaskDone},
			},
		},
		{
			Slug: "dropped-card", Status: work.CardActive,
			Tasks: []work.Task{{ID: "td", Number: 1, State: work.TaskDropped}},
		},
		{
			Slug: "gone", Status: work.CardRemoved,
			Tasks: []work.Task{{ID: "tg", Number: 1, State: work.TaskOpen}},
		},
	}
	matched := MatchExplicitTasks("polter: git-metadata-import#T2 dropped-card T1 gone T1 missing T3", cards)
	if len(matched) != 1 || matched[0].ID != "t2" {
		t.Fatalf("match: %+v", matched)
	}
}
