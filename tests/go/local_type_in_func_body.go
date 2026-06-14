package main

// A Go function that declares a local type inside its body. The local type is
// an implementation detail and must NOT appear in the output. Only the function
// itself should be emitted (consistent with how Rust suppresses local items in
// fn bodies via suppress_locals_in_fn_body, which currently returns false for Go).
func ProcessItems() {
	type item struct {
		ID   int
		Name string
	}
	_ = item{}
}

// A second function to confirm the scanner resumes correctly afterward.
func Real() {}
