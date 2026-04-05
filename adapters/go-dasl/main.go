package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"github.com/hyphacoop/go-dasl/drisl"
	"io"
	"math"
	"os"
	"sort"
	"strings"
)

const TargetVersion = "v0.8.0"

type IPCResult struct {
	Status      string  `json:"status"`
	Version     string  `json:"version"`
	Fingerprint *string `json:"fingerprint"`
	ErrorReason *string `json:"error_reason"`
}

func generateSemanticFingerprint(data interface{}) string {
	switch v := data.(type) {
	case bool:
		return fmt.Sprintf("bool:%t", v)
	case int, int8, int16, int32, int64, uint, uint8, uint16, uint32, uint64:
		return fmt.Sprintf("int:%v", v)
	case float32:
		bits := math.Float64bits(float64(v))
		return fmt.Sprintf("float:0x%016x", bits)
	case float64:
		bits := math.Float64bits(v)
		return fmt.Sprintf("float:0x%016x", bits)
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
	case map[interface{}]interface{}:
		// Catch generic maps that some CBOR decoders use
		keys := make([]string, 0, len(v))
		for k := range v {
			keys = append(keys, fmt.Sprintf("%v", k))
		}
		sort.Strings(keys)
		var parts []string
		for _, k := range keys {
			parts = append(parts, fmt.Sprintf("[%s,%s]", generateSemanticFingerprint(k), generateSemanticFingerprint(v[k])))
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

func printResult(status string, fingerprint *string, errorReason *string) {
	res := IPCResult{
		Status:      status,
		Version:     TargetVersion,
		Fingerprint: fingerprint,
		ErrorReason: errorReason,
	}
	out, _ := json.Marshal(res)
	fmt.Println(string(out))
}

func main() {
	defer func() {
		if r := recover(); r != nil {
			errStr := fmt.Sprintf("panic: %v", r)
			printResult("crash", nil, &errStr)
			os.Exit(0)
		}
	}()

	inputBytes, err := io.ReadAll(os.Stdin)
	if err != nil {
		errStr := "failed to read stdin"
		printResult("crash", nil, &errStr)
		return
	}

	if len(inputBytes) == 0 {
		errStr := "empty input"
		printResult("reject", nil, &errStr)
		return
	}

	// Because go-dasl uses fxamacker/cbor under the hood, Unmarshal is the standard API
	var parsedData interface{}
	err = drisl.Unmarshal(inputBytes, &parsedData)

	if err != nil {
		errStr := err.Error()
		printResult("reject", nil, &errStr)
		return
	}

	fingerprint := generateSemanticFingerprint(parsedData)
	printResult("accept", &fingerprint, nil)
}
