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
