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
