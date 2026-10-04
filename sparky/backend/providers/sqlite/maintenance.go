package sqlite

import (
	"context"
	"fmt"
	"os"
)

func (s *Store) Backup(ctx context.Context, destPath string) (int64, error) {
	if _, err := s.h.Writer().ExecContext(ctx, `VACUUM INTO ?`, destPath); err != nil {
		return 0, fmt.Errorf("vacuum into: %w", err)
	}
	info, err := os.Stat(destPath)
	if err != nil {
		return 0, err
	}
	return info.Size(), nil
}
