package funcs

import (
	"context"
	"encoding/json"

	"github.com/magmast/ai/pkg/chat"
)

type Middleware struct {
}

func (m *Middleware) Run(ctx context.Context, req chat.Request, next chat.Handler) (*chat.Response, error) {
	for {
		res, err := next(ctx, req)
		if err != nil {
			return nil, err
		}

		if res.Message.FunctionCall == nil {
			return res, nil
		}

		req.Messages = append(req.Messages, res.Message)

		out := m.RunFunc(ctx, &req, res.Message.FunctionCall)
		req.Messages = append(req.Messages, chat.Message{
			Role:    chat.RoleFunction,
			Name:    res.Message.FunctionCall.Name,
			Content: out,
		})
	}
}

func (m *Middleware) RunFunc(ctx context.Context, req *chat.Request, fc *chat.FunctionCall) string {
	f := m.FindFunc(req, fc.Name)
	if f == nil {
		return "function not found"
	}

	args := make(Args)
	if err := json.Unmarshal([]byte(fc.Arguments), &args); err != nil {
		return "invalid arguments"
	}

	r := runner(ctx, f.Name)
	out, err := r(ctx, args)
	if err != nil {
		return err.Error()
	}

	return out
}

func (m *Middleware) FindFunc(req *chat.Request, name string) *chat.Function {
	for _, f := range req.Functions {
		if f.Name == name {
			return &f
		}
	}

	return nil
}

type withFunctionMiddleware struct {
	f chat.Function
	r Runner
}

func With(f Function) chat.Middleware {
	return &withFunctionMiddleware{
		f: f.Function(),
		r: f.Run,
	}
}

func (m *withFunctionMiddleware) Run(ctx context.Context, req chat.Request, next chat.Handler) (*chat.Response, error) {
	req.Functions = append(req.Functions, m.f)
	c := WithRunner(ctx, m.f.Name, m.r)
	return next(c, req)
}
