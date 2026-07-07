// ── CASE 1: Variable declarations — spacing and semicolons ─────────────────
const x = "hello";
let   y  =  'world'   ;
var z=42;

// ── CASE 2: Mixed quote styles (should normalize to double) ─────────────────
const single = 'single quote string';
const double = "double quote string";
const escaped = 'can\'t touch this';
const withDouble = 'already has "double" inside';

// ── CASE 3: Trailing whitespace (invisible — these lines end with spaces)   
const padded = "value";   
let another = 1;  

// ── CASE 4: Arrow functions — parens and body style ────────────────────────
const bare = x => x + 1;
const wrapped = (x) => x + 1;
const multi = (x, y) => {
  return x + y;
};
const implicit = (x) => ({ key: x });

// ── CASE 5: Nested blocks — indentation ────────────────────────────────────
function outer() {
    function inner() {
        if (true) {
            for (let i = 0; i < 10; i++) {
                console.log(i);
            }
        }
    }
}

// ── CASE 6: Object literals — bracket spacing ──────────────────────────────
const obj = {key: "value", other: 42};
const spaced = { key: "value", other: 42 };
const nested = {a: {b: {c: 1}}};

// ── CASE 7: Long lines — should wrap at printWidth (80 chars) ──────────────
function veryLongFunctionNameThatExceedsTheLineWidth(argumentOne, argumentTwo, argumentThree, argumentFour) {
  return argumentOne + argumentTwo + argumentThree + argumentFour;
}

const longChain = someObject.methodOne().methodTwo().methodThree().methodFour().methodFive();

// ── CASE 8: Prettier-ignore comment — following statement must be untouched ─
// prettier-ignore
const ignored = [1,   2,   3,   4];
const notIgnored = [1, 2, 3, 4];

// ── CASE 9: Template literals — must not be mangled ────────────────────────
const tmpl = `Hello ${name}, you are ${age} years old.`;
const multiLineTmpl = `
  First line
  Second line
  ${expression}
`;

// ── CASE 10: Array destructuring and spread ────────────────────────────────
const [first, ...rest] = someArray;
const merged = [...arr1, ...arr2, extraItem];

// ── CASE 11: Async / await ────────────────────────────────────────────────
async function fetchData(url) {
  try {
    const response = await fetch(url);
    const data = await response.json();
    return data;
  } catch (error) {
    console.error(error);
  }
}

// ── CASE 12: Class with mixed indentation ──────────────────────────────────
class Animal {
    constructor(name) {
    this.name = name;
  }
    speak() {
        console.log(`${this.name} makes a sound.`);
    }
}

// ── CASE 13: Trailing comma in params and args ─────────────────────────────
function withTrailing(
  a,
  b,
  c,
) {
  return [a, b, c,];
}
