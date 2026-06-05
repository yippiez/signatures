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
