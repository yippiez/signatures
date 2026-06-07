export const meta = {
  name: 'fuzz-all-languages',
  description: 'Fuzz EVERY implemented signatures language by running the fuzz-language workflow once per language (skipping already-fuzzed ones by default), capturing each language\'s distinct bugs as regression fixtures under tests/<lang>/. Captures bugs only — does NOT fix extractors.',
  whenToUse: 'Sweep-fuzz the whole language set. args={} fuzzes all-but-already-done; {languages:[...]} restricts; {skip:[...]} excludes; {fuzzers:N} per-language fuzzer count.',
  phases: [
    { title: 'FuzzAll', detail: 'run fuzz-language per language (parallel; safe — each writes only its own tests/<lang>/)' },
  ],
}

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

const fuzzers = Number.isFinite(cfg.fuzzers) ? Math.max(1, cfg.fuzzers) : 12
// Python + Rust were already fuzzed (their fixtures are committed in the tree),
// so skip them by default to avoid duplicate fixtures. Override with {skip:[]}.
const skip = new Set((Array.isArray(cfg.skip) ? cfg.skip : ['python', 'rust']).map(s => String(s).toLowerCase().trim()))
const requested = Array.isArray(A) ? A.map(String)
  : Array.isArray(cfg.languages) ? cfg.languages.map(String)
  : ALL
const langs = requested.map(s => s.toLowerCase().trim()).filter(Boolean).filter(l => !skip.has(l))

if (langs.length === 0) throw new Error('fuzz-all-languages: no languages left to fuzz after applying skip list.')

phase('FuzzAll')
log('Fuzzing ' + langs.length + ' language(s) with ' + fuzzers + ' fuzzers each: ' + langs.join(', ') +
  (skip.size ? '  (skipping: ' + [...skip].join(', ') + ')' : ''))

const results = await parallel(langs.map(l => () =>
  workflow('fuzz-language', { language: l, fuzzers })
    .then(r => ({ language: l, ok: true, ...r }))
    .catch(e => ({ language: l, ok: false, error: String(e && e.message || e) }))
))

const done = results.filter(Boolean)
const summary = done.map(r => r.ok
  ? '  ' + r.language.padEnd(11) + '→ ' + (r.failures || 0) + ' failure(s), ' + (r.testsCreated || 0) + ' fixture(s) created'
  : '  ' + r.language.padEnd(11) + '→ ERROR: ' + r.error
).join('\n')
log('Sweep complete:\n' + summary)

const totalFixtures = done.reduce((n, r) => n + (r.ok ? (r.testsCreated || 0) : 0), 0)
return {
  languagesFuzzed: done.filter(r => r.ok).length,
  languagesErrored: done.filter(r => !r.ok).map(r => r.language),
  totalFixturesCreated: totalFixtures,
  perLanguage: done,
}
