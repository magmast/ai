package shell

import (
	"os/exec"

	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares/script"
)

func New() chat.Middleware {
	return script.New(script.Config{
		Name:        "execute_shell_script",
		Description: "Executes shell script and returns its stdout, stderr and status",
		Command:     func(s string) *exec.Cmd { return exec.Command("sh", "-c", s) },
	})
}
