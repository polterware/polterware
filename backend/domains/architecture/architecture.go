package architecture

import "errors"

type Layout struct {
	ID        string
	ProjectID string
	Revision  int64
	BodyJSON  string
}

type ViewMember struct {
	NodeID string
	Name   string
	Type   string
	Layer  string
}

type View struct {
	ID            string
	ProjectID     string
	Name          string
	Description   string
	Owner         string
	Purpose       string
	Notes         string
	FilterSummary string
	Environments  []string
	Tags          []string
	UpdatedAt     string
	Members       []ViewMember
}

var ErrConflict = errors.New("layout revision conflict")

func UpdateLayout(current Layout, expectedRevision int64, body string) (Layout, error) {
	if current.Revision != expectedRevision {
		return current, ErrConflict
	}
	current.Revision++
	current.BodyJSON = body
	return current, nil
}
