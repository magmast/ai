package cmd

import (
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/adrg/xdg"
	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares"
	"github.com/magmast/ai/pkg/middlewares/history"
	"github.com/rs/zerolog/log"
	gopenai "github.com/sashabaranov/go-openai"
	"github.com/spf13/cobra"
)

var (
	apiKey  string
	baseUrl string

	rootCmd = &cobra.Command{
		Use:  "ai",
		Args: cobra.MinimumNArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			if apiKey == "" {
				apiKey = os.Getenv("OPENAI_API_KEY")
			}

			config := gopenai.DefaultConfig(apiKey)
			if baseUrl != "" {
				config.BaseURL = baseUrl
			}

			historyPath, err := xdg.StateFile("ai/history.json")
			if err != nil {
				log.Fatal().Err(err).Msg("failed to get history file path")
			}

			chat := &chat.Chat{
				Middlewares: []chat.Middleware{
					middlewares.NewHistory(&history.FileBackend{Path: historyPath}, &history.JSON{}),
					middlewares.NewLimit(15),
					middlewares.NewShell(),
					middlewares.NewPython(),
					middlewares.NewFuncs(),
					middlewares.NewSystem("You're a command line tool 'ai'. Your task is to talk with user and help him solve any problem using the available functions. Please remember that they return stdout, stderr and status when creating scripts (use print statements in python)."),
					middlewares.NewOpenAI(middlewares.OpenAIConfig{
						BaseURL: baseUrl,
						ApiKey:  apiKey,
						Model:   "gpt-3.5-turbo-16k",
					}),
				},
			}

			res, err := chat.Send(context.TODO(), strings.Join(args, " "))
			if err != nil {
				log.Fatal().Err(err).Msg("failed to send message")
			}

			fmt.Println(res.Message.Content)
		},
	}
)

func init() {
	rootCmd.PersistentFlags().StringVarP(&apiKey, "api-key", "k", "", "OpenAI API key")
	rootCmd.PersistentFlags().StringVarP(&baseUrl, "base-url", "u", "", "OpenAI base URL")
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		log.Fatal().Err(err).Msg("failed to execute root command")
	}
}
