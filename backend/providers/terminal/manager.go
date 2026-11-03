package terminal

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/creack/pty"
)

type Session struct {
	ID        string
	ProjectID string
	TargetID  string
	Status    string
	Shell     string
	Cwd       string
	StartedAt time.Time
	Cols      uint16
	Rows      uint16

	cmd    *exec.Cmd
	pty    *os.File
	mu     sync.Mutex
	buffer bytes.Buffer
	closed bool
}

type Manager struct {
	mu       sync.Mutex
	sessions map[string]*Session
}

// SSHProfile is the metadata needed to spawn system ssh inside a PTY.
type SSHProfile struct {
	User         string
	Host         string
	Port         int
	IdentityFile string
}

func NewManager() *Manager {
	return &Manager{sessions: map[string]*Session{}}
}

func defaultShell() string {
	if runtime.GOOS == "windows" {
		return "powershell.exe"
	}
	if shell := os.Getenv("SHELL"); shell != "" {
		return shell
	}
	return "/bin/zsh"
}

func sshBinary() string {
	if runtime.GOOS == "windows" {
		return "ssh.exe"
	}
	return "ssh"
}

// BuildSSHCommand returns argv for an interactive SSH session.
func BuildSSHCommand(profile SSHProfile) (name string, args []string, err error) {
	user := strings.TrimSpace(profile.User)
	host := strings.TrimSpace(profile.Host)
	if user == "" || host == "" {
		return "", nil, fmt.Errorf("ssh user and host are required")
	}
	port := profile.Port
	if port == 0 {
		port = 22
	}
	if port < 1 || port > 65535 {
		return "", nil, fmt.Errorf("ssh port must be between 1 and 65535")
	}
	bin := sshBinary()
	if path, lookErr := exec.LookPath(bin); lookErr != nil {
		return "", nil, fmt.Errorf("ssh binary not found (%s): install OpenSSH client", bin)
	} else {
		bin = path
	}
	args = []string{"-tt", "-p", fmt.Sprintf("%d", port)}
	if identity := strings.TrimSpace(profile.IdentityFile); identity != "" {
		args = append(args, "-i", identity)
	}
	args = append(args, fmt.Sprintf("%s@%s", user, host))
	return bin, args, nil
}

func (m *Manager) Create(id, projectID, targetID, shell, cwd string, cols, rows uint16) (*Session, error) {
	if shell == "" {
		shell = defaultShell()
	}
	return m.createCmd(id, projectID, targetID, shell, cwd, cols, rows, exec.Command(shell))
}

// CreateSSH opens an interactive remote session via system ssh inside a PTY.
func (m *Manager) CreateSSH(id, projectID, targetID, cwd string, cols, rows uint16, profile SSHProfile) (*Session, error) {
	name, args, err := BuildSSHCommand(profile)
	if err != nil {
		return nil, err
	}
	return m.createCmd(id, projectID, targetID, "ssh", cwd, cols, rows, exec.Command(name, args...))
}

func (m *Manager) createCmd(id, projectID, targetID, shellLabel, cwd string, cols, rows uint16, cmd *exec.Cmd) (*Session, error) {
	if cols == 0 {
		cols = 120
	}
	if rows == 0 {
		rows = 32
	}
	if cwd != "" {
		cmd.Dir = cwd
	}
	cmd.Env = os.Environ()
	file, err := pty.StartWithSize(cmd, &pty.Winsize{Cols: cols, Rows: rows})
	if err != nil {
		return nil, err
	}
	session := &Session{
		ID:        id,
		ProjectID: projectID,
		TargetID:  targetID,
		Status:    "active",
		Shell:     shellLabel,
		Cwd:       cwd,
		StartedAt: time.Now().UTC(),
		Cols:      cols,
		Rows:      rows,
		cmd:       cmd,
		pty:       file,
	}
	go session.readLoop()
	m.mu.Lock()
	m.sessions[id] = session
	m.mu.Unlock()
	return session, nil
}

func (s *Session) readLoop() {
	buf := make([]byte, 4096)
	for {
		n, err := s.pty.Read(buf)
		if n > 0 {
			s.mu.Lock()
			_, _ = s.buffer.Write(buf[:n])
			s.mu.Unlock()
		}
		if err != nil {
			s.mu.Lock()
			s.closed = true
			s.Status = "closed"
			s.mu.Unlock()
			return
		}
	}
}

func (m *Manager) Get(id string) (*Session, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	session, ok := m.sessions[id]
	if !ok {
		return nil, fmt.Errorf("terminal session not found")
	}
	return session, nil
}

func (m *Manager) List(projectID string) []*Session {
	m.mu.Lock()
	defer m.mu.Unlock()
	result := make([]*Session, 0, len(m.sessions))
	for _, session := range m.sessions {
		if projectID == "" || session.ProjectID == projectID {
			result = append(result, session)
		}
	}
	return result
}

func (m *Manager) Write(id string, data []byte) error {
	session, err := m.Get(id)
	if err != nil {
		return err
	}
	session.mu.Lock()
	closed := session.closed
	session.mu.Unlock()
	if closed {
		return io.ErrClosedPipe
	}
	_, err = session.pty.Write(data)
	return err
}

func (m *Manager) Resize(id string, cols, rows uint16) (*Session, error) {
	session, err := m.Get(id)
	if err != nil {
		return nil, err
	}
	if cols == 0 || rows == 0 {
		return session, nil
	}
	if err := pty.Setsize(session.pty, &pty.Winsize{Cols: cols, Rows: rows}); err != nil {
		return nil, err
	}
	session.mu.Lock()
	session.Cols = cols
	session.Rows = rows
	session.mu.Unlock()
	return session, nil
}

func (m *Manager) Read(id string, offset int64) (data []byte, next int64, closed bool, err error) {
	session, err := m.Get(id)
	if err != nil {
		return nil, offset, false, err
	}
	session.mu.Lock()
	defer session.mu.Unlock()
	raw := session.buffer.Bytes()
	if offset < 0 {
		offset = 0
	}
	if int(offset) > len(raw) {
		offset = int64(len(raw))
	}
	data = append([]byte(nil), raw[offset:]...)
	return data, int64(len(raw)), session.closed, nil
}

func (m *Manager) Close(id string) (*Session, error) {
	session, err := m.Get(id)
	if err != nil {
		return nil, err
	}
	session.mu.Lock()
	session.closed = true
	session.Status = "closed"
	session.mu.Unlock()
	_ = session.pty.Close()
	if session.cmd.Process != nil {
		_ = session.cmd.Process.Kill()
	}
	return session, nil
}
