// pkg/kernel/cache.go
// Performance optimization: Caching for frequently accessed data

package kernel

import (
	"sync"
	"time"

	"github.com/sruja-ai/sruja/pkg/language"
)

// CacheEntry represents a cached value with expiration.
type CacheEntry struct {
	Value     interface{}
	ExpiresAt time.Time
}

// IsExpired checks if the cache entry has expired.
func (e *CacheEntry) IsExpired() bool {
	return time.Now().After(e.ExpiresAt)
}

// KernelCache provides caching for expensive operations.
type KernelCache struct {
	mu                sync.RWMutex
	parsedPrograms    map[string]*language.Program // cellID -> parsed program
	validationResults map[string]interface{}       // scope -> validation results
	queryResults      map[string]interface{}       // query -> results
	diagramCache      map[string]string            // diagram key -> diagram output
}

// NewKernelCache creates a new kernel cache.
func NewKernelCache() *KernelCache {
	return &KernelCache{
		parsedPrograms:    make(map[string]*language.Program),
		validationResults: make(map[string]interface{}),
		queryResults:      make(map[string]interface{}),
		diagramCache:      make(map[string]string),
	}
}

// GetParsedProgram retrieves a cached parsed program.
func (c *KernelCache) GetParsedProgram(cellID string) (*language.Program, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	program, ok := c.parsedPrograms[cellID]
	return program, ok
}

// SetParsedProgram caches a parsed program.
func (c *KernelCache) SetParsedProgram(cellID string, program *language.Program) {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.parsedPrograms[cellID] = program
}

// InvalidateParsedProgram removes a cached parsed program.
func (c *KernelCache) InvalidateParsedProgram(cellID string) {
	c.mu.Lock()
	defer c.mu.Unlock()

	delete(c.parsedPrograms, cellID)
}

// Clear clears all caches.
func (c *KernelCache) Clear() {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.parsedPrograms = make(map[string]*language.Program)
	c.validationResults = make(map[string]interface{})
	c.queryResults = make(map[string]interface{})
	c.diagramCache = make(map[string]string)
}

// GetQueryResult retrieves a cached query result.
func (c *KernelCache) GetQueryResult(query string) (interface{}, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	result, ok := c.queryResults[query]
	return result, ok
}

// SetQueryResult caches a query result.
func (c *KernelCache) SetQueryResult(query string, result interface{}) {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.queryResults[query] = result
}

// GetDiagram retrieves a cached diagram.
func (c *KernelCache) GetDiagram(key string) (string, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	diagram, ok := c.diagramCache[key]
	return diagram, ok
}

// SetDiagram caches a diagram.
func (c *KernelCache) SetDiagram(key string, diagram string) {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.diagramCache[key] = diagram
}

// InvalidateModelDependent clears caches that depend on the model.
func (c *KernelCache) InvalidateModelDependent() {
	c.mu.Lock()
	defer c.mu.Unlock()

	// Clear query and diagram caches when model changes
	c.queryResults = make(map[string]interface{})
	c.diagramCache = make(map[string]string)
	c.validationResults = make(map[string]interface{})
}
