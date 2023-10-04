package imgen

import (
	"bufio"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"path/filepath"

	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares/funcs"
	"github.com/rs/zerolog/log"
	gopenai "github.com/sashabaranov/go-openai"
)

func New(client *gopenai.Client) chat.Middleware {
	return funcs.With(funcs.Function{
		Name:        "create_images",
		Description: "Generates images and returns paths to them",
		Args: funcs.Args{
			"prompt": {
				Type:        "string",
				Description: "The prompt to generate images for",
			},
			"number": {
				Type:        "integer",
				Description: "Number of images to generate",
			},
			"size": {
				Enum:        []any{"256x256", "512x512", "1024x1024"},
				Description: "Size of the images to generate. Must be one of 256x256, 512x512, 1024x1024",
			},
		},
		Run: func(fCtx *funcs.Context) any {
			prompt, err := fCtx.String("prompt")
			if err != nil {
				return fmt.Errorf("failed to get prompt: %w", err)
			}

			number, err := fCtx.Int("number")
			if err != nil {
				return fmt.Errorf("failed to get number: %w", err)
			}

			size, err := fCtx.String("size")
			if err != nil {
				return fmt.Errorf("failed to get size: %w", err)
			}

			res, err := client.CreateImage(fCtx.Context, gopenai.ImageRequest{
				Prompt: prompt,
				N:      number,
				Size:   size,
			})
			if err != nil {
				return fmt.Errorf("failed to generate images: %w", err)
			}

			paths := make([]string, 0)

			for _, img := range res.Data {
				u, err := url.Parse(img.URL)
				if err != nil {
					log.Error().Str("url", img.URL).Err(err).Msg("failed to parse image url")
					continue
				}

				path := filepath.Base(u.Path)

				f, err := os.Create(path)
				if err != nil {
					log.Error().Str("path", path).Err(err).Msg("failed to create image file")
					continue
				}
				defer f.Close()

				fw := bufio.NewWriter(f)
				defer fw.Flush()

				res, err := http.Get(img.URL)
				if err != nil {
					log.Error().Str("url", img.URL).Err(err).Msg("failed to download image")
					continue
				}
				defer res.Body.Close()

				if _, err := io.Copy(fw, res.Body); err != nil {
					log.Error().Str("path", path).Err(err).Msg("failed to copy image")
					continue
				}

				paths = append(paths, path)
			}

			return paths
		},
	})
}
