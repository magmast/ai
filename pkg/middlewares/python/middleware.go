package python

import (
	"os/exec"

	"github.com/magmast/ai/pkg/chat"
	"github.com/magmast/ai/pkg/middlewares/script"
)

func New() chat.Middleware {
	return script.New(script.Config{
		Name:        "execute_python_script",
		Description: "Executes python script and returns its stdout, stderr and status",
		Command:     func(s string) *exec.Cmd { return exec.Command("python", "-c", s) },
	})
}
