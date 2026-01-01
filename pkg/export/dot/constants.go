package dot

// Layout Constants
const (
	// DefaultNodeSep is the default minimum horizontal spacing between nodes (in pixels).
	DefaultNodeSep = 150
	// DefaultRankSep is the default minimum vertical spacing between ranks (in pixels).
	DefaultRankSep = 180
	// DefaultNodeWidth is the default node width (in pixels).
	DefaultNodeWidth = 200
	// DefaultNodeHeight is the default node height (in pixels).
	DefaultNodeHeight = 120

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
	L1NodeSepScale = 1.15
	// L1RankSepScale is the scaling factor for RankSep in Level 1 (Context) diagrams.
	L1RankSepScale = 1.20

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
	DynamicScalingFactor = 0.25
	// DynamicScalingDivisor is the divisor for the logarithmic component of dynamic scaling.
	DynamicScalingDivisor = 8.0
	// DynamicScalingCap is the maximum allowed value for dynamic scaling factor.
	DynamicScalingCap = 2.2
)

// Edge Weight Constants
const (
	// WeightLabeledEdge is the weight for edges with labels.
	WeightLabeledEdge = 25
	// WeightUnlabeledEdge is the weight for edges without labels.
	WeightUnlabeledEdge = 4
	// WeightInternalBoost is the weight boost for internal edges (within same parent).
	WeightInternalBoost = 2
	// HighDegreeThreshold is the threshold for reducing edge weights on high-degree nodes.
	HighDegreeThreshold = 4
	// HighDegreeReductionNumerator is the numerator for reducing edge weights (e.g., 3/4).
	HighDegreeReductionNumerator = 3
	// HighDegreeReductionDenominator is the denominator for reducing edge weights (e.g., 3/4).
	HighDegreeReductionDenominator = 4
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
	MarginCluster = 40
	// MarginNode is the internal margin for nodes.
	MarginNode = 0.15

	// GraphPad is the graph padding.
	GraphPad = 0.5
)
