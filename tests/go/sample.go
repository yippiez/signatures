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
