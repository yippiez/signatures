// operators.cpp — operator overloads, const/constexpr/static constants,
//                 struct/class/enum/enum class, and method varieties

#include <cstddef>
#include <string>
#include <ostream>

namespace math {

// ----------------------------------------------------------------
// Constants — constexpr, const, and static const
// ----------------------------------------------------------------

constexpr double PI      = 3.141592653589793;
constexpr double E       = 2.718281828459045;
constexpr double SQRT2   = 1.4142135623730951;
constexpr int    MAX_DIM = 4;

static const std::string LIBRARY_NAME = "math-utils";

// ----------------------------------------------------------------
// Plain enum (C-style)
// ----------------------------------------------------------------

enum Axis {
    AXIS_X = 0,
    AXIS_Y = 1,
    AXIS_Z = 2
};

// ----------------------------------------------------------------
// enum class
// ----------------------------------------------------------------

enum class Norm {
    L1,
    L2,
    LInf
};

// ----------------------------------------------------------------
// Struct with operator overloads
// ----------------------------------------------------------------

struct Vec2 {
    double x = 0.0;
    double y = 0.0;

    Vec2() = default;
    Vec2(double x, double y);

    // Arithmetic operators
    Vec2 operator+(const Vec2& rhs) const;
    Vec2 operator-(const Vec2& rhs) const;
    Vec2 operator*(double scalar) const;
    Vec2 operator/(double scalar) const;
    Vec2& operator+=(const Vec2& rhs);
    Vec2& operator-=(const Vec2& rhs);

    // Comparison operators
    bool operator==(const Vec2& rhs) const;
    bool operator!=(const Vec2& rhs) const;

    // Index operator
    double operator[](int i) const;
    double& operator[](int i);

    // Unary
    Vec2 operator-() const;

    // Conversion
    explicit operator bool() const;

    double length() const;
    double dot(const Vec2& other) const;
    Vec2   normalized() const;
};

// Non-member arithmetic (scalar * vec)
Vec2 operator*(double scalar, const Vec2& v);

// Stream output
std::ostream& operator<<(std::ostream& os, const Vec2& v);

// ----------------------------------------------------------------
// Class with static members and constexpr methods
// ----------------------------------------------------------------

class Vec3 {
public:
    static constexpr int DIM = 3;
    static const Vec3    ZERO;
    static const Vec3    UNIT_X;
    static const Vec3    UNIT_Y;
    static const Vec3    UNIT_Z;

    Vec3() = default;
    Vec3(double x, double y, double z);

    double x() const;
    double y() const;
    double z() const;

    double& x();
    double& y();
    double& z();

    Vec3 operator+(const Vec3& rhs) const;
    Vec3 operator-(const Vec3& rhs) const;
    Vec3 operator*(double s) const;
    Vec3& operator+=(const Vec3& rhs);
    Vec3& operator-=(const Vec3& rhs);
    bool  operator==(const Vec3& rhs) const;
    bool  operator!=(const Vec3& rhs) const;
    Vec3  operator-() const;
    double operator[](int i) const;
    double& operator[](int i);

    double length()    const;
    double lengthSq()  const;
    Vec3   normalize() const;
    double dot(const Vec3& o)   const;
    Vec3   cross(const Vec3& o) const;

    Norm dominant_norm() const;
    double norm(Norm kind) const;

private:
    double data_[3] = {};
};

Vec3 operator*(double s, const Vec3& v);
std::ostream& operator<<(std::ostream& os, const Vec3& v);

// ----------------------------------------------------------------
// Matrix class
// ----------------------------------------------------------------

class Mat3 {
public:
    static constexpr int ROWS = 3;
    static constexpr int COLS = 3;
    static const Mat3 IDENTITY;

    Mat3() = default;

    double  operator()(int r, int c) const;
    double& operator()(int r, int c);

    Mat3 operator+(const Mat3& rhs) const;
    Mat3 operator*(const Mat3& rhs) const;
    Vec3 operator*(const Vec3& v)   const;
    Mat3 operator*(double s)        const;
    bool operator==(const Mat3& rhs) const;

    Mat3   transpose() const;
    double determinant() const;
    Mat3   inverse() const;
    bool   invertible() const;

private:
    double m_[3][3] = {};
};

std::ostream& operator<<(std::ostream& os, const Mat3& m);

} // namespace math
