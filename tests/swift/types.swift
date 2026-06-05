// Showcases struct, class, enum, protocol, and extension with computed
// properties, mutating funcs, static funcs, and various accessor forms.

let defaultCapacity: Int = 16
let version: String = "2.1.0"

// MARK: - Protocols

protocol Identifiable {
    associatedtype ID: Hashable
    var id: ID { get }
}

protocol Describable {
    var description: String { get }
    func shortDescription() -> String
}

protocol Resizable {
    mutating func resize(to newCapacity: Int)
    var capacity: Int { get }
    var count: Int { get }
}

// MARK: - Struct with computed properties and mutating funcs

struct Vector2D {
    var x: Double
    var y: Double

    static let zero: Vector2D = Vector2D(x: 0, y: 0)
    static let one: Vector2D = Vector2D(x: 1, y: 1)

    var magnitude: Double {
        get { return (x * x + y * y).squareRoot() }
    }

    var normalized: Vector2D {
        get { return magnitude == 0 ? .zero : Vector2D(x: x / magnitude, y: y / magnitude) }
    }

    var isZero: Bool {
        get { return x == 0 && y == 0 }
    }

    mutating func scale(by factor: Double) {
        x *= factor
        y *= factor
    }

    mutating func translate(dx: Double, dy: Double) {
        x += dx
        y += dy
    }

    static func dot(_ a: Vector2D, _ b: Vector2D) -> Double {
        return a.x * b.x + a.y * b.y
    }

    static func + (lhs: Vector2D, rhs: Vector2D) -> Vector2D {
        return Vector2D(x: lhs.x + rhs.x, y: lhs.y + rhs.y)
    }
}

// MARK: - Enum with associated values and computed properties

enum Shape {
    case circle(radius: Double)
    case rectangle(width: Double, height: Double)
    case triangle(base: Double, height: Double)
    case polygon(sides: Int, sideLength: Double)

    var area: Double {
        get {
            switch self {
            case .circle(let r): return Double.pi * r * r
            case .rectangle(let w, let h): return w * h
            case .triangle(let b, let h): return 0.5 * b * h
            case .polygon(let n, let s):
                return (Double(n) * s * s) / (4.0 * tan(Double.pi / Double(n)))
            }
        }
    }

    var perimeter: Double {
        get {
            switch self {
            case .circle(let r): return 2 * Double.pi * r
            case .rectangle(let w, let h): return 2 * (w + h)
            case .triangle(let b, let h): return b + 2 * (b * b / 4 + h * h).squareRoot()
            case .polygon(let n, let s): return Double(n) * s
            }
        }
    }

    func scale(by factor: Double) -> Shape {
        switch self {
        case .circle(let r): return .circle(radius: r * factor)
        case .rectangle(let w, let h): return .rectangle(width: w * factor, height: h * factor)
        case .triangle(let b, let h): return .triangle(base: b * factor, height: h * factor)
        case .polygon(let n, let s): return .polygon(sides: n, sideLength: s * factor)
        }
    }

    static func unitCircle() -> Shape {
        return .circle(radius: 1.0)
    }
}

// MARK: - Class with stored + computed properties

class Window {
    var title: String
    var frame: (x: Double, y: Double, width: Double, height: Double)
    private var _isVisible: Bool
    weak var delegate: AnyObject?

    var isVisible: Bool {
        get { return _isVisible }
        set { _isVisible = newValue }
    }

    var center: (x: Double, y: Double) {
        get {
            return (frame.x + frame.width / 2, frame.y + frame.height / 2)
        }
        set {
            frame.x = newValue.x - frame.width / 2
            frame.y = newValue.y - frame.height / 2
        }
    }

    var area: Double {
        get { return frame.width * frame.height }
    }

    init(title: String, x: Double, y: Double, width: Double, height: Double) {
        self.title = title
        self.frame = (x, y, width, height)
        self._isVisible = false
    }

    func show() {
        _isVisible = true
    }

    func hide() {
        _isVisible = false
    }

    func move(to point: (x: Double, y: Double)) {
        frame.x = point.x
        frame.y = point.y
    }

    func resize(width: Double, height: Double) {
        frame.width = width
        frame.height = height
    }
}

// MARK: - Extensions conforming to protocols

extension Vector2D: Identifiable {
    typealias ID = String
    var id: String {
        get { return "(\(x),\(y))" }
    }
}

extension Vector2D: Describable {
    var description: String {
        get { return "Vector2D(x: \(x), y: \(y))" }
    }

    func shortDescription() -> String {
        return "(\(x), \(y))"
    }
}

extension Shape: Describable {
    var description: String {
        get {
            switch self {
            case .circle(let r): return "Circle(r=\(r))"
            case .rectangle(let w, let h): return "Rect(\(w)x\(h))"
            case .triangle(let b, let h): return "Triangle(b=\(b),h=\(h))"
            case .polygon(let n, let s): return "Polygon(\(n)-gon,s=\(s))"
            }
        }
    }

    func shortDescription() -> String {
        return description
    }
}
