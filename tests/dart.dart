@@CASE@@ abstract_mixin_enum
// Covers abstract classes, mixins, enums, and the sealed modifier.

import 'dart:math' show Point;

/// Severity levels for log entries.
enum LogLevel {
  trace,
  debug,
  info,
  warning,
  error,
  fatal;

  bool get isAtLeast(LogLevel other) => index >= other.index;

  String get label => name.toUpperCase();
}

/// Cardinal directions with associated degrees.
enum Direction {
  north(0),
  east(90),
  south(180),
  west(270);

  final int degrees;
  const Direction(this.degrees);

  Direction get opposite => Direction.values[(index + 2) % 4];
}

/// Base shape contract.
abstract class Shape {
  const Shape();

  double get area;
  double get perimeter;

  bool contains(Point<double> p);

  @override
  String toString() => '${runtimeType}(area=${area.toStringAsFixed(2)})';
}

/// Mixin that adds serialization capability.
mixin Serializable {
  Map<String, dynamic> toJson();

  String toJsonString() => '{...}';

  static T? tryParse<T>(String json, T Function(Map<String, dynamic>) fromJson) {
    return null;
  }
}

/// Mixin for objects that can be cached by a string key.
mixin Cacheable {
  String get cacheKey;
  Duration get cacheTtl => const Duration(minutes: 5);

  bool isCacheStale(DateTime cachedAt) {
    return DateTime.now().difference(cachedAt) > cacheTtl;
  }
}

/// Abstract sealed base for result types.
abstract class Result<T> {
  const Result();

  bool get isOk;
  T get valueOrThrow;

  R fold<R>({required R Function(T) onOk, required R Function(Object) onError});
}

class Ok<T> extends Result<T> {
  final T value;
  const Ok(this.value);

  @override
  bool get isOk => true;

  @override
  T get valueOrThrow => value;

  @override
  R fold<R>({required R Function(T) onOk, required R Function(Object) onError}) {
    return onOk(value);
  }
}

class Err<T> extends Result<T> {
  final Object error;
  const Err(this.error);

  @override
  bool get isOk => false;

  @override
  T get valueOrThrow => throw error;

  @override
  R fold<R>({required R Function(T) onOk, required R Function(Object) onError}) {
    return onError(error);
  }
}

/// Concrete shape implementing the abstract class and two mixins.
class Circle extends Shape with Serializable, Cacheable {
  final double radius;
  static const double pi = 3.141592653589793;

  const Circle(this.radius);

  @override
  double get area => pi * radius * radius;

  @override
  double get perimeter => 2 * pi * radius;

  @override
  bool contains(Point<double> p) {
    return p.x * p.x + p.y * p.y <= radius * radius;
  }

  @override
  Map<String, dynamic> toJson() => {'radius': radius};

  @override
  String get cacheKey => 'circle:$radius';
}

/// Abstract mixin-class (Dart 3 style).
abstract mixin class Disposable {
  bool get isDisposed;
  void dispose();
  void assertNotDisposed() {
    if (isDisposed) throw StateError('Already disposed');
  }
}
@@CASE@@ comments_strings
// This file contains fake declarations inside comments and strings that
// must NOT appear in the signatures output.

// class FakeInLineComment { void shouldNotAppear() {} }
/* class FakeInBlockComment {
   int alsoFake(int x) => x;
}
*/

/**
 * class FakeInJavadocStyle {
 *   final String nope = '';
 * }
 */

/// Real top-level constant.
const String kAppName = 'MyApp';

/// Real class — its doc comment contains a fake declaration.
///
/// Example usage:
/// ```dart
/// class FakeInDocComment { void fake() {} }
/// final bad = FakeInDocComment();
/// ```
class Documented {
  /// This field is real.
  final int id;

  // Not a declaration: class Phantom { }
  Documented(this.id);

  /// Returns a string. See also: `void fakeMethod() {}` (not real).
  String label() => 'doc-$id';

  /*
   * Block comment inside class:
   * abstract class AlsoFake extends Object {
   *   int notReal();
   * }
   */
  void realMethod() {}
}

// Strings with embedded fake declarations.
const String raw1 = r'class InRawString { const x = 1; }';
const String raw2 = r"void inRawString(int a, int b) => a + b;";
const String interp = 'class InPlainString { void fake() {} }';
const String multiSingle = 'The pattern '
    'class Fake {} '
    'spans lines but is just a string';

/// Triple-quoted strings should also be masked.
const String tripleDouble = """
class FakeInTripleDouble {
  int shouldNotAppear() => 42;
}
""";

const String tripleSingle = '''
abstract class FakeInTripleSingle {
  void alsoFake();
}
''';

/// Real function after all the comment/string noise.
int realAdd(int a, int b) => a + b;

/// Real class with getters and setters.
class Settings {
  int _timeout = 30;
  bool _debug = false;

  int get timeout => _timeout;
  set timeout(int value) {
    _timeout = value.clamp(1, 300);
  }

  bool get debug => _debug;
  set debug(bool value) => _debug = value;

  // This comment has a fake: void notReal() {}
  String get summary => 'timeout=$_timeout debug=$_debug';
}
@@CASE@@ edge
// Edge cases and a slightly malformed-but-parseable file.
// Tests: empty class bodies, one-liner arrow methods, trailing commas,
// annotations, very long parameter lists, factory constructors,
// incomplete braces at EOF (parser must not panic).

import 'dart:async';

// ── Annotations ──────────────────────────────────────────────────────────────

@deprecated
const String oldApiUrl = 'https://old.example.com';

@pragma('vm:prefer-inline')
const int inlineHint = 42;

// ── Empty bodies ─────────────────────────────────────────────────────────────

/// A class with no members.
class Empty {}

/// Abstract class with only abstract methods.
abstract class Marker {
  void mark();
  bool isMarked();
}

// ── Factories and named constructors ─────────────────────────────────────────

class Token {
  final String kind;
  final String value;
  final int line;

  const Token(this.kind, this.value, this.line);

  factory Token.eof() => const Token('EOF', '', -1);
  factory Token.identifier(String name, int line) => Token('ID', name, line);
  factory Token.number(String raw, int line) => Token('NUM', raw, line);

  @override
  String toString() => 'Token($kind, $value)';

  Token copyWith({String? kind, String? value, int? line}) {
    return Token(kind ?? this.kind, value ?? this.value, line ?? this.line);
  }
}

// ── Long / multi-line parameter lists ────────────────────────────────────────

class HttpRequest {
  final String method;
  final Uri url;
  final Map<String, String> headers;
  final String? body;
  final Duration timeout;
  final bool followRedirects;
  final int maxRedirects;

  const HttpRequest({
    required this.method,
    required this.url,
    this.headers = const {},
    this.body,
    this.timeout = const Duration(seconds: 30),
    this.followRedirects = true,
    this.maxRedirects = 5,
  });

  HttpRequest get(String path) => copyWith(method: 'GET', url: url.resolve(path));
  HttpRequest post(String path, {String? body}) =>
      copyWith(method: 'POST', url: url.resolve(path), body: body);

  HttpRequest copyWith({
    String? method,
    Uri? url,
    Map<String, String>? headers,
    String? body,
    Duration? timeout,
    bool? followRedirects,
    int? maxRedirects,
  }) {
    return HttpRequest(
      method: method ?? this.method,
      url: url ?? this.url,
      headers: headers ?? this.headers,
      body: body ?? this.body,
      timeout: timeout ?? this.timeout,
      followRedirects: followRedirects ?? this.followRedirects,
      maxRedirects: maxRedirects ?? this.maxRedirects,
    );
  }
}

// ── Async / stream / future ───────────────────────────────────────────────────

Stream<int> countUp(int to) async* {
  for (int i = 0; i <= to; i++) {
    yield i;
    await Future.delayed(const Duration(milliseconds: 10));
  }
}

Future<void> delay(int ms) => Future.delayed(Duration(milliseconds: ms));

// ── Typedef ───────────────────────────────────────────────────────────────────

typedef Predicate<T> = bool Function(T value);
typedef AsyncMapper<A, B> = Future<B> Function(A input);
typedef Json = Map<String, dynamic>;

// ── Extension ────────────────────────────────────────────────────────────────

extension StringX on String {
  bool get isBlank => trim().isEmpty;
  String capitalize() => isEmpty ? this : '${this[0].toUpperCase()}${substring(1)}';
  int? toIntOrNull() => int.tryParse(this);
}

extension ListX<T> on List<T> {
  T? firstOrNull() => isEmpty ? null : first;
  List<T> separated(T separator) {
    if (isEmpty) return [];
    return [for (int i = 0; i < length; i++) ...[if (i > 0) separator, this[i]]];
  }
}

// ── Slightly malformed: mismatched braces at end (extra close) ────────────────
// The extractor must not panic; it may mis-indent the last item.

class Dangling {
  final int x;
  Dangling(this.x);
  int doubled() => x * 2;
@@CASE@@ expression_bodies
class Calc {
  int add(int a, int b) => a + b;
  String greet(String name) => "hi $name";
  int block(int n) {
    return n;
  }
}

int square(int x) => x * x;
@@CASE@@ generics
// Covers generic classes, generic functions, bounded type params, variance.

import 'dart:collection';

const int kMaxCacheSize = 256;

/// A generic pair holding two values of potentially different types.
class Pair<A, B> {
  final A first;
  final B second;

  const Pair(this.first, this.second);

  Pair<B, A> swap() => Pair(second, first);

  @override
  String toString() => 'Pair($first, $second)';
}

/// A triple.
class Triple<A, B, C> extends Pair<A, B> {
  final C third;

  const Triple(super.first, super.second, this.third);

  @override
  String toString() => 'Triple($first, $second, $third)';
}

/// Generic stack with bounded capacity.
class BoundedStack<T> {
  final int capacity;
  final List<T> _data = [];

  BoundedStack({this.capacity = 32});

  bool push(T item) {
    if (_data.length >= capacity) return false;
    _data.add(item);
    return true;
  }

  T? pop() => _data.isEmpty ? null : _data.removeLast();

  T? peek() => _data.isEmpty ? null : _data.last;

  bool get isFull => _data.length >= capacity;
  bool get isEmpty => _data.isEmpty;
  int get size => _data.length;
}

/// LRU cache backed by a linked hash map.
class LruCache<K, V> {
  final int maxSize;
  final LinkedHashMap<K, V> _map = LinkedHashMap();

  LruCache(this.maxSize);

  V? get(K key) {
    if (!_map.containsKey(key)) return null;
    return _map[key];
  }

  void put(K key, V value) {
    _map.remove(key);
    _map[key] = value;
    if (_map.length > maxSize) {
      _map.remove(_map.keys.first);
    }
  }

  bool containsKey(K key) => _map.containsKey(key);

  int get length => _map.length;
}

/// Generic tree node.
class TreeNode<T> {
  final T value;
  TreeNode<T>? left;
  TreeNode<T>? right;

  TreeNode(this.value, {this.left, this.right});

  bool get isLeaf => left == null && right == null;

  int get height {
    if (isLeaf) return 0;
    return 1 + [left?.height ?? -1, right?.height ?? -1].reduce((a, b) => a > b ? a : b);
  }
}

/// Generic free-standing functions.
T identity<T>(T value) => value;

Pair<A, B> zip<A, B>(A a, B b) => Pair(a, b);

List<B> mapList<A, B>(List<A> items, B Function(A) f) => items.map(f).toList();

R? tryCatch<R>(R Function() block) {
  try {
    return block();
  } catch (_) {
    return null;
  }
}

/// Bounded generic function.
T maxOf<T extends Comparable<T>>(T a, T b) => a.compareTo(b) >= 0 ? a : b;

List<T> flatten<T>(List<List<T>> nested) => [for (final sub in nested) ...sub];

/// A generic repository interface.
abstract class Repository<T, ID extends Comparable<ID>> {
  Future<T?> findById(ID id);
  Future<List<T>> findAll({int limit = 100, int offset = 0});
  Future<T> save(T entity);
  Future<void> deleteById(ID id);
  Future<bool> existsById(ID id);
}

/// In-memory implementation of the generic repository.
class InMemoryRepository<T, ID extends Comparable<ID>> implements Repository<T, ID> {
  final ID Function(T) idOf;
  final Map<ID, T> _store = {};

  InMemoryRepository({required this.idOf});

  @override
  Future<T?> findById(ID id) async => _store[id];

  @override
  Future<List<T>> findAll({int limit = 100, int offset = 0}) async {
    return _store.values.skip(offset).take(limit).toList();
  }

  @override
  Future<T> save(T entity) async {
    return entity;
  }

  @override
  Future<void> deleteById(ID id) async => _store.remove(id);

  @override
  Future<bool> existsById(ID id) async => _store.containsKey(id);
}
@@CASE@@ nested
// Demonstrates deeply nested class members and inner class patterns.

const int kVersion = 2;

class Outer {
  static const String tag = 'Outer';
  final int id;

  Outer(this.id);

  int get value => id * 2;

  void process() {
    // Inner logic here; this body is a function body.
    doWork(id);
  }

  String describe(String label, {bool verbose = false}) {
    return '$tag($label)';
  }

  class Inner {
    static const String tag = 'Inner';
    final Outer parent;

    Inner(this.parent);

    int compute(int x, int y) => x + y + parent.id;

    void reset() {}

    class DeepNested {
      final String name;
      const DeepNested(this.name);

      bool isValid() => name.isNotEmpty;
    }
  }

  class Sibling {
    final double weight;
    Sibling(this.weight);

    double scale(double factor) => weight * factor;
  }
}

class Container<T extends Comparable<T>> {
  final List<T> _items = [];
  static const int defaultCapacity = 16;

  Container();

  void add(T item) => _items.add(item);

  T? find(bool Function(T) predicate) {
    for (final item in _items) {
      if (predicate(item)) return item;
    }
    return null;
  }

  int get length => _items.length;

  bool get isEmpty => _items.isEmpty;

  class View<U> {
    final Container<T> source;
    final U Function(T) transform;

    View({required this.source, required this.transform});

    List<U> toList() => source._items.map(transform).toList();
  }
}

void topLevel(int a, int b) {}
@@CASE@@ realworld
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
@@CASE@@ sample
const int max = 100;
final String name = "app";

class Greeter {
  final String name;
  Greeter(this.name);

  String greet() => "hi $name";
  Future<void> load() async {}
}

abstract class Shape {
  double area();
}

int add(int a, int b) => a + b;
@@CASE@@ unicode
// Non-ASCII (Unicode) identifiers in Dart — fully valid per the language spec.
// Tests that the signature extractor handles multi-byte characters safely.

/// Top-level constant with a Unicode name.
const double π = 3.141592653589793;
const double τ = 2 * π;
const String kÄpfel = 'apples';

/// Class with non-ASCII name: Geometrie (German for Geometry).
class Geometrie {
  final double breite;   // width
  final double höhe;     // height

  const Geometrie({required this.breite, required this.höhe});

  double get fläche => breite * höhe;    // area
  double get umfang => 2 * (breite + höhe);  // perimeter
}

/// Circle using π as a field name.
class Kreis {
  static const double π = 3.141592653589793;
  final double radius;

  const Kreis(this.radius);

  double berechneFlaeche() => π * radius * radius;
  double berechneUmfang() => 2 * π * radius;
}

/// Abstract class using Japanese-inspired names.
abstract class 図形 {
  double get 面積;       // area
  double get 周囲長;     // perimeter
  bool 含む(double x, double y);   // contains
}

/// Spanish-influenced names.
class Configuración {
  final String nombre;
  final int máximo;
  final int mínimo;

  const Configuración({
    required this.nombre,
    this.máximo = 100,
    this.mínimo = 0,
  });

  bool esVálido(int valor) {
    return valor >= mínimo && valor <= máximo;
  }
}

/// Mixin with accented identifiers.
mixin Étiquetable {
  String get étiquette;
  String get description => 'Étiquette: $étiquette';
}

/// Enum with emoji-adjacent name and accented members.
enum Saison {
  printemps,
  été,
  automne,
  hiver;

  String get nomComplet {
    switch (this) {
      case Saison.printemps: return 'Printemps';
      case Saison.été: return 'Été';
      case Saison.automne: return 'Automne';
      case Saison.hiver: return 'Hiver';
    }
  }
}

/// Function with non-ASCII parameter names.
double calculer(double largeur, double hauteur) => largeur * hauteur;

int comparer(Comparable<dynamic> α, Comparable<dynamic> β) => α.compareTo(β);
@@CASE@@ getters
class Foo {
  int _x = 0;
  int get x => _x;
  int get y {
    const localK = 5;
    return _x + 1;
  }
  set x(int v) { _x = v; }
}
int get globalValue => 42;
@@CASE@@ operators
class Complex {
  final double re;
  Complex operator +(Complex other) => this;
  bool operator ==(Object o) => true;
}
@@CASE@@ class_modifiers
base class Animal {
  void speak() {}
}
interface class Drawable {
  void draw() {}
}
final class Config {
  final int x = 0;
}
final answer = 42;
@@CASE@@ nullable_returns_and_index_assign
class A {
  String? foo() {
    final x = 42;
    return null;
  }
  void bar() {}
  dynamic operator [](String key) => null;
  void operator []=(String key, dynamic value) {}
}
String? lookup(String key) {
  return null;
}
@@CASE@@ record_return_type
// Bug: functions with record/tuple return types were silently dropped
(int, String) topLevel() => (1, 'hi');

class Foo {
  (int, int) method() => (1, 2);
  String other() => 'ok';
}
@@CASE@@ local_vars_no_leak
// Bug: local variables at depth >=2 leaked as constants
void gen() {
  if (true) {
    final next = 1;
    var other = 2;
  }
}
@@CASE@@ redirecting_constructor
// Bug: redirecting constructor emitted spurious : this() entry
class Foo {
  Foo.named()
      : this();
  Foo();
}
