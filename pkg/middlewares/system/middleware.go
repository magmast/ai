package system

import (
	"context"

	"github.com/magmast/ai/pkg/chat"
	"golang.org/x/exp/slices"
)

type Middleware struct {
	Message string
}

func (m *Middleware) Run(ctx context.Context, req chat.Request, next chat.Handler) (*chat.Response, error) {
	ms := []chat.Message{
		{
			Role:    chat.RoleSystem,
			Content: m.Message,
		},
	}
	req.Messages = append(ms, req.Messages...)

	res, err := next(ctx, req)
	if err != nil {
		return nil, err
	}

	res.Request.Messages = slices.DeleteFunc(res.Request.Messages, func(m chat.Message) bool {
		return m.Role == chat.RoleSystem
	})

	return res, nil
}
