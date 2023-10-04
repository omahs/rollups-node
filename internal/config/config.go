package config

import (
	"fmt"
	"os"
	"reflect"
	"strconv"
)

const prefix = "CARTESI_"

type Config struct {
	GraphQLPort int `env:"GRAPHQL_PORT" default:"8080"`
	InspectPort int `env:"INSPECT_PORT" default:"8081"`
}

func Load() (*Config, error) {
	config := &Config{}
	v := reflect.Indirect(reflect.ValueOf(config))
	t := reflect.TypeOf(*config)

	for i := 0; i < t.NumField(); i++ {
		field := t.Field(i)
		env, ok := field.Tag.Lookup("env")
		if !ok {
			return nil, fmt.Errorf(`field "%s" must have an "env" tag`, field.Name)
		}

		s := os.Getenv(prefix + env)
		if s == "" {
			s = field.Tag.Get("default")
			if s == "" {
				return nil, fmt.Errorf(`empty value for %s`, env)
			}
		}
		value, err := convert(s, field.Type)
		if err != nil {
			return nil, err
		}

		v.FieldByName(field.Name).Set(value)
	}

	return config, nil
}

func convert(s string, t reflect.Type) (reflect.Value, error) {
	kind := t.Kind()
	switch kind {
	case reflect.Int:
		n, err := strconv.Atoi(s)
		return reflect.ValueOf(n), err
	case reflect.String:
		return reflect.ValueOf(s), nil
	default:
		err := fmt.Errorf(`string "%s" cannot be converted to type %s`, s, kind)
		return reflect.ValueOf(nil), err
	}
}
