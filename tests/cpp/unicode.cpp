// unicode.cpp — non-ASCII identifiers in namespaces, classes, functions, constants

#include <string>
#include <vector>

// Japanese-style namespace and class names
namespace 数学 {

constexpr double 円周率 = 3.14159265358979;
constexpr double 自然対数の底 = 2.71828182845905;

struct ベクトル {
    double x;
    double y;
    double z;
};

class 行列 {
public:
    行列() = default;
    explicit 行列(int 行数, int 列数);

    double 要素(int 行, int 列) const;
    void   設定(int 行, int 列, double 値);

    int 行数() const;
    int 列数() const;

    bool 正方形か() const;

private:
    std::vector<double> データ_;
    int 行数_ = 0;
    int 列数_ = 0;
};

double 内積(const ベクトル& a, const ベクトル& b);
ベクトル 外積(const ベクトル& a, const ベクトル& b);
double ノルム(const ベクトル& v);

} // namespace 数学

// German-style names with umlauts
namespace Größen {

enum class Einheit {
    Meter,
    Kilogramm,
    Sekunde,
    Ampere,
    Kelvin
};

struct Größe {
    double Wert;
    Einheit Einheit_;
};

class Messung {
public:
    explicit Messung(std::string Beschreibung);

    void hinzufügen(Größe g);
    double Durchschnitt() const;
    double Minimum() const;
    double Maximum() const;
    std::size_t Anzahl() const;

private:
    std::string Beschreibung_;
    std::vector<Größe> Werte_;
};

Größe addieren(const Größe& a, const Größe& b);
bool  gleich(const Größe& a, const Größe& b);

} // namespace Größen

// Mixed ASCII and non-ASCII
namespace util {

constexpr int MAX_Größe = 1024;

class Übersetzer {
public:
    explicit Übersetzer(std::string sprache);

    std::string übersetzen(const std::string& text) const;
    bool unterstützt(const std::string& sprache) const;

private:
    std::string sprache_;
};

} // namespace util
