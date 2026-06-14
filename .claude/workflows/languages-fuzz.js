export const meta = {
  name: 'languages-fuzz',
  description: 'Fuzz the implemented signatures languages: per language, ~12 sonnet agents generate examples and hunt for failures (panics / wrong output), then one opus agent turns the distinct failing cases into regression test fixtures under tests/<lang>/. Self-contained sweep — captures bugs only, does NOT fix extractors.',
  whenToUse: 'Sweep-fuzz the language set and pin bugs as tests. args={} fuzzes all-but-already-done; ["Rust","Go"] or {languages:[...]} restricts; {skip:[...]} excludes; {fuzzers:N} per-language fuzzer count.',
  phases: [
    { title: 'Fuzz', detail: 'per language: sonnet agents generate examples and report reproducible failures', model: 'sonnet' },
    { title: 'CreateTests', detail: 'per language: one opus agent writes regression fixtures (input + correct .expected)', model: 'opus' },
  ],
}

// ---- inputs / knobs ---------------------------------------------------------
const REPO = '/home/eren/work2/signature'
const MAX_FAILURES_FOR_TESTS = 40

// Every implemented language (one tests/<lang>/ dir each).
const ALL = [
  'bash', 'c', 'cpp', 'csharp', 'dart', 'go', 'java', 'javascript', 'kotlin',
  'lua', 'php', 'ruby', 'scala', 'swift', 'typescript', 'python', 'rust',
]

let A = args
if (typeof A === 'string') {
  const t = A.trim()
  if (t.startsWith('{') || t.startsWith('[')) { try { A = JSON.parse(t) } catch (_) {} }
}
const cfg = (A && typeof A === 'object' && !Array.isArray(A)) ? A : {}

// Number of sonnet fuzzers per language. Analysis of an early run showed ~8-12
// catches nearly all distinct root causes, so 12 is the default sweet spot.
const FUZZERS = Number.isFinite(cfg.fuzzers) ? Math.max(1, cfg.fuzzers) : 12
// Python + Rust were already fuzzed (their fixtures are committed in the tree),
// so skip them by default to avoid duplicate fixtures. Override with {skip:[]}.
const skip = new Set((Array.isArray(cfg.skip) ? cfg.skip : ['python', 'rust']).map(s => String(s).toLowerCase().trim()))
const requested = Array.isArray(A) ? A.map(String)
  : Array.isArray(cfg.languages) ? cfg.languages.map(String)
  : (typeof cfg.languages === 'string' ? cfg.languages.split(/[,\s]+/)
    : (cfg.language ? [String(cfg.language)] : ALL))
const langs = requested.map(s => s.toLowerCase().trim()).filter(Boolean).filter(l => !skip.has(l))

if (langs.length === 0) throw new Error('languages-fuzz: no languages left to fuzz after applying skip list.')

// ---- which source file holds a language's logic -----------------------------
const ALIAS = {
  'c++': 'cpp', 'cpp': 'cpp', 'cplusplus': 'cpp', 'c#': 'csharp', 'cs': 'csharp', 'csharp': 'csharp',
  'objective-c': 'objc', 'objectivec': 'objc', 'objc': 'objc', 'golang': 'go', 'js': 'javascript', 'ts': 'typescript',
}
const BRACE_FAMILY = new Set(['rust', 'go', 'c', 'cpp', 'csharp', 'java', 'javascript', 'typescript', 'swift', 'kotlin', 'scala', 'php', 'dart', 'objc', 'zig'])
function resolveLang(lang) {
  const key = ALIAS[lang.toLowerCase().trim()] || (lang.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '') || 'lang')
  return { key, dir: key, srcFile: BRACE_FAMILY.has(key) ? 'braces.rs' : key + '.rs' }
}

// 15 fuzzing angles, cycled across the agents to diversify coverage.
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
function fuzzPrompt(lang, srcFile, i, focus) {
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

function createTestsPrompt(lang, dir, failures) {
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
    'For each DISTINCT, genuine failure, create a fixture under tests/' + dir + '/ named like an ORDINARY source file — a short descriptive name reflecting the case (e.g. nested_generics, comment_in_params, unterminated_string), NOT a "fuzz_" or "test" prefix. Use the language\'s normal extension and avoid colliding with files already in tests/' + dir + '/. Write the MINIMAL reproducing input plus a sibling "<same-name>.expected" containing the CORRECT `signatures --no-color` output (the SET/nesting that SHOULD be produced — NOT the current wrong output). Match the tool\'s exact line format (study tests/' + dir + '/*.expected). Merge duplicate/related failures into one fixture each.',
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

// Fuzz a single language end-to-end (Fuzz -> CreateTests), inlined so the sweep
// is fully self-contained (no cross-workflow call).
async function fuzzOneLanguage(lang, fuzzers) {
  const { dir, srcFile } = resolveLang(lang)

  // STAGE 1 — FUZZ: sonnet agents in parallel
  const results = await parallel(
    Array.from({ length: fuzzers }, (_, i) => () =>
      agent(fuzzPrompt(lang, srcFile, i + 1, FOCUS[i % FOCUS.length]), {
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
  log(dir + ': ' + ran.length + '/' + fuzzers + ' fuzzers ran; ' + allFailures.length + ' distinct failure(s) found.')

  if (allFailures.length === 0) {
    return { language: lang, fuzzers: ran.length, failures: 0, testsCreated: 0 }
  }

  // STAGE 2 — CREATE TESTS: one opus agent turns failures into fixtures
  const batch = allFailures.slice(0, MAX_FAILURES_FOR_TESTS)
  if (allFailures.length > batch.length) log(dir + ': capping at ' + batch.length + ' of ' + allFailures.length + ' failures for fixture creation.')
  const tests = await agent(createTestsPrompt(lang, dir, batch), {
    schema: TESTS_SCHEMA,
    model: 'opus',
    agentType: 'general-purpose',
    label: 'create-tests:' + dir,
    phase: 'CreateTests',
  })
  log(dir + ': created ' + (tests.created || 0) + ' regression fixture(s); ' + (tests.currentlyFailing || 0) + ' currently failing (documented bugs).')

  return {
    language: lang,
    fuzzers: ran.length,
    failures: allFailures.length,
    testsCreated: tests.created || 0,
    currentlyFailing: tests.currentlyFailing || 0,
    files: tests.files || [],
  }
}

// ============================================================================
// SWEEP — fuzz every requested language in parallel (each writes only its own
// tests/<lang>/, so parallel edits are conflict-free).
// ============================================================================
phase('Fuzz')
log('Fuzzing ' + langs.length + ' language(s) with ' + FUZZERS + ' fuzzers each: ' + langs.join(', ') +
  (skip.size ? '  (skipping: ' + [...skip].join(', ') + ')' : ''))

const results = await parallel(langs.map(l => () =>
  fuzzOneLanguage(l, FUZZERS)
    .then(r => ({ ok: true, ...r }))
    .catch(e => ({ language: l, ok: false, error: String(e && e.message || e) }))
))

const done = results.filter(Boolean)
const summary = done.map(r => r.ok
  ? '  ' + String(r.language).padEnd(11) + '→ ' + (r.failures || 0) + ' failure(s), ' + (r.testsCreated || 0) + ' fixture(s) created'
  : '  ' + String(r.language).padEnd(11) + '→ ERROR: ' + r.error
).join('\n')
log('Sweep complete:\n' + summary)

const totalFixtures = done.reduce((n, r) => n + (r.ok ? (r.testsCreated || 0) : 0), 0)
return {
  languagesFuzzed: done.filter(r => r.ok).length,
  languagesErrored: done.filter(r => !r.ok).map(r => r.language),
  totalFixturesCreated: totalFixtures,
  perLanguage: done,
}
