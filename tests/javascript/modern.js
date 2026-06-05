// Modern JS: async/await, generators, getters/setters, static fields, private fields

export const VERSION = "2.0.0";

export class AsyncQueue {
  #items = [];
  #waiting = [];

  static empty() {
    return new AsyncQueue();
  }

  get size() {
    return this.#items.length;
  }

  get isEmpty() {
    return this.#items.length === 0;
  }

  set capacity(n) {
    this._cap = n;
  }

  enqueue(item) {
    if (this.#waiting.length > 0) {
      const resolve = this.#waiting.shift();
      resolve(item);
    } else {
      this.#items.push(item);
    }
  }

  async dequeue() {
    if (this.#items.length > 0) {
      return this.#items.shift();
    }
    return new Promise(resolve => this.#waiting.push(resolve));
  }

  async *[Symbol.asyncIterator]() {
    while (true) {
      yield await this.dequeue();
    }
  }
}

export async function fetchAll(urls) {
  return Promise.all(urls.map(url => fetch(url).then(r => r.json())));
}

export async function* streamLines(reader) {
  const decoder = new TextDecoder();
  let buffer = "";
  for await (const chunk of reader) {
    buffer += decoder.decode(chunk, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop();
    for (const line of lines) {
      yield line;
    }
  }
  if (buffer) yield buffer;
}

export function* range(start, end, step = 1) {
  for (let i = start; i < end; i += step) {
    yield i;
  }
}

export function* zip(...iterables) {
  const iterators = iterables.map(it => it[Symbol.iterator]());
  while (true) {
    const results = iterators.map(it => it.next());
    if (results.some(r => r.done)) return;
    yield results.map(r => r.value);
  }
}

const retry = async (fn, times = 3) => {
  for (let i = 0; i < times; i++) {
    try { return await fn(); } catch (e) { if (i === times - 1) throw e; }
  }
};

const debounce = (fn, delay) => {
  let timer;
  return function debounced(...args) {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(this, args), delay);
  };
};
