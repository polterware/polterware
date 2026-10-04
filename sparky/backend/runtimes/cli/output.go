package main

import (
	"fmt"
	"io"
	"os"
	"strings"

	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// jsonMarshal encodes RPC responses as protobuf JSON mapping (camelCase names,
// int64 as strings). Shape version: sparky.cli.json.v1 (contracts v1.6).
var jsonMarshal = protojson.MarshalOptions{
	UseProtoNames:   false,
	EmitUnpopulated: true,
}

func useColor() bool {
	if os.Getenv("NO_COLOR") != "" {
		return false
	}
	if strings.EqualFold(os.Getenv("TERM"), "dumb") {
		return false
	}
	return false
}

func writeJSON(out io.Writer, msg proto.Message) error {
	raw, err := jsonMarshal.Marshal(msg)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintln(out, string(raw))
	return err
}

func writeJSONBytes(out io.Writer, raw []byte) error {
	_, err := fmt.Fprintln(out, string(raw))
	return err
}

func formatStamp(ts *timestamppb.Timestamp) string {
	if ts == nil {
		return "-"
	}
	t := ts.AsTime().UTC()
	if t.IsZero() {
		return "-"
	}
	return t.Format("2006-01-02T15:04:05.000Z")
}

func dash(s string) string {
	if strings.TrimSpace(s) == "" {
		return "-"
	}
	return s
}
