package fake

import "github.com/polterware/polterware/sparky/backend/domains/ids"

// Generator is a repository-port skeleton. Methods panic until W1 replaces them.
type Generator struct{}

var _ ids.Generator = Generator{}

func (Generator) New() string { panic("ids/fake: not implemented") }
