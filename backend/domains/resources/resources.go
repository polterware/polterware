package resources

import "errors"

type Resource struct {
	ID            string
	ProjectID     string
	EnvironmentID string
	Name          string
	Type          string
	ExternalKey   string
	Source        string
	Provider      string
	Status        string
	Region        string
	Address       string
	Tags          []string
	Description   string
}

type Node struct {
	ID            string
	ProjectID     string
	EnvironmentID string
	Name          string
	Platform      string
	Shell         string
	Status        string
	Address       string
	CPU           string
	Memory        string
	LastSeenAt    string
}

var ErrDuplicateKey = errors.New("external_key not unique for project")

func EnsureUniqueExternalKey(existing []Resource, candidate Resource) error {
	if candidate.ExternalKey == "" {
		return nil
	}
	for _, resource := range existing {
		if resource.ID != candidate.ID && resource.ProjectID == candidate.ProjectID && resource.ExternalKey == candidate.ExternalKey {
			return ErrDuplicateKey
		}
	}
	return nil
}
