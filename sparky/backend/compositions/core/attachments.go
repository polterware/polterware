package core

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/attachments"
)

type attachmentMeta struct{ store attachments.Store }

func (a attachmentMeta) List(ctx context.Context, memoryID string) ([]attachments.Attachment, error) {
	return a.store.List(ctx, memoryID)
}

func (a attachmentMeta) Delete(ctx context.Context, id string) error {
	return a.store.Delete(ctx, id)
}

func (a attachmentMeta) Reorder(ctx context.Context, memoryID string, orderedIDs []string) ([]attachments.Attachment, error) {
	if err := a.store.Reorder(ctx, memoryID, orderedIDs); err != nil {
		return nil, err
	}
	return a.store.List(ctx, memoryID)
}

func (attachmentMeta) ImportFromPath(context.Context, string, string, string) (attachments.Attachment, error) {
	return attachments.Attachment{}, unavailable("attachment files")
}

func (attachmentMeta) AddLink(context.Context, string, string, string) (attachments.Attachment, error) {
	return attachments.Attachment{}, unavailable("attachment files")
}

func (attachmentMeta) OpenInfo(context.Context, string) (attachments.OpenInfo, error) {
	return attachments.OpenInfo{}, unavailable("attachment files")
}
