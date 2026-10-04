package core

import (
	"archive/zip"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/attachments"
	"github.com/polterware/polterware/sparky/backend/domains/clock"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/maintenance"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/place"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"github.com/polterware/polterware/sparky/backend/providers/sqlite"
)

type exportMemory struct {
	memory.Memory
	Checklist   []memory.CheckItem       `json:"checklist"`
	TagIDs      []string                 `json:"tagIds"`
	Completions []memory.Completion      `json:"completions"`
	Schedule    *schedule.Schedule       `json:"schedule,omitempty"`
	Place       *place.Place             `json:"place,omitempty"`
	Attachments []attachments.Attachment `json:"attachments"`
}

type exportDoc struct {
	Version    string            `json:"version"`
	ExportedAt time.Time         `json:"exportedAt"`
	Minds      []mind.Mind       `json:"minds"`
	Memories   []exportMemory    `json:"memories"`
	Tags       []tag.Tag         `json:"tags"`
	Settings   settings.Settings `json:"settings"`
	Activity   []activity.Event  `json:"activity"`
}

type maintenanceService struct {
	store       *sqlite.Store
	minds       mind.Repository
	memories    memory.Repository
	tags        tag.Repository
	schedules   schedule.Repository
	places      place.Repository
	attachments attachments.Store
	activity    activity.Repository
	settings    settings.Service
	clock       clock.Clock
}

func (s maintenanceService) Export(ctx context.Context, zipPath string) (int64, error) {
	if zipPath == "" {
		return 0, invalid("zip_path", "required")
	}
	if err := os.MkdirAll(filepath.Dir(zipPath), 0o700); err != nil {
		return 0, fmt.Errorf("create export directory: %w", err)
	}
	doc, err := s.snapshot(ctx)
	if err != nil {
		return 0, err
	}
	file, err := os.CreateTemp(filepath.Dir(zipPath), ".sparky-export-*")
	if err != nil {
		return 0, err
	}
	defer os.Remove(file.Name())
	defer file.Close()
	z := zip.NewWriter(file)
	w, err := z.Create("data.json")
	if err != nil {
		return 0, err
	}
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	if err := enc.Encode(doc); err != nil {
		return 0, err
	}
	if _, err := z.Create("attachments/"); err != nil {
		return 0, err
	}
	if err := z.Close(); err != nil {
		return 0, err
	}
	if err := file.Chmod(0o600); err != nil {
		return 0, err
	}
	if err := file.Close(); err != nil {
		return 0, err
	}
	if err := os.Rename(file.Name(), zipPath); err != nil {
		return 0, err
	}
	info, err := os.Stat(zipPath)
	if err != nil {
		return 0, err
	}
	return info.Size(), nil
}

func (s maintenanceService) snapshot(ctx context.Context) (exportDoc, error) {
	doc := exportDoc{Version: "sparky-export-v1", ExportedAt: s.clock.Now().UTC()}
	var err error
	if doc.Minds, err = s.minds.List(ctx); err != nil {
		return exportDoc{}, err
	}
	if doc.Tags, err = s.tags.List(ctx); err != nil {
		return exportDoc{}, err
	}
	if doc.Settings, err = s.settings.Get(ctx); err != nil {
		return exportDoc{}, err
	}
	cursor := ""
	for {
		page, err := s.memories.List(ctx, memory.ListFilter{Limit: 500, Cursor: cursor})
		if err != nil {
			return exportDoc{}, err
		}
		for _, summary := range page.Memories {
			item, err := s.exportMemory(ctx, summary.ID)
			if err != nil {
				return exportDoc{}, err
			}
			doc.Memories = append(doc.Memories, item)
		}
		if page.NextCursor == "" {
			break
		}
		cursor = page.NextCursor
	}
	cursor = ""
	for {
		page, err := s.activity.List(ctx, cursor, "", 500)
		if err != nil {
			return exportDoc{}, err
		}
		doc.Activity = append(doc.Activity, page.Events...)
		if page.NextCursor == "" {
			break
		}
		cursor = page.NextCursor
	}
	return doc, nil
}

func (s maintenanceService) exportMemory(ctx context.Context, id string) (exportMemory, error) {
	m, err := s.memories.Get(ctx, id)
	if err != nil {
		return exportMemory{}, err
	}
	item := exportMemory{Memory: m}
	if item.Checklist, err = s.memories.ListCheckItems(ctx, id); err != nil {
		return exportMemory{}, err
	}
	if item.TagIDs, err = s.memories.ListTagIDs(ctx, id); err != nil {
		return exportMemory{}, err
	}
	if item.Completions, err = s.memories.ListCompletions(ctx, id); err != nil {
		return exportMemory{}, err
	}
	if item.Attachments, err = s.attachments.List(ctx, id); err != nil {
		return exportMemory{}, err
	}
	if sc, err := s.schedules.Get(ctx, id); err == nil {
		item.Schedule = &sc
	} else if !fault.IsCode(err, fault.CodeNotFound) {
		return exportMemory{}, err
	}
	if p, err := s.places.Get(ctx, id); err == nil {
		item.Place = &p
	} else if !fault.IsCode(err, fault.CodeNotFound) {
		return exportMemory{}, err
	}
	return item, nil
}

func (s maintenanceService) Backup(ctx context.Context, destPath string) (string, int64, error) {
	if destPath == "" {
		return "", 0, invalid("dest_path", "required")
	}
	if err := os.MkdirAll(filepath.Dir(destPath), 0o700); err != nil {
		return "", 0, fmt.Errorf("create backup directory: %w", err)
	}
	n, err := s.store.Backup(ctx, destPath)
	return destPath, n, err
}

func (s maintenanceService) Doctor(ctx context.Context) (maintenance.Report, error) {
	report := maintenance.Report{OK: true}
	add := func(name string, ok bool, detail string) {
		report.Checks = append(report.Checks, maintenance.Check{Name: name, OK: ok, Detail: detail})
		if !ok {
			report.OK = false
		}
	}
	var integrity string
	if err := s.store.Handle().Writer().QueryRowContext(ctx, `PRAGMA integrity_check`).Scan(&integrity); err != nil {
		add("integrity_check", false, err.Error())
	} else {
		add("integrity_check", integrity == "ok", integrity)
	}
	rows, err := s.store.Handle().Writer().QueryContext(ctx, `PRAGMA foreign_key_check`)
	if err != nil {
		add("foreign_key_check", false, err.Error())
	} else {
		defer rows.Close()
		n := 0
		for rows.Next() {
			n++
		}
		add("foreign_key_check", n == 0, fmt.Sprintf("%d violations", n))
	}
	add("sqlite", true, s.store.Handle().Path())
	add("change_feed", true, "committed transactions only")
	return report, nil
}
