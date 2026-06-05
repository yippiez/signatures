// Malformed-but-parseable Swift: missing bodies, unusual spacing, empty
// declarations, attributes with arguments, and operator-like identifiers.

let _: Int = 0
let __version__: String = "1.0.0"

// Attribute with arguments before func
@discardableResult
func computeValue(_ x: Int, _ y: Int) -> Int {
    return x + y
}

// Static and class members
struct MathUtils {
    static let goldenRatio: Double = 1.6180339887
    static var seed: UInt64 = 12345

    static func gcd(_ a: Int, _ b: Int) -> Int {
        var a = a; var b = b
        while b != 0 { let t = b; b = a % b; a = t }
        return a
    }

    static func lcm(_ a: Int, _ b: Int) -> Int {
        return a / gcd(a, b) * b
    }
}

// Class with class funcs (not static)
class Registry {
    var entries: [String: Any]

    class func shared() -> Registry {
        return Registry()
    }

    init() {
        entries = [:]
    }

    subscript(key: String) -> Any? {
        get { return entries[key] }
        set { entries[key] = newValue }
    }

    func register(key: String, value: Any) {
        entries[key] = value
    }
}

// Protocol with default implementations via extension
protocol Greetable {
    var name: String { get }
    func greet() -> String
}

extension Greetable {
    func greet() -> String {
        return "Hello, \(name)!"
    }

    func farewell() -> String {
        return "Goodbye, \(name)!"
    }
}

// Unusual but valid: empty enum body
enum EmptyFlags {}

// Enum with raw value type
enum HTTPMethod: String {
    case get = "GET"
    case post = "POST"
    case put = "PUT"
    case delete = "DELETE"

    var isIdempotent: Bool {
        get {
            switch self {
            case .get, .put, .delete: return true
            case .post: return false
            }
        }
    }
}

// Typealias
typealias CompletionHandler = (Result<Data, Error>) -> Void
typealias JSONDict = [String: Any]

// Actor (Swift concurrency)
actor Counter {
    var value: Int
    let step: Int

    init(start: Int = 0, step: Int = 1) {
        self.value = start
        self.step = step
    }

    func increment() {
        value += step
    }

    func reset() {
        value = 0
    }
}

// Trailing-comma parameter list (unusual formatting)
func multiParam(
    first: Int,
    second: String,
    third: Bool
) -> String {
    return "\(first) \(second) \(third)"
}

// Variadic parameters
func sum(_ numbers: Int...) -> Int {
    return numbers.reduce(0, +)
}

// inout parameter
func swap(_ a: inout Int, _ b: inout Int) {
    let tmp = a; a = b; b = tmp
}
