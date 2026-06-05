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
