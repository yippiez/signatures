export const MAX: number = 5;

interface Shape {
  area(): number;
}

export class Circle implements Shape {
  constructor(public r: number) {}
  area(): number {
    return Math.PI * this.r ** 2;
  }
}

function add(a: number, b: number): number {
  return a + b;
}

type Pair<T> = [T, T];
