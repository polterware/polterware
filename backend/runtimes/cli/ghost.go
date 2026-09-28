package main

import (
	"strings"

	"github.com/charmbracelet/x/ansi"
)

// ghostBody is the Polterware ghost below its antenna; spaces are the eyes.
var ghostBody = []string{
	"  ▄█████▄",
	" ████ █ ██",
	" ████▄█▄██",
	" █▀█▀█▀█▀█",
}

const (
	ghostWidth  = 12
	ghostHeight = 6
)

func renderGhost() []string {
	lines := []string{
		"     " + styleSuccess.Render("●"),
		"     " + styleMuted.Render("│"),
	}
	for _, row := range ghostBody {
		lines = append(lines, styleText.Render(row))
	}
	for i, line := range lines {
		lines[i] = line + strings.Repeat(" ", max(ghostWidth-ansi.StringWidth(line), 0))
	}
	return lines
}

func (m *tuiModel) showGhost() bool {
	return m.height >= 30 && m.width >= 60
}
