// Malformed-but-parseable JavaScript: unclosed generics-like syntax,
// trailing commas in odd places, extra semicolons, weird spacing

const   WEIRD_SPACING   = "extra spaces around";
const TRAILING = {
  a: 1,
  b: 2,
};

// Missing semicolons throughout (ASI handles it)
const noSemi = 42
const alsoNoSemi = "hello"

class BrokenIndent {
constructor(x,y) {
this.x=x
this.y=y
}
    weirdMethod  (  a , b  ) {
   return a+b
    }
static   fromPair([a,b]) {
return new BrokenIndent(a,b)
}
}

function multipleReturns(x) {
if (x > 0) { return x }
if (x < 0) { return -x }
return 0
}

// Empty function
function empty() {}

// Arrow with no body parens
const double = x => x * 2

// Async arrow
const wait = async (ms) => new Promise(r => setTimeout(r, ms))

class EmptyClass {}

class Oneliner { constructor() {} greet() { return "hi" } }

function trailingCommaParams(
  a,
  b,
  c,
) {
  return a + b + c;
}

const obj = {
  method() {
    return 1;
  },
  arrowProp: () => 2,
};
