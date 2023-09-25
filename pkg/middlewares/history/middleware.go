package history

import (
	"context"
	"fmt"

	"github.com/magmast/ai/pkg/chat"
	"github.com/rs/zerolog/log"
)

type Middleware struct {
	Backend Backend
	Format  Format
	// TODO(magmast): Add cache so history isn't read for every request.
}

func (m *Middleware) Run(ctx context.Context, req chat.Request, next chat.Handler) (*chat.Response, error) {
	err := m.PrepareRequest(&req)
	if err != nil {
		return nil, err
	}

	res, err := next(ctx, req)
	if err != nil {
		return nil, err
	}

	if err := m.SaveResponse(res); err != nil {
		log.Error().Err(err).Msg("failed to save history from response")
	}

	return res, nil
}

func (m *Middleware) PrepareRequest(req *chat.Request) error {
	bs, err := m.Backend.Read()
	if err != nil {
		return fmt.Errorf("failed to read history: %w", err)
	}

	ms, err := m.Format.Unmarshal(bs)
	if err != nil {
		return fmt.Errorf("failed to unmarshal history: %w", err)
	}

	ms = append(ms, req.Messages...)
	req.Messages = ms

	return nil
}

func (m *Middleware) SaveResponse(res *chat.Response) error {
	ms := append(res.Request.Messages, res.Message)

	bs, err := m.Format.Marshal(ms)
	if err != nil {
		return fmt.Errorf("failed to marshal history: %w", err)
	}

	if err := m.Backend.Write(bs); err != nil {
		return fmt.Errorf("failed to write history: %w", err)
	}

	return nil
}
