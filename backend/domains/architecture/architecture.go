package architecture

import "errors"

type Layout struct {
	ID        string
	ProjectID string
	Revision  int64
	BodyJSON  string
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
