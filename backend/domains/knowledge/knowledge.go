package knowledge

import (
	"errors"
	"time"
)

type Item struct {
	ID        string
	ProjectID string
	Title     string
	Body      string
	Kind      string
	Tags      []string
	CreatedAt time.Time
	UpdatedAt time.Time
}

const MaxBody = 64 * 1024

var ErrInvalid = errors.New("invalid knowledge item")

func NewItem(id, projectID, title, body, kind string, tags []string) (Item, error) {
	if id == "" || projectID == "" || title == "" || (kind != "doc" && kind != "runbook") || len(body) > MaxBody {
		return Item{}, ErrInvalid
	}
	now := time.Now().UTC()
	return Item{ID: id, ProjectID: projectID, Title: title, Body: body, Kind: kind, Tags: tags, CreatedAt: now, UpdatedAt: now}, nil
}
