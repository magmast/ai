package limit

import (
	"context"

	"github.com/magmast/ai/pkg/chat"
)

type Middleware struct {
	Limit int
}

func (m *Middleware) Run(ctx context.Context, req chat.Request, next chat.Handler) (*chat.Response, error) {
	if len(req.Messages) > m.Limit {
		diff := len(req.Messages) - m.Limit
		req.Messages = req.Messages[diff:]
	}

	return next(ctx, req)
}
