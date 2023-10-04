package state

import "context"

type key int

const stateKey key = 0

func Set(ctx context.Context, state *State) context.Context {
	return context.WithValue(ctx, stateKey, state)
}

func Get(ctx context.Context) *State {
	return ctx.Value(stateKey).(*State)
}
