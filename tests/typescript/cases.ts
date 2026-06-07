@@CASE@@ comments_strings
/**
 * Declarations inside comments and string/template literals must be ignored.
 *
 * interface FakeInJsDoc { hidden(): void; }
 * function notAFunction(x: number): string { return ""; }
 * class InvisibleClass {}
 * const GHOST = 42;
 */

// const IGNORED_LINE_COMMENT: number = 99;
// function lineCommentFn(x: string): void {}
// class LineCommentClass {}
// interface LineCommentInterface {}

export const REAL_CONSTANT: number = 1;

/*
 * Block comment with fake declarations:
 * function blockCommentFn(): void {}
 * class BlockCommentClass {}
 * const BLOCK_GHOST = 0;
 * type BlockType = string;
 * enum BlockEnum { A, B }
 */

export interface RealInterface {
  realMethod(x: number): string;
}

export class RealClass {
  // This comment has: function innerCommentFn(): void {}
  private label: string;

  constructor(label: string) {
    this.label = label;
  }

  describe(): string {
    // function localFake(): number { return 0; }
    // const FAKE_LOCAL = "nope";
    return `label=${this.label}`;
  }

  getTemplate(): string {
    // Template literal below contains fake declarations — all must be ignored.
    const tmpl = `
      function templateFn(): void {}
      class TemplateClass {}
      const TEMPLATE_CONST = 1;
      interface TemplateIface {}
    `;
    return tmpl;
  }

  getSingleQuoted(): string {
    const sq = 'function singleQuoteFn(): void {} class SQClass {} const SQ = 0;';
    return sq;
  }

  getDoubleQuoted(): string {
    const dq = "function doubleQuoteFn(): void {} interface DQIface {} type DQType = number;";
    return dq;
  }
}

// Real function after several comment-only lines.
// function commentedOut(a: number): number { return a + 1; }
export function liveFunction(a: number, b: number): number {
  return a + b;
}

export type RealType = string | number;

export enum RealEnum {
  Alpha,
  Beta,
  Gamma,
}

/* const ANOTHER_GHOST = "ignore me"; interface GhostIface {} */

export const ANOTHER_REAL: string = "present";
@@CASE@@ edge
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
@@CASE@@ generic_methods
class Container<T> {
  identity<U>(x: U): U { return x; }
  map<U>(fn: (x: T) => U): U[] { return []; }
  plain(a: number): number { return a; }
}

function firstOf<T>(items: T[]): T { return items[0]; }
@@CASE@@ generics
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
@@CASE@@ malformed
/**
 * Malformed-but-parseable TypeScript.
 * Syntax is intentionally bent: unclosed generics recovered on the same line,
 * missing semicolons, dangling keywords, mixed indentation, re-opened braces
 * mid-declaration. The extractor must not panic on any of this.
 */

// Missing semicolons (JS/TS allows this via ASI).
export const API_KEY: string = "abc-123"
export const TIMEOUT: number = 30

// Dangling type annotation with no closing brace on same parse path.
export interface Partial {
  id: number
  name: string
  tags: string[]
}

// Class with oddly indented members.
export class Misaligned {
        constructor(public x: number, public y: number) {}
  getValue(): number { return this.x + this.y; }
         scale(factor: number): Misaligned { return new Misaligned(this.x * factor, this.y * factor); }
}

// Multiple statements on a single line (bad style but valid).
export const A: number = 1; export const B: number = 2; export const C: number = 3;

// Enum without trailing comma on last member.
export enum Status {
  Pending = "pending",
  Active = "active",
  Closed = "closed"
}

// Type alias with excessively long union — tests line-join limit.
export type LongUnion =
  | "alpha"
  | "beta"
  | "gamma"
  | "delta"
  | "epsilon"
  | "zeta";

// Function with no return type annotation.
export function legacy(a, b) {
  return a + b;
}

// Arrow with implicit return and no parens around single param.
export const double = n => n * 2;

// Nested ternary in type position (valid TS but visually confusing).
export type Flip<T> = T extends true ? false : true;

// Class with duplicate member names (TS error but parseable).
export class Duplicate {
  constructor(private value: number) {}

  compute(): number {
    return this.value;
  }

  compute(): string {
    return String(this.value);
  }
}

// Interface that extends multiple interfaces with long line.
export interface Mega extends Comparable, Serializable, Cloneable, Iterable {
  megaMethod(a: number, b: string, c: boolean, d: object, e: unknown[]): Promise<void>;
}

// Deeply nested generic that stresses the angle-bracket tracker.
export type Nested<A, B, C> = Map<A, Map<B, Set<C>>>;

// Function with destructured default parameter.
export function withDefaults({ x = 0, y = 0, z = 0 }: { x?: number; y?: number; z?: number } = {}): string {
  return `${x},${y},${z}`;
}

// Immediately-invoked-looking arrow stored as const.
export const getVersion = (): string => "1.0.0";

// A class with a static initializer block comment (parseable TS).
export class Registry {
  private static entries: Map<string, unknown> = new Map();

  static register(key: string, value: unknown): void {
    Registry.entries.set(key, value);
  }

  static lookup(key: string): unknown {
    return Registry.entries.get(key);
  }
}
@@CASE@@ nested
/**
 * Nested namespaces and classes.
 * Covers: namespace, nested namespace, classes inside namespaces,
 * inner classes, static members, depth-3 nesting.
 */

export const VERSION: string = "2.0.0";

export namespace Geometry {
  export const PI: number = 3.14159265358979;

  export interface Point {
    x: number;
    y: number;
  }

  export interface Rect {
    origin: Point;
    width: number;
    height: number;
  }

  export class Vector2D {
    constructor(public x: number, public y: number) {}

    add(other: Vector2D): Vector2D {
      return new Vector2D(this.x + other.x, this.y + other.y);
    }

    scale(factor: number): Vector2D {
      return new Vector2D(this.x * factor, this.y * factor);
    }

    magnitude(): number {
      return Math.sqrt(this.x * this.x + this.y * this.y);
    }
  }

  export namespace Shapes {
    export const DEFAULT_COLOR: string = "black";

    export interface Drawable {
      draw(ctx: CanvasRenderingContext2D): void;
      boundingBox(): Rect;
    }

    export class Circle implements Drawable {
      static readonly unitCircle: Circle = new Circle({ x: 0, y: 0 }, 1);

      constructor(public center: Point, public radius: number) {}

      area(): number {
        return PI * this.radius * this.radius;
      }

      draw(ctx: CanvasRenderingContext2D): void {
        ctx.arc(this.center.x, this.center.y, this.radius, 0, 2 * PI);
      }

      boundingBox(): Rect {
        return {
          origin: { x: this.center.x - this.radius, y: this.center.y - this.radius },
          width: this.radius * 2,
          height: this.radius * 2,
        };
      }
    }

    export class Rectangle implements Drawable {
      constructor(
        public origin: Point,
        public width: number,
        public height: number,
      ) {}

      area(): number {
        return this.width * this.height;
      }

      perimeter(): number {
        return 2 * (this.width + this.height);
      }

      draw(ctx: CanvasRenderingContext2D): void {
        ctx.rect(this.origin.x, this.origin.y, this.width, this.height);
      }

      boundingBox(): Rect {
        return { origin: this.origin, width: this.width, height: this.height };
      }
    }

    export namespace Transform {
      export const IDENTITY_MATRIX: number[] = [1, 0, 0, 1, 0, 0];

      export interface Transformable {
        translate(dx: number, dy: number): void;
        rotate(angle: number): void;
        scale(sx: number, sy: number): void;
      }

      export class AffineTransform implements Transformable {
        private matrix: number[];

        constructor(matrix?: number[]) {
          this.matrix = matrix ?? [...IDENTITY_MATRIX];
        }

        translate(dx: number, dy: number): void {
          this.matrix[4] += dx;
          this.matrix[5] += dy;
        }

        rotate(angle: number): void {
          const cos = Math.cos(angle);
          const sin = Math.sin(angle);
          this.matrix = [cos, sin, -sin, cos, 0, 0];
        }

        scale(sx: number, sy: number): void {
          this.matrix[0] *= sx;
          this.matrix[3] *= sy;
        }

        toArray(): number[] {
          return [...this.matrix];
        }
      }
    }
  }
}

export namespace Config {
  export const DEFAULT_LOCALE: string = "en-US";

  export namespace Logging {
    export const LOG_LEVEL: string = "info";

    export class Logger {
      private name: string;

      constructor(name: string) {
        this.name = name;
      }

      info(msg: string): void {
        console.log(`[${this.name}] INFO: ${msg}`);
      }

      warn(msg: string): void {
        console.warn(`[${this.name}] WARN: ${msg}`);
      }

      error(msg: string, err?: Error): void {
        console.error(`[${this.name}] ERROR: ${msg}`, err);
      }
    }
  }
}

export function createLogger(name: string): Config.Logging.Logger {
  return new Config.Logging.Logger(name);
}
@@CASE@@ realworld
/**
 * Real-world HTTP API client module.
 * Covers: exported constants, interfaces, classes, async methods, getters.
 */

export const BASE_URL: string = "https://api.example.com/v1";
export const DEFAULT_TIMEOUT_MS: number = 5000;
export const MAX_RETRIES: number = 3;

export interface RequestOptions {
  timeout?: number;
  retries?: number;
  headers?: Record<string, string>;
}

export interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

export interface UserProfile {
  id: string;
  username: string;
  email: string;
  createdAt: Date;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

export class ApiError extends Error {
  constructor(
    public readonly statusCode: number,
    public readonly body: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }

  get isClientError(): boolean {
    return this.statusCode >= 400 && this.statusCode < 500;
  }

  get isServerError(): boolean {
    return this.statusCode >= 500;
  }
}

export class HttpClient {
  private readonly baseUrl: string;
  private defaultHeaders: Record<string, string>;

  constructor(baseUrl: string = BASE_URL, token?: string) {
    this.baseUrl = baseUrl;
    this.defaultHeaders = { "Content-Type": "application/json" };
    if (token) {
      this.defaultHeaders["Authorization"] = `Bearer ${token}`;
    }
  }

  get endpoint(): string {
    return this.baseUrl;
  }

  async fetchJson(path: string, opts?: RequestOptions): Promise<unknown> {
    const url = this.baseUrl + path;
    const resp = await fetch(url, { headers: this.defaultHeaders });
    if (!resp.ok) {
      throw new ApiError(resp.status, await resp.text(), "fetch failed");
    }
    return resp.json();
  }

  async sendJson(path: string, body: unknown, opts?: RequestOptions): Promise<unknown> {
    const resp = await fetch(this.baseUrl + path, {
      method: "POST",
      headers: this.defaultHeaders,
      body: JSON.stringify(body),
    });
    if (!resp.ok) {
      throw new ApiError(resp.status, await resp.text(), "send failed");
    }
    return resp.json();
  }

  async removeResource(path: string): Promise<void> {
    const resp = await fetch(this.baseUrl + path, { method: "DELETE" });
    if (!resp.ok) {
      throw new ApiError(resp.status, await resp.text(), "remove failed");
    }
  }
}

export class UserService {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }

  async getUser(id: string): Promise<UserProfile> {
    return this.client.fetchJson(`/users/${id}`) as Promise<UserProfile>;
  }

  async createUser(profile: Omit<UserProfile, "id" | "createdAt">): Promise<UserProfile> {
    return this.client.sendJson("/users", profile) as Promise<UserProfile>;
  }

  async removeUser(id: string): Promise<void> {
    await this.client.removeResource(`/users/${id}`);
  }
}

export function buildClient(token: string): HttpClient {
  return new HttpClient(BASE_URL, token);
}

export async function fetchUserById(id: string, token: string): Promise<UserProfile> {
  const svc = new UserService(buildClient(token));
  return svc.getUser(id);
}
@@CASE@@ sample
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
@@CASE@@ unicode
/**
 * Non-ASCII identifiers and Unicode string content.
 * Covers: CJK, Arabic, Cyrillic, emoji-adjacent identifiers, accented Latin.
 */

export const 最大値: number = 100;
export const минимум: number = 0;
export const näherungswert: number = 3.14;
export const π: number = Math.PI;
export const τ: number = 2 * Math.PI;

export interface Форма {
  площадь(): number;
  периметр(): number;
}

export interface Şekil {
  alan(): number;
  çevre(): number;
}

export class Kreis implements Форма {
  constructor(public radius: number) {}

  площадь(): number {
    return π * this.radius * this.radius;
  }

  периметр(): number {
    return τ * this.radius;
  }
}

export class 円 {
  private 半径: number;

  constructor(半径: number) {
    this.半径 = 半径;
  }

  面積(): number {
    return π * this.半径 * this.半径;
  }

  周長(): number {
    return τ * this.半径;
  }
}

export class DaireŞekli implements Şekil {
  constructor(public yarıçap: number) {}

  alan(): number {
    return π * this.yarıçap * this.yarıçap;
  }

  çevre(): number {
    return τ * this.yarıçap;
  }
}

export type Зміна<T> = {
  старе: T;
  нове: T;
};

export function вітання(імʼя: string): string {
  return `Привіт, ${імʼя}!`;
}

export function grüßen(name: string): string {
  return `Hallo, ${name}!`;
}

export function résoudre(équation: string): number {
  return parseFloat(équation);
}

export function حساب(قيمة: number, معامل: number): number {
  return قيمة * معامل;
}

export const beschreibung: string = "Kreis mit Radius r";
export const описание: string = "Круг с радиусом r";
export const 説明: string = "半径rの円";
