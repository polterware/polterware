// Package attachments stores attachment metadata and copied files.
package attachments

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"errors"
	"fmt"
	"io"
	"mime"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	domain "github.com/polterware/polterware/sparky/backend/domains/attachments"
)

// Store uses the core's SQLite connection and a private attachments directory.
type Store struct {
	db   *sql.DB
	root string
}

var ulidPattern = regexp.MustCompile(`^[0-9A-HJKMNP-TV-Z]{26}$`)

func New(db *sql.DB, dataDir string) (*Store, error) {
	root := filepath.Join(dataDir, "attachments")
	if err := os.MkdirAll(root, 0700); err != nil {
		return nil, fmt.Errorf("create attachment directory: %w", err)
	}
	return &Store{db: db, root: root}, nil
}

func (s *Store) ImportFile(ctx context.Context, memoryID, id, source string) (domain.ImportedFile, error) {
	if !ulidPattern.MatchString(memoryID) || !ulidPattern.MatchString(id) {
		return domain.ImportedFile{}, fmt.Errorf("memory and attachment IDs must be ULIDs")
	}
	info, err := os.Lstat(source)
	if err != nil {
		return domain.ImportedFile{}, fmt.Errorf("stat source: %w", err)
	}
	if info.Mode()&os.ModeSymlink != 0 || !info.Mode().IsRegular() {
		return domain.ImportedFile{}, fmt.Errorf("attachment source must be a regular non-symlink file")
	}
	if info.Size() > domain.HardMaxBytes {
		return domain.ImportedFile{}, fmt.Errorf("attachment exceeds hard size limit")
	}
	in, err := os.Open(source)
	if err != nil {
		return domain.ImportedFile{}, fmt.Errorf("open source: %w", err)
	}
	defer in.Close()
	buf := make([]byte, 512)
	n, readErr := io.ReadFull(in, buf)
	if readErr != nil && !errors.Is(readErr, io.EOF) && !errors.Is(readErr, io.ErrUnexpectedEOF) {
		return domain.ImportedFile{}, readErr
	}
	buf = buf[:n]
	detected := http.DetectContentType(buf)
	if i := strings.IndexByte(detected, ';'); i >= 0 {
		detected = detected[:i]
	}
	ext := ".bin"
	switch detected {
	case "image/jpeg":
		ext = ".jpg"
	case "image/png":
		ext = ".png"
	case "image/gif":
		ext = ".gif"
	case "image/webp":
		ext = ".webp"
	case "audio/mpeg":
		ext = ".mp3"
	case "audio/wav", "audio/x-wav":
		ext = ".wav"
	case "audio/mp4":
		ext = ".m4a"
	case "application/pdf":
		ext = ".pdf"
	case "text/plain; charset=utf-8":
		ext = ".txt"
	}
	if mt := mime.TypeByExtension(ext); mt != "" && detected == "application/octet-stream" {
		detected = mt
	}
	if err := ctx.Err(); err != nil {
		return domain.ImportedFile{}, err
	}
	// Both directory components are generated IDs; source names never enter the stored path.
	dir := filepath.Join(s.root, memoryID)
	if err := os.MkdirAll(dir, 0700); err != nil {
		return domain.ImportedFile{}, err
	}
	rel := filepath.Join(memoryID, id+ext)
	dest := filepath.Join(s.root, rel)
	tmp, err := os.CreateTemp(dir, ".import-*")
	if err != nil {
		return domain.ImportedFile{}, err
	}
	tmpName := tmp.Name()
	defer os.Remove(tmpName)
	h := sha256.New()
	w := io.MultiWriter(tmp, h)
	if _, err = w.Write(buf); err != nil {
		tmp.Close()
		return domain.ImportedFile{}, err
	}
	written, err := io.Copy(w, in)
	if err != nil {
		tmp.Close()
		return domain.ImportedFile{}, err
	}
	size := int64(n) + written
	if size > domain.HardMaxBytes {
		tmp.Close()
		return domain.ImportedFile{}, fmt.Errorf("attachment exceeds hard size limit")
	}
	if err = tmp.Sync(); err != nil {
		tmp.Close()
		return domain.ImportedFile{}, err
	}
	if err = tmp.Close(); err != nil {
		return domain.ImportedFile{}, err
	}
	if err = os.Rename(tmpName, dest); err != nil {
		return domain.ImportedFile{}, err
	}
	return domain.ImportedFile{RelPath: rel, SizeBytes: size, SHA256: fmt.Sprintf("%x", h.Sum(nil)), Mime: detected}, nil
}
func (s *Store) AbsolutePath(rel string) (string, error) {
	p := filepath.Clean(rel)
	if p == "." || filepath.IsAbs(p) || strings.HasPrefix(p, ".."+string(filepath.Separator)) {
		return "", fmt.Errorf("invalid attachment path")
	}
	full := filepath.Join(s.root, p)
	if !within(s.root, full) {
		return "", fmt.Errorf("invalid attachment path")
	}
	return full, nil
}
func within(root, path string) bool {
	r, e := filepath.Rel(root, path)
	return e == nil && r != ".." && !strings.HasPrefix(r, ".."+string(filepath.Separator))
}
func (s *Store) RemoveFile(ctx context.Context, rel string) error {
	if err := ctx.Err(); err != nil {
		return err
	}
	p, err := s.AbsolutePath(rel)
	if err != nil {
		return err
	}
	err = os.Remove(p)
	if errors.Is(err, os.ErrNotExist) {
		return nil
	}
	return err
}
func (s *Store) List(ctx context.Context, memoryID string) ([]domain.Attachment, error) {
	rows, err := s.db.QueryContext(ctx, `SELECT id,memory_id,kind,name,mime,size_bytes,sha256,rel_path,url,title,sort_order,created_at FROM attachments WHERE memory_id=? ORDER BY sort_order,created_at`, memoryID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := []domain.Attachment{}
	for rows.Next() {
		a, e := scan(rows)
		if e != nil {
			return nil, e
		}
		out = append(out, a)
	}
	return out, rows.Err()
}
func (s *Store) Get(ctx context.Context, id string) (domain.Attachment, error) {
	a, e := scan(s.db.QueryRowContext(ctx, `SELECT id,memory_id,kind,name,mime,size_bytes,sha256,rel_path,url,title,sort_order,created_at FROM attachments WHERE id=?`, id))
	if errors.Is(e, sql.ErrNoRows) {
		return a, fmt.Errorf("attachment not found: %s", id)
	}
	return a, e
}

type scanner interface{ Scan(...any) error }

func scan(r scanner) (a domain.Attachment, err error) {
	var name, mime, hash, rel, url, title sql.NullString
	var size sql.NullInt64
	err = r.Scan(&a.ID, &a.MemoryID, &a.Kind, &name, &mime, &size, &hash, &rel, &url, &title, &a.SortOrder, &a.CreatedAt)
	if err != nil {
		return
	}
	if name.Valid {
		a.Name = &name.String
	}
	if mime.Valid {
		a.Mime = &mime.String
	}
	if size.Valid {
		a.SizeBytes = &size.Int64
	}
	if hash.Valid {
		a.SHA256 = &hash.String
	}
	if rel.Valid {
		a.RelPath = &rel.String
	}
	if url.Valid {
		a.URL = &url.String
	}
	if title.Valid {
		a.Title = &title.String
	}
	return
}
func (s *Store) Insert(ctx context.Context, a domain.Attachment) error {
	_, err := s.db.ExecContext(ctx, `INSERT INTO attachments(id,memory_id,kind,name,mime,size_bytes,sha256,rel_path,url,title,sort_order,created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)`, a.ID, a.MemoryID, a.Kind, a.Name, a.Mime, a.SizeBytes, a.SHA256, a.RelPath, a.URL, a.Title, a.SortOrder, a.CreatedAt.UTC().Format("2006-01-02T15:04:05.000Z"))
	return err
}
func (s *Store) Delete(ctx context.Context, id string) error {
	_, err := s.db.ExecContext(ctx, `DELETE FROM attachments WHERE id=?`, id)
	return err
}
func (s *Store) Reorder(ctx context.Context, memoryID string, ids []string) error {
	tx, err := s.db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	defer tx.Rollback()
	for i, id := range ids {
		r, e := tx.ExecContext(ctx, `UPDATE attachments SET sort_order=? WHERE id=? AND memory_id=?`, i, id, memoryID)
		if e != nil {
			return e
		}
		n, e := r.RowsAffected()
		if e != nil {
			return e
		}
		if n != 1 {
			return fmt.Errorf("attachment %q does not belong to memory", id)
		}
	}
	return tx.Commit()
}

// SweepOrphans removes files without rows and rows whose files disappeared.
func (s *Store) SweepOrphans(ctx context.Context) (int, error) {
	rows, err := s.db.QueryContext(ctx, `SELECT id,rel_path FROM attachments WHERE rel_path IS NOT NULL`)
	if err != nil {
		return 0, err
	}
	known := map[string]string{}
	for rows.Next() {
		var id, rel string
		if err = rows.Scan(&id, &rel); err != nil {
			rows.Close()
			return 0, err
		}
		known[rel] = id
	}
	if err = rows.Err(); err != nil {
		rows.Close()
		return 0, err
	}
	rows.Close()
	removed := 0
	err = filepath.WalkDir(s.root, func(path string, d os.DirEntry, e error) error {
		if e != nil {
			return e
		}
		if d.IsDir() {
			return nil
		}
		rel, e := filepath.Rel(s.root, path)
		if e != nil {
			return e
		}
		if _, ok := known[rel]; !ok {
			if e = os.Remove(path); e != nil {
				return e
			}
			removed++
		}
		delete(known, rel)
		return nil
	})
	if err != nil {
		return removed, err
	}
	for rel, id := range known {
		if _, e := s.AbsolutePath(rel); e != nil {
			return removed, e
		}
		if _, e := os.Stat(filepath.Join(s.root, rel)); errors.Is(e, os.ErrNotExist) {
			if _, e = s.db.ExecContext(ctx, `DELETE FROM attachments WHERE id=?`, id); e != nil {
				return removed, e
			}
			removed++
		} else if e != nil {
			return removed, e
		}
	}
	return removed, nil
}

var _ domain.Store = (*Store)(nil)
