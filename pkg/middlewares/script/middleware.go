package script

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"os/exec"

	"github.com/magmast/ai/pkg/ask"
	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares/funcs"
)

var ErrUserDeclined = errors.New("user declined script execution")

type Args struct {
	Script string
}

func (args *Args) UnmarshalJSON(bs []byte) error {
	type tmpArgs struct {
		Script string
	}

	ta := tmpArgs{}
	if err := json.Unmarshal(bs, &ta); err != nil {
		return err
	}

	args.Script = ta.Script

	return nil
}

type Config struct {
	Name        string
	Description string
	Command     func(s string) *exec.Cmd
}

func New(config Config) chat.Middleware {
	return funcs.With(funcs.Function{
		Name:        config.Name,
		Description: config.Description,
		Args: funcs.Args{
			"script": {
				Type:        "string",
				Description: "Script to execute",
			},
		},
		Run: func(fCtx *funcs.Context) any {
			script, err := fCtx.String("script")
			if err != nil {
				return fmt.Errorf("failed to get script: %w", err)
			}

			allowed, err := ask.Bool("I need to execute the following script:\n\n%s\n\nDo you agree?", script)
			if err != nil {
				return fmt.Errorf("failed to ask user if script execution is allowed, so script execution was not allowed: %w", err)
			}
			if !allowed {
				return ErrUserDeclined
			}

			status := 0
			stdout := new(bytes.Buffer)
			stderr := new(bytes.Buffer)

			cmd := config.Command(script)
			cmd.Stdout = stdout
			cmd.Stderr = stderr
			err = cmd.Run()
			if errors.Is(err, &exec.ExitError{}) {
				status = err.(*exec.ExitError).ExitCode()
			} else if err != nil {
				return fmt.Errorf("failed to execute script: %w", err)
			}

			return struct {
				Status int
				Stdout string
				Stderr string
			}{
				Status: status,
				Stdout: stdout.String(),
				Stderr: stderr.String(),
			}
		},
	})
}
