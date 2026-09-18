package seed

import storepkg "github.com/polterware/polterware/backend/providers/sqlite"

func seedProcesses(store *storepkg.Store) error {
	sessions := []storepkg.ProcessSession{
		{ID: "proc-api", ProjectID: ProjectID, EnvironmentID: EnvDevelopmentID, Name: "example-api", NodeID: "node-local", Status: "running", PID: 4122, Command: "./example-api serve", StartedAt: "2026-09-17T22:00:00Z"},
		{ID: "proc-postgres", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "postgres", NodeID: "node-nitro", Status: "running", PID: 1012, Command: "postgres -D /var/lib/postgresql/data", StartedAt: "2026-09-14T00:00:00Z"},
		{ID: "proc-worker", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "settlement-worker", NodeID: "node-worker-01", Status: "running", PID: 2280, Command: "./settlement-worker", StartedAt: "2026-09-17T18:00:00Z"},
		{ID: "proc-cache", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "redis", NodeID: "node-prod-vps", Status: "failed", Command: "redis-server /etc/redis.conf", StartedAt: "2026-09-17T23:42:00Z"},
	}
	for _, session := range sessions {
		if err := store.SaveProcess(session); err != nil {
			return err
		}
	}
	return nil
}
