package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/stretchr/testify/require"
)

func TestParseCourseExample(t *testing.T) {
	parser, err := language.NewParser()
	require.NoError(t, err)

	_, _, err = parser.ParseFile("../../examples/course/ecommerce.sruja")
	require.NoError(t, err)
}
