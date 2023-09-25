package openai

import (
	"context"
	"fmt"

	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/utils"
	"github.com/sashabaranov/go-openai"
)

const DefaultModel = "gpt-3.5-turbo-16k"

// Middleware sends accepted messages to the openai API and returns the
// response.
type Middleware struct {
	Client *openai.Client
	Model  string
}

func (m *Middleware) Run(ctx context.Context, req chat.Request, next chat.Handler) (*chat.Response, error) {
	model := m.Model
	if model == "" {
		model = DefaultModel
	}

	res, err := m.Client.CreateChatCompletion(ctx, openai.ChatCompletionRequest{
		Model: model,
		Messages: utils.Map(req.Messages, func(m chat.Message) openai.ChatCompletionMessage {
			var fc *openai.FunctionCall
			if m.FunctionCall != nil {
				fc = &openai.FunctionCall{
					Name:      m.FunctionCall.Name,
					Arguments: m.FunctionCall.Arguments,
				}
			}

			return openai.ChatCompletionMessage{
				Role:         string(m.Role),
				Name:         m.Name,
				Content:      m.Content,
				FunctionCall: fc,
			}
		}),
		Functions: utils.Map(req.Functions, func(f chat.Function) openai.FunctionDefinition {
			return openai.FunctionDefinition{
				Name:        f.Name,
				Description: f.Description,
				Parameters:  (&f).Parameters(),
			}
		}),
	})
	if err != nil {
		return nil, fmt.Errorf("failed to create chat completion: %w", err)
	}

	msg := res.Choices[0].Message

	var fc *chat.FunctionCall
	if msg.FunctionCall != nil {
		fc = &chat.FunctionCall{
			Name:      msg.FunctionCall.Name,
			Arguments: msg.FunctionCall.Arguments,
		}
	}

	return &chat.Response{
		Request: req,
		Message: chat.Message{
			Role:         chat.Role(msg.Role),
			Name:         msg.Name,
			Content:      msg.Content,
			FunctionCall: fc,
		},
	}, nil
}
