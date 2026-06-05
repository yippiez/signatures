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
