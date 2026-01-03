package dot

// Layout Constants
const (
	// DefaultNodeSep is the default minimum horizontal spacing between nodes (in pixels).
	// DefaultNodeSep is the default minimum horizontal spacing between nodes (in pixels).
	// Matched to LikeC4 (110px @ 72dpi = ~1.5 inches) for balanced spacing with dynamic nodes.
	DefaultNodeSep = 110
	// DefaultRankSep is the default minimum vertical spacing between ranks (in pixels).
	// Matched to LikeC4 (120px @ 72dpi = ~1.6 inches).
	DefaultRankSep = 120
	// DefaultNodeWidth is the default node width (in pixels).
	// Set to 0 to imply dynamic sizing.
	DefaultNodeWidth = 0
	// DefaultNodeHeight is the default node height (in pixels).
	// Set to 0 to imply dynamic sizing.
	DefaultNodeHeight = 0

	// MinWidthPerson is the minimum width for Person nodes.
	MinWidthPerson = 200.0
	// MinHeightPerson is the minimum height for Person nodes.
	MinHeightPerson = 180.0

	// MinWidthSystem is the minimum width for System nodes.
	MinWidthSystem = 220.0
	// MinHeightSystem is the minimum height for System nodes.
	MinHeightSystem = 140.0

	// MinWidthContainer is the minimum width for Container nodes.
	MinWidthContainer = 200.0
	// MinHeightContainer is the minimum height for Container nodes.
	MinHeightContainer = 120.0

	// MinWidthComponent is the minimum width for Component nodes.
	MinWidthComponent = 180.0
	// MinHeightComponent is the minimum height for Component nodes.
	MinHeightComponent = 100.0

	// MinWidthInfrastructure is the minimum width for Datastore and Queue nodes.
	MinWidthInfrastructure = 200.0
	// MinHeightInfrastructure is the minimum height for Datastore and Queue nodes.
	MinHeightInfrastructure = 100.0

	// BufferPaddingPercent is the percentage of buffer padding to add to node dimensions.
	BufferPaddingPercent = 0.05

	// MaxNodeWidth is the maximum allowed node width.
	MaxNodeWidth = 500.0
	// MaxNodeHeight is the maximum allowed node height.
	MaxNodeHeight = 300.0
)

// Scaling Factors
const (
	// L1NodeSepScale is the scaling factor for NodeSep in Level 1 (Context) diagrams.
	L1NodeSepScale = 1.10
	// L1RankSepScale is the scaling factor for RankSep in Level 1 (Context) diagrams.
	L1RankSepScale = 1.10

	// L2NodeSepScale is the base scaling factor for NodeSep in Level 2 (Container) diagrams.
	L2NodeSepScale = 1.05
	// L2RankSepScale is the base scaling factor for RankSep in Level 2 (Container) diagrams.
	L2RankSepScale = 1.10

	// L3NodeSepScale is the base scaling factor for NodeSep in Level 3 (Component) diagrams.
	L3NodeSepScale = 1.15
	// L3RankSepScale is the base scaling factor for RankSep in Level 3 (Component) diagrams.
	L3RankSepScale = 1.20

	// HubDegreeThreshold is the number of connections required for a node to be considered a "hub".
	HubDegreeThreshold = 12
	// HubScaleWidth is the scaling factor for hub node width.
	HubScaleWidth = 1.2
	// HubScaleHeight is the scaling factor for hub node height.
	HubScaleHeight = 1.1

	// DenseGraphThreshold is the number of nodes that triggers dense graph optimizations.
	DenseGraphThreshold = 8
	// ComplexGraphThreshold is the number of nodes that triggers complex graph optimizations.
	ComplexGraphThreshold = 20

	// DynamicScalingBase is the base value for dynamic scaling logarithmic formula.
	DynamicScalingBase = 1.0
	// DynamicScalingFactor is the multiplier for the logarithmic component of dynamic scaling.
	DynamicScalingFactor = 0.15 // Reduced from 0.25
	// DynamicScalingDivisor is the divisor for the logarithmic component of dynamic scaling.
	DynamicScalingDivisor = 10.0 // Increased from 8.0
	// DynamicScalingCap is the maximum allowed value for dynamic scaling factor.
	DynamicScalingCap = 1.5 // Reduced from 2.2
)

// Edge Weight Constants
const (
	// WeightLabeledEdge is the weight for edges with labels.
	WeightLabeledEdge = 10 // Reduced from 25 to allow more flexibility
	// WeightUnlabeledEdge is the weight for edges without labels.
	WeightUnlabeledEdge = 2 // Reduced from 4
	// WeightInternalBoost is the weight boost for internal edges (within same parent).
	WeightInternalBoost = 2
	// HighDegreeThreshold is the threshold for reducing edge weights on high-degree nodes.
	HighDegreeThreshold = 5
	// HighDegreeReductionNumerator is the numerator for reducing edge weights (e.g., 3/4).
	HighDegreeReductionNumerator = 2
	// HighDegreeReductionDenominator is the denominator for reducing edge weights (e.g., 3/4).
	HighDegreeReductionDenominator = 3

	// EdgeBundlingThreshold is the minimum number of edges to trigger bundling.
	EdgeBundlingThreshold = 3
	// EdgeBundlingStrength controls how strongly edges are bundled (0.0-1.0).
	EdgeBundlingStrength = 0.8
	// EdgeBundlingDistance controls the minimum distance between bundled edges.
	EdgeBundlingDistance = 0.5
)

// Styling Constants
const (
	// FontName is the default font family.
	FontName = "Arial"
	// FontSizeGlobal is the default font size for the graph.
	FontSizeGlobal = 12
	// FontSizeNode is the default font size for nodes (implied, often same as global or specific).
	FontSizeNode = 12 // Using global default
	// FontSizeEdge is the default font size for edge labels.
	FontSizeEdge = 11
	// FontSizeCluster is the font size for cluster labels.
	FontSizeCluster = 14

	// ColorSlate500 is the color used for edge lines.
	ColorSlate500 = "#596980"
	// ColorSlate700 is the color used for text (nodes, edges).
	ColorSlate700 = "#4A5568"
	// ColorSlate800 is the color used for cluster titles.
	ColorSlate800 = "#2D3748"
	// ColorGrayBg is the background color for clusters.
	ColorGrayBg = "#f8f9fa"
	// ColorTransparent is transparent color.
	ColorTransparent = "transparent"

	// PenWidthEdge is the thickness of edge lines.
	PenWidthEdge = 2
	// PenWidthNode is the thickness of node borders (0 for flat/filled).
	PenWidthNode = 0

	// ArrowSize is the size of edge arrows.
	ArrowSize = 0.75

	// MarginCluster is the margin inside clusters.
	MarginCluster = 24 // Reduced from 40
	// MarginNode is the internal margin for nodes.
	MarginNode = 0.10 // Reduced from 0.15

	// GraphPad is the graph padding.
	GraphPad = 0.2 // Reduced from 0.5
)
