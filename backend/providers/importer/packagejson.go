package importer

import (
	"encoding/json"
	"fmt"
)

func parsePackageJSON(rel string, data []byte) FileResult {
	source := ImportSource(rel)
	evidence := DraftEvidence{
		Path:    rel,
		Summary: "package.json package manifest",
		Source:  source,
	}
	var payload struct {
		Name            string            `json:"name"`
		Dependencies    map[string]string `json:"dependencies"`
		DevDependencies map[string]string `json:"devDependencies"`
	}
	if err := json.Unmarshal(data, &payload); err != nil {
		return FileResult{Path: rel, Skipped: true, Reason: "invalid package.json"}
	}
	var facts []DraftFact
	if payload.Name != "" {
		facts = append(facts, DraftFact{
			Key:       "name",
			Statement: "npm package name is " + payload.Name,
			Source:    source,
		})
	}
	for name := range payload.Dependencies {
		facts = append(facts, DraftFact{
			Key:       "dep:" + name,
			Statement: fmt.Sprintf("npm dependency %s", name),
			Source:    source,
		})
	}
	for name := range payload.DevDependencies {
		facts = append(facts, DraftFact{
			Key:       "devDep:" + name,
			Statement: fmt.Sprintf("npm devDependency %s", name),
			Source:    source,
		})
	}
	if len(facts) == 0 {
		facts = append(facts, DraftFact{
			Key:       "present",
			Statement: "Project has package.json",
			Source:    source,
		})
	}
	title := "package.json"
	if payload.Name != "" {
		title = payload.Name
	}
	return FileResult{Path: rel, Evidence: evidence, Facts: facts, Title: title}
}
