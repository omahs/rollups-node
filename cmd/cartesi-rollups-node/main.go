// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

package main

import (
	"github.com/cartesi/rollups-node/internal/pkg/logger"
)

func main() {
	if err := rootCmd.Execute(); err != nil {
		logger.Error.Fatal(err)
	}
}
