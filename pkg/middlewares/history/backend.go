package history

import (
	"errors"
	"os"
	"path/filepath"
)

type Backend interface {
	Read() ([]byte, error)
	Write([]byte) error
}

type FileBackend struct {
	Path string
}

func (f *FileBackend) Read() ([]byte, error) {
	b, err := os.ReadFile(f.Path)
	if errors.Is(err, os.ErrNotExist) {
		return []byte("[]"), nil
	}

	return b, err
}

func (f *FileBackend) Write(b []byte) error {
	dir := filepath.Dir(f.Path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}

	return os.WriteFile(f.Path, b, 0755)
}
