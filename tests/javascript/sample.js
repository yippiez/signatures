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
