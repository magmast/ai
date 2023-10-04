package cmd

import (
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/adrg/xdg"
	"github.com/magmast/ai/internal/state"
	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares"
	"github.com/magmast/ai/pkg/middlewares/history"
	"github.com/rs/zerolog/log"
	gopenai "github.com/sashabaranov/go-openai"
	"github.com/spf13/cobra"
)

func init() {
	rootCmd.PersistentFlags().StringVarP(&apiKey, "api-key", "k", os.Getenv("OPENAI_API_KEY"), "OpenAI API key")
	rootCmd.PersistentFlags().StringVarP(&baseUrl, "base-url", "u", os.Getenv("OPENAI_BASE_URL"), "OpenAI API base URL")
}

var (
	apiKey  string
	baseUrl string

	rootCmd = &cobra.Command{
		Use:   "ai",
		Short: "Extend terminal with OpenAI",
		Args:  cobra.MinimumNArgs(1),
		PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
			if apiKey == "" {
				return errors.New("either OPENAI_API_KEY environment variable or --api-key flag must be set")
			}

			config := gopenai.DefaultConfig(apiKey)
			if baseUrl != "" {
				config.BaseURL = baseUrl
			}

			client := gopenai.NewClientWithConfig(config)

			cmd.SetContext(state.Set(cmd.Context(), &state.State{
				Client: client,
			}))

			return nil
		},
		RunE: func(cmd *cobra.Command, args []string) error {
			s := state.Get(cmd.Context())

			historyPath, err := xdg.StateFile("ai/history.json")
			if err != nil {
				return fmt.Errorf("failed to get history file path: %w", err)
			}

			chat := &chat.Chat{
				Middlewares: []chat.Middleware{
					middlewares.NewHistory(&history.FileBackend{Path: historyPath}, &history.JSON{}),
					middlewares.NewLimit(15),
					middlewares.NewShell(),
					middlewares.NewPython(),
					middlewares.NewImgen(s.Client),
					middlewares.NewFuncs(),
					middlewares.NewSystem("You're a command line tool 'ai'. Your task is to talk with user and help him solve any problem using the available functions."),
					middlewares.NewOpenAI(middlewares.OpenAIConfig{
						Client: s.Client,
						Model:  "localmodels__llama-2-7b-chat-ggml__llama-2-7b-chat.ggmlv3.q8_0.bin",
					}),
				},
			}

			res, err := chat.Send(cmd.Context(), strings.Join(args, " "))
			if err != nil {
				return fmt.Errorf("failed to send message: %w", err)
			}

			_, err = fmt.Println(res.Message.Content)
			return err
		},
	}
)

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		log.Fatal().Err(err).Msg("failed to execute root command")
	}
}
