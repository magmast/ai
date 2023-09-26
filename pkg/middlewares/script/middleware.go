package script

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os/exec"

	"github.com/magmast/ai/pkg/ask"
	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares/funcs"
)

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
		Args: funcs.ArgsConfig{
			"script": {
				Type:        "string",
				Description: "Script to execute",
			},
		},
		Run: func(ctx context.Context, args funcs.Args) (string, error) {
			script := funcs.Arg[string](args, "script")

			allowed, err := ask.Bool("I need to execute the following script:\n\n%s\n\nDo you agree?", script)
			if err != nil {
				return "failed to ask user if script execution is allowed, so script execution was not allowed", nil
			}
			if !allowed {
				return "user declined script execution", nil
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
				return fmt.Sprintf("failed to execute script: %s", err), nil
			}

			result := struct {
				Status int
				Stdout string
				Stderr string
			}{
				Status: status,
				Stdout: stdout.String(),
				Stderr: stderr.String(),
			}

			bs, err := json.Marshal(result)
			if err != nil {
				return fmt.Sprintf("failed to marshal stdout: %s", err), nil
			}

			return string(bs), nil
		},
	})
}
