package main

// A const group where members have string values. After masking, each string
// becomes spaces — so the assignment line ends with `=`, which incorrectly
// triggers the continuation flag (cont=true) and causes the NEXT member to be
// skipped. All four constants must appear in the output.
const (
	Greeting = "hello"
	Farewell = "goodbye"
	Count    = 42
	Version  = "1.0.0"
)

func After() {}
