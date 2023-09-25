package middlewares

import (
	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares/funcs"
	"github.com/magmast/ai/pkg/middlewares/history"
	"github.com/magmast/ai/pkg/middlewares/limit"
	"github.com/magmast/ai/pkg/middlewares/openai"
	"github.com/magmast/ai/pkg/middlewares/python"
	"github.com/magmast/ai/pkg/middlewares/shell"
	"github.com/magmast/ai/pkg/middlewares/system"
	gopenai "github.com/sashabaranov/go-openai"
)

func NewFuncs() chat.Middleware {
	return &funcs.Middleware{}
}

func NewHistory(backend history.Backend, format history.Format) chat.Middleware {
	return &history.Middleware{
		Backend: backend,
		Format:  format,
	}
}

func NewLimit(n int) chat.Middleware {
	return &limit.Middleware{
		Limit: n,
	}
}

func NewOpenAI(config ...OpenAIConfig) chat.Middleware {
	var c *OpenAIConfig
	if len(config) > 0 {
		c = &config[0]
	}

	var apiKey string
	if c != nil {
		apiKey = c.ApiKey
	}

	clientConfig := gopenai.DefaultConfig(apiKey)
	if c != nil && c.BaseURL != "" {
		clientConfig.BaseURL = c.BaseURL
	}

	var model string
	if c != nil && c.Model != "" {
		model = c.Model
	}

	return &openai.Middleware{
		Client: gopenai.NewClientWithConfig(clientConfig),
		Model:  model,
	}
}

type OpenAIConfig struct {
	BaseURL string
	ApiKey  string
	Model   string
}

var DefaultOpenAIConfig = OpenAIConfig{
	BaseURL: "https://api.openai.com",
	ApiKey:  "",
	Model:   "gpt-3.5-turbo-16k",
}

func NewPython() chat.Middleware {
	return python.New()
}

func NewShell() chat.Middleware {
	return shell.New()
}

func NewSystem(m string) chat.Middleware {
	return &system.Middleware{
		Message: m,
	}
}
