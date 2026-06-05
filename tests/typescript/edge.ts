/**
 * Edge cases: arrow functions as constants, abstract classes, intersection /
 * union types, index signatures, optional chaining in types, rest params,
 * destructured params, and re-exports.
 */

// Arrow functions stored as constants (should appear as Function kind).
export const greet = (name: string): string => `Hello, ${name}!`;

export const add = (a: number, b: number): number => a + b;

export const noop = (): void => {};

export const compose = <T>(fns: Array<(x: T) => T>): ((x: T) => T) =>
  fns.reduce((f, g) => (x) => f(g(x)));

// Plain value constants (no arrow — Constant kind).
export const ZERO: number = 0;
export const EMPTY: string = "";
export const FLAGS: readonly string[] = ["verbose", "debug", "trace"];

// Abstract class.
export abstract class Shape {
  abstract area(): number;
  abstract perimeter(): number;

  describe(): string {
    return `area=${this.area()} perimeter=${this.perimeter()}`;
  }
}

// Concrete subclass.
export class Square extends Shape {
  constructor(public side: number) {
    super();
  }

  area(): number {
    return this.side * this.side;
  }

  perimeter(): number {
    return 4 * this.side;
  }
}

// Index signatures in interfaces.
export interface StringMap {
  [key: string]: string;
  readonly size: number;
}

export interface EventRegistry {
  [eventName: string]: Array<(...args: unknown[]) => void>;
}

// Intersection and union types.
export type Serializable = object & { toJSON(): string };

export type StringOrNumber = string | number;

export type AnyPrimitive = string | number | boolean | null | undefined;

// Tuple types.
export type Point2D = [number, number];

export type Entry<K, V> = [key: K, value: V];

// Rest parameters and destructured params.
export function logAll(...messages: string[]): void {
  messages.forEach((m) => console.log(m));
}

export function mergeObjects(
  target: Record<string, unknown>,
  ...sources: Record<string, unknown>[]
): Record<string, unknown> {
  return Object.assign(target, ...sources);
}

export function swapPair([a, b]: [number, number]): [number, number] {
  return [b, a];
}

// Function overloads with complex signatures.
export function coerce(value: string): number;
export function coerce(value: number): string;
export function coerce(value: boolean): number;
export function coerce(value: string | number | boolean): string | number {
  if (typeof value === "boolean") {
    return value ? 1 : 0;
  }
  if (typeof value === "string") {
    return Number(value);
  }
  return String(value);
}

// Type predicates and assertion functions.
export function isString(value: unknown): value is string {
  return typeof value === "string";
}

export function assertIsNumber(value: unknown): asserts value is number {
  if (typeof value !== "number") {
    throw new TypeError("Expected number");
  }
}

// Class with private fields (#) and static block.
export class Counter {
  static #instances: number = 0;
  #count: number;

  constructor(start: number = 0) {
    this.#count = start;
    Counter.#instances++;
  }

  increment(by: number = 1): this {
    this.#count += by;
    return this;
  }

  reset(): void {
    this.#count = 0;
  }

  get value(): number {
    return this.#count;
  }

  static instanceCount(): number {
    return Counter.#instances;
  }
}

// Immediately exported type-only constructs.
export type Awaited<T> = T extends Promise<infer U> ? U : T;
export type Flatten<T> = T extends Array<infer U> ? U : T;
