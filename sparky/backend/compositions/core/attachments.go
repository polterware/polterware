package core

import (
	"context"
	"fmt"
	"path/filepath"
	"sync"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/attachments"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/platform/clock"
	"github.com/polterware/polterware/sparky/backend/platform/ids"
	attachprovider "github.com/polterware/polterware/sparky/backend/providers/attachments"
	"github.com/polterware/polterware/sparky/backend/providers/sqlite"
)

var attachmentServices sync.Map
var attachmentServiceMu sync.Mutex

type attachmentMeta struct {
	store attachments.Store
}

type attachmentComposite struct {
	meta  attachments.Store
	files *attachprovider.Store
}

func (c attachmentComposite) List(ctx context.Context, memoryID string) ([]attachments.Attachment, error) {
	return c.meta.List(ctx, memoryID)
}

func (c attachmentComposite) Get(ctx context.Context, id string) (attachments.Attachment, error) {
	return c.meta.Get(ctx, id)
}

func (c attachmentComposite) Insert(ctx context.Context, a attachments.Attachment) error {
	return c.meta.Insert(ctx, a)
}

func (c attachmentComposite) Delete(ctx context.Context, id string) error {
	return c.meta.Delete(ctx, id)
}

func (c attachmentComposite) Reorder(ctx context.Context, memoryID string, orderedIDs []string) error {
	return c.meta.Reorder(ctx, memoryID, orderedIDs)
}

func (c attachmentComposite) ImportFile(ctx context.Context, memoryID, attachmentID, sourcePath string) (attachments.ImportedFile, error) {
	if c.files == nil {
		return attachments.ImportedFile{}, fault.Unavailable("attachment files are not available")
	}
	return c.files.ImportFile(ctx, memoryID, attachmentID, sourcePath)
}

func (c attachmentComposite) AbsolutePath(relPath string) (string, error) {
	if c.files == nil {
		return "", fault.Unavailable("attachment files are not available")
	}
	return c.files.AbsolutePath(relPath)
}

func (c attachmentComposite) RemoveFile(ctx context.Context, relPath string) error {
	if c.files == nil {
		return fault.Unavailable("attachment files are not available")
	}
	return c.files.RemoveFile(ctx, relPath)
}

func buildAttachmentService(store attachments.Store) (attachments.Service, error) {
	sqliteMeta, ok := store.(sqlite.Attachments)
	if !ok {
		return nil, unavailable("attachment files")
	}
	audit := activity.Audit{Repository: sqliteMeta.Activity(), Transactions: sqliteMeta.Store, Clock: clock.Clock{}, IDs: ids.Generator{}}
	write := func(ctx context.Context, id, op string, fn func(context.Context) error) error {
		return audit.Write(ctx, caller(ctx), "attachment", id, op, "Attachment "+op, fn)
	}
	composite := attachmentComposite{meta: store}
	dbPath := sqliteMeta.Handle().Path()
	if dbPath != ":memory:" {
		files, err := attachprovider.New(sqliteMeta.Handle().Writer(), filepath.Dir(dbPath))
		if err != nil {
			return nil, fmt.Errorf("open attachment files: %w", err)
		}
		// Initialize recovery before publishing the service or accepting imports.
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()
		if _, err := files.SweepOrphansWith(ctx, func(ctx context.Context, id string) error {
			return audit.Write(ctx, actor.System(), "attachment", id, "delete", "Remove attachment with missing file", func(ctx context.Context) error { return store.Delete(ctx, id) })
		}); err != nil {
			return nil, fmt.Errorf("recover attachment files: %w", err)
		}
		composite.files = files
	}
	return attachments.NewService(composite, ids.Generator{}, clock.Clock{}).WithMemoryRev(sqliteMeta.Memories(), sqliteMeta.Store).WithAudit(write), nil
}

func (a attachmentMeta) service() (attachments.Service, error) {
	attachmentServiceMu.Lock()
	defer attachmentServiceMu.Unlock()
	if cached, ok := attachmentServices.Load(a.store); ok {
		return cached.(attachments.Service), nil
	}
	svc, err := buildAttachmentService(a.store)
	if err != nil {
		return nil, err
	}
	actual, _ := attachmentServices.LoadOrStore(a.store, svc)
	return actual.(attachments.Service), nil
}

func (a attachmentMeta) ImportFromPath(ctx context.Context, memoryID, sourcePath, name string, expectedRev int64) (attachments.Attachment, error) {
	svc, err := a.service()
	if err != nil {
		return attachments.Attachment{}, err
	}
	return svc.ImportFromPath(ctx, memoryID, sourcePath, name, expectedRev)
}

func (a attachmentMeta) AddLink(ctx context.Context, memoryID, rawURL, title string, expectedRev int64) (attachments.Attachment, error) {
	svc, err := a.service()
	if err != nil {
		return attachments.Attachment{}, err
	}
	return svc.AddLink(ctx, memoryID, rawURL, title, expectedRev)
}

func (a attachmentMeta) List(ctx context.Context, memoryID string) ([]attachments.Attachment, error) {
	return a.store.List(ctx, memoryID)
}

func (a attachmentMeta) Delete(ctx context.Context, id string) error {
	svc, err := a.service()
	if err != nil {
		return err
	}
	return svc.Delete(ctx, id)
}

func (a attachmentMeta) Reorder(ctx context.Context, memoryID string, orderedIDs []string) ([]attachments.Attachment, error) {
	svc, err := a.service()
	if err != nil {
		return nil, err
	}
	return svc.Reorder(ctx, memoryID, orderedIDs)
}

func (a attachmentMeta) OpenInfo(ctx context.Context, id string) (attachments.OpenInfo, error) {
	svc, err := a.service()
	if err != nil {
		return attachments.OpenInfo{}, err
	}
	return svc.OpenInfo(ctx, id)
}
