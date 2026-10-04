package store

import (
	"math"
	"math/rand/v2"
	"sort"
	"sync"
)

// Scored is one top-K hit. Score is the dot product of L2-normalized vectors
// (cosine similarity).
type Scored struct {
	Index int
	Score float32
}

// Generate returns n L2-normalized float32 vectors of dims, packed row-major.
func Generate(n, dims int, seed uint64) []float32 {
	rng := rand.New(rand.NewPCG(seed, seed^0x9e3779b97f4a7c15))
	buf := make([]float32, n*dims)
	for i := 0; i < n; i++ {
		row := buf[i*dims : (i+1)*dims]
		var sum float64
		for d := 0; d < dims; d++ {
			v := float32(rng.NormFloat64())
			row[d] = v
			sum += float64(v) * float64(v)
		}
		inv := float32(1 / math.Sqrt(sum))
		for d := 0; d < dims; d++ {
			row[d] *= inv
		}
	}
	return buf
}

// TopK returns the k highest dot products against query. query must already be
// L2-normalized and have length dims.
func TopK(data []float32, dims, k int, query []float32) []Scored {
	return topKRange(data, dims, k, 0, len(data)/dims, query)
}

// TopKParallel splits the scan across workers goroutines and merges the heaps.
// The measurement includes goroutine start, which is the cost of the obvious implementation.
func TopKParallel(data []float32, dims, k, workers int, query []float32) []Scored {
	n := len(data) / dims
	if workers < 2 || n == 0 {
		return TopK(data, dims, k, query)
	}
	if workers > n {
		workers = n
	}
	parts := make([][]Scored, workers)
	var wg sync.WaitGroup
	chunk := (n + workers - 1) / workers
	for w := 0; w < workers; w++ {
		from := w * chunk
		if from >= n {
			break
		}
		to := from + chunk
		if to > n {
			to = n
		}
		wg.Add(1)
		go func(w, from, to int) {
			defer wg.Done()
			parts[w] = topKRange(data, dims, k, from, to, query)
		}(w, from, to)
	}
	wg.Wait()
	merged := make([]Scored, 0, workers*k)
	for _, part := range parts {
		merged = append(merged, part...)
	}
	return keepTop(merged, k)
}

func topKRange(data []float32, dims, k, from, to int, query []float32) []Scored {
	if k <= 0 || from >= to {
		return nil
	}
	h := make([]Scored, 0, k)
	for i := from; i < to; i++ {
		row := data[i*dims : (i+1)*dims]
		score := dot(row, query)
		if len(h) < k {
			h = append(h, Scored{Index: i, Score: score})
			if len(h) == k {
				heapifyMin(h)
			}
			continue
		}
		if score > h[0].Score {
			h[0] = Scored{Index: i, Score: score}
			siftDownMin(h, 0)
		}
	}
	sort.Slice(h, func(a, b int) bool {
		if h[a].Score == h[b].Score {
			return h[a].Index < h[b].Index
		}
		return h[a].Score > h[b].Score
	})
	return h
}

func keepTop(items []Scored, k int) []Scored {
	if len(items) <= k {
		sort.Slice(items, func(a, b int) bool { return items[a].Score > items[b].Score })
		return items
	}
	h := append([]Scored(nil), items[:k]...)
	heapifyMin(h)
	for _, item := range items[k:] {
		if item.Score > h[0].Score {
			h[0] = item
			siftDownMin(h, 0)
		}
	}
	sort.Slice(h, func(a, b int) bool {
		if h[a].Score == h[b].Score {
			return h[a].Index < h[b].Index
		}
		return h[a].Score > h[b].Score
	})
	return h
}

func dot(a, b []float32) float32 {
	var sum float32
	for i := range a {
		sum += a[i] * b[i]
	}
	return sum
}

func heapifyMin(h []Scored) {
	for i := len(h)/2 - 1; i >= 0; i-- {
		siftDownMin(h, i)
	}
}

func siftDownMin(h []Scored, i int) {
	n := len(h)
	for {
		left := i*2 + 1
		if left >= n {
			return
		}
		smallest := left
		right := left + 1
		if right < n && h[right].Score < h[left].Score {
			smallest = right
		}
		if h[i].Score <= h[smallest].Score {
			return
		}
		h[i], h[smallest] = h[smallest], h[i]
		i = smallest
	}
}

// Float32LE encodes vectors as little-endian float32 bytes, the embeddings.vector layout.
func Float32LE(v []float32) []byte {
	buf := make([]byte, len(v)*4)
	for i, x := range v {
		bits := math.Float32bits(x)
		off := i * 4
		buf[off] = byte(bits)
		buf[off+1] = byte(bits >> 8)
		buf[off+2] = byte(bits >> 16)
		buf[off+3] = byte(bits >> 24)
	}
	return buf
}
