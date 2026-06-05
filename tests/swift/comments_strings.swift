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
