package main

import (
	"encoding/base64"
	"fmt"
	"math"
	"sort"
	"strings"
)

func generateSemanticFingerprint(data interface{}) string {
	switch v := data.(type) {
	case bool:
		return fmt.Sprintf("bool:%t", v)
	case int, int8, int16, int32, int64, uint, uint8, uint16, uint32, uint64:
		return fmt.Sprintf("int:%v", v)
	case float32:
		return fmt.Sprintf("float:0x%016x", math.Float64bits(float64(v)))
	case float64:
		return fmt.Sprintf("float:0x%016x", math.Float64bits(v))
	case string:
		return fmt.Sprintf("str:%s", v)
	case []byte:
		return fmt.Sprintf("bytes:%s", base64.StdEncoding.EncodeToString(v))
	case []interface{}:
		var parts []string
		for _, item := range v {
			parts = append(parts, generateSemanticFingerprint(item))
		}
		return fmt.Sprintf("[%s]", strings.Join(parts, ","))
	case map[string]interface{}:
		keys := make([]string, 0, len(v))
		for k := range v {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		var parts []string
		for _, k := range keys {
			parts = append(parts, fmt.Sprintf("[%s,%s]", generateSemanticFingerprint(k), generateSemanticFingerprint(v[k])))
		}
		return fmt.Sprintf("[%s]", strings.Join(parts, ","))
	default:
		return fmt.Sprintf("unknown:%T", v)
	}
}

func main() {
	data := map[string]interface{}{
		"z_string":   "hello",
		"a_float":    float64(3.14159),
		"an_int":     int(42),
		"a_bool":     true,
		"nested_map": map[string]interface{}{"c": "c", "b": "b", "a": "a"},
		"an_array":   []interface{}{int(1), float64(2.5), false},
		"some_bytes": []byte{0xDE, 0xAD, 0xBE, 0xEF},
	}
	fmt.Println(generateSemanticFingerprint(data))
}
