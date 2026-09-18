package seed

import storepkg "github.com/polterware/polterware/backend/providers/sqlite"

func seedProcesses(store *storepkg.Store) error {
	// Process sessions are created only when the user starts a local process.
	// Do not seed fake remote PIDs.
	_ = store
	return nil
}
