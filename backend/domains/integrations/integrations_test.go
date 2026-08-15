package integrations

import "testing"

func TestCLIRegistryHasSupportedProvidersOnly(t *testing.T) {
	if len(CLIProviders) != 9 {
		t.Fatalf("got %d providers", len(CLIProviders))
	}
	for _, provider := range CLIProviders {
		if provider.Binary == "" || len(provider.AuthCommands) == 0 {
			t.Fatalf("incomplete provider: %s", provider.ID)
		}
	}
}
