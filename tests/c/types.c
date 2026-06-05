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
