export const meta = {
  name: 'implement-new-language',
  description: 'Add one or many languages to the signatures CLI: opus implements each extractor (brace-family share braces.rs serially; others in parallel) -> sonnet generates many test fixtures + expected snapshots under tests/<lang>/ -> opus runs tests/run.sh and fixes until it passes. Agents are time-boxed so the run never stalls.',
  whenToUse: 'Add + test source languages. args="Rust" | ["Kotlin","Ruby"] | {languages:[...], rounds:3}. Implement (opus) -> generate tests (sonnet) -> test & fix until pass (opus).',
  phases: [
    { title: 'ImplementModules', detail: 'parallel opus, one per dedicated-module language (temp-isolated)', model: 'opus' },
    { title: 'ImplementBraces', detail: 'one serial opus extends src/lang/braces.rs for all brace-family languages', model: 'opus' },
    { title: 'Register', detail: 'one serial sonnet wires module languages into shared files + builds/tests', model: 'sonnet' },
    { title: 'GenerateTests', detail: 'one sonnet per language writes many fixtures + .expected snapshots under tests/<lang>/', model: 'sonnet' },
    { title: 'Test', detail: 'run tests/run.sh and collect failures', model: 'sonnet' },
    { title: 'Fix', detail: 'one bounded opus fixes failures (extractor or snapshot) and re-runs until green', model: 'opus' },
  ],
}

// ---- inputs / knobs ---------------------------------------------------------
const REPO = '/home/eren/work2/signature'
const FILES_PER_LANG = 8

let A = args
if (typeof A === 'string') {
  const t = A.trim()
  if (t.startsWith('{') || t.startsWith('[')) { try { A = JSON.parse(t) } catch (_) {} }
}
const cfg = (A && typeof A === 'object' && !Array.isArray(A)) ? A : {}
const MAX_ROUNDS = Number.isFinite(cfg.rounds) ? Math.max(1, cfg.rounds) : 3

const rawLangs = Array.isArray(A) ? A
  : Array.isArray(cfg.languages) ? cfg.languages
  : (typeof cfg.languages === 'string' ? cfg.languages.split(/[,\s]+/)
    : (typeof A === 'string' ? A.split(/[,\s]+/) : []))
const RESERVED = new Set(['languages', 'fuzzers', 'rounds'])
const langs = rawLangs.map(s => String(s).trim()).filter(Boolean)
  .filter(s => !RESERVED.has(s.toLowerCase()) && !/^\d+$/.test(s))
if (langs.length === 0) throw new Error('implement-new-language needs a language. args="Rust" or ["Kotlin","Ruby"].')

const TIMEBOX_GEN = 'TIME-BOX: ~4 minutes — generate the files, do not loop forever.'
const TIMEBOX_FIX = 'TIME-BOX: ~6 minutes and at most 3 build/test cycles. If not fully green in time, return partial progress in "remaining" — never loop indefinitely.'

// ---- language families ------------------------------------------------------
const ALIAS = {
  'c++': 'cpp', 'cpp': 'cpp', 'cplusplus': 'cpp', 'c#': 'csharp', 'cs': 'csharp', 'csharp': 'csharp',
  'objective-c': 'objc', 'objectivec': 'objc', 'objc': 'objc', 'f#': 'fsharp',
  'golang': 'go', 'js': 'javascript', 'ts': 'typescript',
}
const BRACE_FAMILY = new Set(['rust', 'go', 'c', 'cpp', 'csharp', 'java', 'javascript', 'typescript', 'swift', 'kotlin', 'scala', 'php', 'dart', 'objc', 'zig'])
function normKey(lang) {
  const low = lang.toLowerCase().trim()
  return ALIAS[low] || (low.replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '') || 'lang')
}
const moduleOf = {}, sourceFileOf = {}, isBrace = {}, dirOf = {}
const used = new Set()
for (const L of langs) {
  const key = normKey(L)
  const brace = BRACE_FAMILY.has(key)
  isBrace[L] = brace
  sourceFileOf[L] = brace ? 'braces.rs' : key + '.rs'
  dirOf[L] = key
  let name = key, n = 2
  while (!brace && used.has(name)) { name = key + '_' + n; n++ }
  used.add(name)
  moduleOf[L] = name
}
const braceLangs = langs.filter(L => isBrace[L])
const moduleLangs = langs.filter(L => !isBrace[L])

// ---- prompts ----------------------------------------------------------------
function moduleImplPrompt(lang, mod) {
  return [
    'Implement "signatures" support for ' + lang + ' as a SELF-CONTAINED extractor module, IN ISOLATION (parallel with other languages). Publish only your own module file; REPORT shared-file registration data.',
    'Real repo: ' + REPO + '. A language = a module implementing the Language trait (fn extract(&self, source:&str)->Vec<Signature>) producing Signature { indent: usize (nesting level, 0=top), kind: Kind(Function|Class|Constant), text (declaration, body removed), line }.',
    'STEP 1 — PRIVATE TEMP COPY: WS=$(mktemp -d) && cp ' + REPO + '/Cargo.toml "$WS"/ && cp -r ' + REPO + '/src "$WS"/src. Iterate inside "$WS".',
    'STEP 2 — Read $WS/src/lang/python.rs (reference), mod.rs, signature.rs, render.rs, main.rs. Create $WS/src/lang/' + mod + '.rs (module name EXACTLY "' + mod + '"): heuristic std-only scanner. Functions/methods->Function; class/type-like->Class; module-level constants->Constant. indent=nesting level via a stack. text=normalized one-line declaration, body removed, multi-line joined. Ignore comments and string/char literals. NEVER panic; respect char boundaries. Wire it up in the copy (mod line, for_path arm, render keywords, help) and add #[cfg(test)] tests. cargo build && cargo test until green.',
    'STEP 3 — Publish only your file: cp "$WS/src/lang/' + mod + '.rs" ' + REPO + '/src/lang/' + mod + '.rs. Do NOT edit real shared files.',
    'STEP 4 — Report modLine, forPathArm, functionKeywords, classKeywords, constantKeywords, extensions, helpEntry, buildOk/testOk, published.',
  ].join('\n')
}
function braceImplPrompt(list) {
  return [
    'You are the SERIAL BRACE-FAMILY implementer. Extend the EXISTING shared engine src/lang/braces.rs (a Lang enum + per-language config) for: ' + list.join(', ') + '. You are the ONLY writer of braces.rs and shared files here — work directly in the real repo: ' + REPO + ' (cwd).',
    'For EACH language: add its Lang variant + config (comment syntax, Function/Class/Constant keyword sets, extensions), a for_path() arm in src/lang/mod.rs, any new keywords in src/render.rs, and an entry in print_help in src/main.rs. Reuse the config-driven machinery; do not duplicate logic. NEVER panic; respect char boundaries. Add #[cfg(test)] tests for each. cargo build && cargo test until green (all languages).',
    'Return perLanguage[].{language,compiles}, buildOk, testOk, notes.',
  ].join('\n')
}
function registerPrompt(reports) {
  const data = reports.map(r => ['## ' + r.language + ' (module ' + r.module + ')',
    'modLine: ' + (r.modLine || ('mod ' + r.module + ';')),
    'forPathArm: ' + (r.forPathArm || '(from extensions ' + JSON.stringify(r.extensions || []) + ')'),
    'functionKeywords: ' + JSON.stringify(r.functionKeywords || []),
    'classKeywords: ' + JSON.stringify(r.classKeywords || []),
    'constantKeywords: ' + JSON.stringify(r.constantKeywords || []),
    'helpEntry: ' + (r.helpEntry || r.language)].join('\n')).join('\n\n')
  return [
    'SERIAL REGISTRAR. Dedicated-module languages were implemented in parallel and published their own src/lang/<module>.rs but did NOT touch shared files. Wire them all in one pass and verify.',
    'Repo (cwd): ' + REPO + '. Edit src/lang/mod.rs, src/render.rs, src/main.rs. Do NOT rewrite extractor files or braces.rs.',
    'LANGUAGES:', data || '(none)',
    '1. mod.rs: add each modLine + for_path arm (before "_ => None"). 2. render.rs: keep colorize generic/keyword-driven, add new keywords; existing render tests MUST stay green. 3. main.rs: extend print_help. 4. cargo build && cargo test. Report buildOk/testOk + perLanguage[].{language,compiles}.',
  ].join('\n')
}
function genTestsPrompt(lang, dir, srcFile) {
  return [
    'Generate a thorough TEST FIXTURE SET for ' + lang + ' for the "signatures" CLI, under tests/' + dir + '/. ' + TIMEBOX_GEN,
    'Repo (cwd): ' + REPO + '. Build once: cargo build -q. Binary: ./target/debug/signatures.',
    'The tool prints one line per signature (functions/methods, classes/types, constants), body removed, nested members indented 2 spaces/level, declarations inside comments/strings ignored. ' + lang + ' logic lives in src/lang/' + srcFile + '. STUDY the EXACT output format from existing snapshots: tests/python/*.expected and tests/' + dir + '/*.expected if present.',
    '',
    'Create ' + FILES_PER_LANG + ' varied ' + lang + ' source files in tests/' + dir + '/ with descriptive names, covering: a realistic real-world-style file; deep nesting; multi-line signatures / generics; comments & strings containing FAKE declarations (must be ignored); unicode identifiers; constants vs non-constant lookalikes; modifiers/decorators/attributes; and one malformed-but-parseable file.',
    'For EACH source file ALSO write a sibling snapshot "<file>.expected" containing the CORRECT expected `signatures --no-color <file>` output. Match the tool\'s exact formatting, but the SET of signatures and their NESTING must be CORRECT for the language — author it from the real grammar, do NOT blindly paste current tool output. Run the tool to compare; if the tool output is wrong, keep your CORRECT expected (the fixer will make the tool match). If you realize your expected was wrong, correct it.',
    '',
    'Do not edit any src/ file. Return: filesCreated (int), and disagreements: short notes on cases where current tool output differs from your correct expected.',
  ].join('\n')
}
function runTestsPrompt(activeLangs) {
  return [
    'Run the fixture suite and report results. Repo (cwd): ' + REPO + '.',
    'Run: ./tests/run.sh ' + activeLangs.map(L => dirOf[L]).join(' '),
    'Do NOT edit anything. Report allPass (bool), counts, and for each FAIL the file path plus the expected-vs-actual diff (concise).',
  ].join('\n')
}
function fixPrompt(activeLangs, round) {
  return [
    'SINGLE SERIAL FIXER, round ' + round + '. You are the only writer — edit the real repo directly: ' + REPO + ' (cwd). ' + TIMEBOX_FIX,
    'Run ./tests/run.sh ' + activeLangs.map(L => dirOf[L]).join(' ') + ' to see failures (expected-vs-actual diffs).',
    'For EACH failing fixture decide the root cause: usually the EXTRACTOR is wrong → fix it (a per-language module, the shared src/lang/braces.rs, src/render.rs, or src/lang/mod.rs). Only edit a "<file>.expected" snapshot if the expected was genuinely incorrect. Group shared-code bugs (e.g. in braces.rs/render.rs) and fix each root cause once. Zero new deps; never introduce a panic; respect UTF-8 boundaries.',
    'Re-run ./tests/run.sh until it passes, and keep "cargo test" green (no regressions). Return: allPass, summary, changedFiles, remaining.',
  ].join('\n')
}

// ---- schemas ----------------------------------------------------------------
const MODULE_IMPL_SCHEMA = { type: 'object', required: ['language', 'module', 'buildOk', 'testOk', 'published'], properties: {
  language: { type: 'string' }, module: { type: 'string' }, extensions: { type: 'array', items: { type: 'string' } },
  buildOk: { type: 'boolean' }, testOk: { type: 'boolean' }, published: { type: 'boolean' },
  modLine: { type: 'string' }, forPathArm: { type: 'string' },
  functionKeywords: { type: 'array', items: { type: 'string' } }, classKeywords: { type: 'array', items: { type: 'string' } },
  constantKeywords: { type: 'array', items: { type: 'string' } }, helpEntry: { type: 'string' }, notes: { type: 'string' } } }
const REGISTER_SCHEMA = { type: 'object', required: ['buildOk', 'testOk', 'perLanguage'], properties: {
  buildOk: { type: 'boolean' }, testOk: { type: 'boolean' },
  perLanguage: { type: 'array', items: { type: 'object', required: ['language', 'compiles'], properties: { language: { type: 'string' }, compiles: { type: 'boolean' }, note: { type: 'string' } } } },
  summary: { type: 'string' }, notes: { type: 'string' } } }
const GEN_SCHEMA = { type: 'object', required: ['filesCreated'], properties: {
  filesCreated: { type: 'integer' }, disagreements: { type: 'array', items: { type: 'string' } }, notes: { type: 'string' } } }
const TEST_SCHEMA = { type: 'object', required: ['allPass'], properties: {
  allPass: { type: 'boolean' }, pass: { type: 'integer' }, fail: { type: 'integer' },
  failures: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, diff: { type: 'string' } } } } } }
const FIX_SCHEMA = { type: 'object', required: ['allPass', 'summary'], properties: {
  allPass: { type: 'boolean' }, summary: { type: 'string' }, changedFiles: { type: 'array', items: { type: 'string' } }, remaining: { type: 'array', items: { type: 'string' } } } }

// ============================================================================
// STAGE A — IMPLEMENT (modules parallel, brace-family serial)
// ============================================================================
log('Languages: ' + langs.join(', ') + ' | brace: [' + braceLangs.join(', ') + '] | modules: [' + moduleLangs.join(', ') + ']')
phase('ImplementModules')
const stageA = await parallel([
  ...(braceLangs.length ? [() => { phase('ImplementBraces'); return agent(braceImplPrompt(braceLangs), { schema: REGISTER_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'impl-braces', phase: 'ImplementBraces' }) }] : []),
  ...moduleLangs.map(L => () => agent(moduleImplPrompt(L, moduleOf[L]), { schema: MODULE_IMPL_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'impl:' + L, phase: 'ImplementModules' })),
])
const braceResult = braceLangs.length ? stageA[0] : null
const moduleResults = (braceLangs.length ? stageA.slice(1) : stageA).filter(Boolean)
const publishedModules = moduleResults.filter(r => r.published)
for (const r of moduleResults) log('impl ' + r.language + ': build=' + r.buildOk + ' test=' + r.testOk + ' published=' + r.published)
if (braceResult) log('impl-braces: build=' + braceResult.buildOk + ' test=' + braceResult.testOk)

// ============================================================================
// STAGE B — REGISTER module languages (serial)
// ============================================================================
let registered = []
if (publishedModules.length) {
  phase('Register')
  const reg = await agent(registerPrompt(publishedModules), { schema: REGISTER_SCHEMA, model: 'sonnet', agentType: 'general-purpose', label: 'register', phase: 'Register' })
  log('Register: build=' + reg.buildOk + ' test=' + reg.testOk)
  registered = (reg.perLanguage || []).filter(p => p.compiles).map(p => p.language)
}
const braceCompiled = braceResult ? (braceResult.perLanguage || []).filter(p => p.compiles).map(p => p.language) : []
const active = new Set([...braceCompiled, ...registered])
for (const L of langs) if (!active.has(L) && (braceResult ? braceResult.buildOk : true)) active.add(L)
const activeLangs = [...active]
log('Generating tests for: ' + activeLangs.join(', '))

// ============================================================================
// STAGE C — GENERATE TESTS (one sonnet per language, parallel)
// ============================================================================
phase('GenerateTests')
const gens = await parallel(activeLangs.map(L => () =>
  agent(genTestsPrompt(L, dirOf[L], sourceFileOf[L]), { schema: GEN_SCHEMA, model: 'sonnet', agentType: 'general-purpose', label: 'gen-tests:' + L, phase: 'GenerateTests' })))
let totalFiles = 0
gens.filter(Boolean).forEach((g, i) => { totalFiles += g.filesCreated || 0; if ((g.disagreements || []).length) log(activeLangs[i] + ' gen: ' + g.disagreements.length + ' tool/expected disagreement(s)') })
log('Generated ~' + totalFiles + ' fixtures across ' + activeLangs.length + ' language(s).')

// ============================================================================
// STAGE D — TEST -> FIX until pass (bounded)
// ============================================================================
let round = 0
let passed = false
while (round < MAX_ROUNDS) {
  round++
  phase('Test r' + round)
  const test = await agent(runTestsPrompt(activeLangs), { schema: TEST_SCHEMA, model: 'sonnet', agentType: 'general-purpose', label: 'test:r' + round, phase: 'Test r' + round })
  log('Round ' + round + ' test: allPass=' + test.allPass + ' (pass=' + (test.pass || 0) + ' fail=' + (test.fail || 0) + ')')
  if (test.allPass) { passed = true; break }

  phase('Fix r' + round)
  const fix = await agent(fixPrompt(activeLangs, round), { schema: FIX_SCHEMA, model: 'opus', agentType: 'general-purpose', label: 'fix:r' + round, phase: 'Fix r' + round })
  log('Round ' + round + ' fix: allPass=' + fix.allPass + ' — ' + (fix.summary || '').slice(0, 120))
  if (fix.allPass) { passed = true; break }
}
if (!passed) log('Reached MAX_ROUNDS=' + MAX_ROUNDS + ' without a fully green suite.')

return { languages: langs, braceLangs, moduleLangs, active: activeLangs, rounds: round, passed, fixturesGenerated: totalFiles }
