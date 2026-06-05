// Package generics demonstrates Go 1.18+ generic types and functions.
// It covers type parameters, constraints, multi-line signatures, and
// methods on generic types.
package generics

import "cmp"

// Ordered is a constraint that permits any ordered type.
type Ordered interface {
	~int | ~int8 | ~int16 | ~int32 | ~int64 |
		~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 | ~uintptr |
		~float32 | ~float64 | ~string
}

// Pair holds two values of potentially different types.
type Pair[A, B any] struct {
	First  A
	Second B
}

// NewPair constructs a Pair.
func NewPair[A, B any](first A, second B) Pair[A, B] {
	return Pair[A, B]{First: first, Second: second}
}

func (p Pair[A, B]) Swap() Pair[B, A] {
	return Pair[B, A]{}
}

// Stack is a generic LIFO container.
type Stack[T any] struct {
	items []T
}

func NewStack[T any]() *Stack[T] {
	return &Stack[T]{}
}

func (s *Stack[T]) Push(item T) {
	// body elided
}

func (s *Stack[T]) Pop() (T, bool) {
	var zero T
	return zero, false
}

func (s *Stack[T]) Peek() (T, bool) {
	var zero T
	return zero, false
}

func (s *Stack[T]) Len() int {
	return 0
}

// Map applies a transform function to each element.
func Map[T, U any](slice []T, f func(T) U) []U {
	return nil
}

// Filter retains only elements satisfying the predicate.
func Filter[T any](slice []T, pred func(T) bool) []T {
	return nil
}

// Reduce folds a slice into a single value.
func Reduce[T, U any](slice []T, init U, f func(U, T) U) U {
	return init
}

// Min returns the smaller of two ordered values.
func Min[T cmp.Ordered](a, b T) T {
	if a < b {
		return a
	}
	return b
}

// Max returns the larger of two ordered values.
func Max[T cmp.Ordered](a, b T) T {
	if a > b {
		return a
	}
	return b
}

// Set is a generic hash set backed by a map.
type Set[T comparable] struct {
	m map[T]struct{}
}

func NewSet[T comparable]() *Set[T] {
	return &Set[T]{m: make(map[T]struct{})}
}

func (s *Set[T]) Add(v T) {
	// body elided
}

func (s *Set[T]) Contains(v T) bool {
	return false
}

func (s *Set[T]) Remove(v T) {
	// body elided
}

func (s *Set[T]) Len() int {
	return 0
}

// Result wraps a value or an error, similar to Rust's Result.
type Result[T any] struct {
	value T
	err   error
}

func Ok[T any](v T) Result[T] {
	return Result[T]{value: v}
}

func Err[T any](err error) Result[T] {
	return Result[T]{err: err}
}

func (r Result[T]) Unwrap() T {
	return r.value
}

func (r Result[T]) IsOk() bool {
	return r.err == nil
}
