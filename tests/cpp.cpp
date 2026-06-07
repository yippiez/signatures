@@CASE@@ comments_strings
// comments_strings.cpp — fake declarations inside comments and strings must be ignored

#include <string>

namespace tricks {

// The following line comment contains fake code that must NOT appear:
// int fake_line_comment_fn(int x) { return x; }
// class FakeClass { void method(); };

/*
 * Block comment with fake declarations:
 * struct FakeStruct { int field; };
 * namespace fake_ns { void fake_fn(); }
 * enum FakeEnum { A, B, C };
 */

constexpr int REAL_CONST = 42;

class StringHolder {
public:
    explicit StringHolder();

    // Returns a string that looks like code but is just data
    std::string get_fake_code() const;

    std::string get_raw_fake() const;

    bool is_valid() const;

    // Operator inside class (string containing fake decl in body — ignored)
    StringHolder operator+(const StringHolder& other) const;
    bool operator==(const StringHolder& other) const;

private:
    // member_ holds strings that contain declaration-like text — not declarations
    std::string member_;
    std::string raw_member_;
};

// Real function after the class
int real_function(int a, int b);

/*
 * Another block comment.
 * void another_fake(bool x);
 * class AnotherFake {};
 */

struct RealStruct {
    int x;
    int y;
    // int fake_field_comment — not a real field
    double z;
};

// Inline comment on a real declaration: // struct FakeInline {};
enum class Color {
    Red,
    Green,
    Blue
};

// Real standalone functions after all the noise
double compute(double a, double b);
void   process(const RealStruct& s);

} // namespace tricks
@@CASE@@ constructors
class Widget {
public:
    Widget();
    Widget(int value);
    Widget(const Widget& other);
    Widget(Widget&& other) noexcept;
    ~Widget();
};
@@CASE@@ edge
// edge.cpp — one malformed-but-parseable file with edge cases

// Missing closing brace at top level (malformed but parseable)
namespace open_ns {

// Empty struct
struct Empty {};

// Struct with only static constexpr members
struct Constants {
    static constexpr int    ZERO  = 0;
    static constexpr double PI    = 3.14;
    static constexpr bool   FALSE = false;
};

// Function declared with trailing return type
auto make_pair(int a, int b) -> std::pair<int,int>;

// Const pointer vs pointer to const
const int* pointer_to_const(int* p);
int* const const_pointer(int* p);

// Multiple return types via output params
void multi_out(int in, int* out1, double* out2, bool* ok);

// Variadic C-style
int printf_like(const char* fmt, ...);

// Deeply nested type in return
std::map<std::string, std::vector<int>> build_index(const std::vector<std::string>& words);

// Unnamed enum (C-style)
enum {
    FLAG_A = 1 << 0,
    FLAG_B = 1 << 1,
    FLAG_C = 1 << 2
};

// Typedef'd struct
typedef struct {
    int code;
    char message[256];
} ErrorInfo;

// Function that takes a function pointer
void register_callback(void (*cb)(int, void*), void* userdata);

// constexpr function
constexpr int factorial(int n);

// static const at namespace scope (not a constexpr — should be detected by const+equals)
static const int LIMIT = 100;

// Class with unusual features
class Peculiar {
public:
    Peculiar() = default;
    ~Peculiar() = default;
    Peculiar(const Peculiar&) = delete;
    Peculiar(Peculiar&&) = default;

    // Conversion operator
    explicit operator bool() const;
    explicit operator int()  const;

    // Nested typedef
    typedef int value_t;

    // Pure virtual
    virtual void update(float dt) = 0;
    virtual int  priority() const = 0;

    // Non-virtual with default arg
    void reset(int mode = 0);

protected:
    int state_ = 0;
};

// Deliberately un-closed namespace (malformed — parser must not panic)
namespace unclosed {

struct Dangling {
    int a;
    int b;
};

int dangle(int x);

// File ends here without closing brace for 'unclosed' or 'open_ns'
@@CASE@@ nested
// nested.cpp — deeply nested namespaces and classes

namespace corp {

namespace engine {

namespace render {

namespace detail {

struct Vertex {
    float x, y, z;
    float nx, ny, nz;
    float u, v;
};

struct Triangle {
    unsigned a, b, c;
};

class Mesh {
public:
    Mesh() = default;

    void add_vertex(Vertex v);
    void add_triangle(Triangle t);

    unsigned vertex_count() const;
    unsigned triangle_count() const;

    bool empty() const;
    void clear();

    class Builder {
    public:
        Builder& vertex(float x, float y, float z);
        Builder& normal(float nx, float ny, float nz);
        Builder& uv(float u, float v);
        Builder& triangle(unsigned a, unsigned b, unsigned c);
        Mesh build();

    private:
        Mesh mesh_;
    };

private:
    std::vector<Vertex>   vertices_;
    std::vector<Triangle> triangles_;
};

} // namespace detail

enum class BlendMode {
    Opaque,
    Alpha,
    Additive,
    Multiply
};

class Material {
public:
    explicit Material(std::string name);

    const std::string& name() const;
    void set_blend(BlendMode b);
    BlendMode blend() const;

    class Param {
    public:
        explicit Param(std::string key);
        void set_float(float v);
        void set_int(int v);
        void set_string(std::string v);
        const std::string& key() const;

    private:
        std::string key_;
    };

    void add_param(Param p);

private:
    std::string name_;
    BlendMode blend_ = BlendMode::Opaque;
    std::vector<Param> params_;
};

class Scene {
public:
    void add_mesh(detail::Mesh m, Material mat);
    void remove_mesh(unsigned id);
    unsigned mesh_count() const;

    class Node {
    public:
        Node(unsigned id, detail::Mesh m, Material mat);

        unsigned id() const;
        const detail::Mesh& mesh() const;
        const Material& material() const;

    private:
        unsigned id_;
        detail::Mesh mesh_;
        Material material_;
    };

private:
    std::vector<Node> nodes_;
    unsigned next_id_ = 0;
};

} // namespace render

} // namespace engine

namespace physics {

struct AABB {
    float min_x, min_y, min_z;
    float max_x, max_y, max_z;
};

class RigidBody {
public:
    explicit RigidBody(float mass);

    float mass() const;
    void apply_force(float fx, float fy, float fz);
    void step(float dt);

    AABB bounding_box() const;

private:
    float mass_;
    float vx_ = 0, vy_ = 0, vz_ = 0;
    float px_ = 0, py_ = 0, pz_ = 0;
};

} // namespace physics

} // namespace corp
@@CASE@@ operator_overloads
struct Foo {
    Foo operator+(const Foo& o) const;
    Foo& operator+=(const Foo& o);
    bool operator==(const Foo& o) const;
    bool operator<(const Foo& o) const;
    Foo& operator[](std::size_t i);
    Foo* operator->();
    Foo operator()(int x);
    Foo& operator=(const Foo& o);
    explicit operator bool() const;
};
std::ostream& operator<<(std::ostream& os, const Foo& f);
@@CASE@@ operators
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
@@CASE@@ raw_string_literal
void before();
const char* code = R"(
void fake() {}
{ { {
)";
const char* tagged = R"sql(
SELECT } ; FROM t
)sql";
void after();
@@CASE@@ realworld
// realworld.cpp — realistic HTTP client / server mini-library

#include <cstdint>
#include <string>
#include <vector>
#include <map>
#include <functional>

#define HTTP_VERSION "1.1"
#define MAX_HEADER_SIZE 8192

namespace http {

// ------------------------------------------------------------------
// Types
// ------------------------------------------------------------------

enum class Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD
};

enum class StatusCode {
    OK              = 200,
    Created         = 201,
    NoContent       = 204,
    BadRequest      = 400,
    Unauthorized    = 401,
    NotFound        = 404,
    InternalError   = 500
};

struct Header {
    std::string name;
    std::string value;
};

class Request {
public:
    Request() = default;
    explicit Request(Method m, std::string url);

    Method method() const;
    const std::string& url() const;
    const std::string& body() const;
    std::string header(const std::string& name) const;

    void set_body(std::string b);
    void add_header(std::string name, std::string value);

    bool is_secure() const;

private:
    Method method_ = Method::GET;
    std::string url_;
    std::string body_;
    std::vector<Header> headers_;
};

class Response {
public:
    Response() = default;
    explicit Response(StatusCode code);

    StatusCode status() const;
    int status_int() const;
    const std::string& body() const;

    void set_status(StatusCode code);
    void set_body(std::string b);
    void set_header(std::string name, std::string value);

    bool ok() const;

private:
    StatusCode status_ = StatusCode::OK;
    std::string body_;
    std::map<std::string, std::string> headers_;
};

// ------------------------------------------------------------------
// Router
// ------------------------------------------------------------------

using Handler = std::function<Response(const Request&)>;

class Router {
public:
    void get(std::string path, Handler h);
    void post(std::string path, Handler h);
    void put(std::string path, Handler h);
    void del(std::string path, Handler h);

    Response dispatch(const Request& req) const;

    bool has_route(Method m, const std::string& path) const;

private:
    struct Route {
        Method method;
        std::string path;
        Handler handler;
    };
    std::vector<Route> routes_;
};

// ------------------------------------------------------------------
// Server
// ------------------------------------------------------------------

constexpr uint16_t DEFAULT_PORT = 8080;
constexpr int      BACKLOG      = 128;

class Server {
public:
    explicit Server(uint16_t port = DEFAULT_PORT);
    ~Server();

    Server(const Server&) = delete;
    Server& operator=(const Server&) = delete;

    void set_router(Router r);
    void listen();
    void stop();

    uint16_t port() const;
    bool running() const;

private:
    uint16_t port_;
    bool running_ = false;
    Router router_;
    int sock_fd_ = -1;
};

// ------------------------------------------------------------------
// Utilities
// ------------------------------------------------------------------

std::string url_encode(const std::string& s);
std::string url_decode(const std::string& s);
std::string base64_encode(const std::vector<uint8_t>& data);
std::vector<uint8_t> base64_decode(const std::string& s);

Method parse_method(const std::string& s);
StatusCode parse_status(int code);

} // namespace http
@@CASE@@ sample
#include <vector>

namespace app {

constexpr int MAX = 10;

template <typename T>
class Vec {
public:
    T at(int i) const { return data_[i]; }
private:
    std::vector<T> data_;
};

struct Point { int x, y; };

int add(int a, int b) { return a + b; }

}  // namespace app
@@CASE@@ templates
// templates.cpp — templates with multi-line signatures and specialisations

#include <cstddef>
#include <type_traits>
#include <utility>

namespace tpl {

// Single-line template function
template <typename T>
T clamp(T value, T lo, T hi);

// Multi-line template function signature
template <
    typename InputIt,
    typename OutputIt,
    typename UnaryPredicate
>
OutputIt copy_if_transformed(InputIt first,
                             InputIt last,
                             OutputIt d_first,
                             UnaryPredicate pred);

// Template class with nested template member
template <typename T, std::size_t N>
class StaticArray {
public:
    using value_type      = T;
    using size_type       = std::size_t;
    using reference       = T&;
    using const_reference = const T&;

    StaticArray() = default;

    template <typename... Args>
    explicit StaticArray(Args&&... args);

    reference       at(size_type i);
    const_reference at(size_type i) const;

    reference       operator[](size_type i);
    const_reference operator[](size_type i) const;

    size_type size() const;
    bool      empty() const;

    T*       begin();
    T*       end();
    const T* begin() const;
    const T* end() const;

    template <typename Compare>
    void sort(Compare comp);

private:
    T data_[N];
};

// Full specialization
template <>
class StaticArray<bool, 0> {
public:
    std::size_t size() const;
    bool        empty() const;
};

// Template alias
template <typename T>
using Pair = StaticArray<T, 2>;

// Non-type template parameter
template <int Base, int Exp>
struct Power {
    static constexpr int value = Base * Power<Base, Exp - 1>::value;
};

// Variadic template
template <typename First, typename... Rest>
struct TypeList {
    using head = First;
    using tail = TypeList<Rest...>;

    static constexpr std::size_t size = 1 + sizeof...(Rest);
};

// Template with concept-style enable_if
template <
    typename T,
    typename = std::enable_if_t<std::is_arithmetic_v<T>>
>
class NumericRange {
public:
    NumericRange(T lo, T hi);

    T lo() const;
    T hi() const;
    T length() const;
    bool contains(T v) const;
    bool overlaps(const NumericRange& other) const;

private:
    T lo_, hi_;
};

// Standalone template functions
template <typename T>
constexpr T square(T x);

template <typename Container>
typename Container::value_type sum(const Container& c);

template <typename T, typename U>
auto safe_cast(U value) -> T;

} // namespace tpl
@@CASE@@ unicode
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
