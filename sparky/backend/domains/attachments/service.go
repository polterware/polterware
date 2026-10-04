package attachments

import (
	"context"
	"errors"
	"fmt"
	"net/url"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/clock"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/ids"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/tx"
)

type ServiceImpl struct {
	mu       sync.Mutex
	store    Store
	ids      ids.Generator
	clock    clock.Clock
	memories memory.Repository
	tx       tx.Manager
	audit    func(context.Context, string, string, func(context.Context) error) error
}

func NewService(store Store, generator ids.Generator, c clock.Clock) *ServiceImpl {
	return &ServiceImpl{store: store, ids: generator, clock: c}
}

func (s *ServiceImpl) WithMemoryRev(memories memory.Repository, tx tx.Manager) *ServiceImpl {
	s.memories = memories
	s.tx = tx
	return s
}

// WithAudit joins metadata mutations and activity in the same transaction.
func (s *ServiceImpl) WithAudit(write func(context.Context, string, string, func(context.Context) error) error) *ServiceImpl {
	s.audit = write
	return s
}

func (s *ServiceImpl) mutate(ctx context.Context, id, op string, fn func(context.Context) error) error {
	if s.audit != nil {
		return s.audit(ctx, id, op, fn)
	}
	return s.within(ctx, fn)
}

func (s *ServiceImpl) cleanup(ctx context.Context, rel string, cause error) error {
	cleanupCtx, cancel := context.WithTimeout(context.WithoutCancel(ctx), 5*time.Second)
	defer cancel()
	if err := s.store.RemoveFile(cleanupCtx, rel); err != nil {
		return errors.Join(cause, fmt.Errorf("cleanup attachment %s: %w", rel, err))
	}
	return cause
}

func (s *ServiceImpl) within(ctx context.Context, fn func(context.Context) error) error {
	if s.tx == nil {
		return fn(ctx)
	}
	return s.tx.Within(ctx, fn)
}

func (s *ServiceImpl) checkMemoryRev(ctx context.Context, memoryID string, expectedRev int64) error {
	if expectedRev == 0 {
		return nil
	}
	if s.memories == nil {
		return fault.Unavailable("memory revision check is not wired")
	}
	m, err := s.memories.Get(ctx, memoryID)
	if err != nil {
		return err
	}
	if m.Rev != expectedRev {
		return fault.Conflict("memory", memoryID, m.Rev, m)
	}
	return nil
}

func (s *ServiceImpl) ImportFromPath(ctx context.Context, memoryID, sourcePath, name string, expectedRev int64) (Attachment, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if memoryID == "" || sourcePath == "" {
		return Attachment{}, fault.Invalid(fault.Violation{Field: "source_path", Reason: "required"})
	}
	id := s.ids.New()
	file, err := s.store.ImportFile(ctx, memoryID, id, sourcePath)
	if err != nil {
		return Attachment{}, err
	}
	kind := KindFile
	if strings.HasPrefix(file.Mime, "image/") {
		kind = KindPhoto
	}
	if strings.HasPrefix(file.Mime, "audio/") {
		kind = KindAudio
	}
	limit := int64(MaxFileBytes)
	if kind == KindPhoto {
		limit = MaxPhotoBytes
	}
	if kind == KindAudio {
		limit = MaxAudioBytes
	}
	if file.SizeBytes > limit {
		return Attachment{}, s.cleanup(ctx, file.RelPath, fault.Exhausted("attachment exceeds kind size limit"))
	}
	if name == "" {
		name = filepath.Base(sourcePath)
	}
	mime, size, hash, rel := file.Mime, file.SizeBytes, file.SHA256, file.RelPath
	a := Attachment{ID: id, MemoryID: memoryID, Kind: kind, Name: &name, Mime: &mime, SizeBytes: &size, SHA256: &hash, RelPath: &rel, CreatedAt: s.clock.Now().UTC()}
	var duplicate *Attachment
	if err = s.within(ctx, func(ctx context.Context) error {
		if err := s.checkMemoryRev(ctx, memoryID, expectedRev); err != nil {
			return err
		}
		prior, err := s.store.List(ctx, memoryID)
		if err != nil {
			return err
		}
		for _, existing := range prior {
			if existing.SHA256 != nil && *existing.SHA256 == file.SHA256 {
				duplicate = &existing
				return nil
			}
		}
		return s.mutate(ctx, a.ID, "create", func(ctx context.Context) error { return s.store.Insert(ctx, a) })
	}); err != nil {
		return Attachment{}, s.cleanup(ctx, rel, err)
	}
	if duplicate != nil {
		if err := s.cleanup(ctx, rel, nil); err != nil {
			return Attachment{}, err
		}
		return *duplicate, nil
	}
	return a, nil
}

func (s *ServiceImpl) AddLink(ctx context.Context, memoryID, rawURL, title string, expectedRev int64) (Attachment, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	u, err := url.ParseRequestURI(rawURL)
	if err != nil || (u.Scheme != "http" && u.Scheme != "https") || u.Hostname() == "" {
		return Attachment{}, fault.Invalid(fault.Violation{Field: "url", Reason: "must be an http or https URL"})
	}
	a := Attachment{ID: s.ids.New(), MemoryID: memoryID, Kind: KindLink, URL: &rawURL, CreatedAt: s.clock.Now().UTC()}
	if title != "" {
		a.Title = &title
	}
	err = s.mutate(ctx, a.ID, "create", func(ctx context.Context) error {
		if err := s.checkMemoryRev(ctx, memoryID, expectedRev); err != nil {
			return err
		}
		return s.store.Insert(ctx, a)
	})
	if err != nil {
		return Attachment{}, err
	}
	return a, nil
}
func (s *ServiceImpl) List(ctx context.Context, memoryID string) ([]Attachment, error) {
	return s.store.List(ctx, memoryID)
}
func (s *ServiceImpl) Reorder(ctx context.Context, memoryID string, ids []string) ([]Attachment, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if err := s.mutate(ctx, memoryID, "update", func(ctx context.Context) error { return s.store.Reorder(ctx, memoryID, ids) }); err != nil {
		return nil, err
	}
	return s.store.List(ctx, memoryID)
}
func (s *ServiceImpl) OpenInfo(ctx context.Context, id string) (OpenInfo, error) {
	a, err := s.store.Get(ctx, id)
	if err != nil {
		return OpenInfo{}, err
	}
	if a.RelPath == nil {
		return OpenInfo{}, fault.Invalid(fault.Violation{Field: "id", Reason: "links have no local file"})
	}
	p, err := s.store.AbsolutePath(*a.RelPath)
	if err != nil {
		return OpenInfo{}, err
	}
	mime, name := "application/octet-stream", ""
	if a.Mime != nil {
		mime = *a.Mime
	}
	if a.Name != nil {
		name = *a.Name
	}
	return OpenInfo{ID: a.ID, AbsolutePath: p, Mime: mime, Name: name, Kind: a.Kind}, nil
}
func (s *ServiceImpl) Delete(ctx context.Context, id string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	a, err := s.store.Get(ctx, id)
	if err != nil {
		return err
	}
	if err = s.mutate(ctx, id, "delete", func(ctx context.Context) error { return s.store.Delete(ctx, id) }); err != nil {
		return err
	}
	if a.RelPath != nil {
		if err = s.cleanup(ctx, *a.RelPath, nil); err != nil {
			return err
		}
	}
	return nil
}

var _ Service = (*ServiceImpl)(nil)
