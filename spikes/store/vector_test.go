package store

import (
	"math"
	"testing"
)

func TestTopKPlantedMatch(t *testing.T) {
	const n, dims, k = 64, 32, 5
	data := Generate(n, dims, 7)
	query := data[3*dims : (3+1)*dims]
	hits := TopK(data, dims, k, query)
	if len(hits) != k {
		t.Fatalf("len = %d", len(hits))
	}
	if hits[0].Index != 3 {
		t.Fatalf("top index = %d score = %g", hits[0].Index, hits[0].Score)
	}
	if math.Abs(float64(hits[0].Score)-1) > 1e-5 {
		t.Fatalf("top score = %g", hits[0].Score)
	}
	par := TopKParallel(data, dims, k, 4, query)
	if len(par) != len(hits) {
		t.Fatalf("parallel len = %d", len(par))
	}
	for i := range hits {
		if par[i].Index != hits[i].Index {
			t.Fatalf("parallel hit %d = %+v, single = %+v", i, par[i], hits[i])
		}
		if math.Abs(float64(par[i].Score-hits[i].Score)) > 1e-5 {
			t.Fatalf("score mismatch %+v vs %+v", par[i], hits[i])
		}
	}
}

func TestFTSQuery(t *testing.T) {
	if got := FTSQuery(`café reunião`); got != `"café" "reunião"*` {
		t.Fatalf("got %s", got)
	}
	if got := FTSQuery(`"café da manhã"`); got != `"café da manhã"` {
		t.Fatalf("phrase got %s", got)
	}
	if got := FTSQuery(`foo OR bar*`); got != `"foo" "OR" "bar"*` {
		t.Fatalf("operators got %s", got)
	}
}
