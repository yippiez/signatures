export const meta = {
  name: 'fuzz-language',
  description: 'Fuzz one already-implemented signatures language: 20 sonnet agents generate language examples and hunt for failures (panics / wrong output); then one opus agent turns the distinct failing cases into regression test fixtures under tests/<lang>/. It only captures bugs as tests — it does NOT fix the extractor.',
  whenToUse: 'Stress-test a single existing language and pin its bugs as tests. args="Rust" or {language:"Rust", fuzzers:20}.',
  phases: [
    { title: 'Fuzz', detail: '20 sonnet agents generate examples and report reproducible failures', model: 'sonnet' },
    { title: 'CreateTests', detail: 'one opus agent writes regression fixtures (input + correct .expected) for the failures', model: 'opus' },
  ],
}

// ---- inputs / knobs ---------------------------------------------------------
const REPO = '/home/eren/work2/signature'
const MAX_FAILURES_FOR_TESTS = 40

let A = args
if (typeof A === 'string') {
  const t = A.trim()
  if (t.startsWith('{') || t.startsWith('[')) { try { A = JSON.parse(t) } catch (_) {} }
}
const cfg = (A && typeof A === 'object' && !Array.isArray(A)) ? A : {}
const FUZZERS = Number.isFinite(cfg.fuzzers) ? Math.max(1, cfg.fuzzers) : 20

const rawLangs = Array.isArray(A) ? A
  : Array.isArray(cfg.languages) ? cfg.languages
  : (cfg.language ? [cfg.language]
    : (typeof cfg.languages === 'string' ? cfg.languages.split(/[,\s]+/)
      : (typeof A === 'string' ? A.split(/[,\s]+/) : [])))
const RESERVED = new Set(['language', 'languages', 'fuzzers', 'rounds'])
const cleaned = rawLangs.map(s => String(s).trim()).filter(Boolean)
  .filter(s => !RESERVED.has(s.toLowerCase()) && !/^\d+$/.test(s))
if (cleaned.length === 0) throw new Error('fuzz-language needs a language. args="Rust" or {language:"Rust", fuzzers:20}.')
const lang = cleaned[0]
if (cleaned.length > 1) log('Multiple languages given; fuzzing only the first: ' + lang)

// ---- which source file holds this language's logic --------------------------
const ALIAS = {
  'c++': 'cpp', 'cpp': 'cpp', 'cplusplus': 'cpp', 'c#': 'csharp', 'cs': 'csharp', 'csharp': 'csharp',
  'objective-c': 'objc', 'objectivec': 'objc', 'objc': 'objc', 'golang': 'go', 'js': 'javascript', 'ts': 'typescript',
}
const BRACE_FAMILY = new Set(['rust', 'go', 'c', 'cpp', 'csharp', 'java', 'javascript', 'typescript', 'swift', 'kotlin', 'scala', 'php', 'dart', 'objc', 'zig'])
const key = ALIAS[lang.toLowerCase().trim()] || (lang.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '') || 'lang')
const dir = key
const srcFile = BRACE_FAMILY.has(key) ? 'braces.rs' : key + '.rs'

// 15 fuzzing angles, cycled across the 20 agents to diversify coverage.
const FOCUS = [
  'Deeply nested declarations — verify nesting indent at every depth.',
  'Multi-line signatures with generics / long parameter lists split across lines.',
  'Comments & strings (line/block/doc/raw) that CONTAIN fake declarations — must be ignored.',
  'Unicode / non-ASCII identifiers, emoji, multibyte content — must never panic, never split a char.',
  'Constants vs lookalikes: annotated/typed assignments, augmented ops, comparisons.',
  'Whitespace torture: tabs vs spaces, mixed indentation, CRLF, no trailing newline.',
  'Degenerate inputs: empty / whitespace-only / comments-only / one very long line.',
  'A very large generated file (thousands of declarations) — correctness and no hang.',
  'Decorators / attributes / annotations / modifiers / visibility qualifiers.',
  'Async / generator / operator / special declaration forms and language-specific variants.',
  'Operators/symbols resembling signatures but not (assignments, casts, calls, type aliases).',
  'A realistic idiomatic real-world file — typical code should produce a clean outline.',
  'Malformed / truncated / incomplete code, unbalanced brackets — must never crash.',
  'Randomly mutated inputs: truncate, duplicate lines, inject random bytes/quotes/brackets.',
  'Expression-bodied members, default parameters, and trailing-comma parameter lists.',
]

// ---- prompts ----------------------------------------------------------------
function fuzzPrompt(i, focus) {
  return [
    'You are FUZZER #' + i + ' stress-testing the "signatures" CLI support for ' + lang + ' (an ALREADY-implemented language). ' +
      'TIME-BOX: ~3 minutes — find failures, then stop; never loop.',
    'Repo (cwd): ' + REPO + '. Build once: cargo build -q (if it FAILS, report that as a panic-severity failure and stop). Binary: ./target/debug/signatures.',
    'Work in a private scratch dir: WS=$(mktemp -d). Write your example files THERE — do NOT write under tests/ and do NOT edit any src/ file.',
    '',
    'The tool prints one line per signature (functions/methods, classes/types, constants), body removed, nested members indented 2 spaces/level, declarations inside comments/strings ignored. ' + lang + ' logic lives in src/lang/' + srcFile + ' (study it + the real ' + lang + ' grammar to know correct output).',
    'YOUR FOCUS: ' + focus,
    '',
    'Generate varied ' + lang + ' examples (realistic + adversarial + a few mechanically-mutated), run "./target/debug/signatures --no-color <file>" on each, and hunt for FAILURES:',
    '  - PANIC / crash / unexpected non-zero exit (a panic prints "thread \'main\' panicked"; the only OK errors are "unsupported file type" / "No such file or directory").',
    '  - WRONG output: a real declaration missing; a fake one (from a comment/string) emitted; garbled/incorrect text; wrong nesting indent; body not removed; multi-line not joined.',
    '',
    'For EACH genuine, reproducible failure record: a MINIMAL input (inline it), the ACTUAL (wrong) output, and the CORRECT expected output (from the real grammar). Report ONLY real failures — no nitpicks. Zero failures is a fine result. Return the structured result.',
  ].join('\n')
}

function createTestsPrompt(failures) {
  const list = failures.map((f, n) => [
    '### Failure ' + (n + 1) + ': ' + (f.title || 'untitled') + ' [' + (f.severity || 'unknown') + ']',
    f.note ? 'Note: ' + f.note : '',
    f.input ? 'Input:\n' + f.input : '',
    f.actual ? 'Actual (wrong) output:\n' + f.actual : '',
    f.expected ? 'Correct expected output:\n' + f.expected : '',
  ].filter(Boolean).join('\n')).join('\n\n')

  return [
    'You are creating REGRESSION TEST FIXTURES from fuzzer-found failures for ' + lang + '. Repo (cwd): ' + REPO + '.',
    'Scope: ONLY capture these bugs as tests. Do NOT modify any src/ file and do NOT fix the extractor — fixing is out of scope for this run.',
    'TIME-BOX: ~6 minutes.',
    '',
    'The fuzzers reported these failures (deduplicated):',
    list || '(none — the fuzzers found no failures; create no fixtures.)',
    '',
    'For each DISTINCT, genuine failure, create a fixture under tests/' + dir + '/ named fuzz_<short-slug>.<ext> holding the MINIMAL reproducing input, and a sibling "<same-name>.expected" containing the CORRECT `signatures --no-color` output (the SET/nesting that SHOULD be produced — NOT the current wrong output). Match the tool\'s exact line format (study tests/' + dir + '/*.expected). Merge duplicate/related failures into one fixture each.',
    'These fixtures will currently FAIL the suite (they document real bugs) — that is expected and intended. Verify each input does not crash the binary, then run "./tests/run.sh ' + dir + '" and report how many of your new fixtures currently fail.',
    '',
    'Return: created (int), files (paths), currentlyFailing (int), notes.',
  ].join('\n')
}

// ---- schemas ----------------------------------------------------------------
const FUZZ_SCHEMA = {
  type: 'object',
  required: ['anyFailures', 'failures'],
  properties: {
    anyFailures: { type: 'boolean' },
    failures: {
      type: 'array',
      items: {
        type: 'object',
        required: ['title', 'severity'],
        properties: {
          title: { type: 'string' },
          severity: { type: 'string', enum: ['panic', 'wrong-output', 'other'] },
          input: { type: 'string' },
          actual: { type: 'string' },
          expected: { type: 'string' },
          note: { type: 'string' },
        },
      },
    },
  },
}
const TESTS_SCHEMA = {
  type: 'object',
  required: ['created'],
  properties: {
    created: { type: 'integer' },
    files: { type: 'array', items: { type: 'string' } },
    currentlyFailing: { type: 'integer' },
    notes: { type: 'string' },
  },
}

// ---- helpers ----------------------------------------------------------------
function dedupe(failures) {
  const seen = new Set()
  const out = []
  for (const f of failures || []) {
    const k = (f.title || '').toLowerCase().replace(/\s+/g, ' ').trim() + '|' + (f.input || '')
    if (seen.has(k)) continue
    seen.add(k)
    out.push(f)
  }
  return out
}

// ============================================================================
// STAGE 1 — FUZZ: 20 sonnet agents in parallel
// ============================================================================
phase('Fuzz')
log('Fuzzing ' + lang + ' (logic in src/lang/' + srcFile + ') with ' + FUZZERS + ' sonnet agents...')
const results = await parallel(
  Array.from({ length: FUZZERS }, (_, i) => () =>
    agent(fuzzPrompt(i + 1, FOCUS[i % FOCUS.length]), {
      schema: FUZZ_SCHEMA,
      model: 'sonnet',
      agentType: 'general-purpose',
      label: 'fuzz:' + dir + ':a' + (i + 1),
      phase: 'Fuzz',
    })
  )
)

const ran = results.filter(Boolean)
const allFailures = dedupe(ran.flatMap(r => Array.isArray(r.failures) ? r.failures : []))
log(ran.length + '/' + FUZZERS + ' fuzzers ran; ' + allFailures.length + ' distinct failure(s) found.')

if (allFailures.length === 0) {
  log('No failures found — nothing to capture. Done.')
  return { language: lang, fuzzers: ran.length, failures: 0, testsCreated: 0 }
}

// ============================================================================
// STAGE 2 — CREATE TESTS: one opus agent turns failures into fixtures
// ============================================================================
const batch = allFailures.slice(0, MAX_FAILURES_FOR_TESTS)
if (allFailures.length > batch.length) log('Capping at ' + batch.length + ' of ' + allFailures.length + ' failures for fixture creation.')
phase('CreateTests')
const tests = await agent(createTestsPrompt(batch), {
  schema: TESTS_SCHEMA,
  model: 'opus',
  agentType: 'general-purpose',
  label: 'create-tests:' + dir,
  phase: 'CreateTests',
})
log('Created ' + (tests.created || 0) + ' regression fixture(s); ' + (tests.currentlyFailing || 0) + ' currently failing (documented bugs).')

return {
  language: lang,
  fuzzers: ran.length,
  failures: allFailures.length,
  testsCreated: tests.created || 0,
  currentlyFailing: tests.currentlyFailing || 0,
  files: tests.files || [],
}
