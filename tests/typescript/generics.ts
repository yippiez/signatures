/**
 * Generics with constraints, conditional types, mapped types, decorators,
 * utility types, and overload signatures.
 */

export const EMPTY_ARRAY: readonly never[] = [];

// ---- Interfaces with generic parameters ----

export interface Comparable<T> {
  compareTo(other: T): number;
  equals(other: T): boolean;
}

export interface Repository<T, Id> {
  findById(id: Id): Promise<T | undefined>;
  findAll(): Promise<T[]>;
  save(entity: T): Promise<T>;
  removeById(id: Id): Promise<void>;
}

export interface Transformer<In, Out> {
  transform(input: In): Out;
  transformAll(inputs: In[]): Out[];
}

// ---- Type aliases and conditional/mapped types ----

export type Nullable<T> = T | null;

export type Optional<T> = T | undefined;

export type DeepReadonly<T> = T extends (infer U)[]
  ? ReadonlyArray<DeepReadonly<U>>
  : T extends object
    ? { readonly [P in keyof T]: DeepReadonly<T[P]> }
    : T;

export type Keys<T> = keyof T;

export type Values<T> = T[keyof T];

export type PickByValue<T, V> = {
  [K in keyof T as T[K] extends V ? K : never]: T[K];
};

export type Result<T, E extends Error = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };

// ---- Enums ----

export enum Direction {
  North = "NORTH",
  South = "SOUTH",
  East = "EAST",
  West = "WEST",
}

export enum HttpStatus {
  Ok = 200,
  Created = 201,
  BadRequest = 400,
  Unauthorized = 401,
  NotFound = 404,
  InternalServerError = 500,
}

export const enum LogLevel {
  Trace = 0,
  Debug = 1,
  Info = 2,
  Warn = 3,
  Error = 4,
}

// ---- Generic classes with constraints ----

export class SortedList<T extends Comparable<T>> {
  private items: T[];

  constructor(initial?: T[]) {
    this.items = initial ? [...initial].sort((a, b) => a.compareTo(b)) : [];
  }

  insert(item: T): void {
    this.items.push(item);
    this.items.sort((a, b) => a.compareTo(b));
  }

  min(): T | undefined {
    return this.items[0];
  }

  max(): T | undefined {
    return this.items[this.items.length - 1];
  }

  toArray(): T[] {
    return [...this.items];
  }
}

export class Cache<K, V> {
  private store: Map<K, { value: V; expiresAt: number }>;

  constructor(private ttlMs: number = 60_000) {
    this.store = new Map();
  }

  set(key: K, value: V): void {
    this.store.set(key, { value, expiresAt: Date.now() + this.ttlMs });
  }

  has(key: K): boolean {
    const entry = this.store.get(key);
    return !!entry && entry.expiresAt > Date.now();
  }

  clear(): void {
    this.store.clear();
  }
}

// ---- Decorators (TypeScript experimental) ----

@sealed
export class BankAccount {
  private balance: number;

  constructor(
    public readonly owner: string,
    initialBalance: number,
  ) {
    this.balance = initialBalance;
  }

  @logCall
  deposit(amount: number): void {
    this.balance += amount;
  }

  @logCall
  withdraw(amount: number): boolean {
    if (amount > this.balance) {
      return false;
    }
    this.balance -= amount;
    return true;
  }

  get currentBalance(): number {
    return this.balance;
  }
}

// ---- Overload signatures ----

export function parseValue(input: string): number;
export function parseValue(input: number): string;
export function parseValue(input: string | number): string | number {
  if (typeof input === "string") {
    return parseFloat(input);
  }
  return input.toString();
}

// ---- Generic top-level functions ----

export function identity<T>(value: T): T {
  return value;
}

export function mapArray<T, U>(arr: T[], fn: (item: T) => U): U[] {
  return arr.map(fn);
}

export function pipe<A, B, C>(fn1: (a: A) => B, fn2: (b: B) => C): (a: A) => C {
  return (a) => fn2(fn1(a));
}

export function assertNonNull<T>(value: T | null | undefined, msg?: string): T {
  if (value == null) {
    throw new Error(msg ?? "Unexpected null or undefined");
  }
  return value;
}

// Dummy decorator implementations so file is parseable in isolation.
function sealed(target: Function): void {}
function logCall(target: unknown, key: string, descriptor: PropertyDescriptor): void {}
