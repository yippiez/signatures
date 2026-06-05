class Container<T> {
  identity<U>(x: U): U { return x; }
  map<U>(fn: (x: T) => U): U[] { return []; }
  plain(a: number): number { return a; }
}

function firstOf<T>(items: T[]): T { return items[0]; }
