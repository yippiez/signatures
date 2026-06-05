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
