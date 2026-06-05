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
