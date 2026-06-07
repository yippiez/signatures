@@CASE@@ comments_strings
// This file tests that declarations inside comments and strings are ignored.

// function ignoredInLineComment(x, y) {}
// class IgnoredClass {}
// const IGNORED_CONST = 42;

/*
 * Block comment with fake declarations:
 * function blockCommentFn(a, b) { return a + b; }
 * class BlockCommentClass { constructor() {} }
 * const BLOCK_COMMENT_CONST = "hello";
 * async function asyncInComment() {}
 */

const REAL_CONST = "I am real";

function parseTemplate(template) {
  // The next line is a comment, not code:
  // const fakeConst = `function notReal() {}`;
  const fake = "function alsoFake(x) { return x; }";
  const alsoFake = 'class FakeClass { method() {} }';
  const tmplFake = `
    function templateFake() {}
    class TemplateClass {
      constructor() {}
      method() {}
    }
    const TEMPLATE_CONST = 99;
  `;
  return template.replace(/\{\{.*?\}\}/g, "");
}

class StringHolder {
  constructor() {
    // class FakeInComment extends Base {}
    this.single = 'function notAFunction() {}';
    this.double = "const NOT_CONST = true;";
    this.backtick = `class NotAClass { get prop() { return 1; } }`;
  }

  getSingle() {
    /* function multilineCommentFn() {
         return 42;
       }
    */
    return this.single;
  }

  getDouble() {
    return this.double;
  }

  getBacktick() {
    return this.backtick;
  }
}

function realFunction(a, b) {
  const msg = "class Embedded { constructor(x) { this.x = x; } }";
  const tmpl = `function ${a}() { return ${b}; }`;
  return msg + tmpl;
}

const arrowReal = (x) => x * 2;
@@CASE@@ edge
// Edge cases: destructuring, default params, rest/spread, computed methods,
// immediately invoked, multiple assignments

export const CONFIG = Object.freeze({ debug: false, version: 3 });

// Multiline arrow assigned to const
export const transform = (input, options = {}) =>
  input.toString().trim();

export const compose = (...fns) => x => fns.reduceRight((v, f) => f(v), x);

class Validator {
  #rules = new Map();

  constructor({ strict = false, maxLength = 255 } = {}) {
    this.strict = strict;
    this.maxLength = maxLength;
  }

  addRule(name, fn) {
    this.#rules.set(name, fn);
    return this;
  }

  validate(value, ruleName) {
    const rule = this.#rules.get(ruleName);
    if (!rule) throw new Error(`Unknown rule: ${ruleName}`);
    return rule(value);
  }

  get ruleCount() {
    return this.#rules.size;
  }

  [Symbol.iterator]() {
    return this.#rules.entries();
  }
}

class ExtendedArray extends Array {
  constructor(...args) {
    super(...args);
  }

  first() {
    return this[0];
  }

  last() {
    return this[this.length - 1];
  }

  groupBy(keyFn) {
    return this.reduce((acc, item) => {
      const key = keyFn(item);
      (acc[key] = acc[key] || []).push(item);
      return acc;
    }, {});
  }
}

function* fibonacci() {
  let [a, b] = [0, 1];
  while (true) {
    yield a;
    [a, b] = [b, a + b];
  }
}

export function parseArgs(argv = process.argv.slice(2)) {
  const flags = {};
  const positional = [];
  for (const arg of argv) {
    if (arg.startsWith('--')) {
      const [key, val = true] = arg.slice(2).split('=');
      flags[key] = val;
    } else {
      positional.push(arg);
    }
  }
  return { flags, positional };
}

export default Validator;
@@CASE@@ malformed
// Malformed-but-parseable JavaScript: unclosed generics-like syntax,
// trailing commas in odd places, extra semicolons, weird spacing

const   WEIRD_SPACING   = "extra spaces around";
const TRAILING = {
  a: 1,
  b: 2,
};

// Missing semicolons throughout (ASI handles it)
const noSemi = 42
const alsoNoSemi = "hello"

class BrokenIndent {
constructor(x,y) {
this.x=x
this.y=y
}
    weirdMethod  (  a , b  ) {
   return a+b
    }
static   fromPair([a,b]) {
return new BrokenIndent(a,b)
}
}

function multipleReturns(x) {
if (x > 0) { return x }
if (x < 0) { return -x }
return 0
}

// Empty function
function empty() {}

// Arrow with no body parens
const double = x => x * 2

// Async arrow
const wait = async (ms) => new Promise(r => setTimeout(r, ms))

class EmptyClass {}

class Oneliner { constructor() {} greet() { return "hi" } }

function trailingCommaParams(
  a,
  b,
  c,
) {
  return a + b + c;
}

const obj = {
  method() {
    return 1;
  },
  arrowProp: () => 2,
};
@@CASE@@ modern
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
@@CASE@@ nested
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
@@CASE@@ realworld
/**
 * A realistic HTTP client module with classes, exports, and constants.
 * function notASignature() -- inside block comment, should be ignored
 */

import { EventEmitter } from 'events';

export const DEFAULT_TIMEOUT = 5000;
export const BASE_URL = "https://api.example.com";
const MAX_RETRIES = 3;

export class HttpError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
    this.name = "HttpError";
  }

  isClientError() {
    return this.status >= 400 && this.status < 500;
  }

  isServerError() {
    return this.status >= 500;
  }

  toString() {
    return `${this.name}: ${this.status} ${this.message}`;
  }
}

export class HttpClient extends EventEmitter {
  constructor(baseUrl, options) {
    super();
    this.baseUrl = baseUrl || BASE_URL;
    this.timeout = options?.timeout || DEFAULT_TIMEOUT;
    this._interceptors = [];
  }

  get defaultHeaders() {
    return {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };
  }

  set authToken(token) {
    this._token = token;
  }

  addInterceptor(fn) {
    this._interceptors.push(fn);
  }

  async get(path, params) {
    return this._request('GET', path, null, params);
  }

  async post(path, body) {
    return this._request('POST', path, body);
  }

  async put(path, body) {
    return this._request('PUT', path, body);
  }

  async delete(path) {
    return this._request('DELETE', path);
  }

  async _request(method, path, body, params) {
    const url = new URL(this.baseUrl + path);
    if (params) {
      Object.entries(params).forEach(([k, v]) => url.searchParams.set(k, v));
    }
    let attempt = 0;
    while (attempt < MAX_RETRIES) {
      try {
        const res = await fetch(url, {
          method,
          headers: this.defaultHeaders,
          body: body ? JSON.stringify(body) : undefined,
        });
        if (!res.ok) throw new HttpError(res.status, await res.text());
        return await res.json();
      } catch (err) {
        attempt++;
        if (attempt >= MAX_RETRIES) throw err;
      }
    }
  }
}

export function createClient(baseUrl, options) {
  return new HttpClient(baseUrl, options);
}

export default HttpClient;
@@CASE@@ sample
const MAX = 100;
export const NAME = "app";

export class Greeter {
  constructor(name) {
    this.name = name;
  }
  greet() {
    return `hi ${this.name}`;
  }
  async load() {}
}

function add(a, b) {
  // function notReal() in a comment
  return a + b;
}

const mul = (a, b) => a * b;
@@CASE@@ unicode
// Unicode identifiers: non-ASCII variable, function, and class names

const π = Math.PI;
const τ = 2 * π;
const ε = 1e-10;

function вычислить(значение, коэффициент) {
  return значение * коэффициент;
}

function calculerAire(rayon) {
  return π * rayon * rayon;
}

class Ñoño {
  constructor(nombre) {
    this.nombre = nombre;
  }

  saludar() {
    return `¡Hola, ${this.nombre}!`;
  }

  get descripción() {
    return `Soy ${this.nombre}`;
  }

  set descripción(valor) {
    this.nombre = valor;
  }
}

class 数学Utils {
  static 加法(甲, 乙) {
    return 甲 + 乙;
  }

  static 乗法(甲, 乙) {
    return 甲 * 乙;
  }
}

const μ = (σ, n) => σ / Math.sqrt(n);

const ƒormatter = new Intl.NumberFormat('de-DE');

function résoudre(équation, variable) {
  return équation.solve(variable);
}

async function récupérerDonnées(url) {
  const réponse = await fetch(url);
  return réponse.json();
}
@@CASE@@ arrow_comparison_ops
const f = (a, b) => a > b;
const g = (a, b) => a < b;
const h = (x) => x >= 0;
@@CASE@@ computed_method_names
class Foo {
  normalMethod() { return 1; }
  [Symbol.iterator]() {}
  static [Symbol.hasInstance](obj) { return false; }
}
