package localcore

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"errors"
	"fmt"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/domains/integrations"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	cliprobe "github.com/polterware/polterware/backend/providers/cli"
)

var listInventoryFn = cliprobe.ListInventory

func (core *Core) SyncIntegrationConnection(_ context.Context, request *connect.Request[localv1.SyncIntegrationConnectionRequest]) (*connect.Response[localv1.SyncIntegrationConnectionResponse], error) {
	connectionID := request.Msg.GetConnectionId()
	if connectionID == "" {
		return nil, invalid("connection id is required")
	}

	connection, err := core.Store.GetConnection(connectionID)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, connect.NewError(connect.CodeNotFound, fmt.Errorf("connection not found"))
	}
	if err != nil {
		return nil, internal(err)
	}

	startedAt := time.Now().UTC()
	inventory := listInventoryFn(connection.ProviderID)
	finishedAt := time.Now().UTC()

	if inventory.Err != nil {
		run, saveErr := core.recordFailedSync(connection, startedAt, finishedAt, inventory.Message)
		if saveErr != nil {
			return nil, internal(saveErr)
		}
		return connect.NewResponse(&localv1.SyncIntegrationConnectionResponse{Run: syncRunProto(run)}), nil
	}

	run, saveErr := core.replaceInventory(connection, inventory, startedAt, finishedAt)
	if saveErr != nil {
		return nil, internal(saveErr)
	}
	return connect.NewResponse(&localv1.SyncIntegrationConnectionResponse{Run: syncRunProto(run)}), nil
}

func (core *Core) recordFailedSync(connection integrations.Connection, startedAt, finishedAt time.Time, message string) (integrations.SyncRun, error) {
	if message == "" {
		message = "CLI inventory sync failed"
	}
	run := integrations.SyncRun{
		ID:           syncRunID(connection.ID, finishedAt),
		ConnectionID: connection.ID,
		StartedAt:    startedAt.Format(time.RFC3339),
		FinishedAt:   finishedAt.Format(time.RFC3339),
		Result:       "failed",
		Imported:     0,
		Changed:      0,
		Message:      message,
	}
	if err := core.Store.SaveIntegrationSyncRun(run); err != nil {
		return integrations.SyncRun{}, err
	}
	connection.Freshness = "stale"
	connection.ErrorMessage = message
	connection.LastSyncAt = finishedAt.Format(time.RFC3339)
	if err := core.Store.SaveConnection(connection); err != nil {
		return integrations.SyncRun{}, err
	}
	return run, nil
}

func (core *Core) replaceInventory(connection integrations.Connection, inventory cliprobe.InventoryResult, startedAt, finishedAt time.Time) (integrations.SyncRun, error) {
	oldCount, err := core.Store.CountIntegrationResourcesByConnection(connection.ID)
	if err != nil {
		return integrations.SyncRun{}, err
	}
	if err := core.Store.DeleteIntegrationResourcesByConnection(connection.ID); err != nil {
		return integrations.SyncRun{}, err
	}
	for _, item := range inventory.Items {
		resource := integrations.Resource{
			ID:           integrationResourceID(connection.ID, item.ExternalID),
			ConnectionID: connection.ID,
			Name:         item.Name,
			Kind:         item.Kind,
			ExternalID:   item.ExternalID,
			EnvironmentID: connection.EnvironmentID,
			Status:       "synced",
		}
		if err := core.Store.SaveIntegrationResource(resource); err != nil {
			return integrations.SyncRun{}, err
		}
	}

	imported := len(inventory.Items)
	changed := absInt(imported - oldCount)
	message := inventory.Message
	if message == "" {
		message = fmt.Sprintf("Imported %d items from %s.", imported, connection.ProviderID)
	}
	run := integrations.SyncRun{
		ID:           syncRunID(connection.ID, finishedAt),
		ConnectionID: connection.ID,
		StartedAt:    startedAt.Format(time.RFC3339),
		FinishedAt:   finishedAt.Format(time.RFC3339),
		Result:       "success",
		Imported:     imported,
		Changed:      changed,
		Message:      message,
	}
	if err := core.Store.SaveIntegrationSyncRun(run); err != nil {
		return integrations.SyncRun{}, err
	}

	connection.LastSyncAt = finishedAt.Format(time.RFC3339)
	connection.Freshness = "fresh"
	connection.ErrorMessage = ""
	connection.Summary = message
	if err := core.Store.SaveConnection(connection); err != nil {
		return integrations.SyncRun{}, err
	}
	return run, nil
}

func integrationResourceID(connectionID, externalID string) string {
	sum := sha256.Sum256([]byte(connectionID + "\x00" + externalID))
	return "ires-" + connectionID + "-" + hex.EncodeToString(sum[:8])
}

func syncRunID(connectionID string, at time.Time) string {
	return fmt.Sprintf("sync-%s-%d", connectionID, at.UnixNano())
}

func absInt(value int) int {
	if value < 0 {
		return -value
	}
	return value
}
