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
