package sqlite

import "time"

// HookSwitches returns the stored switches; a hook with no row is on.
func (s *Store) HookSwitches() (map[string]bool, error) {
	rows, err := s.DB.Query(`SELECT name, enabled FROM agent_hook_switches`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := map[string]bool{}
	for rows.Next() {
		var name string
		var enabled int
		if err := rows.Scan(&name, &enabled); err != nil {
			return nil, err
		}
		out[name] = enabled != 0
	}
	return out, rows.Err()
}

// SetHookSwitch turns one hook on or off.
func (s *Store) SetHookSwitch(name string, enabled bool) error {
	value := 0
	if enabled {
		value = 1
	}
	_, err := s.DB.Exec(
		`INSERT INTO agent_hook_switches(name, enabled, updated_at) VALUES(?,?,?)
		 ON CONFLICT(name) DO UPDATE SET enabled=excluded.enabled, updated_at=excluded.updated_at`,
		name, value, time.Now().UTC().Format(time.RFC3339),
	)
	return err
}
