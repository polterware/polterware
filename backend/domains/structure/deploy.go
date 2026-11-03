package structure

// Deployment is one environment link for a Structure node. Provider is a catalog
// id and Resource names the remote target. Credentials are never stored.
type Deployment struct {
	ID            string `json:"id"`
	EnvironmentID string `json:"environment_id"`
	// Environment is the environment name, for display. The table stores the id.
	Environment string `json:"environment,omitempty"`
	Provider    string `json:"provider"`
	Resource    string `json:"resource"`
	// TargetID is the platform's stable id for the resource, empty until linked.
	TargetID string `json:"target_id,omitempty"`
	// URL is the resource's page on the platform's dashboard, filled by the core.
	URL string `json:"url,omitempty"`
	// Check is the latest deploy the platform's CLI reported; nil when never checked.
	Check *DeployCheck `json:"-"`
}

// DeployCheck is the latest deploy of a link as its platform reported it.
// State is ok, building, failed, stopped, or unknown.
type DeployCheck struct {
	State     string
	Summary   string
	URL       string
	CheckedAt string
}

// DeploymentsFrom reads a deploys field from a planned op or a replayed
// change_ops JSON value. The bool is false when value is not a deployment list.
func DeploymentsFrom(value any) ([]Deployment, bool) {
	switch v := value.(type) {
	case nil:
		return []Deployment{}, true
	case []Deployment:
		return v, true
	case []any:
		out := make([]Deployment, 0, len(v))
		for _, item := range v {
			fields, ok := item.(map[string]any)
			if !ok {
				return nil, false
			}
			out = append(out, Deployment{
				ID:            fieldString(fields, "id"),
				EnvironmentID: fieldString(fields, "environment_id"),
				Environment:   fieldString(fields, "environment"),
				Provider:      fieldString(fields, "provider"),
				Resource:      fieldString(fields, "resource"),
				TargetID:      fieldString(fields, "target_id"),
			})
		}
		return out, true
	default:
		return nil, false
	}
}

func fieldString(fields map[string]any, key string) string {
	text, _ := fields[key].(string)
	return text
}
