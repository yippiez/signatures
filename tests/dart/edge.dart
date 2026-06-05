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
