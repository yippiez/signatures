// A realistic HTTP client and repository layer modelled on a Flutter app.

import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;

const String kBaseUrl = 'https://api.example.com/v1';
const int kDefaultTimeout = 30;
final Uri _defaultUri = Uri.parse(kBaseUrl);

/// Represents a paginated API response envelope.
class ApiResponse<T> {
  final T data;
  final int statusCode;
  final String? message;

  const ApiResponse({required this.data, required this.statusCode, this.message});

  factory ApiResponse.fromJson(Map<String, dynamic> json, T Function(dynamic) fromJson) {
    return ApiResponse(
      data: fromJson(json['data']),
      statusCode: json['status'] as int,
      message: json['message'] as String?,
    );
  }

  bool get isSuccess => statusCode >= 200 && statusCode < 300;

  @override
  String toString() => 'ApiResponse($statusCode, $message)';
}

/// A user entity returned by the API.
class User {
  final int id;
  final String name;
  final String email;
  DateTime? lastLogin;

  User({required this.id, required this.name, required this.email, this.lastLogin});

  factory User.fromJson(Map<String, dynamic> json) {
    return User(
      id: json['id'] as int,
      name: json['name'] as String,
      email: json['email'] as String,
      lastLogin: json['last_login'] != null
          ? DateTime.parse(json['last_login'] as String)
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'email': email,
      'last_login': lastLogin?.toIso8601String(),
    };
  }
}

/// HTTP client wrapper with retry and timeout.
class ApiClient {
  final http.Client _inner;
  final Duration timeout;
  static const int maxRetries = 3;

  ApiClient({http.Client? client, this.timeout = const Duration(seconds: kDefaultTimeout)})
      : _inner = client ?? http.Client();

  Future<Map<String, dynamic>> get(String path, {Map<String, String>? headers}) async {
    final uri = Uri.parse('$kBaseUrl$path');
    final response = await _inner.get(uri, headers: headers).timeout(timeout);
    return _parseResponse(response);
  }

  Future<Map<String, dynamic>> post(String path, {required Map<String, dynamic> body}) async {
    final uri = Uri.parse('$kBaseUrl$path');
    final response = await _inner.post(
      uri,
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode(body),
    ).timeout(timeout);
    return _parseResponse(response);
  }

  Map<String, dynamic> _parseResponse(http.Response response) {
    if (response.statusCode >= 400) {
      throw ApiException(response.statusCode, response.body);
    }
    return jsonDecode(response.body) as Map<String, dynamic>;
  }

  void close() => _inner.close();
}

/// Repository for user-related API operations.
class UserRepository {
  final ApiClient _client;
  final Map<int, User> _cache = {};

  UserRepository(this._client);

  Future<User> fetchUser(int id) async {
    if (_cache.containsKey(id)) return _cache[id]!;
    final json = await _client.get('/users/$id');
    final user = User.fromJson(json);
    _cache[id] = user;
    return user;
  }

  Future<List<User>> fetchAll({int page = 1, int perPage = 20}) async {
    final json = await _client.get('/users?page=$page&per_page=$perPage');
    final list = (json['items'] as List).cast<Map<String, dynamic>>();
    return list.map(User.fromJson).toList();
  }

  Future<User> updateUser(int id, {String? name, String? email}) async {
    final body = <String, dynamic>{};
    if (name != null) body['name'] = name;
    if (email != null) body['email'] = email;
    final json = await _client.post('/users/$id', body: body);
    return User.fromJson(json);
  }

  void invalidate(int id) => _cache.remove(id);
}

/// Thrown when the API returns a 4xx/5xx response.
class ApiException implements Exception {
  final int statusCode;
  final String body;

  const ApiException(this.statusCode, this.body);

  @override
  String toString() => 'ApiException($statusCode): $body';
}

List<T> paginate<T>(List<T> items, {required int page, required int size}) {
  return items.skip((page - 1) * size).take(size).toList();
}
