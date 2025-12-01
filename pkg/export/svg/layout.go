// pkg/export/svg/layout.go
package svg

// LayoutConfig defines the layout parameters for SVG diagram generation.
// This allows for configurable diagram dimensions and spacing instead of hardcoded values.
type LayoutConfig struct {
	// Canvas dimensions
	CanvasWidth  int
	CanvasHeight int
	CanvasX      int
	CanvasY      int

	// System layout
	SystemBoxWidth  int
	SystemBoxHeight int
	SystemBoxX      int
	SystemBoxY      int
	SystemSpacingY  int

	// Container layout
	ContainerBoxWidth  int
	ContainerBoxHeight int
	ContainerSpacingX  int
	ContainerStartX    int
	ContainerStartY    int

	// Component layout
	ComponentBoxWidth  int
	ComponentBoxHeight int
	ComponentSpacingX  int

	// Person layout
	PersonBoxWidth  int
	PersonBoxHeight int
	PersonStartX    int
	PersonStartY    int
	PersonSpacingY  int

	// DataStore layout
	DataStoreBoxWidth  int
	DataStoreBoxHeight int
	DataStoreStartY    int
}

// DefaultLayoutConfig returns the default layout configuration.
// These values match the original hardcoded values for backward compatibility.
func DefaultLayoutConfig() *LayoutConfig {
	return &LayoutConfig{
		// Canvas
		CanvasWidth:  1100,
		CanvasHeight: 1200,
		CanvasX:      50,
		CanvasY:      180,

		// System
		SystemBoxWidth:  600,
		SystemBoxHeight: 250,
		SystemBoxX:      500,
		SystemBoxY:      300,
		SystemSpacingY:  300,

		// Container
		ContainerBoxWidth:  200,
		ContainerBoxHeight: 150,
		ContainerSpacingX:  250,
		ContainerStartX:    450,
		ContainerStartY:    400,

		// Component
		ComponentBoxWidth:  180,
		ComponentBoxHeight: 120,
		ComponentSpacingX:  220,

		// Person
		PersonBoxWidth:  120,
		PersonBoxHeight: 150,
		PersonStartX:    150,
		PersonStartY:    300,
		PersonSpacingY:  200,

		// DataStore
		DataStoreBoxWidth:  200,
		DataStoreBoxHeight: 150,
		DataStoreStartY:    600,
	}
}

// CompactLayoutConfig returns a more compact layout configuration.
// Useful for smaller diagrams or when space is limited.
func CompactLayoutConfig() *LayoutConfig {
	cfg := DefaultLayoutConfig()
	cfg.CanvasWidth = 900
	cfg.CanvasHeight = 1000
	cfg.SystemBoxWidth = 500
	cfg.SystemBoxHeight = 200
	cfg.ContainerBoxWidth = 160
	cfg.ContainerBoxHeight = 120
	cfg.ComponentBoxWidth = 140
	cfg.ComponentBoxHeight = 100
	return cfg
}

// LargeLayoutConfig returns a larger layout configuration.
// Useful for complex diagrams with many elements.
func LargeLayoutConfig() *LayoutConfig {
	cfg := DefaultLayoutConfig()
	cfg.CanvasWidth = 1400
	cfg.CanvasHeight = 1500
	cfg.SystemBoxWidth = 800
	cfg.SystemBoxHeight = 300
	cfg.ContainerBoxWidth = 240
	cfg.ContainerBoxHeight = 180
	cfg.ComponentBoxWidth = 220
	cfg.ComponentBoxHeight = 140
	cfg.ContainerSpacingX = 280
	cfg.ComponentSpacingX = 250
	return cfg
}
