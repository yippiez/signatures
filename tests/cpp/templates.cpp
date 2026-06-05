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
