package utils

func MapS[I any, O any](s []I, f func(I) O) []O {
	result := make([]O, len(s))
	for i, v := range s {
		result[i] = f(v)
	}
	return result
}

func MapM[K comparable, I any, O any](m map[K]I, f func(I) O) map[K]O {
	result := make(map[K]O)
	for k, v := range m {
		result[k] = f(v)
	}
	return result
}

func FoldR[I any, O any](s []I, init O, f func(O, I) O) O {
	acc := init
	for i := len(s) - 1; i >= 0; i-- {
		elem := s[i]
		acc = f(acc, elem)
	}
	return acc
}
