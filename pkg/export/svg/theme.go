// pkg/export/svg/theme.go
package svg

// Theme defines color schemes and styling for professional diagrams
type Theme struct {
	Name          string
	Background    string
	GridColor     string
	TitleColor    string
	TitleFontSize int
	FontFamily    string

	// Node colors
	PersonFill      string
	PersonStroke    string
	SystemFill      string
	SystemStroke    string
	ContainerFill   string
	ContainerStroke string
	DatabaseFill    string
	DatabaseStroke  string
	QueueFill       string
	QueueStroke     string

	// Edge colors
	EdgeColor     string
	EdgeWidth     int
	LabelColor    string
	LabelFontSize int

	// Effects
	UseGradients   bool
	UseShadows     bool
	UseCurvedEdges bool
	ShadowOffset   int
	ShadowBlur     int
}

// DefaultTheme returns a professional blue-gray theme
func DefaultTheme() *Theme {
	return &Theme{
		Name:          "default",
		Background:    "#ffffff",
		GridColor:     "#f5f5f5",
		TitleColor:    "#1a1a1a",
		TitleFontSize: 20,
		FontFamily:    "system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",

		PersonFill:      "#ff6b6b",
		PersonStroke:    "#c92a2a",
		SystemFill:      "#4dabf7",
		SystemStroke:    "#1971c2",
		ContainerFill:   "#74c0fc",
		ContainerStroke: "#1c7ed6",
		DatabaseFill:    "#51cf66",
		DatabaseStroke:  "#2f9e44",
		QueueFill:       "#ffa94d",
		QueueStroke:     "#e67700",

		EdgeColor:     "#495057",
		EdgeWidth:     2,
		LabelColor:    "#495057",
		LabelFontSize: 12,

		UseGradients:   true,
		UseShadows:     true,
		UseCurvedEdges: false,
		ShadowOffset:   2,
		ShadowBlur:     4,
	}
}

// ProfessionalTheme returns a professional dark theme
func ProfessionalTheme() *Theme {
	return &Theme{
		Name:          "professional",
		Background:    "#ffffff",
		GridColor:     "#f8f9fa",
		TitleColor:    "#212529",
		TitleFontSize: 22,
		FontFamily:    "'Inter', system-ui, -apple-system, sans-serif",

		PersonFill:      "#ff8787",
		PersonStroke:    "#c92a2a",
		SystemFill:      "#339af0",
		SystemStroke:    "#1864ab",
		ContainerFill:   "#66d9ef",
		ContainerStroke: "#1c7ed6",
		DatabaseFill:    "#51cf66",
		DatabaseStroke:  "#2b8a3e",
		QueueFill:       "#ffd43b",
		QueueStroke:     "#f59f00",

		EdgeColor:     "#495057",
		EdgeWidth:     2,
		LabelColor:    "#495057",
		LabelFontSize: 12,

		UseGradients:   true,
		UseShadows:     true,
		UseCurvedEdges: true,
		ShadowOffset:   3,
		ShadowBlur:     6,
	}
}

// C4Theme returns a theme matching C4 model conventions
func C4Theme() *Theme {
	return &Theme{
		Name:          "c4",
		Background:    "#ffffff",
		GridColor:     "#f5f5f5",
		TitleColor:    "#333333",
		TitleFontSize: 20,
		FontFamily:    "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif",

		// C4 model colors (based on Structurizr/C4-PlantUML conventions)
		PersonFill:      "#ffffff", // White background for persons
		PersonStroke:    "#08427b", // Dark blue border
		SystemFill:      "#438dd5", // Blue for systems
		SystemStroke:    "#08427b", // Dark blue border
		ContainerFill:   "#85bbf0", // Light blue for containers
		ContainerStroke: "#08427b", // Dark blue border
		DatabaseFill:    "#ffffff", // White for databases
		DatabaseStroke:  "#08427b", // Dark blue border
		QueueFill:       "#ffffff", // White for queues
		QueueStroke:     "#08427b", // Dark blue border

		EdgeColor:     "#999999",
		EdgeWidth:     1,
		LabelColor:    "#333333",
		LabelFontSize: 11,

		UseGradients:   false, // C4 uses flat colors
		UseShadows:     false, // C4 doesn't use shadows
		UseCurvedEdges: false,
		ShadowOffset:   0,
		ShadowBlur:     0,
	}
}

// MinimalTheme returns a minimal clean theme
func MinimalTheme() *Theme {
	return &Theme{
		Name:          "minimal",
		Background:    "#ffffff",
		GridColor:     "#ffffff",
		TitleColor:    "#000000",
		TitleFontSize: 18,
		FontFamily:    "system-ui, sans-serif",

		PersonFill:      "#fee2e2",
		PersonStroke:    "#dc2626",
		SystemFill:      "#dbeafe",
		SystemStroke:    "#2563eb",
		ContainerFill:   "#e0f2fe",
		ContainerStroke: "#0284c7",
		DatabaseFill:    "#dcfce7",
		DatabaseStroke:  "#16a34a",
		QueueFill:       "#fef3c7",
		QueueStroke:     "#d97706",

		EdgeColor:     "#6b7280",
		EdgeWidth:     1,
		LabelColor:    "#374151",
		LabelFontSize: 11,

		UseGradients:   false,
		UseShadows:     false,
		UseCurvedEdges: false,
		ShadowOffset:   0,
		ShadowBlur:     0,
	}
}
