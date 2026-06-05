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
