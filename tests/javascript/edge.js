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
