package main

import (
	"fmt"
	"os"
	"os/exec"
	"syscall"
)

// movedCommands are the delegation commands Polterware Dispatch took over;
// the value is dispatch's name for each.
var movedCommands = map[string]string{
	"run":       "run",
	"runs":      "runs",
	"wait":      "wait",
	"result":    "result",
	"cancel":    "cancel",
	"route":     "route",
	"classify":  "classify",
	"models":    "models",
	"merge":     "merge",
	"clean":     "clean",
	"recover":   "recover",
	"subagents": "subagents",
	"tiers":     "tiers",
	"usage":     "usage",
	"agents":    "agents",
	"doctor":    "doctor",
	"config":    "config",
	"skill":     "skill",
	"update":    "update",
}

// legacyDispatch maps `polterware dispatch <sub>` to dispatch's command.
var legacyDispatch = map[string]string{"ls": "runs", "stats": "usage"}

// runMoved hands a delegation command to dispatch, the Polterware Dispatch
// binary, when it is on PATH, and otherwise says where the command went.
func runMoved(command string, args []string) error {
	bin, err := exec.LookPath("dispatch")
	if err != nil {
		return fmt.Errorf("moved to Dispatch: dispatch %s (install Dispatch to get it)", command)
	}
	return syscall.Exec(bin, append([]string{"dispatch", command}, args...), os.Environ())
}

// runLegacyDispatch handles `polterware dispatch <sub>`. The SessionStart hook
// Polterware registered (`dispatch hook`) stays silent without dispatch, so
// a stale hook is harmless.
func runLegacyDispatch(args []string) error {
	if len(args) == 0 {
		return runMoved("help", nil)
	}
	sub := args[0]
	if name, ok := legacyDispatch[sub]; ok {
		sub = name
	}
	if sub == "hook" {
		if _, err := exec.LookPath("dispatch"); err != nil {
			return nil
		}
	}
	return runMoved(sub, args[1:])
}
