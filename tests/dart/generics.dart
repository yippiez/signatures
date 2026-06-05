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
