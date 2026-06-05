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
