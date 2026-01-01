package mermaid

// Styling Constants
const (
	// Node Styles
	StylePerson    = "fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000"
	StyleSystem    = "fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000"
	StyleContainer = "fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000"
	StyleDatabase  = "fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000"
	StyleQueue     = "fill:#ffe5cc,stroke:#333,stroke-width:2px,color:#000"
	StyleExternal  = "fill:#eeeeee,stroke:#666,stroke-width:2px,color:#000,stroke-dasharray: 3 3"
	StyleComponent = "fill:#e6f7ff,stroke:#333,stroke-width:2px,color:#000"

	// Class Names
	ClassPerson    = "personStyle"
	ClassSystem    = "systemStyle"
	ClassContainer = "containerStyle"
	ClassDatabase  = "databaseStyle"
	ClassQueue     = "queueStyle"
	ClassExternal  = "externalStyle"
	ClassComponent = "componentStyle"
)

// Formatting Constants
const (
	// Indent2 is 2 spaces.
	Indent2 = "  "
	// Indent4 is 4 spaces.
	Indent4 = "    "
	// Indent8 is 8 spaces.
	Indent8 = "        "

	// DefaultTheme is the default Mermaid theme.
	DefaultTheme = "default"
	// DefaultDirection is the default layout direction.
	DefaultDirection = "LR"
)
