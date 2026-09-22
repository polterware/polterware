package importer

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
)

// StableFactID returns a deterministic fact id for re-import upserts.
func StableFactID(projectID, relPath, key string) string {
	return "fact-import-" + shortHash(projectID+"|"+relPath+"|"+key)
}

// StableEvidenceID returns a deterministic evidence id for a relative path.
func StableEvidenceID(projectID, relPath string) string {
	return "ev-import-" + shortHash(projectID+"|"+relPath)
}

// StableSourceID returns a deterministic knowledge source id for a relative path.
func StableSourceID(projectID, relPath string) string {
	return "src-import-" + shortHash(projectID+"|"+relPath)
}

// ImportSource tags provenance as import:<relpath>.
func ImportSource(relPath string) string {
	return fmt.Sprintf("import:%s", relPath)
}

func shortHash(input string) string {
	sum := sha256.Sum256([]byte(input))
	return hex.EncodeToString(sum[:])[:12]
}
