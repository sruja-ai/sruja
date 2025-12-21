// internal/converter/metadata.go
// Metadata conversion functions
package converter

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

func convertMetadataToJSON(src *language.MetadataBlock) []MetadataEntryJSON {
	var entries []MetadataEntryJSON
	for _, entry := range src.Entries {
		if entry.Value != nil {
			val := *entry.Value
			entries = append(entries, MetadataEntryJSON{
				Key:   entry.Key,
				Value: &val,
			})
		} else if len(entry.Array) > 0 {
			entries = append(entries, MetadataEntryJSON{
				Key:   entry.Key,
				Array: entry.Array,
			})
		}
	}
	return entries
}

func convertMetadataFromJSON(src []MetadataEntryJSON) *language.MetadataBlock {
	if len(src) == 0 {
		return nil
	}
	block := &language.MetadataBlock{}
	for _, entry := range src {
		if entry.Value != nil {
			block.Entries = append(block.Entries, &language.MetaEntry{
				Key:   entry.Key,
				Value: entry.Value,
			})
		} else if len(entry.Array) > 0 {
			block.Entries = append(block.Entries, &language.MetaEntry{
				Key:   entry.Key,
				Array: entry.Array,
			})
		}
	}
	return block
}
