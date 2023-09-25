package chat

import (
	"context"
	"encoding/json"
	"errors"
	"reflect"

	"github.com/magmast/ai/pkg/utils"
	"github.com/rs/zerolog"
)

var (
	ErrNotStruct = errors.New("Function.Args must return a structure")
)

type Request struct {
	Messages  []Message
	Functions []Function
}

func (r Request) MarshalZerologObject(e *zerolog.Event) {
	type TmpFunc struct {
		Name        string
		Description string
		Parameters  map[string]any
	}

	type TmpReq struct {
		Messages  []Message
		Functions []TmpFunc
	}

	e.Any("req", TmpReq{
		Messages: r.Messages,
		Functions: utils.Map(r.Functions, func(f Function) TmpFunc {
			return TmpFunc{
				Name:        f.Name,
				Description: f.Description,
				Parameters:  (&f).Parameters(),
			}
		}),
	})
}

type Response struct {
	Request Request
	Message Message
}

type Message struct {
	Role         Role
	Name         string
	Content      string
	FunctionCall *FunctionCall
}

type Role string

const (
	RoleSystem    Role = "system"
	RoleAssistant Role = "assistant"
	RoleUser      Role = "user"
	RoleFunction  Role = "function"
)

type FunctionCall struct {
	Name      string
	Arguments string
}

type Function struct {
	Name        string
	Description string
	Args        func() json.Unmarshaler
	Run         func(ctx context.Context, args json.Unmarshaler) (string, error)
}

func (f *Function) Parameters() map[string]any {
	a := f.Args()
	pt := reflect.TypeOf(a)
	if pt.Kind() != reflect.Ptr || pt.Elem().Kind() != reflect.Struct {
		panic("function arguments must be a pointer to a struct")
	}

	t := pt.Elem()

	p := make(map[string]any)

	for i := 0; i < t.NumField(); i++ {
		f := t.Field(i)

		if f.Type.Kind() != reflect.String {
			panic("only string arguments are supported at the moment")
		}

		p[f.Name] = map[string]any{"type": "string"}
	}

	return map[string]any{
		"type":       "object",
		"properties": p,
	}
}

func Args[T any](args json.Unmarshaler) T {
	return args.(T)
}
