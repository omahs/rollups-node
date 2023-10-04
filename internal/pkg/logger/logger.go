// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

package logger

import (
	"io"
	"log"
	"os"
	"strings"
)

var (
	Error   *log.Logger
	Warning *log.Logger
	Info    *log.Logger
	Debug   *log.Logger
)

func init() {
	//FIXME: There might be a problem if this runs before the `config` package
	// has altered the env vars
	logLevel := os.Getenv("CARTESI_LOG_LEVEL")
	_, enableTimestamp := os.LookupEnv("CARTESI_LOG_ENABLE_TIMESTAMP") //LOG_ENABLE_TIMESTAMP
	var flags int
	if enableTimestamp {
		flags |= log.Ldate | log.Ltime
	}

	Error = log.New(os.Stderr, "ERROR ", flags)
	Warning = log.New(os.Stderr, "WARN ", flags)
	Info = log.New(os.Stdout, "INFO ", flags)
	Debug = log.New(os.Stdout, "DEBUG ", flags)

	if strings.EqualFold(logLevel, "error") {
		Warning.SetOutput(io.Discard)
		Info.SetOutput(io.Discard)
		Debug.SetOutput(io.Discard)
	} else if strings.EqualFold(logLevel, "warn") {
		Info.SetOutput(io.Discard)
		Debug.SetOutput(io.Discard)
	} else if strings.EqualFold(logLevel, "debug") {
		flags |= log.Llongfile
		Error.SetFlags(flags)
		Warning.SetFlags(flags)
		Info.SetFlags(flags)
		Debug.SetFlags(flags)
	} else {
		// INFO is the default option
		Debug.SetOutput(io.Discard)
	}
}
