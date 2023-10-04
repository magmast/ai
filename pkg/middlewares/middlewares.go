package middlewares

import (
	"os"

	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares/funcs"
	"github.com/magmast/ai/pkg/middlewares/history"
	"github.com/magmast/ai/pkg/middlewares/imgen"
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

func NewImgen(client *gopenai.Client) chat.Middleware {
	return imgen.New(client)
}

func NewLimit(n int) chat.Middleware {
	return &limit.Middleware{
		Limit: n,
	}
}

type OpenAIConfig struct {
	Client *gopenai.Client
	Model  string
}

func NewOpenAI(config ...OpenAIConfig) chat.Middleware {
	var c *OpenAIConfig
	if len(config) > 0 {
		c = &config[0]
	}

	var client *gopenai.Client
	if c != nil && c.Client != nil {
		client = c.Client
	} else {
		apiKey := os.Getenv("OPENAI_API_KEY")
		if apiKey == "" {
			panic("OPENAI_API_KEY is not set")
		}

		client = gopenai.NewClient(apiKey)
	}

	var model string
	if c != nil && c.Model != "" {
		model = c.Model
	}

	return &openai.Middleware{
		Client: client,
		Model:  model,
	}
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
