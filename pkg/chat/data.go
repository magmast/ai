package chat

import (
	"errors"
)

var (
	ErrNotStruct = errors.New("Function.Args must return a structure")
)

type Request struct {
	Messages  []Message
	Functions []Function
}

type Response struct {
	Request Request
	Message Message
}

type Message struct {
	Role         Role
	Name         string
	Content      string
	FunctionCall *FunctionCall
}

type Role string

const (
	RoleSystem    Role = "system"
	RoleAssistant Role = "assistant"
	RoleUser      Role = "user"
	RoleFunction  Role = "function"
)

type FunctionCall struct {
	Name      string
	Arguments string
}

type Function struct {
	Name        string
	Description string
	Parameters  map[string]any
}
