package language

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParser_PropertiesAndStyle(t *testing.T) {
	dsl := `
architecture "Test System" {
	system Sys "My System" {
		properties {
			"qps": "1000"
			"latency": "50ms"
		}
		style {
			shape: "cylinder"
			color: "#ff0000"
		}

		container Web "Web App" {
			technology "React"
			properties {
				"framework": "nextjs"
			}
			style {
				icon: "react.png"
			}
		}
	}
}
`
	parser, err := NewParser()
	require.NoError(t, err)

	program, _, err := parser.Parse("test.sruja", dsl)
	require.NoError(t, err)

	sys := program.Architecture.Systems[0]
	assert.Equal(t, "1000", sys.Properties["qps"])
	assert.Equal(t, "50ms", sys.Properties["latency"])
	assert.Equal(t, "cylinder", sys.Style["shape"])
	assert.Equal(t, "#ff0000", sys.Style["color"])

	web := sys.Containers[0]
	assert.Equal(t, "nextjs", web.Properties["framework"])
	assert.Equal(t, "react.png", web.Style["icon"])
}
