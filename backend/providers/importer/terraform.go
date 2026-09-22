package importer

import (
	"path/filepath"
	"regexp"
)

var (
	tfProviderRE = regexp.MustCompile(`(?m)provider\s+"([^"]+)"`)
	tfResourceRE = regexp.MustCompile(`(?m)resource\s+"([^"]+)"\s+"([^"]+)"`)
)

func parseTerraform(rel string, data []byte) FileResult {
	source := ImportSource(rel)
	base := filepath.Base(rel)
	evidence := DraftEvidence{
		Path:    rel,
		Summary: "Terraform configuration " + base,
		Source:  source,
	}
	text := string(data)
	var facts []DraftFact
	seenProviders := map[string]struct{}{}
	for _, match := range tfProviderRE.FindAllStringSubmatch(text, -1) {
		name := match[1]
		if _, ok := seenProviders[name]; ok {
			continue
		}
		seenProviders[name] = struct{}{}
		facts = append(facts, DraftFact{
			Key:       "provider:" + name,
			Statement: "Terraform provider " + name,
			Source:    source,
		})
	}
	seenResources := map[string]struct{}{}
	for _, match := range tfResourceRE.FindAllStringSubmatch(text, -1) {
		rtype, rname := match[1], match[2]
		key := rtype + "." + rname
		if _, ok := seenResources[key]; ok {
			continue
		}
		seenResources[key] = struct{}{}
		facts = append(facts, DraftFact{
			Key:       "resource:" + key,
			Statement: "Terraform resource " + key,
			Source:    source,
		})
	}
	if len(facts) == 0 {
		facts = append(facts, DraftFact{
			Key:       "present",
			Statement: "Project has Terraform file " + rel,
			Source:    source,
		})
	}
	return FileResult{Path: rel, Evidence: evidence, Facts: facts, Title: base}
}
