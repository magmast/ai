package funcs

import (
	"github.com/magmast/ai/pkg/chat"
)

type Function struct {
	Name        string
	Description string
	Args        Args
	Run         Runner
}

func (b *Function) Function() chat.Function {
	return chat.Function{
		Name:        b.Name,
		Description: b.Description,
		Parameters: map[string]any{
			"type":       "object",
			"properties": b.Args.Map(),
		},
	}
}

type Runner func(*Context) any

type Args map[string]Arg

func (a Args) Map() map[string]map[string]any {
	m := make(map[string]map[string]any, len(a))
	for k, v := range a {
		m[k] = v.Map()
	}
	return m
}

type Arg struct {
	Type        string
	Description string
	Enum        []any
}

func (a *Arg) Map() map[string]any {
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
