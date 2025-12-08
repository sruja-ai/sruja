package svg

type Exporter struct {
	Width             int
	Height            int
	Direction         string
	NodeWidth         int
	NodeHeight        int
	HorizontalSpacing int
	VerticalSpacing   int
	Padding           int
	ShowTitle         bool
	TitleFontSize     int
	Theme             *Theme
	ShowGrid          bool
	ShowLegend        bool
	EmbedFonts        bool
	Metadata          map[string]string
}

func NewExporter() *Exporter {
	return &Exporter{
		Width:             1200,
		Height:            800,
		Direction:         "LR",
		NodeWidth:         180,
		NodeHeight:        70,
		HorizontalSpacing: 140,
		VerticalSpacing:   50,
		Padding:           50,
		ShowTitle:         true,
		TitleFontSize:     20,
		Theme:             C4Theme(),
		ShowGrid:          false,
		ShowLegend:        false,
	}
}
