package knowledge

import (
	"crypto/rand"
	"encoding/hex"
	"errors"
	"time"
)

func NewItemID() string {
	var buf [12]byte
	_, _ = rand.Read(buf[:])
	return "know-" + hex.EncodeToString(buf[:])
}

type Item struct {
	ID        string
	ProjectID string
	Title     string
	Body      string
	Kind      string
	Tags      []string
	Summary   string
	Owner     string
	Status    string
	ExtraJSON string
	CreatedAt time.Time
	UpdatedAt time.Time
}

const MaxBody = 64 * 1024

var ErrInvalid = errors.New("invalid knowledge item")

func ValidKind(kind string) bool {
	switch kind {
	case "doc", "runbook", "adr", "constitution", "source":
		return true
	default:
		return false
	}
}

func NewItem(id, projectID, title, body, kind string, tags []string) (Item, error) {
	if id == "" || projectID == "" || title == "" || !ValidKind(kind) || len(body) > MaxBody {
		return Item{}, ErrInvalid
	}
	now := time.Now().UTC()
	return Item{ID: id, ProjectID: projectID, Title: title, Body: body, Kind: kind, Tags: tags, ExtraJSON: "{}", CreatedAt: now, UpdatedAt: now}, nil
}
