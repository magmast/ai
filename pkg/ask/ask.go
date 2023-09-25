package ask

import (
	"bufio"
	"fmt"
	"os"
	"unicode"
)

func Bool(format string, a ...any) (bool, error) {
	prompt := fmt.Sprintf(format, a...)
	fmt.Printf("%s [Y/n]", prompt)

	r := bufio.NewReader(os.Stdin)
	c, _, err := r.ReadRune()
	if err != nil {
		return false, err
	}

	return unicode.ToLower(c) == 'y', nil
}
