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
