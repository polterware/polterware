package sqlite

import (
	"database/sql"
	"time"

	"github.com/polterware/polterware/backend/domains/projects"
)

func (s *Store) ListProjectRepos(projectID string) ([]projects.Repository, error) {
	rows, err := s.DB.Query(`SELECT id, project_id, name, path, created_at, updated_at FROM project_repos WHERE project_id=? ORDER BY name`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []projects.Repository
	for rows.Next() {
		item, err := scanRepository(rows)
		if err != nil {
			return nil, err
		}
		out = append(out, item)
	}
	return out, rows.Err()
}

func (s *Store) SaveProjectRepo(repo projects.Repository) error {
	_, err := s.DB.Exec(`INSERT INTO project_repos(id,project_id,name,path,created_at,updated_at) VALUES(?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET name=excluded.name,path=excluded.path,updated_at=excluded.updated_at`,
		repo.ID, repo.ProjectID, repo.Name, repo.Path, repo.CreatedAt.Format(time.RFC3339), repo.UpdatedAt.Format(time.RFC3339))
	return err
}

func (s *Store) DeleteProjectRepo(projectID, name string) error {
	result, err := s.DB.Exec(`DELETE FROM project_repos WHERE project_id=? AND name=?`, projectID, name)
	if err != nil {
		return err
	}
	n, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if n == 0 {
		return sql.ErrNoRows
	}
	return nil
}

type repoScanner interface {
	Scan(dest ...any) error
}

func scanRepository(row repoScanner) (projects.Repository, error) {
	var item projects.Repository
	var created, updated string
	err := row.Scan(&item.ID, &item.ProjectID, &item.Name, &item.Path, &created, &updated)
	if err != nil {
		return projects.Repository{}, err
	}
	item.CreatedAt, _ = time.Parse(time.RFC3339, created)
	item.UpdatedAt, _ = time.Parse(time.RFC3339, updated)
	return item, nil
}
