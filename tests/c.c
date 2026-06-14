@@CASE@@ comments_strings
/*
 * comments_strings.c - Fake declarations inside comments and string/char
 * literals that must be ignored by the signatures CLI.
 *
 * struct FakeInBlockComment { int x; };
 * void fake_in_block_comment(void);
 * #define FAKE_MACRO_IN_COMMENT 999
 * enum FakeEnum { A, B };
 */

#include <stdio.h>
#include <string.h>

// int fake_line_comment_proto(int a, int b);
// struct FakeLineStruct { char c; };
// typedef int FakeLineTypedef;

#define GREETING "struct NotAStruct { int x; };"
#define DECL_STRING "void not_a_function(int fake);"
#define CHAR_LITERAL_BRACE '{'

static const char *SQL = "SELECT id FROM users WHERE active = 1;";
static const int VERSION = 3;

/* Real declarations start here */

struct Config {
    int debug;
    int verbose;
    char logfile[256];
};

typedef struct Config Config;

void print_fake_declarations(void) {
    /* enum Hidden { X = 0 }; should be ignored */
    const char *msg = "typedef int NotReal;";
    printf("%s\n", msg);
    // void also_fake(void);
    puts("struct AlsoFake { double d; };");
}

int parse_config(const char *filename, struct Config *cfg) {
    /* int nested_fake(void) { return 0; } */
    if (!filename || !cfg) return -1;
    cfg->debug = 0;
    cfg->verbose = 0;
    return 0;
}

static int validate(const struct Config *cfg) {
    (void)cfg;
    return 1;
}

/* Another block comment with fake stuff:
 * static const int FAKE_CONST = 42;
 * union FakeUnion { int a; float b; };
 */
void dump_config(const struct Config *cfg) {
    const char *fmt = "int not_printed(void);";
    (void)fmt;
    printf("debug=%d verbose=%d\n", cfg->debug, cfg->verbose);
}
@@CASE@@ edge
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
@@CASE@@ line_comment_continuation
// comment \
int hidden_fn(void) { return 0; }
int real_fn(void) { return 1; }
@@CASE@@ macros
/*
 * macros.c - Heavy use of #define constants, static consts, and
 * function-like macros (bodies elided from signatures).
 */

#include <stddef.h>
#include <stdint.h>

/* Simple constants */
#define PI              3.14159265358979323846
#define E               2.71828182845904523536
#define MAX_UINT8       255
#define MAX_UINT16      65535
#define MAX_UINT32      4294967295U
#define CACHE_LINE      64
#define PAGE_SIZE       4096

/* Bit manipulation */
#define BIT(n)          (1u << (n))
#define BITMASK(lo, hi) (((1u << ((hi) - (lo) + 1)) - 1u) << (lo))
#define SET_BIT(x, n)   ((x) |=  BIT(n))
#define CLR_BIT(x, n)   ((x) &= ~BIT(n))
#define TST_BIT(x, n)   (!!((x) & BIT(n)))

/* Min/max/clamp */
#define MIN(a, b)       ((a) < (b) ? (a) : (b))
#define MAX(a, b)       ((a) > (b) ? (a) : (b))
#define CLAMP(v, lo, hi) MIN(MAX((v), (lo)), (hi))

/* Array helpers */
#define ARRAY_LEN(arr)  (sizeof(arr) / sizeof((arr)[0]))
#define ARRAY_END(arr)  ((arr) + ARRAY_LEN(arr))

/* Alignment */
#define ALIGN_UP(x, align)   (((x) + (align) - 1) & ~((align) - 1))
#define ALIGN_DOWN(x, align) ((x) & ~((align) - 1))
#define IS_ALIGNED(x, align) (((x) & ((align) - 1)) == 0)

/* Assert / unreachable */
#define STATIC_ASSERT(cond, msg) typedef char static_assert_##msg[(cond) ? 1 : -1]
#define UNREACHABLE()   do { __builtin_unreachable(); } while (0)

/* Stringify / concatenate */
#define STRINGIFY(x)    #x
#define TOSTRING(x)     STRINGIFY(x)
#define CONCAT(a, b)    a##b

/* Static consts */
static const uint32_t FNV_OFFSET_BASIS = 2166136261u;
static const uint32_t FNV_PRIME        = 16777619u;
static const size_t   MAX_KEY_LEN      = 128;
static const size_t   HASH_TABLE_SIZE  = 1024;
static const double   GOLDEN_RATIO     = 1.61803398874989484820;

/* Struct using macro constants */
struct MemBlock {
    uint8_t  data[PAGE_SIZE];
    size_t   used;
    uint32_t checksum;
};

/* Functions */
uint32_t fnv1a_hash(const void *data, size_t len) {
    const uint8_t *p = (const uint8_t *)data;
    uint32_t h = FNV_OFFSET_BASIS;
    for (size_t i = 0; i < len; i++) {
        h ^= p[i];
        h *= FNV_PRIME;
    }
    return h;
}

size_t align_up(size_t x, size_t align) {
    return ALIGN_UP(x, align);
}

int mem_block_write(struct MemBlock *blk, const void *src, size_t n) {
    if (!blk || !src) return -1;
    if (blk->used + n > PAGE_SIZE) return -1;
    (void)src;
    blk->used += n;
    return 0;
}

static uint32_t crc32_byte(uint32_t crc, uint8_t byte) {
    (void)crc; (void)byte;
    return 0;
}

uint32_t crc32(const void *data, size_t len) {
    const uint8_t *p = (const uint8_t *)data;
    uint32_t crc = 0xFFFFFFFFu;
    for (size_t i = 0; i < len; i++)
        crc = crc32_byte(crc, p[i]);
    return crc ^ 0xFFFFFFFFu;
}
@@CASE@@ multiline_define
#define M \
    struct HiddenInMacro { int x; };
int after(void) { return 0; }
@@CASE@@ nested
/*
 * nested.c - Deeply nested structs, unions, and anonymous members.
 */

#include <stdint.h>

struct Point2D {
    float x;
    float y;
};

struct Point3D {
    float x;
    float y;
    float z;
};

union FloatInt {
    float f;
    uint32_t i;
};

struct Color {
    union {
        struct {
            uint8_t r;
            uint8_t g;
            uint8_t b;
            uint8_t a;
        } rgba;
        uint32_t packed;
    };
};

struct Transform {
    struct {
        float m00; float m01; float m02;
        float m10; float m11; float m12;
        float m20; float m21; float m22;
    } rotation;
    struct Point3D translation;
    float scale;
};

struct SceneNode {
    char name[64];
    struct Transform transform;
    struct Color tint;
    struct SceneNode *parent;
    struct SceneNode **children;
    int child_count;
};

union Register {
    uint64_t qword;
    struct {
        uint32_t lo;
        uint32_t hi;
    } dwords;
    struct {
        uint16_t w0;
        uint16_t w1;
        uint16_t w2;
        uint16_t w3;
    } words;
    uint8_t bytes[8];
};

/* Function that works with nested types */
void transform_apply(const struct Transform *t, const struct Point3D *in, struct Point3D *out) {
    (void)t; (void)in; (void)out;
}

struct Point3D point3d_add(struct Point3D a, struct Point3D b) {
    struct Point3D r;
    r.x = a.x + b.x;
    r.y = a.y + b.y;
    r.z = a.z + b.z;
    return r;
}

uint32_t color_lerp(struct Color a, struct Color b, float t) {
    (void)a; (void)b; (void)t;
    return 0;
}

void scene_node_attach(struct SceneNode *parent, struct SceneNode *child) {
    (void)parent; (void)child;
}
@@CASE@@ realworld
/*
 * realworld.c - Simulated real-world network server module
 * Covers: includes, macros, static consts, structs, enums, function
 * prototypes and definitions, mixed declarations.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

#define SERVER_VERSION "1.4.2"
#define MAX_CONNECTIONS 1024
#define DEFAULT_PORT 8080
#define BUFFER_SIZE 4096

static const int BACKLOG = 128;
static const double TIMEOUT_SEC = 30.0;
static const char *DEFAULT_HOST = "0.0.0.0";

enum ConnectionState {
    CONN_IDLE,
    CONN_HANDSHAKE,
    CONN_ACTIVE,
    CONN_CLOSING,
    CONN_ERROR
};

enum LogLevel {
    LOG_DEBUG,
    LOG_INFO,
    LOG_WARN,
    LOG_ERROR
};

struct ServerConfig {
    int port;
    int max_conn;
    double timeout;
    char host[256];
};

struct Connection {
    int fd;
    enum ConnectionState state;
    char remote_addr[64];
    int remote_port;
    char buffer[BUFFER_SIZE];
    size_t buf_len;
};

struct Server {
    int listen_fd;
    struct ServerConfig config;
    struct Connection *connections;
    int conn_count;
};

/* Prototypes */
int server_init(struct Server *srv, const struct ServerConfig *cfg);
void server_shutdown(struct Server *srv);
int server_accept(struct Server *srv);
int connection_read(struct Connection *conn);
int connection_write(struct Connection *conn, const char *data, size_t len);
void connection_close(struct Connection *conn);
static void log_message(enum LogLevel level, const char *fmt, ...);

int server_init(struct Server *srv, const struct ServerConfig *cfg) {
    if (!srv || !cfg) return -1;
    srv->listen_fd = -1;
    srv->config = *cfg;
    srv->conn_count = 0;
    srv->connections = calloc(cfg->max_conn, sizeof(struct Connection));
    if (!srv->connections) return -ENOMEM;
    return 0;
}

void server_shutdown(struct Server *srv) {
    if (!srv) return;
    for (int i = 0; i < srv->conn_count; i++) {
        connection_close(&srv->connections[i]);
    }
    free(srv->connections);
    srv->connections = NULL;
}

int server_accept(struct Server *srv) {
    /* int fake_proto(void) — this is in a comment, ignore */
    if (srv->conn_count >= srv->config.max_conn) return -ENOBUFS;
    return 0;
}

int connection_read(struct Connection *conn) {
    if (!conn || conn->state != CONN_ACTIVE) return -1;
    return (int)conn->buf_len;
}

int connection_write(struct Connection *conn, const char *data, size_t len) {
    (void)data;
    (void)len;
    if (!conn) return -1;
    return 0;
}

void connection_close(struct Connection *conn) {
    if (!conn) return;
    conn->state = CONN_CLOSING;
    conn->fd = -1;
}

static void log_message(enum LogLevel level, const char *fmt, ...) {
    (void)level;
    (void)fmt;
}
@@CASE@@ sample
#include <stdio.h>

#define MAX 100
static const int LIMIT = 5;

struct Point {
    int x;
    int y;
};

int add(int a, int b) {
    return a + b;
}

void process(struct Point *p) {
    /* void not_real(void) inside a comment */
}
@@CASE@@ types
/*
 * types.c - Typedefs, enums, and type aliases.
 */

#include <stdint.h>
#include <stdbool.h>

/* Primitive typedefs */
typedef unsigned char  u8;
typedef unsigned short u16;
typedef unsigned int   u32;
typedef unsigned long long u64;
typedef signed char    i8;
typedef signed short   i16;
typedef signed int     i32;
typedef signed long long i64;
typedef float          f32;
typedef double         f64;

/* Function pointer typedefs */
typedef int (*CompareFn)(const void *a, const void *b);
typedef void (*CallbackFn)(int event, void *user_data);
typedef char *(*AllocStrFn)(size_t n);

/* Enum with explicit values */
enum HttpStatus {
    HTTP_OK                = 200,
    HTTP_CREATED           = 201,
    HTTP_NO_CONTENT        = 204,
    HTTP_BAD_REQUEST       = 400,
    HTTP_UNAUTHORIZED      = 401,
    HTTP_FORBIDDEN         = 403,
    HTTP_NOT_FOUND         = 404,
    HTTP_INTERNAL_ERROR    = 500,
    HTTP_NOT_IMPLEMENTED   = 501,
    HTTP_SERVICE_UNAVAIL   = 503
};

typedef enum HttpStatus HttpStatus;

enum Direction {
    DIR_NORTH,
    DIR_SOUTH,
    DIR_EAST,
    DIR_WEST
};

/* Struct with typedef in one shot */
typedef struct {
    u32 id;
    u8  flags;
    u16 length;
    u8  payload[256];
} Packet;

typedef struct {
    i32 x;
    i32 y;
    i32 width;
    i32 height;
} Rect;

typedef struct Node {
    void *data;
    struct Node *next;
    struct Node *prev;
} Node;

typedef struct {
    Node *head;
    Node *tail;
    u32   count;
    CompareFn compare;
} LinkedList;

/* Union typedef */
typedef union {
    u32  raw;
    struct {
        u32 mantissa : 23;
        u32 exponent :  8;
        u32 sign     :  1;
    } parts;
} Float32Bits;

/* Functions operating on these types */
LinkedList *list_create(CompareFn cmp);
void        list_destroy(LinkedList *list);
int         list_push_front(LinkedList *list, void *data);
int         list_push_back(LinkedList *list, void *data);
void       *list_pop_front(LinkedList *list);
Node       *list_find(const LinkedList *list, const void *key);

const char *http_status_str(HttpStatus status) {
    switch (status) {
        case HTTP_OK: return "OK";
        case HTTP_NOT_FOUND: return "Not Found";
        default: return "Unknown";
    }
}

Rect rect_intersect(Rect a, Rect b) {
    (void)a; (void)b;
    Rect r = {0, 0, 0, 0};
    return r;
}

bool rect_contains(const Rect *r, i32 x, i32 y) {
    return x >= r->x && x < r->x + r->width &&
           y >= r->y && y < r->y + r->height;
}
@@CASE@@ leading_attribute_declspec
__attribute__((noinline)) int f(int x);
__declspec(dllexport) int exported(int x);
__attribute__((noreturn)) void die(void) { for(;;); }
int normal(void);
@@CASE@@ typedef_fn_ptr
// Bug: typedef of pointer-return function pointer classified as class
typedef void *(*VoidPtrFn)(int);
typedef int *(*IntPtrFn)(int);
typedef void (*VoidFn)(int);
@@CASE@@ struct_with_attribute
// Bug: struct with __attribute__ annotation classified as function
struct __attribute__((packed)) PackedOne {
    char a;
    int b;
};
@@CASE@@ multiline_const_full
// Bug: multi-line constant initializer truncated in --output full
static const int X =
    42;
int f(void) { return X; }
@@CASE@@ anon_struct_return
// Bug: function with anonymous struct return type classified as class
struct { int code; char msg[64]; } get_error(void);
struct Named { int x; };
struct Named get_named(void);
@@CASE@@ ptr_to_array_return
// Bug: functions with pointer-to-array return type dropped
int (*get_row(int matrix[][10], int row))[10];
int normal(int x);
