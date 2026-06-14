# signatures

CLI that prints the signatures of source file.

Supported: Python (`.py`, `.pyi`), Rust (`.rs`), Go (`.go`), JavaScript (`.js`, `.mjs`, `.cjs`, `.jsx`), TypeScript (`.ts`, `.tsx`, `.mts`, `.cts`), Java (`.java`), C (`.c`, `.h`), C++ (`.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx`), C# (`.cs`, `.csx`), Kotlin (`.kt`, `.kts`), Swift (`.swift`), PHP (`.php`, `.phtml`), Scala (`.scala`, `.sc`), Dart (`.dart`), Ruby (`.rb`, `.rake`, `.gemspec`), Lua (`.lua`), Bash (`.sh`, `.bash`).

## Install

```
bash scripts/install.sh
```

Builds the release binary and installs it to `~/.local/bin/signatures`.

## Usage

```
usage: signatures [--format <plain|jsonl>] [--output <truncated|full>] [--stream] [--no-color] [--help] [--version] <file>...

Options:
  --format <plain|jsonl>      output format (default: plain). jsonl emits one JSON object per signature per line (no color).
  --output <truncated|full>   detail level (default: truncated). full prints the complete verbatim source of each top-level decl.
  --stream                    stream each finding as produced (flush per line)
  --no-color                  disable ANSI colors (colors are on by default)
  -h, --help                  show this help
      --version               show version
```

A single file (indentation reflects nesting):

```
$ signatures sample.py
MAX_RETRIES = …
DEFAULT_NAME: str = …
def greet(name="world"):
async def fetch(url, *, timeout=30, retries=MAX_RETRIES):
class Greeter(Base):
  GREETING = …
  def __init__(self, name):
  async def greet(self) -> str:
```

Multiple files or a whole directory:

```
$ signatures src/*.rs lib/utils.py
```

Print the complete source of each top-level declaration (`--output full`) — the
outline expanded to whole bodies; nested members appear inside their parent's
block rather than as separate lines:

```
$ signatures --output full demo.py
MAX = 5

class Greeter:
    def greet(self):
        return "hi"
```

Machine-readable output (one JSON object per signature per line):

```
$ signatures --format jsonl sample.py
```
