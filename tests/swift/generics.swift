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
