package projects

import (
	"errors"
	"time"
)

const (
	StatusActive   = "active"
	StatusArchived = "archived"
)

type Project struct {
	ID         string
	Name       string
	RootPath   string
	CreatedAt  time.Time
	UpdatedAt  time.Time
	Status     string
	ArchivedAt time.Time
}

type Environment struct {
	ID          string
	ProjectID   string
	Name        string
	Kind        string
	Protection  string
	IsDefault   bool
	Description string
}

type Settings struct {
	ProjectID            string
	Description          string
	DefaultEnvironmentID string
	Repository           string
}

var ErrInvalid = errors.New("invalid project entity")

func NewProject(id, name, rootPath string) (Project, error) {
	if id == "" || name == "" || rootPath == "" {
		return Project{}, ErrInvalid
	}
	now := time.Now().UTC()
	return Project{
		ID:        id,
		Name:      name,
		RootPath:  rootPath,
		CreatedAt: now,
		UpdatedAt: now,
		Status:    StatusActive,
	}, nil
}

func NewEnvironment(id, projectID, name, kind string) (Environment, error) {
	if id == "" || projectID == "" || name == "" {
		return Environment{}, ErrInvalid
	}
	if kind == "" {
		kind = "local"
	}
	return Environment{ID: id, ProjectID: projectID, Name: name, Kind: kind, Protection: "standard"}, nil
}
