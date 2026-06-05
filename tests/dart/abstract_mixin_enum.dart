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
