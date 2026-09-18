package cli

import (
	"encoding/json"
	"os"
	"path/filepath"
)

func dockerConfigPath() string {
	if home, err := os.UserHomeDir(); err == nil && home != "" {
		return filepath.Join(home, ".docker", "config.json")
	}
	return ""
}

func terraformCredentialsPath() string {
	if home, err := os.UserHomeDir(); err == nil && home != "" {
		return filepath.Join(home, ".terraform.d", "credentials.tfrc.json")
	}
	return ""
}

// dockerHasRegistryAuth reports whether Docker config lists any registry auth
// entries. It inspects keys only and never returns credential values.
func dockerHasRegistryAuth(configPath string) bool {
	if configPath == "" {
		return false
	}
	raw, err := os.ReadFile(configPath)
	if err != nil {
		return false
	}
	var parsed struct {
		Auths map[string]json.RawMessage `json:"auths"`
	}
	if err := json.Unmarshal(raw, &parsed); err != nil {
		return false
	}
	return len(parsed.Auths) > 0
}

func terraformCredentialsPresent(credentialsPath string) bool {
	if credentialsPath == "" {
		return false
	}
	info, err := os.Stat(credentialsPath)
	if err != nil || info.IsDir() || info.Size() == 0 {
		return false
	}
	return true
}
