package seed

import (
	"time"

	"github.com/polterware/polterware/backend/domains/operations"
	"github.com/polterware/polterware/backend/domains/workflows"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func seedOperations(store *storepkg.Store) error {
	workflowsToSave := []workflows.Workflow{
		{
			ID: "workflow-deploy", ProjectID: ProjectID, Name: "Deploy service", Status: "ready",
			Description: "Validate, build, deploy, and check one service.", Trigger: "manual",
			LastRunAt: "2026-09-18T00:00:00Z",
			Steps: []workflows.Step{
				{ID: "validate", Name: "Validate source", OperationType: "validate"},
				{ID: "build", Name: "Build image", OperationType: "build", Params: map[string]string{"command": "docker build ."}},
				{ID: "deploy", Name: "Deploy service", OperationType: "deploy"},
				{ID: "health", Name: "Check health", OperationType: "health_check"},
			},
		},
		{
			ID: "workflow-backup", ProjectID: ProjectID, Name: "Backup database", Status: "ready",
			Description: "Check the database and create a backup preview.", Trigger: "schedule",
			LastRunAt: "2026-09-17T03:00:00Z",
			Steps: []workflows.Step{
				{ID: "validate", Name: "Validate database", OperationType: "validate"},
				{ID: "backup", Name: "Create backup", OperationType: "command", Params: map[string]string{"command": "pg_dump --format=custom example"}},
				{ID: "notify", Name: "Record result", OperationType: "notify"},
			},
		},
	}
	for _, workflow := range workflowsToSave {
		if err := store.SaveWorkflow(workflow); err != nil {
			return err
		}
	}

	startSuccess := time.Date(2026, 9, 18, 14, 42, 0, 0, time.UTC)
	finishSuccess := startSuccess.Add(18 * time.Second)
	startRunning := time.Date(2026, 9, 18, 14, 50, 0, 0, time.UTC)
	startFailed := time.Date(2026, 9, 18, 13, 18, 0, 0, time.UTC)
	finishFailed := startFailed.Add(42 * time.Second)

	runs := []*operations.Operation{
		{
			ID: "run-restart-api", CorrelationID: "corr-restart-api", ProjectID: ProjectID, EnvironmentID: EnvProductionID,
			Type: "command", Status: operations.StatusSucceeded, Title: "Restart API service", Target: "api-service",
			Command: "systemctl restart example-api", ResultSummary: "Completed without changing remote systems.",
			RequestedAt: startSuccess, StartedAt: &startSuccess, FinishedAt: &finishSuccess,
			StepsJSON: `[{"id":"step-validate","name":"Validate target","status":"success","duration":"2s"},{"id":"step-restart","name":"Restart service","status":"success","duration":"12s"},{"id":"step-health","name":"Check health","status":"success","duration":"4s"}]`,
		},
		{
			ID: "run-sync-resources", CorrelationID: "corr-sync-resources", ProjectID: ProjectID, EnvironmentID: EnvDevelopmentID,
			Type: "refresh", Status: operations.StatusRunning, Title: "Refresh resource inventory", Target: "local-project",
			Command: "polter resources refresh", ResultSummary: "Inventory refresh is still running.",
			RequestedAt: startRunning, StartedAt: &startRunning,
			StepsJSON: `[{"id":"step-read","name":"Read registry","status":"success","duration":"1s"},{"id":"step-merge","name":"Merge resource metadata","status":"running"}]`,
		},
		{
			ID: "run-backup", CorrelationID: "corr-backup", ProjectID: ProjectID, EnvironmentID: EnvProductionID,
			Type: "backup", Status: operations.StatusFailed, Title: "Database backup preview", Target: "prod-postgres",
			Command: "pg_dump --format=custom example", ResultSummary: "Failed without changing any local or remote data.",
			ErrorMessage: "Backup destination unavailable.", RequestedAt: startFailed, StartedAt: &startFailed, FinishedAt: &finishFailed,
			StepsJSON: `[{"id":"step-connect","name":"Check database","status":"success","duration":"2s"},{"id":"step-backup","name":"Create backup preview","status":"failed","duration":"40s"}]`,
		},
	}
	for _, run := range runs {
		if err := store.SaveOperation(run); err != nil {
			return err
		}
	}
	logs := map[string][]string{
		"run-restart-api":    {"Recorded run only.", "Service restart preview completed.", "Health check is healthy."},
		"run-sync-resources": {"Recorded run only.", "Refreshing resource metadata…"},
		"run-backup":         {"Recorded run only.", "Backup destination unavailable."},
	}
	for operationID, lines := range logs {
		for _, line := range lines {
			if _, err := store.AppendLog(operationID, line, false); err != nil {
				return err
			}
		}
	}

	schedules := []operations.Schedule{
		{
			ID: "schedule-backup", ProjectID: ProjectID, EnvironmentID: EnvProductionID,
			Name: "Nightly database backup", Description: "Starts the backup workflow on the local schedule.",
			Status: "active", WorkflowID: "workflow-backup", Expression: "0 3 * * *", Timezone: "UTC",
			NextRunAt: "2026-09-19T03:00:00Z", LastRunAt: "2026-09-18T03:00:00Z",
		},
		{
			ID: "schedule-refresh", ProjectID: ProjectID, EnvironmentID: EnvDevelopmentID,
			Name: "Resource metadata refresh", Description: "Refreshes the inventory on a paused schedule.",
			Status: "paused", WorkflowID: "workflow-deploy", Expression: "*/30 * * * *", Timezone: "UTC",
			LastRunAt: "2026-09-17T23:32:00Z",
		},
	}
	for _, schedule := range schedules {
		if err := store.SaveSchedule(schedule); err != nil {
			return err
		}
	}
	return nil
}
