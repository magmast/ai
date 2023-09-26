package funcs

import (
	"context"

	"github.com/magmast/ai/pkg/chat"
)

type Function struct {
	Name        string
	Description string
	Args        ArgsConfig
	Run         Runner
}

func (b *Function) Function() chat.Function {
	return chat.Function{
		Name:        b.Name,
		Description: b.Description,
		Parameters: map[string]any{
			"type":       "object",
			"properties": b.Args,
		},
	}
}

type Runner func(context.Context, Args) (string, error)

type ArgsConfig map[string]ArgConfig

type ArgConfig struct {
	Type        string
	Description string
	Enum        []any
}

func (a *ArgConfig) Map() map[string]any {
	m := map[string]any{}

	if a.Type != "" {
		m["type"] = a.Type
	}

	if a.Description != "" {
		m["description"] = a.Description
	}

	if len(a.Enum) > 0 {
		m["enum"] = a.Enum
	}

	return m
}

type Args map[string]any

func Arg[T any](a Args, n string) T {
	return a[n].(T)
}
