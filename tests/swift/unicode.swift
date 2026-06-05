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
