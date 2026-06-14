@@CASE@@ comments_strings
// This file tests that declarations inside comments and string literals
// are correctly ignored by the extractor.

let apiVersion: String = "v2"
let helpText: String = "Usage: mytool [options]\n  func notAFunc() {}\n  struct NotAStruct {}"

// The following comment contains fake declarations that must be ignored:
// struct FakeStruct { var x: Int }
// func fakeFunction() -> Bool { return true }
// let FAKE_CONST = 42
// class FakeClass {}
// protocol FakeProtocol {}

struct RealModel {
    let id: Int

    /* Another block comment with fake decls:
       func hiddenMethod() -> String { "hidden" }
       var hiddenVar: Bool = false
       struct HiddenNested {}
    */
    var label: String

    // Single-line comment: func alsoFake() {}
    func realMethod() -> String {
        // Inside body comment: let innerFake = 99
        let inner = "not a sig"
        return inner
    }
}

// Multiline string literal containing fake declarations:
let documentation: String = """
struct DocumentedType {
    var field: Int
    func method() {}
}
protocol DocumentedProtocol {
    func requirement()
}
let DOCUMENTED_CONST = 100
"""

// Interpolated string with fake code:
let snippet: String = "func \("interpolated")() -> Int { 0 }"

protocol Serializable {
    func serialize() -> Data
    func deserialize(from data: Data) throws
}

class Cache {
    var maxSize: Int
    var entries: [String: Any]

    init(maxSize: Int) {
        self.maxSize = maxSize
        self.entries = [:]
    }

    // func evict() is documented elsewhere; this is the real one:
    func evict() {
        entries.removeAll()
    }

    var count: Int {
        get { return entries.count }
    }
}

/* Top-level fake declarations in a block comment:
struct TopFake {}
func topFakeFunc() {}
let TOP_FAKE = 0
*/

let logPrefix: String = "// class NotReal {}"
@@CASE@@ edge
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
@@CASE@@ generics
// Generic functions, types, and where-clauses covering a variety of constraints.

let emptySlotSentinel: Int = -1

// MARK: - Generic free functions

func identity<T>(_ value: T) -> T {
    return value
}

func zip<A, B>(_ a: [A], _ b: [B]) -> [(A, B)] {
    return Swift.zip(a, b).map { ($0.0, $0.1) }
}

func sorted<T: Comparable>(_ collection: [T]) -> [T] {
    return collection.sorted()
}

func transform<Input, Output>(
    _ values: [Input],
    using transform: (Input) throws -> Output
) rethrows -> [Output] {
    return try values.map(transform)
}

func merge<Key: Hashable, Value>(
    _ lhs: [Key: Value],
    _ rhs: [Key: Value],
    resolving conflict: (Value, Value) -> Value
) -> [Key: Value] {
    var result = lhs
    for (k, v) in rhs {
        result[k] = result[k].map { conflict($0, v) } ?? v
    }
    return result
}

// MARK: - Generic types

struct Stack<Element> {
    private var storage: [Element]

    init() {
        storage = []
    }

    mutating func push(_ element: Element) {
        storage.append(element)
    }

    mutating func pop() -> Element? {
        return storage.popLast()
    }

    var top: Element? {
        get { return storage.last }
    }

    var isEmpty: Bool {
        get { return storage.isEmpty }
    }
}

struct Pair<First, Second> {
    let first: First
    let second: Second

    init(_ first: First, _ second: Second) {
        self.first = first
        self.second = second
    }

    func swapped() -> Pair<Second, First> {
        return Pair<Second, First>(second, first)
    }

    func mapFirst<T>(_ f: (First) -> T) -> Pair<T, Second> {
        return Pair<T, Second>(f(first), second)
    }
}

// MARK: - Constrained generics

protocol Measurable {
    var measure: Double { get }
}

func largest<T: Measurable>(_ items: [T]) -> T? {
    return items.max(by: { $0.measure < $1.measure })
}

func allEqual<T: Equatable>(_ lhs: [T], _ rhs: [T]) -> Bool {
    return lhs == rhs
}

extension Array where Element: Comparable {
    func clamp(min lower: Element, max upper: Element) -> [Element] {
        return map { Swift.min(upper, Swift.max(lower, $0)) }
    }

    var minMax: (min: Element, max: Element)? {
        get {
            guard let lo = self.min(), let hi = self.max() else { return nil }
            return (lo, hi)
        }
    }
}

extension Stack where Element: Equatable {
    func contains(_ element: Element) -> Bool {
        return storage.contains(element)
    }
}

// MARK: - Protocol with associated types

protocol Container {
    associatedtype Item
    var count: Int { get }
    func item(at index: Int) -> Item
    mutating func append(_ item: Item)
}

struct BoundedQueue<T>: Container {
    typealias Item = T
    private var items: [T]
    let capacity: Int

    var count: Int {
        get { return items.count }
    }

    init(capacity: Int) {
        self.capacity = capacity
        self.items = []
    }

    func item(at index: Int) -> T {
        return items[index]
    }

    mutating func append(_ item: T) {
        if items.count < capacity {
            items.append(item)
        }
    }

    mutating func dequeue() -> T? {
        guard !items.isEmpty else { return nil }
        return items.removeFirst()
    }
}
@@CASE@@ nested
// Nested types: struct inside class, enum inside struct, protocol inside extension.

let schemaVersion: Int = 2

struct Database {
    let name: String
    var isOpen: Bool

    struct Table {
        let tableName: String
        var rowCount: Int

        struct Column {
            let columnName: String
            var dataType: String
            var isNullable: Bool

            func describe() -> String {
                return "\(columnName) \(dataType)"
            }
        }

        func column(named name: String) -> Column? {
            return nil
        }

        mutating func addColumn(_ col: Column) {
            rowCount += 1
        }
    }

    enum StorageFormat {
        case inMemory
        case persistent(path: String)

        struct Options {
            var pageSize: Int
            var cacheSize: Int

            func isValid() -> Bool {
                return pageSize > 0 && cacheSize > 0
            }
        }

        var options: Options {
            get { return Options(pageSize: 4096, cacheSize: 64) }
        }
    }

    func table(named name: String) -> Table? {
        return nil
    }

    func open() {
    }
}

class DocumentStore {
    var path: String

    class Section {
        var title: String
        var items: [String]

        init(title: String) {
            self.title = title
            self.items = []
        }

        func addItem(_ item: String) {
            items.append(item)
        }

        class SubSection {
            var heading: String

            init(heading: String) {
                self.heading = heading
            }

            func render() -> String {
                return heading
            }
        }
    }

    init(path: String) {
        self.path = path
    }

    func newSection(title: String) -> Section {
        return Section(title: title)
    }
}

extension DocumentStore {
    protocol Exportable {
        func export() -> Data
        var mimeType: String { get }
    }

    func exportAll() -> [Data] {
        return []
    }
}
@@CASE@@ realworld
// A realistic networking + model layer, similar to what you'd find in a
// production iOS app.

import Foundation

let defaultTimeoutInterval: TimeInterval = 30.0
let maxRetryCount: Int = 3

// MARK: - Models

struct User {
    let id: UUID
    var username: String
    var email: String
    var avatarURL: URL?

    func displayName() -> String {
        return username
    }

    mutating func updateEmail(_ newEmail: String) {
        email = newEmail
    }
}

struct APIError {
    let code: Int
    let message: String

    var isRetryable: Bool {
        get { return code >= 500 }
    }
}

// MARK: - Networking

protocol NetworkSession {
    func data(from url: URL) async throws -> (Data, URLResponse)
    func upload(request: URLRequest, data: Data) async throws -> (Data, URLResponse)
}

protocol Endpoint {
    var path: String { get }
    var method: String { get }
    var headers: [String: String] { get }
}

class APIClient {
    let baseURL: URL
    var session: NetworkSession
    private let decoder: JSONDecoder

    init(baseURL: URL, session: NetworkSession) {
        self.baseURL = baseURL
        self.session = session
        self.decoder = JSONDecoder()
    }

    func fetch<T: Decodable>(endpoint: any Endpoint) async throws -> T {
        let url = baseURL.appendingPathComponent(endpoint.path)
        let (data, _) = try await session.data(from: url)
        return try decoder.decode(T.self, from: data)
    }

    func post<Body: Encodable, Response: Decodable>(
        endpoint: any Endpoint,
        body: Body
    ) async throws -> Response {
        var request = URLRequest(url: baseURL.appendingPathComponent(endpoint.path))
        request.httpMethod = endpoint.method
        request.httpBody = try JSONEncoder().encode(body)
        let (data, _) = try await session.upload(request: request, data: request.httpBody!)
        return try decoder.decode(Response.self, from: data)
    }
}

// MARK: - Repository

class UserRepository {
    private let client: APIClient
    private var cache: [UUID: User] = [:]

    init(client: APIClient) {
        self.client = client
    }

    func fetchUser(id: UUID) async throws -> User {
        if let cached = cache[id] {
            return cached
        }
        let user: User = try await client.fetch(endpoint: UserEndpoint.get(id: id))
        cache[id] = user
        return user
    }

    func updateUser(_ user: User) async throws -> User {
        let updated: User = try await client.post(endpoint: UserEndpoint.update, body: user)
        cache[updated.id] = updated
        return updated
    }

    func clearCache() {
        cache.removeAll()
    }
}

enum UserEndpoint: Endpoint {
    case get(id: UUID)
    case update
    case list

    var path: String {
        switch self {
        case .get(let id): return "/users/\(id)"
        case .update: return "/users"
        case .list: return "/users"
        }
    }

    var method: String {
        switch self {
        case .get: return "GET"
        case .update: return "PUT"
        case .list: return "GET"
        }
    }

    var headers: [String: String] {
        return ["Content-Type": "application/json"]
    }
}
@@CASE@@ sample
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
@@CASE@@ types
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
@@CASE@@ unicode
// Non-ASCII identifiers, emoji, and Unicode operator names.

let приветствие: String = "Привет, мир!"
let π: Double = 3.14159265358979
let ε: Double = 0.0001
let résumé: String = "curriculum vitae"

struct Координата {
    var широта: Double
    var долгота: Double

    func описание() -> String {
        return "(\(широта), \(долгота))"
    }

    mutating func сдвинуть(дельта: Double) {
        широта += дельта
    }
}

class ПространствоИмён {
    var имя: String
    var дочерние: [ПространствоИмён]

    init(имя: String) {
        self.имя = имя
        self.дочерние = []
    }

    func добавить(_ ребёнок: ПространствоИмён) {
        дочерние.append(ребёнок)
    }
}

protocol Описываемый {
    func описать() -> String
    var название: String { get }
}

enum Направление {
    case север
    case юг
    case восток
    case запад

    func противоположное() -> Направление {
        switch self {
        case .север: return .юг
        case .юг: return .север
        case .восток: return .запад
        case .запад: return .восток
        }
    }
}

// Japanese identifiers
struct 点 {
    var x座標: Double
    var y座標: Double

    func 距離(から other: 点) -> Double {
        let dx = x座標 - other.x座標
        let dy = y座標 - other.y座標
        return (dx * dx + dy * dy).squareRoot()
    }
}

// Arabic identifier
let رسالة: String = "مرحبا"

extension 点: Описываемый {
    func описать() -> String {
        return "(\(x座標), \(y座標))"
    }

    var название: String {
        get { return "点" }
    }
}
@@CASE@@ prefix_postfix_nonisolated_modifiers
struct S {
    prefix static func - (v: S) -> S { return v }
    static func + (a: S, b: S) -> S { return a }
}
postfix func +++ (value: Int) -> Int { return value + 1 }
actor A {
    nonisolated func count() -> Int { return 0 }
}
@@CASE@@ multiline_where_clause
func foo<T, U>() where T: Hashable,
                        U: Equatable {
}
@@CASE@@ where_on_separate_line
struct S<T>
    where T: Hashable {
    var x: T
}
@@CASE@@ emoji_identifier
let 🎉: String = "x"
let ok: String = "x"
@@CASE@@ macro_declaration
macro foo() -> Int = #externalMacro(module: "M", type: "T")
func bar() -> Int { return 0 }
@@CASE@@ package_access_modifier
struct Before {
    var beforeMember: Int
}
package struct Pkg {
    var pkgMember: String
}
@@CASE@@ private_set_modifier
struct S {
    private(set) var x: Int
    var y: Int
}
