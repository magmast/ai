package history

import (
	"encoding/json"

	"github.com/magmast/ai/pkg/chat"
)

type Format interface {
	Marshal([]chat.Message) ([]byte, error)
	Unmarshal([]byte) ([]chat.Message, error)
}

type JSON struct {
}

func (m *JSON) Marshal(msgs []chat.Message) ([]byte, error) {
	return json.Marshal(msgs)
}

func (m *JSON) Unmarshal(bytes []byte) ([]chat.Message, error) {
	var msgs []chat.Message
	if err := json.Unmarshal(bytes, &msgs); err != nil {
		return nil, err
	}
	return msgs, nil
}
