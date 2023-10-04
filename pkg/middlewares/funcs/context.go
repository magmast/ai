package funcs

import (
	"context"
	"errors"
)

var (
	ErrInvalidArgType = errors.New("invalid argument type")
	ErrArgNotFound    = errors.New("argument not found")
)

type Context struct {
	Context context.Context
	Args    map[string]any
}

func (c *Context) String(n string) (string, error) {
	a, ok := c.Args[n]
	if !ok {
		return "", ErrArgNotFound
	}

	v, ok := a.(string)
	if !ok {
		return "", ErrInvalidArgType
	}

	return v, nil
}

func (c *Context) Int(n string) (int, error) {
	a, ok := c.Args[n]
	if !ok {
		return 0, ErrArgNotFound
	}

	v, ok := a.(float64)
	if !ok {
		return 0, ErrInvalidArgType
	}

	return int(v), nil
}

type key int

const runnersKey key = 0

func runners(ctx context.Context) map[string]Runner {
	rs := ctx.Value(runnersKey)
	if rs == nil {
		return make(map[string]Runner)
	}
	return rs.(map[string]Runner)
}

func runner(ctx context.Context, n string) Runner {
	return runners(ctx)[n]
}

func WithRunner(ctx context.Context, n string, r Runner) context.Context {
	rs := runners(ctx)
	rs[n] = r
	return context.WithValue(ctx, runnersKey, rs)
}
