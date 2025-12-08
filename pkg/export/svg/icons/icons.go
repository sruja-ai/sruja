package icons

import (
	_ "embed"
	"encoding/json"
	"os"
	"regexp"
	"strings"
	"sync"
)

//go:embed icons.json
var jsonData []byte

var (
	once  sync.Once
	cache map[string]string
)

func load() {
	cache = map[string]string{}
	if b, err := os.ReadFile("pkg/export/svg/icons/icons.json"); err == nil {
		_ = json.Unmarshal(b, &cache)
		re := regexp.MustCompile(`(?is)<svg[^>]*>(.*)</svg>`)
		for k, v := range cache {
			if strings.Contains(v, "<svg") {
				m := re.FindStringSubmatch(v)
				if len(m) == 2 {
					cache[k] = strings.TrimSpace(m[1])
				}
			}
		}
		return
	}
	_ = json.Unmarshal(jsonData, &cache)
}

func Get(name string) string {
	once.Do(load)
	if cache == nil {
		return ""
	}
	return cache[name]
}
