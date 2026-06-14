@@CASE@@ comments_strings
// Package comments_strings verifies that fake declarations inside comments and
// string/raw-string literals are NOT extracted by the signatures CLI.
//
// The following lines inside this block comment must be ignored:
// func fakeInLineComment(x int) int
// type FakeCommentType struct
// const FakeConst = 99
package commentsstrings

// This is a real constant.
const RealConst = 42

// SomeFunc has a comment above it that mentions func fake1(a, b string) bool
// which must not appear in the output.
func SomeFunc(a, b string) bool {
	// Inside the body: func innerFake() -- must be ignored (inside function body)
	_ = "func fakeInDoubleQuoteString(x int) int"
	_ = `func fakeInRawString(y float64) float64
type FakeRawType struct {
	const FakeRawConst = 0
}`
	return false
}

/* Block comment containing:
   func fakeInBlockComment(z int) {}
   type FakeBlock struct { X int }
   const FakeBlockConst = "hello"
*/

// RealType is a real struct declaration.
type RealType struct {
	Name  string
	Value int
}

// FakeMethodComment demonstrates that a method reference inside a comment is ignored:
// func (r RealType) FakeCommentMethod() error
func (r RealType) RealMethod() error {
	return nil
}

// multiLineStr holds a raw string with embedded fake declarations.
var multiLineStr = `
func notAFunc(a int) {
}

type notAType struct {
	X int
}

const notAConst = 7
`

// anotherFn has a double-quoted string with fake Go code.
func anotherFn() string {
	return "type Ghost struct { func fake() }"
}

// RealInterface is a real interface.
type RealInterface interface {
	Do() error
	Undo() error
}
@@CASE@@ edge
// Package edge covers tricky edge cases: blank identifiers, multi-return,
// variadic params, function-typed fields, init(), blank imports, and a
// deliberately minimal / malformed-but-parseable structure.
package edge

// const block with iota.
const (
	Sunday = iota
	Monday
	Tuesday
	Wednesday
	Thursday
	Friday
	Saturday
)

// Bit-flag constants.
const (
	FlagRead    = 1 << iota
	FlagWrite
	FlagExecute
)

// Multi-return function.
func Divide(a, b float64) (float64, error) {
	return 0, nil
}

// Named return values.
func MinMax(nums []int) (min, max int) {
	return 0, 0
}

// Variadic function.
func Sum(vals ...int) int {
	return 0
}

// Function accepting another function as argument.
func Apply(f func(int) int, v int) int {
	return 0
}

// Function returning a function.
func Adder(x int) func(int) int {
	return nil
}

// init is a special Go function; it appears in the output.
func init() {
	// package-level setup
}

// FuncField is a struct with a function-typed field.
type FuncField struct {
	Name    string
	Handler func(string) error
	Filter  func(int) bool
}

// Doer is a minimal one-method interface.
type Doer interface {
	Do() error
}

// Multi-return with named results and an embedded interface.
type ReadWriteCloser interface {
	Read(p []byte) (n int, err error)
	Write(p []byte) (n int, err error)
	Close() error
}

// Blank-identifier assignment at package level (not a declaration).
var _ Doer = (*FuncField)(nil)

// LongSig has a multi-line parameter list that must be joined into one output line.
func LongSig(
	firstName string,
	lastName string,
	age int,
	active bool,
) (string, error) {
	return "", nil
}

// var block.
var (
	DefaultTimeout = 30
	DefaultRetries = 3
	DefaultHost    = "localhost"
)

// Recursive type alias (type definition that references itself indirectly).
type Node struct {
	Val  int
	Next *Node
}

func (n *Node) Append(val int) *Node {
	return nil
}

// Empty struct used as a set element or signal.
type Token struct{}

func NewToken() Token {
	return Token{}
}
@@CASE@@ generics
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
@@CASE@@ nested
// Package nested demonstrates deeply-nested type definitions, embedded structs,
// anonymous structs in fields, and methods on those types.
package nested

// Address is embedded inside Person.
type Address struct {
	Street  string
	City    string
	Country string
	ZipCode string
}

// Contact groups communication channels.
type Contact struct {
	Email   string
	Phone   string
	Address Address
}

// Person is a top-level entity with a nested anonymous struct.
type Person struct {
	Name    string
	Age     int
	Contact Contact
	Meta    struct {
		CreatedAt int64
		UpdatedAt int64
		Tags      []string
	}
}

func (p Person) FullName() string {
	return ""
}

func (p *Person) UpdateContact(c Contact) {
	// body elided
}

// Organization owns a collection of persons.
type Organization struct {
	Name    string
	Members []Person
	Board   struct {
		Chair   Person
		Members []Person
	}
}

func NewOrganization(name string) *Organization {
	return nil
}

func (o *Organization) AddMember(p Person) {
	// body elided
}

func (o *Organization) FindMember(name string) (Person, bool) {
	return Person{}, false
}

// Tree is a recursive data structure.
type Tree struct {
	Value    int
	Children []*Tree
	Parent   *Tree
}

func NewTree(value int) *Tree {
	return nil
}

func (t *Tree) Insert(value int) *Tree {
	return nil
}

func (t *Tree) Height() int {
	return 0
}

func (t *Tree) Walk(fn func(int)) {
	// body elided
}

// Matrix is a 2-D container.
type Matrix struct {
	Rows int
	Cols int
	Data [][]float64
}

func NewMatrix(rows, cols int) *Matrix {
	return nil
}

func (m *Matrix) At(row, col int) float64 {
	return 0
}

func (m *Matrix) Set(row, col int, val float64) {
	// body elided
}

func (m *Matrix) Multiply(other *Matrix) (*Matrix, error) {
	return nil, nil
}
@@CASE@@ realworld
// Package httpserver provides a minimal HTTP API server used in production.
// It demonstrates realistic patterns: structs with many fields, interfaces,
// constructor functions, methods with pointer receivers, and constants.
package httpserver

import (
	"context"
	"net/http"
	"time"
)

const DefaultPort = 8080
const MaxConnections = 1024
const ReadTimeout = 30 * time.Second

var ErrNotFound = errors.New("resource not found")
var ErrUnauthorized = errors.New("unauthorized")

type Config struct {
	Host           string
	Port           int
	ReadTimeout    time.Duration
	WriteTimeout   time.Duration
	MaxHeaderBytes int
	TLSCertFile    string
	TLSKeyFile     string
}

type Handler interface {
	ServeHTTP(w http.ResponseWriter, r *http.Request)
	Pattern() string
}

type Middleware interface {
	Wrap(next http.Handler) http.Handler
}

type Server struct {
	config  Config
	mux     *http.ServeMux
	logger  Logger
	started bool
}

type Logger interface {
	Info(msg string, args ...interface{})
	Error(msg string, args ...interface{})
}

type RequestContext struct {
	TraceID   string
	UserID    string
	StartTime time.Time
}

func NewServer(cfg Config, logger Logger) *Server {
	return nil
}

func (s *Server) RegisterHandler(h Handler) {
	// body elided
}

func (s *Server) Use(mw Middleware) {
	// body elided
}

func (s *Server) Start(ctx context.Context) error {
	return nil
}

func (s *Server) Stop(ctx context.Context) error {
	return nil
}

func (s *Server) Addr() string {
	return ""
}

func extractRequestContext(r *http.Request) RequestContext {
	return RequestContext{}
}

func writeJSON(w http.ResponseWriter, status int, v interface{}) error {
	return nil
}
@@CASE@@ sample
package main

const Pi = 3.14159
const MaxUsers = 100

type Point struct {
	X int
	Y int
}

type Shape interface {
	Area() float64
}

func (p Point) Distance(o Point) float64 {
	return 0
}

func Add(a, b int) int {
	return a + b
}
@@CASE@@ unicode
// Package unicode exercises non-ASCII identifiers and string contents.
// Go allows Unicode letters in identifiers; the extractor must handle them.
package unicode

// Числа contains numeric constants for internationalized apps.
type Числа struct {
	Значение int
	Метка    string
}

// Δ is the Greek letter delta used as a threshold constant.
const Δ = 0.001

// 最大値 is a Japanese identifier meaning "maximum value".
const 最大値 = 1000

// Grüßen is a German verb meaning "to greet".
func Grüßen(name string) string {
	return ""
}

// 挨拶 is a Japanese function meaning "greeting".
func 挨拶(名前 string) string {
	return ""
}

// MathOps defines arithmetic operations with Unicode method names.
type MathOps interface {
	Сложить(a, b int) int
	Умножить(a, b int) int
}

// Résumé is a struct with accented field names.
type Résumé struct {
	Prénom string
	Nom    string
	Âge    int
}

func (r Résumé) Afficher() string {
	return ""
}

// αβγ is a Greek-alphabet constant group.
const (
	α = 1
	β = 2
	γ = 3
)

// φ is the golden ratio approximation.
const φ = 1.6180339887

// Töölö is a Finnish place name used as an identifier.
type Töölö struct {
	Koordinaten [2]float64
}

func NeuesToölö(lat, lon float64) Töölö {
	return Töölö{}
}
@@CASE@@ type_group
package main

type (
	ID      int64
	Name    string
	Handler func(ID, Name) error
)
@@CASE@@ type_group_struct
package main

type (
	Simple     int
	Aliased    = string
	WithStruct struct {
		X, Y float64
	}
)

func After() {}
@@CASE@@ raw_string_backslash
package main

var s = `x\`

func Invisible() {}
@@CASE@@ send_chan_type
package main

type SendChan chan<- string

func After() {}
@@CASE@@ typed_var_no_init
package main

var counter int

func After() {}
@@CASE@@ grouped_var_types
package main

var (
	MaxRetries int
	BaseURL    string
	Enabled    bool
)

func Real() {}
@@CASE@@ group_value_continuation
package fuzz

const (
	X = 1 +
		2

	Y = 42
)

func After() {}
@@CASE@@ group_inline_paren_call
package main

var (
	A = f(
		1)
	B = 2
)
@@CASE@@ group_closing_paren_own_line
package main

var (
	A = f(
		1,
	)
	B = 3
)

func After() {}
