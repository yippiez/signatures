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
