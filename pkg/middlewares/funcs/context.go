package funcs

import "context"

type key int

const runnersKey key = 0

func runners(ctx context.Context) map[string]Runner {
	rs := ctx.Value(runnersKey)
	if rs == nil {
		return make(map[string]Runner)
	}
	return rs.(map[string]Runner)
}

func runner(ctx context.Context, n string) Runner {
	return runners(ctx)[n]
}

func WithRunner(ctx context.Context, n string, r Runner) context.Context {
	rs := runners(ctx)
	rs[n] = r
	return context.WithValue(ctx, runnersKey, rs)
}
