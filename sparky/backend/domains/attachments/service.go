package attachments

import (
	"context"
	"net/url"
	"path/filepath"
	"strings"
	"sync"

	"github.com/polterware/polterware/sparky/backend/domains/clock"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/ids"
)

type ServiceImpl struct {
	mu    sync.Mutex
	store Store
	ids   ids.Generator
	clock clock.Clock
}

func NewService(store Store, generator ids.Generator, c clock.Clock) *ServiceImpl {
	return &ServiceImpl{store: store, ids: generator, clock: c}
}

func (s *ServiceImpl) ImportFromPath(ctx context.Context, memoryID, sourcePath, name string) (Attachment, error) {
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
		_ = s.store.RemoveFile(ctx, file.RelPath)
		return Attachment{}, fault.Exhausted("attachment exceeds kind size limit")
	}
	prior, err := s.store.List(ctx, memoryID)
	if err != nil {
		_ = s.store.RemoveFile(ctx, file.RelPath)
		return Attachment{}, err
	}
	for _, existing := range prior {
		if existing.SHA256 != nil && *existing.SHA256 == file.SHA256 {
			_ = s.store.RemoveFile(ctx, file.RelPath)
			return existing, nil
		}
	}
	if name == "" {
		name = filepath.Base(sourcePath)
	}
	mime, size, hash, rel := file.Mime, file.SizeBytes, file.SHA256, file.RelPath
	a := Attachment{ID: id, MemoryID: memoryID, Kind: kind, Name: &name, Mime: &mime, SizeBytes: &size, SHA256: &hash, RelPath: &rel, CreatedAt: s.clock.Now().UTC()}
	if err = s.store.Insert(ctx, a); err != nil {
		_ = s.store.RemoveFile(ctx, rel)
		return Attachment{}, err
	}
	return a, nil
}

func (s *ServiceImpl) AddLink(ctx context.Context, memoryID, rawURL, title string) (Attachment, error) {
	u, err := url.ParseRequestURI(rawURL)
	if err != nil || (u.Scheme != "http" && u.Scheme != "https") || u.Hostname() == "" {
		return Attachment{}, fault.Invalid(fault.Violation{Field: "url", Reason: "must be an http or https URL"})
	}
	a := Attachment{ID: s.ids.New(), MemoryID: memoryID, Kind: KindLink, URL: &rawURL, CreatedAt: s.clock.Now().UTC()}
	if title != "" {
		a.Title = &title
	}
	if err = s.store.Insert(ctx, a); err != nil {
		return Attachment{}, err
	}
	return a, nil
}
func (s *ServiceImpl) List(ctx context.Context, memoryID string) ([]Attachment, error) {
	return s.store.List(ctx, memoryID)
}
func (s *ServiceImpl) Reorder(ctx context.Context, memoryID string, ids []string) ([]Attachment, error) {
	if err := s.store.Reorder(ctx, memoryID, ids); err != nil {
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
	a, err := s.store.Get(ctx, id)
	if err != nil {
		return err
	}
	if err = s.store.Delete(ctx, id); err != nil {
		return err
	}
	if a.RelPath != nil {
		if err = s.store.RemoveFile(ctx, *a.RelPath); err != nil {
			return err
		}
	}
	return nil
}

var _ Service = (*ServiceImpl)(nil)
