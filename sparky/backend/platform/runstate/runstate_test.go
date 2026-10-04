package runstate

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"testing"
)

func TestLockAndPrivateFiles(t *testing.T) {
	db := filepath.Join(t.TempDir(), "sparky.db")
	lock, err := LockCore(db)
	if err != nil {
		t.Fatal(err)
	}
	_, err = LockCore(db)
	var locked LockedError
	if !errors.As(err, &locked) {
		t.Fatalf("second lock: %v", err)
	}
	state := Core{PID: os.Getpid(), DBPath: db, Endpoint: "http://127.0.0.1:1234", Token: "secret", Build: "1-1"}
	if err := WriteJSON(CorePath(db), state); err != nil {
		t.Fatal(err)
	}
	if got, ok := ReadCore(db); !ok || got != state {
		t.Fatalf("read state: %+v %v", got, ok)
	}
	for _, path := range []string{CorePath(db), LockPath(db)} {
		info, err := os.Stat(path)
		if err != nil || info.Mode().Perm() != 0o600 {
			t.Fatalf("private %s: %v %v", path, info, err)
		}
	}
	if err := RemoveCore(db, os.Getpid()+1); err != nil {
		t.Fatal(err)
	}
	if _, ok := ReadCore(db); !ok {
		t.Fatal("different owner removed state")
	}
	if err := RemoveCore(db, os.Getpid()); err != nil {
		t.Fatal(err)
	}
	if err := lock.Close(); err != nil {
		t.Fatal(err)
	}
	lock, err = LockCore(db)
	if err != nil {
		t.Fatal(err)
	}
	defer lock.Close()
}

func TestConcurrentPublicationAndReaders(t *testing.T) {
	db := filepath.Join(t.TempDir(), "sparky.db")
	state := Core{PID: os.Getpid(), DBPath: db, Endpoint: "http://127.0.0.1:1234", Token: "secret"}
	if err := WriteJSON(CorePath(db), state); err != nil {
		t.Fatal(err)
	}
	var wg sync.WaitGroup
	for i := 0; i < 8; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < 40; j++ {
				if err := WriteJSON(CorePath(db), state); err != nil {
					t.Error(err)
				}
				if got, ok := ReadCore(db); !ok || got.PID != state.PID {
					t.Errorf("torn publication: %+v %t", got, ok)
				}
			}
		}()
	}
	wg.Wait()
	// Malformed/stale readers must not delete a concurrently published state.
	if err := os.WriteFile(CorePath(db), []byte("{"), 0o600); err != nil {
		t.Fatal(err)
	}
	if _, ok := ReadCore(db); ok {
		t.Fatal("malformed state accepted")
	}
	if _, err := os.Stat(CorePath(db)); err != nil {
		t.Fatal("read removed state", err)
	}
}
func TestLogRotationAndConcurrentClose(t *testing.T) {
	db := filepath.Join(t.TempDir(), "sparky.db")
	w, err := OpenLog(db)
	if err != nil {
		t.Fatal(err)
	}
	w.limit = 64
	for i := 0; i < 10; i++ {
		if _, err := fmt.Fprintln(w, "a log entry that occupies enough bytes to rotate"); err != nil {
			t.Fatal(err)
		}
	}
	for _, suffix := range []string{"", ".1", ".2", ".3"} {
		info, err := os.Stat(LogPath(db) + suffix)
		if err != nil || info.Mode().Perm() != 0o600 {
			t.Fatalf("log archive %s: %v", suffix, err)
		}
	}
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, err := w.Write([]byte("entry\n"))
			if err != nil && !errors.Is(err, os.ErrClosed) {
				t.Error(err)
			}
		}()
	}
	if err := w.Close(); err != nil {
		t.Fatal(err)
	}
	wg.Wait()
	if err := w.Close(); err != nil {
		t.Fatal(err)
	}
}
