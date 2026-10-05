package attachments

import "context"

// Store persists attachment metadata and files under the data directory.
type Store interface {
	List(ctx context.Context, memoryID string) ([]Attachment, error)
	Get(ctx context.Context, id string) (Attachment, error)
	Insert(ctx context.Context, a Attachment) error
	Delete(ctx context.Context, id string) error
	Reorder(ctx context.Context, memoryID string, orderedIDs []string) error
	ImportFile(ctx context.Context, memoryID, attachmentID, sourcePath string) (ImportedFile, error)
	AbsolutePath(relPath string) (string, error)
	RemoveFile(ctx context.Context, relPath string) error
}

// Service is the attachment use-case port.
// expectedRev 0 skips the memory revision check. Non-zero is checked in the
// same transaction as the metadata write.
type Service interface {
	ImportFromPath(ctx context.Context, memoryID, sourcePath, name string, expectedRev int64) (Attachment, error)
	AddLink(ctx context.Context, memoryID, url, title string, expectedRev int64) (Attachment, error)
	List(ctx context.Context, memoryID string) ([]Attachment, error)
	Delete(ctx context.Context, id string) error
	Reorder(ctx context.Context, memoryID string, orderedIDs []string) ([]Attachment, error)
	OpenInfo(ctx context.Context, id string) (OpenInfo, error)
}
