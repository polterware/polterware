package localcore

import (
	"context"
	"runtime"
	"strconv"
	"strings"

	"github.com/polterware/polterware/backend/domains/operations"
	processpkg "github.com/polterware/polterware/backend/providers/process"
)

func (core *Core) processManager() *processpkg.Manager {
	core.runnerOnce.Do(func() {
		core.Runner = processpkg.NewManager()
	})
	return core.Runner
}

func (core *Core) enqueueOperationExecution(operationID string) {
	go core.executeOperation(operationID)
}

func (core *Core) executeOperation(operationID string) {
	item, err := core.Store.LoadOperation(operationID)
	if err != nil || item.Status != operations.StatusQueued {
		return
	}
	if err := item.Transition(operations.StatusRunning); err != nil {
		return
	}
	if err := core.Store.SaveOperation(item); err != nil {
		return
	}
	_, _ = core.Store.AppendLog(item.ID, "Operation started on local host.", false)

	command, args, cwd := resolveOperationCommand(item)
	if command == "" {
		failOperation(core, item, "no command to execute")
		return
	}

	ctx, cancel := context.WithCancel(context.Background())
	manager := core.processManager()
	manager.RegisterCancel(item.ID, cancel)
	defer manager.ClearCancel(item.ID)

	_, _ = core.Store.AppendLog(item.ID, "$ "+shellPreview(command, args), false)
	result := processpkg.Run(ctx, command, args, cwd)

	latest, err := core.Store.LoadOperation(item.ID)
	if err != nil {
		return
	}
	if latest.IsTerminal() {
		return
	}

	if strings.TrimSpace(result.Stdout) != "" {
		_, _ = core.Store.AppendLog(item.ID, truncateLog(result.Stdout), false)
	}
	if strings.TrimSpace(result.Stderr) != "" {
		_, _ = core.Store.AppendLog(item.ID, truncateLog(result.Stderr), false)
	}

	if ctx.Err() != nil {
		_ = latest.Transition(operations.StatusCancelled)
		latest.ResultSummary = "cancelled"
		latest.ErrorMessage = "cancelled"
		_ = core.Store.SaveOperation(latest)
		_, _ = core.Store.AppendLog(item.ID, "Operation cancelled.", false)
		return
	}

	if result.Err != nil || result.ExitCode != 0 {
		_ = latest.Transition(operations.StatusFailed)
		latest.ResultSummary = "failed"
		if result.Err != nil {
			latest.ErrorMessage = result.Err.Error()
		} else {
			latest.ErrorMessage = "exit code " + strconv.Itoa(result.ExitCode)
		}
		_ = core.Store.SaveOperation(latest)
		_, _ = core.Store.AppendLog(item.ID, "Operation failed: "+latest.ErrorMessage, false)
		return
	}

	_ = latest.Transition(operations.StatusSucceeded)
	latest.ResultSummary = "succeeded"
	_ = core.Store.SaveOperation(latest)
	_, _ = core.Store.AppendLog(item.ID, "Operation succeeded.", false)
}

func failOperation(core *Core, item *operations.Operation, message string) {
	_ = item.Transition(operations.StatusFailed)
	item.ErrorMessage = message
	item.ResultSummary = "failed"
	_ = core.Store.SaveOperation(item)
	_, _ = core.Store.AppendLog(item.ID, message, false)
}

func resolveOperationCommand(item *operations.Operation) (name string, args []string, cwd string) {
	if item.Params != nil {
		cwd = item.Params["cwd"]
	}
	command := strings.TrimSpace(item.Command)
	if command == "" && item.Params != nil {
		command = strings.TrimSpace(item.Params["command"])
	}
	if command == "" {
		return "", nil, cwd
	}
	if runtime.GOOS == "windows" {
		return "cmd", []string{"/C", command}, cwd
	}
	return "sh", []string{"-c", command}, cwd
}

func shellPreview(name string, args []string) string {
	if len(args) == 2 && (args[0] == "-c" || args[0] == "/C") {
		return args[1]
	}
	return strings.TrimSpace(name + " " + strings.Join(args, " "))
}

func truncateLog(body string) string {
	const max = 4000
	if len(body) <= max {
		return body
	}
	return body[:max] + "…"
}
