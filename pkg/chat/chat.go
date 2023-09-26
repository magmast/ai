package chat

import (
	"context"
	"errors"

	"github.com/magmast/ai/pkg/utils"
	"github.com/rs/zerolog/log"
)

var ErrNoMiddlewares = errors.New("no middlewares")

type Chat struct {
	Middlewares []Middleware
}

func (c *Chat) Send(ctx context.Context, s string) (*Response, error) {
	var initRun Handler = func(ctx context.Context, req Request) (*Response, error) {
		return nil, ErrNoMiddlewares
	}

	run := utils.FoldR(c.Middlewares, initRun, func(acc Handler, middleware Middleware) Handler {
		return func(ctx context.Context, req Request) (*Response, error) {
			log.Trace().Ctx(ctx).Type("middleware", middleware).Any("request", req).Msg("running middleware")
			return middleware.Run(ctx, req, acc)
		}
	})

	return run(ctx, Request{
		Messages: []Message{
			{
				Role:    RoleUser,
				Content: s,
			},
		},
	})
}
