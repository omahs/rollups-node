package config

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestConfigDefaults(t *testing.T) {
	_, err := Load()
	assert.Nil(t, err)
}

func TestConfigTodo(t *testing.T) {
	if os.Setenv("CARTESI_INSPECT_PORT", "aaa") != nil {
		assert.Fail(t, "internal")
	}
	_, err := Load()
	assert.Nil(t, err)
}
