// realworld.cpp — realistic HTTP client / server mini-library

#include <cstdint>
#include <string>
#include <vector>
#include <map>
#include <functional>

#define HTTP_VERSION "1.1"
#define MAX_HEADER_SIZE 8192

namespace http {

// ------------------------------------------------------------------
// Types
// ------------------------------------------------------------------

enum class Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD
};

enum class StatusCode {
    OK              = 200,
    Created         = 201,
    NoContent       = 204,
    BadRequest      = 400,
    Unauthorized    = 401,
    NotFound        = 404,
    InternalError   = 500
};

struct Header {
    std::string name;
    std::string value;
};

class Request {
public:
    Request() = default;
    explicit Request(Method m, std::string url);

    Method method() const;
    const std::string& url() const;
    const std::string& body() const;
    std::string header(const std::string& name) const;

    void set_body(std::string b);
    void add_header(std::string name, std::string value);

    bool is_secure() const;

private:
    Method method_ = Method::GET;
    std::string url_;
    std::string body_;
    std::vector<Header> headers_;
};

class Response {
public:
    Response() = default;
    explicit Response(StatusCode code);

    StatusCode status() const;
    int status_int() const;
    const std::string& body() const;

    void set_status(StatusCode code);
    void set_body(std::string b);
    void set_header(std::string name, std::string value);

    bool ok() const;

private:
    StatusCode status_ = StatusCode::OK;
    std::string body_;
    std::map<std::string, std::string> headers_;
};

// ------------------------------------------------------------------
// Router
// ------------------------------------------------------------------

using Handler = std::function<Response(const Request&)>;

class Router {
public:
    void get(std::string path, Handler h);
    void post(std::string path, Handler h);
    void put(std::string path, Handler h);
    void del(std::string path, Handler h);

    Response dispatch(const Request& req) const;

    bool has_route(Method m, const std::string& path) const;

private:
    struct Route {
        Method method;
        std::string path;
        Handler handler;
    };
    std::vector<Route> routes_;
};

// ------------------------------------------------------------------
// Server
// ------------------------------------------------------------------

constexpr uint16_t DEFAULT_PORT = 8080;
constexpr int      BACKLOG      = 128;

class Server {
public:
    explicit Server(uint16_t port = DEFAULT_PORT);
    ~Server();

    Server(const Server&) = delete;
    Server& operator=(const Server&) = delete;

    void set_router(Router r);
    void listen();
    void stop();

    uint16_t port() const;
    bool running() const;

private:
    uint16_t port_;
    bool running_ = false;
    Router router_;
    int sock_fd_ = -1;
};

// ------------------------------------------------------------------
// Utilities
// ------------------------------------------------------------------

std::string url_encode(const std::string& s);
std::string url_decode(const std::string& s);
std::string base64_encode(const std::vector<uint8_t>& data);
std::vector<uint8_t> base64_decode(const std::string& s);

Method parse_method(const std::string& s);
StatusCode parse_status(int code);

} // namespace http
