export const meta = {
  name: 'implement-new-language',
  description: 'Implement signatures support for one or many languages and harden them. Brace-family langs share braces.rs (serial extend); dedicated-module langs implement in parallel (temp-isolated). Then bounded parallel fuzz -> dedup across languages -> single bounded serial fix, looped until clean. Every agent is time-boxed so the run never stalls on a straggler.',
  whenToUse: 'Add one or several source languages to the signatures CLI and harden them. Pass args="Rust" or args=["Kotlin","Ruby","C++"] or args={languages:[...], fuzzers:5, rounds:2}. Fuzzing fans out in parallel; shared-file edits (braces.rs / mod.rs / render.rs / main.rs) are serialized; fixes are deduped and applied by one bounded agent.',
  phases: [
    { title: 'ImplementModules', detail: 'parallel opus agents, one per dedicated-module language, temp-isolated', model: 'opus' },
    { title: 'ImplementBraces', detail: 'one serial opus agent extends src/lang/braces.rs for all brace-family languages', model: 'opus' },
    { title: 'Register', detail: 'one serial sonnet agent wires module languages into the shared files + builds/tests', model: 'sonnet' },
    { title: 'Fuzz', detail: 'time-boxed sonnet fuzzers per language, all at once (read-only)', model: 'sonnet' },
    { title: 'Fix', detail: 'one bounded serial opus agent fixes the deduped batch directly in the real repo', model: 'opus' },
  ],
}

// ---- inputs / knobs ---------------------------------------------------------
// args may be: "Rust" | ["Rust","Go"] | {languages:[...], fuzzers:N, rounds:N}
const REPO = '/home/eren/work2/signature'
const MAX_ERRORS_TO_FIX = 50

// Robust arg parsing: args may arrive as a real object/array, OR as a
// JSON-encoded string (which previously got split into garbage "languages",
// wasting a whole run). Parse JSON strings before anything else.
let A = args
if (typeof A === 'string') {
  const t = A.trim()
  if (t.startsWith('{') || t.startsWith('[')) {
    try { A = JSON.parse(t) } catch (_) { /* fall through to plain split */ }
  }
}
const cfg = (A && typeof A === 'object' && !Array.isArray(A)) ? A : {}
const FUZZERS = Number.isFinite(cfg.fuzzers) ? Math.max(1, cfg.fuzzers) : 5
const MAX_ROUNDS = Number.isFinite(cfg.rounds) ? Math.max(1, cfg.rounds) : 3

const rawLangs = Array.isArray(A) ? A
  : Array.isArray(cfg.languages) ? cfg.languages
  : (typeof cfg.languages === 'string' ? cfg.languages.split(/[,\s]+/)
    : (typeof A === 'string' ? A.split(/[,\s]+/) : []))
// Drop stray config-key tokens in case a plain-string split still sneaks through.
const RESERVED = new Set(['languages', 'fuzzers', 'rounds'])
const langs = rawLangs
  .map(s => String(s).trim())
  .filter(Boolean)
  .filter(s => !RESERVED.has(s.toLowerCase()) && !/^\d+$/.test(s))
if (langs.length === 0) {
  throw new Error('implement-new-language requires at least one language. args="Rust" or args=["Kotlin","Ruby"].')
}

// Time-box guidance injected into every worker prompt so no agent runs unbounded
// (the root cause of the earlier stall: one fix agent ran ~33 min and blocked a barrier).
const FUZZ_TIMEBOX = 'TIME-BOX: spend at most ~3 minutes total; create up to 10 files; once you have clear, reproducible defects (or are confident there are none), STOP and return — never keep looping.'
const FIX_TIMEBOX = 'TIME-BOX: at most ~6 minutes and at most 3 full cargo build/test cycles. Fix the highest-impact defects first; if not everything is green in time, return partial progress with what remains in "remaining" — do NOT loop indefinitely.'

// ---- language families ------------------------------------------------------
// Brace-family languages reuse the shared src/lang/braces.rs engine (a Lang enum
// + per-language config). Everything else gets a dedicated extractor module.
const ALIAS = {
  'c++': 'cpp', 'cpp': 'cpp', 'cplusplus': 'cpp',
  'c#': 'csharp', 'cs': 'csharp', 'csharp': 'csharp',
  'objective-c': 'objc', 'objectivec': 'objc', 'objc': 'objc',
  'f#': 'fsharp', 'golang': 'go', 'js': 'javascript', 'ts': 'typescript',
}
const BRACE_FAMILY = new Set([
  'rust', 'go', 'c', 'cpp', 'csharp', 'java', 'javascript', 'typescript',
  'swift', 'kotlin', 'scala', 'php', 'dart', 'objc', 'zig',
])
function normKey(lang) {
  const low = lang.toLowerCase().trim()
  if (ALIAS[low]) return ALIAS[low]
  return low.replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '') || 'lang'
}

const moduleOf = {}   // dedicated-module file name per language
const sourceFileOf = {} // which src/lang/*.rs file holds this language's logic
const isBrace = {}
const usedModules = new Set()
for (const L of langs) {
  const key = normKey(L)
  const brace = BRACE_FAMILY.has(key)
  isBrace[L] = brace
  sourceFileOf[L] = brace ? 'braces.rs' : key + '.rs'
  if (!brace) {
    let name = key
    let n = 2
    while (usedModules.has(name)) { name = key + '_' + n; n++ }
    usedModules.add(name)
    moduleOf[L] = name
  } else {
    moduleOf[L] = key
  }
}
const braceLangs = langs.filter(L => isBrace[L])
const moduleLangs = langs.filter(L => !isBrace[L])

// 15 distinct fuzzing angles (used round-robin across however many fuzzers).
const FOCUS = [
  'Deeply nested declarations: types within types within functions; verify nesting indent at every depth.',
  'Multi-line signatures with complex generics / type params / long parameter lists split across lines; verify they join into one tidy line, body removed.',
  'Comments and strings that CONTAIN fake declarations (line/block/doc comments, string/char/raw literals) — all ignored, no false positives.',
  'Unicode / non-ASCII identifiers, emoji, multibyte content — never panic; never split a multi-byte char (the known class of bug).',
  'Constants: tricky/annotated assignments, augmented ops, comparisons, lookalikes that must NOT be reported.',
  'Whitespace torture: tabs vs spaces, mixed indentation, trailing whitespace, CRLF, no trailing newline.',
  'Degenerate inputs: empty / whitespace-only / comments-only file, one very long line, file with no signatures.',
  'A very large generated file (thousands of declarations) — correctness and no hang/panic.',
  'Decorators / attributes / annotations / modifiers / visibility qualifiers preceding declarations.',
  'Async / generator / operator / special declaration forms and language-specific variants.',
  'Operators/symbols that resemble signatures but are not (assignments, casts, calls, type aliases).',
  'A realistic idiomatic real-world file — typical code should produce a clean, useful outline.',
  'Malformed / truncated / incomplete code, unbalanced brackets/braces — never crash.',
  'Randomly mutated inputs: truncate, duplicate lines, inject random bytes/quotes/brackets, scramble whitespace — hunt panics.',
  'Dense interleaving of all declaration kinds with minimal/edge spacing, back-to-back declarations.',
]

// ---- prompts ----------------------------------------------------------------
function moduleImplPrompt(lang, mod) {
  return [
    'Implement "signatures" support for ' + lang + ' as a SELF-CONTAINED extractor module, IN ISOLATION (parallel with other languages). Publish only your own module file; REPORT shared-file registration data.',
    '',
    'Real repo: ' + REPO + '. A language = a module implementing the Language trait (fn extract(&self, source:&str)->Vec<Signature>) producing Signature { indent: usize (nesting level, 0=top), kind: Kind(Function|Class|Constant), text (declaration, body removed), line }.',
    '',
    'STEP 1 — PRIVATE TEMP COPY (never build the real repo): WS=$(mktemp -d) && cp ' + REPO + '/Cargo.toml "$WS"/ && cp -r ' + REPO + '/src "$WS"/src. Iterate inside "$WS".',
    'STEP 2 — Read $WS/src/lang/python.rs (reference), mod.rs, signature.rs, render.rs, main.rs. Create $WS/src/lang/' + mod + '.rs (use EXACTLY module name "' + mod + '"): a heuristic, std-only scanner. Functions/methods->Function; class/type-like->Class; module-level constants->Constant. indent=nesting level via a stack. text=normalized one-line declaration, body removed, multi-line joined. Ignore comments and string/char literals. NEVER panic; operate on char boundaries (never index mid-char). To verify it compiles, wire it up IN THE COPY (mod line, for_path arm, render keywords, help). Add #[cfg(test)] tests (top-level fn, nested w/ indent, multi-line join, constant accept+reject, decl-in-comment ignored). Run cargo build && cargo test until green.',
    'STEP 3 — Publish only your file to the real repo: cp "$WS/src/lang/' + mod + '.rs" ' + REPO + '/src/lang/' + mod + '.rs. Do NOT edit real shared files.',
    'STEP 4 — Report: modLine, forPathArm, functionKeywords, classKeywords, constantKeywords, extensions, helpEntry, buildOk/testOk (from the copy), published.',
  ].join('\n')
}

function braceImplPrompt(braceList) {
  return [
    'You are the SERIAL BRACE-FAMILY implementer. Extend the EXISTING shared engine src/lang/braces.rs (a Lang enum + per-language config) to support these brace-family languages: ' + braceList.join(', ') + '. You are the ONLY writer of braces.rs and the shared files in this phase — work directly in the real repo: ' + REPO + ' (cwd).',
    '',
    'For EACH language: add its Lang enum variant and config (comment syntax, keyword sets for Function/Class/Constant, extension(s)), and a for_path() arm in src/lang/mod.rs mapping its extension(s). Extend src/render.rs keyword coloring if it needs new keywords, and add it to print_help in src/main.rs. Reuse the existing config-driven machinery — do NOT duplicate logic. NEVER panic; respect char boundaries (no slicing mid multi-byte char).',
    'Add #[cfg(test)] tests in braces.rs for each new language (a representative declaration of each kind, nesting, a decl-in-comment ignored).',
    'Run "cargo build" and "cargo test" in the real repo until BOTH are green (all existing languages included).',
    'If a listed language is genuinely NOT brace-style and does not fit braces.rs, say so in notes and skip it (it will be handled separately).',
    '',
    'Return: perLanguage[].{language,compiles}, buildOk, testOk, notes.',
  ].join('\n')
}

function registerPrompt(reports) {
  const data = reports.map(r => [
    '## ' + r.language + ' (module ' + r.module + ')',
    'modLine: ' + (r.modLine || ('mod ' + r.module + ';')),
    'forPathArm: ' + (r.forPathArm || '(derive from extensions ' + JSON.stringify(r.extensions || []) + ')'),
    'functionKeywords: ' + JSON.stringify(r.functionKeywords || []),
    'classKeywords: ' + JSON.stringify(r.classKeywords || []),
    'constantKeywords: ' + JSON.stringify(r.constantKeywords || []),
    'helpEntry: ' + (r.helpEntry || r.language),
  ].join('\n')).join('\n\n')
  return [
    'You are the SERIAL REGISTRAR. Dedicated-module languages were implemented in parallel; each published its own src/lang/<module>.rs but did NOT touch shared files. Wire them all into the shared files in ONE pass and verify the crate.',
    'Repo (cwd): ' + REPO + '. Files you edit: src/lang/mod.rs, src/render.rs, src/main.rs. Do NOT rewrite extractor files (already published) or braces.rs (owned by the brace phase).',
    '',
    'LANGUAGES TO REGISTER:',
    data || '(none)',
    '',
    '1. src/lang/mod.rs: add every modLine and for_path arm (before "_ => None"). 2. src/render.rs: keep colorize generic/keyword-driven, adding any new function/class/constant keywords; existing render tests MUST stay green. 3. src/main.rs: extend the print_help supported-languages line. 4. cargo build && cargo test. Report buildOk/testOk and perLanguage[].{language,compiles}.',
  ].join('\n')
}

function fuzzPrompt(lang, srcFile, dir, i, round, focus) {
  return [
    'You are FUZZER #' + i + ' (round ' + round + ') stress-testing the "signatures" CLI support for ' + lang + '. ' + FUZZ_TIMEBOX,
    'Repo (cwd): ' + REPO + '. Binary: ./target/debug/signatures (run "cargo build -q" once; if it FAILS report that as a critical build error and stop). READ src and RUN the binary only — do NOT edit any source file. Put your files only under tests/fuzz/' + dir + '/r' + round + '-a' + i + '/.',
    '',
    'INTENDED BEHAVIOR: one line per signature (functions/methods, classes/types, constants); body removed; nested members indented 2 spaces/level; declarations inside comments/strings ignored; use --no-color for stable diffing. This language\'s logic lives in src/lang/' + srcFile + ' (study it + src/lang/python.rs + the real ' + lang + ' grammar).',
    'FOCUS: ' + focus,
    '',
    '1. mkdir -p tests/fuzz/' + dir + '/r' + round + '-a' + i + '/ and write a handful of HARD ' + lang + ' files (adversarial + a few mutated/random variants to surface panics). 2. Run "./target/debug/signatures --no-color <file>" on each. Watch for: PANICS/crashes/unexpected non-zero exits (panic prints "thread \'main\' panicked"; only OK errors are "unsupported file type"/"No such file or directory"); build failures; WRONG output (real decl missing; fake decl from comment/string emitted; garbled text; wrong indent; body not removed; multi-line not joined). 3. For each genuine defect save a MINIMAL repro and record exact input, command, expected, actual.',
    '',
    'Report ONLY clear, reproducible defects (no nitpicks). Zero errors = success. Return the structured result.',
  ].join('\n')
}

function fixPrompt(errors, round) {
  const list = errors.map((e, n) => [
    '### Defect ' + (n + 1) + ': [' + (e.lang || '?') + '] ' + (e.title || 'untitled') + ' [' + (e.severity || 'unknown') + ']',
    e.description ? 'Description: ' + e.description : '',
    e.repro ? 'Repro file: ' + e.repro : '',
    e.command ? 'Command: ' + e.command : '',
    e.input ? 'Input:\n' + e.input : '',
    e.expected ? 'Expected:\n' + e.expected : '',
    e.actual ? 'Actual:\n' + e.actual : '',
  ].filter(Boolean).join('\n')).join('\n\n')
  return [
    'You are the SINGLE SERIAL FIXER for round ' + round + '. You are the ONLY writer this phase — edit the real repo directly: ' + REPO + ' (cwd). ' + FIX_TIMEBOX,
    '',
    'Many of these defects are the SAME underlying bug reported by different fuzzers/languages (they have been deduped, but related ones often share a root cause — e.g. a single bug in the shared src/lang/braces.rs or src/render.rs affects all brace languages). Group by root cause and fix each root cause ONCE.',
    '',
    'DEFECTS (' + errors.length + '):',
    list,
    '',
    'For each root cause: reproduce (./target/debug/signatures on the input), fix it (in the right file: a per-language module, the shared braces.rs, render.rs, or mod.rs). Zero new deps. Never introduce a panic; respect UTF-8 char boundaries. Add a regression #[cfg(test)] test per fix. Run "cargo build" and "cargo test" until green (no regressions to ANY language). If a reported defect is actually correct behavior, do not change code — note it in "remaining".',
    '',
    'Return: fixed, summary, changedFiles, buildOk, testOk, remaining.',
  ].join('\n')
}

// ---- schemas ----------------------------------------------------------------
const MODULE_IMPL_SCHEMA = {
  type: 'object',
  required: ['language', 'module', 'buildOk', 'testOk', 'published'],
  properties: {
    language: { type: 'string' }, module: { type: 'string' },
    extensions: { type: 'array', items: { type: 'string' } },
    buildOk: { type: 'boolean' }, testOk: { type: 'boolean' }, published: { type: 'boolean' },
    modLine: { type: 'string' }, forPathArm: { type: 'string' },
    functionKeywords: { type: 'array', items: { type: 'string' } },
    classKeywords: { type: 'array', items: { type: 'string' } },
    constantKeywords: { type: 'array', items: { type: 'string' } },
    helpEntry: { type: 'string' }, notes: { type: 'string' },
  },
}
const REGISTER_SCHEMA = {
  type: 'object',
  required: ['buildOk', 'testOk', 'perLanguage'],
  properties: {
    buildOk: { type: 'boolean' }, testOk: { type: 'boolean' },
    perLanguage: { type: 'array', items: { type: 'object', required: ['language', 'compiles'], properties: { language: { type: 'string' }, compiles: { type: 'boolean' }, note: { type: 'string' } } } },
    summary: { type: 'string' }, notes: { type: 'string' },
  },
}
const FUZZ_SCHEMA = {
  type: 'object',
  required: ['anyErrors', 'errors'],
  properties: {
    casesCreated: { type: 'integer' }, anyErrors: { type: 'boolean' },
    errors: { type: 'array', items: { type: 'object', required: ['title', 'severity'], properties: {
      title: { type: 'string' }, severity: { type: 'string', enum: ['panic', 'build', 'wrong-output', 'other'] },
      description: { type: 'string' }, input: { type: 'string' }, command: { type: 'string' },
      expected: { type: 'string' }, actual: { type: 'string' }, repro: { type: 'string' },
    } } },
  },
}
const FIX_SCHEMA = {
  type: 'object',
  required: ['fixed', 'summary', 'buildOk', 'testOk'],
  properties: {
    fixed: { type: 'boolean' }, summary: { type: 'string' },
    changedFiles: { type: 'array', items: { type: 'string' } },
    buildOk: { type: 'boolean' }, testOk: { type: 'boolean' },
    remaining: { type: 'array', items: { type: 'string' } },
  },
}

// ---- helpers ----------------------------------------------------------------
function dedupe(errors) {
  const seen = new Set()
  const out = []
  for (const e of errors || []) {
    const key = (e.title || '').toLowerCase().replace(/\s+/g, ' ').trim() + '|' + (e.input || e.repro || '')
    if (seen.has(key)) continue
    seen.add(key)
    out.push(e)
  }
  return out
}

// ============================================================================
// STAGE A — IMPLEMENT: modules in parallel, brace-family serial
// ============================================================================
log('Languages: ' + langs.join(', ') + ' | brace-family: [' + braceLangs.join(', ') + '] | dedicated modules: [' + moduleLangs.join(', ') + ']')

// Run the (serial) brace extender and the (parallel) module implementers concurrently:
// the brace agent owns braces.rs + shared files; module agents only write their own new
// files in temp copies and report data — so these never touch the same file.
phase('ImplementModules')
const stageA = await parallel([
  ...(braceLangs.length ? [() => {
    phase('ImplementBraces')
    return agent(braceImplPrompt(braceLangs), { schema: REGISTER_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'impl-braces', phase: 'ImplementBraces' })
  }] : []),
  ...moduleLangs.map(L => () =>
    agent(moduleImplPrompt(L, moduleOf[L]), { schema: MODULE_IMPL_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'impl:' + L, phase: 'ImplementModules' })
  ),
])

const braceResult = braceLangs.length ? stageA[0] : null
const moduleResults = (braceLangs.length ? stageA.slice(1) : stageA).filter(Boolean)
const publishedModules = moduleResults.filter(r => r.published)
for (const r of moduleResults) log('impl ' + r.language + ': build=' + r.buildOk + ' test=' + r.testOk + ' published=' + r.published)
if (braceResult) log('impl-braces: build=' + braceResult.buildOk + ' test=' + braceResult.testOk)

// ============================================================================
// STAGE B — REGISTER dedicated-module languages (serial)
// ============================================================================
let registered = []
if (publishedModules.length) {
  phase('Register')
  const reg = await agent(registerPrompt(publishedModules), { schema: REGISTER_SCHEMA, model: 'sonnet', agentType: 'general-purpose', label: 'register', phase: 'Register' })
  log('Register: build=' + reg.buildOk + ' test=' + reg.testOk)
  registered = (reg.perLanguage || []).filter(p => p.compiles).map(p => p.language)
}

// Active languages = those that compiled (brace langs that compiled + registered modules).
const braceCompiled = braceResult ? (braceResult.perLanguage || []).filter(p => p.compiles).map(p => p.language) : []
const activeSet = new Set([...braceCompiled, ...registered])
// Fall back to "all requested" if reporting was sparse but the build is green.
for (const L of langs) if (!activeSet.has(L) && (braceResult ? braceResult.buildOk : true)) activeSet.add(L)
const pending = new Set([...activeSet])
log('Entering hardening for: ' + [...pending].join(', '))

// ============================================================================
// STAGE C — HARDEN: bounded parallel fuzz -> dedup across langs -> single bounded fix
// ============================================================================
const cleanLangs = []
const perLangHistory = {}
let round = 0

while (round < MAX_ROUNDS && pending.size > 0) {
  round++
  const activeLangs = [...pending]

  phase('Fuzz r' + round)
  log('Round ' + round + ': fuzzing ' + activeLangs.length + ' language(s) x ' + FUZZERS + ' (time-boxed): ' + activeLangs.join(', '))
  const jobMeta = []
  const jobs = []
  for (const L of activeLangs) {
    for (let i = 0; i < FUZZERS; i++) {
      jobMeta.push({ lang: L, i: i + 1 })
      jobs.push(() => agent(fuzzPrompt(L, sourceFileOf[L], moduleOf[L], i + 1, round, FOCUS[i % FOCUS.length]), {
        schema: FUZZ_SCHEMA, model: 'sonnet', agentType: 'general-purpose',
        label: 'fuzz:' + L + ':r' + round + '-a' + (i + 1), phase: 'Fuzz r' + round,
      }))
    }
  }
  const fuzzResults = await parallel(jobs)

  // Group per language (to mark clean), and collect a global deduped error batch.
  const errByLang = {}
  const globalErrors = []
  fuzzResults.forEach((r, idx) => {
    if (!r) return
    const L = jobMeta[idx].lang
    const errs = Array.isArray(r.errors) ? r.errors : []
    if (!errByLang[L]) errByLang[L] = []
    errByLang[L].push(...errs)
    for (const e of errs) globalErrors.push({ ...e, lang: L })
  })

  for (const L of activeLangs) {
    const n = dedupe(errByLang[L] || []).length
    if (!perLangHistory[L]) perLangHistory[L] = []
    perLangHistory[L].push({ round, errors: n })
    if (n === 0) { pending.delete(L); cleanLangs.push(L); log(L + ': CLEAN after round ' + round) }
    else log(L + ': ' + n + ' distinct defect(s) in round ' + round)
  }

  const batch = dedupe(globalErrors)
  if (batch.length === 0) { log('Round ' + round + ': no defects across any language.'); break }

  // ---- single bounded serial fixer (dedup means shared bugs fixed once) ----
  phase('Fix r' + round)
  log('Round ' + round + ': one bounded fixer handling ' + Math.min(batch.length, MAX_ERRORS_TO_FIX) + ' deduped defect(s) across ' + activeLangs.length + ' language(s)...')
  const fix = await agent(fixPrompt(batch.slice(0, MAX_ERRORS_TO_FIX), round), {
    schema: FIX_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'fix:r' + round, phase: 'Fix r' + round,
  })
  log('Round ' + round + ' fix: fixed=' + fix.fixed + ' build=' + fix.buildOk + ' test=' + fix.testOk)
  // languages with defects stay pending and get re-fuzzed next round.
}

const stillPending = [...pending]
if (stillPending.length) log('Reached MAX_ROUNDS=' + MAX_ROUNDS + ' with defects remaining for: ' + stillPending.join(', '))

return {
  languages: langs,
  braceLangs, moduleLangs,
  clean: cleanLangs,
  stillPending,
  rounds: round,
  perLangHistory,
}
