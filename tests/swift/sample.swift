let MAX = 5

struct Point {
    var x: Int
    var y: Int
}

class Circle {
    var radius: Double = 0
    func area() -> Double {
        return Double.pi * radius * radius
    }
}

protocol Shape {
    func area() -> Double
}

enum Color { case red, green, blue }
