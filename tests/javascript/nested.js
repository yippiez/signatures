// Nested classes, functions, and closures

const ROOT_KEY = "root";

class Outer {
  constructor(value) {
    this.value = value;
  }

  getInner() {
    class Inner {
      constructor(x) {
        this.x = x;
      }
      compute() {
        return this.x * 2;
      }
    }
    return new Inner(this.value);
  }

  makeCounter() {
    let count = 0;
    function increment() {
      count += 1;
      return count;
    }
    function reset() {
      count = 0;
    }
    return { increment, reset };
  }

  static create(value) {
    return new Outer(value);
  }
}

function pipeline(...fns) {
  return function execute(input) {
    return fns.reduce((acc, fn) => fn(acc), input);
  };
}

const memoize = (fn) => {
  const cache = new Map();
  return function memoized(...args) {
    const key = JSON.stringify(args);
    if (cache.has(key)) return cache.get(key);
    const result = fn(...args);
    cache.set(key, result);
    return result;
  };
};

function makeAdder(x) {
  return function add(y) {
    return x + y;
  };
}

class EventBus {
  constructor() {
    this._handlers = {};
  }

  on(event, handler) {
    if (!this._handlers[event]) {
      this._handlers[event] = [];
    }
    this._handlers[event].push(handler);
    return () => this.off(event, handler);
  }

  off(event, handler) {
    if (this._handlers[event]) {
      this._handlers[event] = this._handlers[event].filter(h => h !== handler);
    }
  }

  emit(event, ...args) {
    (this._handlers[event] || []).forEach(h => h(...args));
  }
}
