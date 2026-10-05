// Package mind is the mind-tree domain.
package mind

import "time"

// Mind is one node. A nil ParentID is a root.
type Mind struct {
	ID        string
	ParentID  *string
	Name      string
	Color     *string
	Icon      *string
	SortOrder int
	CreatedAt time.Time
	UpdatedAt time.Time
	Rev       int64
}

// Patch sets the fields that are non-nil. Clear flags write NULL.
type Patch struct {
	Name        *string
	ParentID    *string
	ClearParent bool
	Color       *string
	ClearColor  bool
	Icon        *string
	ClearIcon   bool
	SortOrder   *int
}

// Create is the input for a new mind.
type Create struct {
	ParentID  *string
	Name      string
	Color     *string
	Icon      *string
	SortOrder int
}

// DeleteResult is the effect of deleting a mind.
// Descendants are deleted. Memories that pointed at them move to Inbox.
type DeleteResult struct {
	DeletedMindIDs   []string
	MovedMemoryCount int
}
