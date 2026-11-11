-- A domain organizes components; only a component has a checkout. Domains
-- drop their repo and folder, and a repository no node or task uses any more
-- goes, as when the last component unlinks it.

UPDATE structure_nodes SET repo = '', folder = '' WHERE level = 'domain';

DELETE FROM repositories
  WHERE NOT EXISTS (SELECT 1 FROM structure_nodes n WHERE n.project_id = repositories.project_id AND n.repo = repositories.name AND n.status <> 'removed')
    AND NOT EXISTS (SELECT 1 FROM work_tasks t JOIN work_cards c ON c.id = t.card_id WHERE c.project_id = repositories.project_id AND t.repo = repositories.name);
