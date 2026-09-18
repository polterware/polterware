package localcore

import "testing"

func TestIsLocalProcessPlatform(t *testing.T) {
	t.Parallel()
	cases := []struct {
		platform string
		want     bool
	}{
		{platform: "local", want: true},
		{platform: "Local", want: true},
		{platform: "local · arm64", want: true},
		{platform: "macOS · arm64", want: false},
		{platform: "linux", want: false},
		{platform: "", want: false},
	}
	for _, tc := range cases {
		if got := isLocalProcessPlatform(tc.platform); got != tc.want {
			t.Fatalf("platform %q: got %v want %v", tc.platform, got, tc.want)
		}
	}
}
