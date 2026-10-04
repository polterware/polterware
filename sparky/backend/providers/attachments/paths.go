package attachments

import (
	"crypto/rand"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"golang.org/x/sys/unix"
)

// The data directory is trusted. Every component below it is opened relative to
// its parent descriptor, with no symlink traversal, including during renames.
func (s *Store) openRoot() (*os.File, error) {
	parent, err := os.Open(filepath.Dir(s.root))
	if err != nil {
		return nil, err
	}
	defer parent.Close()
	return openDirectory(parent, filepath.Base(s.root))
}

func openDirectory(parent *os.File, name string) (*os.File, error) {
	var st unix.Stat_t
	if err := unix.Fstatat(int(parent.Fd()), name, &st, unix.AT_SYMLINK_NOFOLLOW); err != nil {
		return nil, err
	}
	if st.Mode&unix.S_IFMT != unix.S_IFDIR {
		return nil, fmt.Errorf("attachment ancestor is not a directory")
	}
	fd, err := unix.Openat(int(parent.Fd()), name, unix.O_RDONLY|unix.O_DIRECTORY|unix.O_NOFOLLOW|unix.O_CLOEXEC, 0)
	if err != nil {
		return nil, err
	}
	return os.NewFile(uintptr(fd), name), nil
}

func (s *Store) openMemory(memoryID string, create bool) (*os.File, error) {
	if !ulidPattern.MatchString(memoryID) {
		return nil, fmt.Errorf("invalid memory ID")
	}
	root, err := s.openRoot()
	if err != nil {
		return nil, err
	}
	defer root.Close()
	if create {
		if err = unix.Mkdirat(int(root.Fd()), memoryID, 0700); err != nil && err != unix.EEXIST {
			return nil, err
		}
	}
	return openDirectory(root, memoryID)
}

func splitPath(rel string) (string, string, error) {
	parts := strings.Split(rel, string(filepath.Separator))
	if len(parts) != 2 || !ulidPattern.MatchString(parts[0]) || parts[1] == "" || parts[1] == "." || parts[1] == ".." {
		return "", "", fmt.Errorf("invalid attachment path")
	}
	return parts[0], parts[1], nil
}

func createTemp(dir *os.File) (*os.File, string, error) {
	b := make([]byte, 16)
	if _, err := rand.Read(b); err != nil {
		return nil, "", err
	}
	name := fmt.Sprintf(".import-%x", b)
	fd, err := unix.Openat(int(dir.Fd()), name, unix.O_WRONLY|unix.O_CREAT|unix.O_EXCL|unix.O_NOFOLLOW|unix.O_CLOEXEC, 0600)
	if err != nil {
		return nil, "", err
	}
	return os.NewFile(uintptr(fd), name), name, nil
}
