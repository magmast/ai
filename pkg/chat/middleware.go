package chat

import "context"

type Middleware interface {
	Run(ctx context.Context, req Request, next Handler) (*Response, error)
}

type Handler func(ctx context.Context, req Request) (*Response, error)
