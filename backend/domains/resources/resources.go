package resources

import (
	"errors"
	"strings"
)

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
	ID              string
	ProjectID       string
	EnvironmentID   string
	Name            string
	Platform        string
	Shell           string
	Status          string
	Address         string
	CPU             string
	Memory          string
	LastSeenAt      string
	ConnectionKind  string
	SSHUser         string
	SSHPort         int
	SSHIdentityFile string
}

var (
	ErrDuplicateKey   = errors.New("external_key not unique for project")
	ErrInvalidSSHNode = errors.New("invalid ssh host profile")
	ErrSSHKeyMaterial = errors.New("ssh identity must be a file path, not key material")
)

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

// NormalizeNode fills defaults for connection fields.
func NormalizeNode(node *Node) {
	if node.Shell == "" {
		node.Shell = "bash"
	}
	if node.Status == "" {
		node.Status = "unknown"
	}
	if node.ConnectionKind == "" {
		node.ConnectionKind = "local"
	}
	if node.SSHPort == 0 {
		node.SSHPort = 22
	}
}

// ValidateNode checks SSH host profiles. Local nodes only require id/project/name at the RPC layer.
func ValidateNode(node Node) error {
	NormalizeNode(&node)
	if node.ConnectionKind != "local" && node.ConnectionKind != "ssh" {
		return ErrInvalidSSHNode
	}
	if node.ConnectionKind != "ssh" {
		return nil
	}
	if strings.TrimSpace(node.Name) == "" || strings.TrimSpace(node.Address) == "" || strings.TrimSpace(node.SSHUser) == "" {
		return ErrInvalidSSHNode
	}
	if node.SSHPort < 1 || node.SSHPort > 65535 {
		return ErrInvalidSSHNode
	}
	if looksLikeKeyMaterial(node.SSHIdentityFile) {
		return ErrSSHKeyMaterial
	}
	return nil
}

func looksLikeKeyMaterial(value string) bool {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return false
	}
	lower := strings.ToLower(trimmed)
	if strings.Contains(lower, "begin ") && strings.Contains(lower, "private key") {
		return true
	}
	if strings.Contains(trimmed, "\n") || strings.Contains(trimmed, "\r") {
		return true
	}
	return false
}
