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
