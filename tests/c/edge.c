/*
 * edge.c - Edge cases and a slightly malformed-but-parseable file.
 * Covers: empty structs (GCC extension), zero-length arrays, flexible
 * array members, old-style K&R prototype hints, multiline macro defs,
 * function pointers in structs, variadic functions, attributes, and
 * unusual but valid C constructs.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

/* Multiline macro */
#define SWAP(type, a, b) \
    do {                 \
        type _tmp = (a); \
        (a) = (b);       \
        (b) = _tmp;      \
    } while (0)

#define UNUSED(x) ((void)(x))

#define LIKELY(x)   __builtin_expect(!!(x), 1)
#define UNLIKELY(x) __builtin_expect(!!(x), 0)

static const int MAGIC = 0xDEADBEEF;
static const size_t FLEX_HEADER_SIZE = offsetof(struct { int n; char d[1]; }, d);

/* Struct with function pointers */
struct Ops {
    int   (*open)(const char *path, int flags);
    int   (*close)(int fd);
    long  (*read)(int fd, void *buf, size_t n);
    long  (*write)(int fd, const void *buf, size_t n);
    int   (*ioctl)(int fd, unsigned long cmd, void *arg);
};

/* Flexible array member */
struct Packet {
    uint16_t type;
    uint16_t length;
    uint8_t  payload[];
};

/* Struct with zero-length trailing buffer (MSVC-style) */
struct Buffer {
    size_t  capacity;
    size_t  used;
    uint8_t data[0];
};

/* Bitfield struct */
struct Flags {
    unsigned int readable   : 1;
    unsigned int writable   : 1;
    unsigned int executable : 1;
    unsigned int sticky     : 1;
    unsigned int            : 4;
    unsigned int uid        : 1;
    unsigned int gid        : 1;
    unsigned int reserved   : 22;
};

/* Anonymous union inside struct */
struct Value {
    int type;
    union {
        long   ival;
        double dval;
        char  *sval;
        void  *pval;
    };
};

/* Variadic function */
int string_format(char *buf, size_t size, const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    (void)buf; (void)size; (void)fmt;
    va_end(ap);
    return 0;
}

/* Pointer-to-function return type */
typedef void (*HandlerFn)(int sig);
HandlerFn signal_set(int signum, HandlerFn handler) {
    (void)signum; (void)handler;
    return (HandlerFn)0;
}

/* const-qualified pointer params */
int memcmp_safe(const void *a, size_t la, const void *b, size_t lb) {
    (void)a; (void)la; (void)b; (void)lb;
    return 0;
}

/* Returning a struct by value */
struct Value value_make_int(long v) {
    struct Value val;
    val.type = 0;
    val.ival = v;
    return val;
}

struct Value value_make_str(char *s) {
    struct Value val;
    val.type = 2;
    val.sval = s;
    return val;
}

/* static inline */
static inline uint32_t rotl32(uint32_t x, int k) {
    return (x << k) | (x >> (32 - k));
}

/* K&R-style parameter list (old-style, still valid C89) */
int old_style_add(a, b)
    int a;
    int b;
{
    return a + b;
}
