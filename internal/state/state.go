package state

import (
	"context"

	gopenai "github.com/sashabaranov/go-openai"
	"github.com/urfave/cli/v2"
)

var RequiredFlags = []cli.Flag{
	&cli.StringFlag{
		Name:     "api-key",
		Aliases:  []string{"k"},
		Usage:    "OpenAI API key",
		EnvVars:  []string{"OPENAI_API_KEY"},
		Required: true,
	},
	&cli.StringFlag{
		Name:    "base-url",
		Aliases: []string{"u"},
		Usage:   "OpenAI base URL",
		EnvVars: []string{"OPENAI_BASE_URL"},
	},
}

type State struct {
	Client *gopenai.Client
}

func New(cCtx *cli.Context) (context.Context, *State) {
	config := gopenai.DefaultConfig(cCtx.String("api-key"))

	baseUrl := cCtx.String("base-url")
	if baseUrl != "" {
		config.BaseURL = baseUrl
	}

	state := &State{
		Client: gopenai.NewClientWithConfig(config),
	}

	return Set(cCtx.Context, state), state
}
