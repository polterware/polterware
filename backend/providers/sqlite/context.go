package sqlite

import (
	"strings"

	contextpkg "github.com/polterware/polterware/backend/domains/context"
	"github.com/polterware/polterware/backend/domains/ppm"
)

func (s *Store) CompileContext(req contextpkg.Request) (contextpkg.Package, error) {
	candidates := make([]contextpkg.Item, 0, 32)

	if req.EntityID != "" {
		entity, err := s.GetPpmEntity(req.EntityID)
		if err == nil {
			candidates = append(candidates, contextpkg.Item{
				Kind: "entity", ID: entity.ID, Title: entity.Title,
				Excerpt: entity.Kind + " · " + entity.RefKind + "/" + entity.RefID,
				TruthClass: entity.TruthClass, Source: entity.Source, Path: ppm.FormatPath(entity),
			})
			links, err := s.ListKnowledgeLinks(req.ProjectID, "", entity.ID)
			if err != nil {
				return contextpkg.Package{}, err
			}
			for _, link := range links {
				item, err := s.getKnowledgeByID(link.KnowledgeID)
				if err != nil {
					continue
				}
				candidates = append(candidates, contextpkg.Item{
					Kind: item.Kind, ID: item.ID, Title: item.Title,
					Excerpt: contextpkg.Excerpt(firstNonEmpty(item.Summary, item.Body), 280),
					TruthClass: link.TruthClass, Source: link.Source,
					Path: "knowledge/" + item.Kind + "/" + item.ID,
				})
			}
			rels, err := s.ListPpmRelationships(req.ProjectID, entity.ID)
			if err != nil {
				return contextpkg.Package{}, err
			}
			for _, rel := range rels {
				otherID := rel.ObjectEntityID
				if otherID == entity.ID {
					otherID = rel.SubjectEntityID
				}
				other, err := s.GetPpmEntity(otherID)
				if err != nil {
					continue
				}
				candidates = append(candidates, contextpkg.Item{
					Kind: "entity", ID: other.ID, Title: other.Title,
					Excerpt: rel.Predicate + " · " + other.Kind,
					TruthClass: rel.TruthClass, Source: rel.Source, Path: ppm.FormatPath(other),
				})
			}
		}
	}

	if strings.TrimSpace(req.Query) != "" {
		hits, err := s.SearchKnowledge(req.ProjectID, req.Query, "", 12)
		if err != nil {
			return contextpkg.Package{}, err
		}
		for _, hit := range hits {
			candidates = append(candidates, contextpkg.Item{
				Kind: hit.Item.Kind, ID: hit.Item.ID, Title: hit.Item.Title,
				Excerpt: contextpkg.Excerpt(firstNonEmpty(hit.Snippet, hit.Item.Summary, hit.Item.Body), 280),
				TruthClass: hit.TruthClass, Source: hit.Source,
				Path: "knowledge/" + hit.Item.Kind + "/" + hit.Item.ID,
			})
		}
		entities, err := s.ListPpmEntities(req.ProjectID, "")
		if err != nil {
			return contextpkg.Package{}, err
		}
		needle := strings.ToLower(strings.TrimSpace(req.Query))
		for _, entity := range entities {
			if !strings.Contains(strings.ToLower(entity.Title), needle) &&
				!strings.Contains(strings.ToLower(entity.RefID), needle) {
				continue
			}
			candidates = append(candidates, contextpkg.Item{
				Kind: "entity", ID: entity.ID, Title: entity.Title,
				Excerpt: entity.Kind + " · " + entity.RefID,
				TruthClass: entity.TruthClass, Source: entity.Source, Path: ppm.FormatPath(entity),
			})
		}
	}

	if len(candidates) == 0 && req.EntityID == "" && strings.TrimSpace(req.Query) == "" {
		entities, err := s.ListPpmEntities(req.ProjectID, "")
		if err != nil {
			return contextpkg.Package{}, err
		}
		for i, entity := range entities {
			if i >= 8 {
				break
			}
			candidates = append(candidates, contextpkg.Item{
				Kind: "entity", ID: entity.ID, Title: entity.Title,
				Excerpt: entity.Kind + " · " + entity.RefID,
				TruthClass: entity.TruthClass, Source: entity.Source, Path: ppm.FormatPath(entity),
			})
		}
	}

	return contextpkg.Compile(req, dedupeContextItems(candidates)), nil
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return value
		}
	}
	return ""
}

func dedupeContextItems(items []contextpkg.Item) []contextpkg.Item {
	seen := map[string]struct{}{}
	out := make([]contextpkg.Item, 0, len(items))
	for _, item := range items {
		key := item.Kind + ":" + item.ID
		if _, ok := seen[key]; ok {
			continue
		}
		seen[key] = struct{}{}
		out = append(out, item)
	}
	return out
}
