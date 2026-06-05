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
