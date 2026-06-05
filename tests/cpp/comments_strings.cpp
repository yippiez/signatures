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
