// A realistic networking + model layer, similar to what you'd find in a
// production iOS app.

import Foundation

let defaultTimeoutInterval: TimeInterval = 30.0
let maxRetryCount: Int = 3

// MARK: - Models

struct User {
    let id: UUID
    var username: String
    var email: String
    var avatarURL: URL?

    func displayName() -> String {
        return username
    }

    mutating func updateEmail(_ newEmail: String) {
        email = newEmail
    }
}

struct APIError {
    let code: Int
    let message: String

    var isRetryable: Bool {
        get { return code >= 500 }
    }
}

// MARK: - Networking

protocol NetworkSession {
    func data(from url: URL) async throws -> (Data, URLResponse)
    func upload(request: URLRequest, data: Data) async throws -> (Data, URLResponse)
}

protocol Endpoint {
    var path: String { get }
    var method: String { get }
    var headers: [String: String] { get }
}

class APIClient {
    let baseURL: URL
    var session: NetworkSession
    private let decoder: JSONDecoder

    init(baseURL: URL, session: NetworkSession) {
        self.baseURL = baseURL
        self.session = session
        self.decoder = JSONDecoder()
    }

    func fetch<T: Decodable>(endpoint: any Endpoint) async throws -> T {
        let url = baseURL.appendingPathComponent(endpoint.path)
        let (data, _) = try await session.data(from: url)
        return try decoder.decode(T.self, from: data)
    }

    func post<Body: Encodable, Response: Decodable>(
        endpoint: any Endpoint,
        body: Body
    ) async throws -> Response {
        var request = URLRequest(url: baseURL.appendingPathComponent(endpoint.path))
        request.httpMethod = endpoint.method
        request.httpBody = try JSONEncoder().encode(body)
        let (data, _) = try await session.upload(request: request, data: request.httpBody!)
        return try decoder.decode(Response.self, from: data)
    }
}

// MARK: - Repository

class UserRepository {
    private let client: APIClient
    private var cache: [UUID: User] = [:]

    init(client: APIClient) {
        self.client = client
    }

    func fetchUser(id: UUID) async throws -> User {
        if let cached = cache[id] {
            return cached
        }
        let user: User = try await client.fetch(endpoint: UserEndpoint.get(id: id))
        cache[id] = user
        return user
    }

    func updateUser(_ user: User) async throws -> User {
        let updated: User = try await client.post(endpoint: UserEndpoint.update, body: user)
        cache[updated.id] = updated
        return updated
    }

    func clearCache() {
        cache.removeAll()
    }
}

enum UserEndpoint: Endpoint {
    case get(id: UUID)
    case update
    case list

    var path: String {
        switch self {
        case .get(let id): return "/users/\(id)"
        case .update: return "/users"
        case .list: return "/users"
        }
    }

    var method: String {
        switch self {
        case .get: return "GET"
        case .update: return "PUT"
        case .list: return "GET"
        }
    }

    var headers: [String: String] {
        return ["Content-Type": "application/json"]
    }
}
