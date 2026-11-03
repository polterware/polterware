package knowledge

import (
	"crypto/sha1"
	"encoding/hex"
	"errors"
	"time"
)

type EntityLink struct {
	ID          string
	ProjectID   string
	KnowledgeID string
	EntityID    string
	Predicate   string
	TruthClass  string
	Source      string
	Actor       string
	ObservedAt  time.Time
	CreatedAt   time.Time
}

type SearchHit struct {
	Item       Item
	Score      float64
	Snippet    string
	TruthClass string
	Source     string
}

var ErrInvalidLink = errors.New("invalid knowledge entity link")

func LinkID(knowledgeID, entityID, predicate string) string {
	sum := sha1.Sum([]byte(knowledgeID + "|" + entityID + "|" + predicate))
	return "klink-" + hex.EncodeToString(sum[:12])
}

func NewEntityLink(projectID, knowledgeID, entityID, predicate, truthClass, source, actor string) (EntityLink, error) {
	if projectID == "" || knowledgeID == "" || entityID == "" || predicate == "" {
		return EntityLink{}, ErrInvalidLink
	}
	if truthClass == "" {
		truthClass = "declared"
	}
	if source == "" {
		source = "polterware"
	}
	if actor == "" {
		actor = "human"
	}
	now := time.Now().UTC()
	return EntityLink{
		ID:          LinkID(knowledgeID, entityID, predicate),
		ProjectID:   projectID,
		KnowledgeID: knowledgeID,
		EntityID:    entityID,
		Predicate:   predicate,
		TruthClass:  truthClass,
		Source:      source,
		Actor:       actor,
		ObservedAt:  now,
		CreatedAt:   now,
	}, nil
}
