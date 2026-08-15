package workflows

import "errors"

type Step struct {
	ID            string
	Name          string
	OperationType string
	Params        map[string]string
}

type Workflow struct {
	ID        string
	ProjectID string
	Name      string
	Status    string
	Steps     []Step
}

var ErrInvalid = errors.New("invalid workflow")

func Validate(workflow Workflow) error {
	if workflow.ID == "" || workflow.ProjectID == "" || workflow.Name == "" {
		return ErrInvalid
	}
	for _, step := range workflow.Steps {
		if step.ID == "" || step.Name == "" || step.OperationType == "" {
			return ErrInvalid
		}
	}
	return nil
}
