export const meta = {
  name: 'impl-output-flags',
  description: 'Implement --format jsonl, --stream, and --output full for the signatures CLI: opus builds the core (cli/render/main + Signature.full field + tests/flags.sh), then per-extractor-file agents (braces=opus, others=sonnet) populate the full text, then per-language sonnet agents verify the flags, then opus fixes any failures until green.',
  phases: [
    { title: 'Core', detail: 'opus: cli flags + Signature.full + render(jsonl/full/stream) + main wiring + tests/flags.sh', model: 'opus' },
    { title: 'Extractors', detail: 'parallel per file: populate Signature.full where values are elided (braces=opus, python/ruby/lua/bash=sonnet)' },
    { title: 'Gate', detail: 'sonnet: cargo build + cargo test + tests/run.sh + tests/flags.sh' },
    { title: 'VerifyLangs', detail: '17 sonnet agents (read-only): run the 3 flags per language fixture and report failures', model: 'sonnet' },
    { title: 'Fix', detail: 'opus: fix all reported failures, rebuild, re-run every gate until green', model: 'opus' },
  ],
}

const REPO = '/home/eren/work2/signature'
const ELLIPSIS = '…' // the U+2026 char used by the extractors for elided values

// ---------------------------------------------------------------------------
// Shared contract — every agent is told the SAME rules so the pieces compose.
// ---------------------------------------------------------------------------
const CONTRACT = [
  'PROJECT: a zero-dependency (std-only) Rust CLI "signatures" that prints the signatures',
  '(functions/classes/constants) of source files, body removed, nested members indented 2 spaces/level.',
  'cwd / repo root: ' + REPO + '. Binary: ./target/debug/signatures. Existing regression suite: ./tests/run.sh.',
  '',
  'THREE NEW FLAGS to add (this whole effort):',
  '  --format <plain|jsonl>   output format. DEFAULT plain (current behavior, byte-for-byte unchanged).',
  '                           jsonl = one JSON object per signature per line (JSON Lines).',
  '  --stream                 stream each finding as it is produced (flush per signature) instead of',
  '                           buffering all output and writing once. Works across BOTH formats and must',
  '                           produce BYTE-IDENTICAL output to the non-streaming path (it is purely incremental).',
  '  --output <truncated|full>  DEFAULT truncated (current behavior). full = show the full code part instead',
  '                           of the elided "' + ELLIPSIS + '" (U+2026) value. Applies to the value/RHS that extractors',
  '                           currently elide (e.g. constants emit "NAME = ' + ELLIPSIS + '"). It does NOT dump function bodies.',
  '',
  'CORE DATA-MODEL CONTRACT (authoritative — every extractor follows it):',
  '  struct Signature gains a new field:  pub full: Option<String>',
  '    * full = Some(<collapsed full declaration text WITHOUT the "' + ELLIPSIS + '" elision>) when this signature',
  '      elided a value/RHS. e.g. text="MAX = ' + ELLIPSIS + '"  ->  full=Some("MAX = 1 << 20").',
  '    * full = None when nothing was elided (functions, classes whose text is already complete).',
  '      Renderers MUST treat None as "use text" (i.e. full output falls back to text). This keeps the',
  '      tool correct even for extractors that have not been updated yet.',
  '  Every existing place that constructs a Signature must set the new field (None unless it elided a value).',
  '',
  'RENDERING RULES:',
  '  - plain + truncated (DEFAULT): identical to today (uses Signature.text, colorized).',
  '  - plain + full: same as plain, but the chosen text is full.unwrap_or(text).',
  '  - jsonl (any --output): NO ANSI color ever. One compact JSON object per signature per line:',
  '      {"file":<path>,"line":<int>,"indent":<int>,"kind":"function|class|constant","text":<chosen text>}',
  '    * file = the same display path used for the plain-mode header (always present in jsonl, even for 1 file).',
  '    * kind lowercased. text respects --output (full.unwrap_or(text) when full, else text).',
  '    * Hand-roll JSON string escaping (no deps): escape  \\  and  "  and control chars (\\n \\r \\t and \\u00XX for <0x20). Keep UTF-8 as-is.',
  '    * jsonl never prints file headers and never prints blank separator lines between files.',
  '  - --stream: write+flush stdout after each emitted unit; non-stream buffers then writes once. Both byte-identical.',
  '',
  'INVARIANTS (must hold for ALL languages; tests/flags.sh enforces them):',
  '  I1. Default (no new flags) output is byte-for-byte unchanged -> ./tests/run.sh still passes.',
  '  I2. For every fixture and every format f in {plain,jsonl}: output(f) == output(f,--stream)  (byte-identical).',
  '  I3. In jsonl, every non-empty output line parses as valid JSON.',
  '  I4. jsonl record count == number of plain-mode signature lines (plain lines minus any header lines).',
  '  I5. --output full never changes the NUMBER of signatures/lines vs truncated (it only expands text).',
].join('\n')

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------
const CORE_SCHEMA = {
  type: 'object',
  required: ['ok', 'buildPassed', 'cargoTestPassed', 'runShPassed', 'summary'],
  properties: {
    ok: { type: 'boolean' },
    buildPassed: { type: 'boolean' },
    cargoTestPassed: { type: 'boolean' },
    runShPassed: { type: 'boolean' },
    flagsShCreated: { type: 'boolean' },
    summary: { type: 'string' },
    filesTouched: { type: 'array', items: { type: 'string' } },
    notes: { type: 'string' },
  },
}
const EXTRACTOR_SCHEMA = {
  type: 'object',
  required: ['file', 'ok', 'buildPassed', 'summary'],
  properties: {
    file: { type: 'string' },
    ok: { type: 'boolean' },
    buildPassed: { type: 'boolean' },
    changed: { type: 'boolean' },
    sitesUpdated: { type: 'integer' },
    summary: { type: 'string' },
    notes: { type: 'string' },
  },
}
const GATE_SCHEMA = {
  type: 'object',
  required: ['buildPassed', 'cargoTestPassed', 'runShPassed', 'flagsShPassed'],
  properties: {
    buildPassed: { type: 'boolean' },
    cargoTestPassed: { type: 'boolean' },
    runShPassed: { type: 'boolean' },
    flagsShPassed: { type: 'boolean' },
    details: { type: 'string' },
  },
}
const VERIFY_SCHEMA = {
  type: 'object',
  required: ['language', 'passed', 'failures'],
  properties: {
    language: { type: 'string' },
    passed: { type: 'boolean' },
    failures: {
      type: 'array',
      items: {
        type: 'object',
        required: ['check', 'detail'],
        properties: {
          check: { type: 'string' },
          detail: { type: 'string' },
          repro: { type: 'string' },
        },
      },
    },
    notes: { type: 'string' },
  },
}
const FIX_SCHEMA = {
  type: 'object',
  required: ['buildPassed', 'cargoTestPassed', 'runShPassed', 'flagsShPassed'],
  properties: {
    buildPassed: { type: 'boolean' },
    cargoTestPassed: { type: 'boolean' },
    runShPassed: { type: 'boolean' },
    flagsShPassed: { type: 'boolean' },
    fixed: { type: 'array', items: { type: 'string' } },
    remaining: { type: 'array', items: { type: 'string' } },
    summary: { type: 'string' },
  },
}

// ---------------------------------------------------------------------------
// PHASE 1 — CORE (opus, serial)
// ---------------------------------------------------------------------------
phase('Core')
const corePrompt = [
  'You implement the CORE of three new CLI flags for the signatures Rust tool. TIME-BOX ~12 min.',
  CONTRACT,
  '',
  'YOUR SCOPE (core only — do NOT touch src/lang/*.rs extractor logic except to set the new field to None where Signatures are built):',
  ' 1. src/signature.rs: add  pub full: Option<String>  to struct Signature.',
  ' 2. src/cli.rs: extend Config with the parsed options. Add an output format enum {Plain, Jsonl}, an output',
  '    mode enum {Truncated, Full}, and a `stream: bool`. Parse --format <plain|jsonl> and --format=<v>,',
  '    --output <truncated|full> and --output=<v>, and --stream. Missing/invalid values -> Parsed::Error.',
  '    Keep --no-color, --help, --version, -- handling. Add unit tests for the new parsing (success + error cases).',
  ' 3. src/render.rs: implement rendering per the RENDERING RULES. Refactor so there is one function that renders',
  '    ONE signature (given colors, format, output-mode) to a String, used by both the buffered and streaming paths.',
  '    Plain+truncated must remain byte-identical to today. Add a hand-rolled json string-escape helper + tests.',
  ' 4. src/main.rs: wire it. Build the file list as today. For each file emit its signatures via the chosen format;',
  '    when --stream, write+flush stdout after each emitted unit (and after each plain header); else buffer + write once.',
  '    jsonl: always include the file field, never print headers/blank separators. Update USAGE and print_help() text.',
  ' 5. Every Signature{...} constructor across src/ (including each src/lang/*.rs) must compile: set full: None there',
  '    for now (the per-language agents will fill real values next). This is the ONLY edit you make to extractors.',
  ' 6. Create tests/flags.sh (chmod +x): a language-agnostic checker that enforces invariants I1-I5 above across',
  '    EVERY fixture under tests/ (reuse the fixture discovery idea from tests/run.sh; for merged @@CASE@@ files you',
  '    may test the whole file at once — these invariants are per-invocation, not per-case). Use python3 for JSON',
  '    validation (json.loads per line). It must exit non-zero on any violation and print what failed. Keep it std',
  '    POSIX bash. Do NOT modify tests/run.sh.',
  '',
  'VERIFY before returning: `cargo build` clean, `cargo test` green, `./tests/run.sh` green (I1), `bash tests/flags.sh` green.',
  'Note: until the per-language agents run, --output full will equal truncated for value-eliding constants (full=None fallback) — that is fine and expected now; flags.sh invariants I2-I5 must still pass.',
  'Return the structured result. List every file you touched.',
].join('\n')
const core = await agent(corePrompt, { schema: CORE_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'core', phase: 'Core' })
log('Core: build=' + core.buildPassed + ' cargo-test=' + core.cargoTestPassed + ' run.sh=' + core.runShPassed + ' — ' + core.summary)

// ---------------------------------------------------------------------------
// PHASE 2 — EXTRACTORS (parallel by file; disjoint files, safe)
// ---------------------------------------------------------------------------
phase('Extractors')
const EXTRACTORS = [
  { file: 'src/lang/braces.rs', model: 'opus', langs: 'Rust, Go, JS, TS, Java, C, C++, C#, Kotlin, Swift, PHP, Scala, Dart (ALL 13 brace languages)',
    hint: 'The single choke point is const_text() (builds "{} = ' + ELLIPSIS + '"); also the Go grouped const/var path in go_group_block (formats "{kw} {} = ' + ELLIPSIS + '") and any inline "= ' + ELLIPSIS + '" / setmetatable-style class cases. Thread an optional full value out of classify()/the const builders to the Signature{...} push site. full = the collapsed declaration WITH its real RHS (the gathered header already contains "NAME = value"; just keep the value instead of eliding it). full=None for functions and for already-complete texts.' },
  { file: 'src/lang/python.rs', model: 'sonnet', langs: 'Python', hint: 'Find where it emits "NAME = ' + ELLIPSIS + '" for module/class constants; capture the real RHS as full.' },
  { file: 'src/lang/ruby.rs', model: 'sonnet', langs: 'Ruby', hint: 'See the "{name} = ' + ELLIPSIS + '" builder (~line 334); capture the real RHS value as full.' },
  { file: 'src/lang/lua.rs', model: 'sonnet', langs: 'Lua', hint: 'See "{prefix}{name} = ' + ELLIPSIS + '" (~line 328) and the setmetatable(...) class case (~321); capture the real value as full.' },
  { file: 'src/lang/bash.rs', model: 'sonnet', langs: 'Bash', hint: 'See "{name} = ' + ELLIPSIS + '" (~line 456); capture the real RHS as full.' },
]
function extractorPrompt(e) {
  return [
    'You populate the new Signature.full field in ONE extractor file: ' + e.file + ' (language(s): ' + e.langs + '). TIME-BOX ~8 min.',
    CONTRACT,
    '',
    'The CORE is already implemented: struct Signature has `pub full: Option<String>`, currently set to None everywhere.',
    'YOUR JOB: in ' + e.file + ' ONLY, wherever this extractor ELIDES a value to "' + ELLIPSIS + '" (U+2026) — set full = Some(<the collapsed full text WITH the real value>) on that Signature instead of None.',
    'HINT: ' + e.hint,
    'Rules: edit ONLY ' + e.file + '. Do NOT change the truncated `text` (it must keep "' + ELLIPSIS + '"). full applies to value/RHS elision only; functions and complete texts stay full=None. Collapse internal whitespace the same way `text` is collapsed. Be conservative: if a value cannot be cleanly captured, leave full=None (the renderer falls back to text).',
    'Add/extend a unit test in this file asserting full is the expected complete string for at least one constant.',
    'VERIFY: `cargo build` clean and `cargo test` green. Then sanity-check by hand: build the binary and run',
    '  ./target/debug/signatures --no-color --output full <a fixture for this language under tests/>',
    'and confirm constants now show real values (no "' + ELLIPSIS + '") while default (no --output) still shows "' + ELLIPSIS + '".',
    'Return the structured result (sitesUpdated = how many elision sites you converted).',
  ].join('\n')
}
const extractors = await parallel(EXTRACTORS.map(e => () =>
  agent(extractorPrompt(e), { schema: EXTRACTOR_SCHEMA, model: e.model, agentType: 'general-purpose', label: 'extractor:' + e.file.split('/').pop(), phase: 'Extractors' })
))
for (const r of extractors.filter(Boolean)) log('Extractor ' + r.file + ': build=' + r.buildPassed + ' sites=' + (r.sitesUpdated || 0) + ' — ' + r.summary)

// ---------------------------------------------------------------------------
// PHASE 3 — GATE (sonnet): build once, run all suites, so VerifyLangs uses a fresh binary
// ---------------------------------------------------------------------------
phase('Gate')
const gate = await agent([
  'BUILD GATE. cwd ' + REPO + '. Run, in order, and report each result truthfully (do NOT fix anything — just report):',
  '  1. cargo build            (binary at ./target/debug/signatures)',
  '  2. cargo test',
  '  3. ./tests/run.sh         (default-output regression — invariant I1)',
  '  4. bash tests/flags.sh    (cross-language flag invariants I2-I5)',
  'For any failure, capture the key error lines into `details`. Leave the freshly built debug binary in place.',
].join('\n'), { schema: GATE_SCHEMA, model: 'sonnet', agentType: 'general-purpose', label: 'gate', phase: 'Gate' })
log('Gate: build=' + gate.buildPassed + ' cargo-test=' + gate.cargoTestPassed + ' run.sh=' + gate.runShPassed + ' flags.sh=' + gate.flagsShPassed)

// ---------------------------------------------------------------------------
// PHASE 4 — VERIFY LANGS (parallel, read-only, sonnet). One per language fixture.
// ---------------------------------------------------------------------------
phase('VerifyLangs')
const LANGS = [
  ['bash', 'tests/bash.sh'], ['c', 'tests/c.c'], ['cpp', 'tests/cpp.cpp'], ['csharp', 'tests/csharp.cs'],
  ['dart', 'tests/dart.dart'], ['go', 'tests/go.go'], ['java', 'tests/java.java'], ['javascript', 'tests/javascript.js'],
  ['kotlin', 'tests/kotlin.kt'], ['lua', 'tests/lua.lua'], ['php', 'tests/php.php'], ['ruby', 'tests/ruby.rb'],
  ['scala', 'tests/scala.scala'], ['swift', 'tests/swift.swift'], ['typescript', 'tests/typescript.ts'],
  ['python', 'tests/python.py'], ['rust', 'tests/rust.rs'],
]
function verifyPrompt(language, fixture) {
  return [
    'You VERIFY the three new flags for ' + language + ' only. READ-ONLY: do NOT edit any source and do NOT run cargo build (the binary ./target/debug/signatures is already built). TIME-BOX ~4 min.',
    CONTRACT,
    '',
    'Fixture: ' + fixture + ' (a @@CASE@@-merged file: split cases into a scratch dir with mktemp if you want per-case checks, OR just run the whole file — invariants are per-invocation). Also feel free to write tiny extra ' + language + ' snippets to a mktemp scratch dir (NOT under tests/).',
    'Run the binary with combinations and CHECK:',
    '  A. I2 stream==buffered: for fmt in plain,jsonl:  diff <(bin --no-color --format fmt FILE) <(bin --no-color --format fmt --stream FILE)  must be empty.',
    '  B. I3 valid JSON: every non-empty line of  bin --no-color --format jsonl FILE  parses (pipe to: python3 -c "import sys,json;[json.loads(l) for l in sys.stdin if l.strip()]").',
    '  C. I4 counts: jsonl line count == plain signature-line count (plain minus header lines; for a single FILE arg plain prints no header).',
    '  D. --output full: vs default truncated, line count is identical (I5); and for ' + language + ' CONSTANTS that have a real value, full shows the value while default shows "' + ELLIPSIS + '". Spot-check 1-2 constants. (If this language has no value constants in the fixture, note that and skip D\'s value check.)',
    '  E. jsonl has NO ANSI escape bytes; jsonl "kind" is one of function/class/constant; "file"/"line"/"indent"/"text" present.',
    'Report passed=true only if A-E hold. For each failure give check (A-E), a concrete detail, and a copy-pasteable repro command. Be precise; this drives the fix phase.',
  ].join('\n')
}
const verifs = (await parallel(LANGS.map(([language, fixture]) => () =>
  agent(verifyPrompt(language, fixture), { schema: VERIFY_SCHEMA, model: 'sonnet', agentType: 'general-purpose', label: 'verify:' + language, phase: 'VerifyLangs' })
))).filter(Boolean)
const failingLangs = verifs.filter(v => !v.passed)
log('VerifyLangs: ' + (verifs.length - failingLangs.length) + '/' + verifs.length + ' passed' +
  (failingLangs.length ? '. Failing: ' + failingLangs.map(v => v.language).join(', ') : ''))

// ---------------------------------------------------------------------------
// PHASE 5 — FIX (opus): fix everything reported + any red gate, until all green.
// ---------------------------------------------------------------------------
phase('Fix')
const allGreen = gate.buildPassed && gate.cargoTestPassed && gate.runShPassed && gate.flagsShPassed && failingLangs.length === 0
let fix = null
if (allGreen) {
  log('All gates green and every language verified — no fix phase needed.')
} else {
  const failureReport = [
    'GATE: build=' + gate.buildPassed + ' cargo-test=' + gate.cargoTestPassed + ' run.sh=' + gate.runShPassed + ' flags.sh=' + gate.flagsShPassed,
    gate.details ? 'GATE DETAILS:\n' + gate.details : '',
    '',
    'PER-LANGUAGE FAILURES:',
    failingLangs.length ? failingLangs.map(v =>
      '## ' + v.language + '\n' + (v.failures || []).map(f => '- [' + f.check + '] ' + f.detail + (f.repro ? '\n  repro: ' + f.repro : '')).join('\n')
    ).join('\n\n') : '(none — only gate failures)',
  ].filter(Boolean).join('\n')
  fix = await agent([
    'You FIX all reported failures in the signatures CLI so every gate is green. cwd ' + REPO + '. TIME-BOX ~15 min.',
    CONTRACT,
    '',
    'Reported failures to resolve:',
    failureReport,
    '',
    'Fix the root causes in src/ (core files and/or the relevant src/lang/*.rs). Preserve invariant I1 (default output byte-identical -> tests/run.sh).',
    'Loop until ALL of these pass, re-running after each change: `cargo build`, `cargo test`, `./tests/run.sh`, `bash tests/flags.sh`.',
    'If a per-language semantic issue (e.g. --output full not showing a value) is genuinely out of scope to fix safely, leave full=None for that case (renderer falls back to text) and note it in remaining[].',
    'Return the structured result with the final state of all four checks.',
  ].join('\n'), { schema: FIX_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'fix', phase: 'Fix' })
  log('Fix: build=' + fix.buildPassed + ' cargo-test=' + fix.cargoTestPassed + ' run.sh=' + fix.runShPassed + ' flags.sh=' + fix.flagsShPassed + ' — ' + (fix.summary || ''))
}

return {
  core: { build: core.buildPassed, cargoTest: core.cargoTestPassed, runSh: core.runShPassed, files: core.filesTouched || [] },
  extractors: extractors.filter(Boolean).map(r => ({ file: r.file, build: r.buildPassed, sites: r.sitesUpdated || 0 })),
  gate,
  verify: { passed: verifs.length - failingLangs.length, total: verifs.length, failing: failingLangs.map(v => v.language) },
  fix: fix ? { build: fix.buildPassed, cargoTest: fix.cargoTestPassed, runSh: fix.runShPassed, flagsSh: fix.flagsShPassed, remaining: fix.remaining || [] } : 'skipped (all green)',
  finalGreen: fix ? (fix.buildPassed && fix.cargoTestPassed && fix.runShPassed && fix.flagsShPassed) : allGreen,
}
