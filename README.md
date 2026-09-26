# diver
Find knowledge, not just papers, on ArXiv

## Commands

| Command | Description |
|---------|-------------|
| `diver search <query>` | Search arXiv (remote) |
| `diver collect <query>` | Search arXiv and ingest all matching papers |
| `diver ingest <arxiv-id>` | Ingest a single paper by arXiv ID |
| `diver find <query>` | Search your local corpus (FTS) |
| `diver inspect <arxiv-id>` | Show full metadata, taxonomy-resolved categories, and version history |
| `diver extract <arxiv-id>` | Extract grounded, supported claims from a stored paper's abstract (uses the Claude API) and persist them |
| `diver assertions <arxiv-id>` | Show the assertions previously extracted and stored for a paper |
| `diver dive <concept>` | Explore a concept: papers that assert about it and how they connect |
| `diver list` | List all ingested papers |

## Concept exploration (`diver dive`)

`diver dive <term>` traverses the **extracted knowledge graph**. It resolves the term
to a **concept**, shows every stored claim that asserts about that concept, and lists
the papers each is related to via deterministic edges.

### Concepts

Concepts are formed from your stored claims, deterministically — no model decides
what a concept is — and rebuilt automatically whenever claims change:

- **Words.** Every distinctive word is a concept. Plural forms are the same concept:
  `diver dive networks` also finds the papers that only say `network`, and the header
  lists every surface form seen (`forms: network ×5, networks ×5`).
- **Phrases.** Two adjacent words that at least two papers share are a concept in their
  own right — `machine translation`, `diffusion models`, `attention mechanism`. A phrase
  is *linked to* its words, not merged into them: the header of a word lists its
  `narrower:` phrases, and the header of a phrase lists its `broader:` words.
- **Filler never stands alone.** Generic words (`model`, `method`, `results`) and common
  English are never concepts by themselves, though filler can end a phrase
  (`language model`). A common-word stoplist and a research-filler stoplist decide this.
- **Unresolved terms suggest.** A term that is not a concept says so and lists the
  concepts containing it: `diver dive model` offers `diffusion models`,
  `transformer model`, … For a multi-word query, concepts covering more of its words
  come first (`quantum error correction` offers `error correction`, `quantum error`),
  and a misspelling gets a *did you mean* (`atention` → `attention`).
- **No substring matching.** Earlier versions matched any claim *containing* the text,
  so `gan` matched "or**gan**ized". Concepts are whole words, which also means
  derivational variants are distinct: `attention` does not reach `attentional`.

### Edges

- **shared category** or **shared author** (structural), and
- **co-assertion** — the two papers' claims share a concept, so `dive` links papers by
  *what they assert*, not only their metadata. Each edge names the concept and is
  weighted by its inverse document frequency across the corpus (rarer concepts score
  higher). When two papers share a phrase, they are linked by the phrase alone — not
  additionally by each of its words.

Each paper in a dive lists its related papers **grouped and ranked**: first a summary
(`Related: 12 by shared concepts; 1 by shared authors only; 90 sharing only a category
(cs.LG, cs.CL)`), then the top five papers linked by concepts or authors, with titles
and their strongest shared concepts (`shares positional encoding (0.83), lstm (0.80)`).
Papers that only share an arXiv category are counted, never listed — at corpus scale a
category links hundreds of papers and says nothing about what they claim.

### Temperature (`--temperature`)

`diver dive <term> --temperature <t>` tunes how permissive co-assertion linking is,
with `t` in `[0.0, 1.0]` (default **0.5**):

- **low** (→ 0.0) links papers only on rare, distinctive shared concepts — a sparse,
  high-signal graph;
- **high** (→ 1.0) also links on common shared concepts — a denser graph. `1.0` links
  every shared concept, except words already covered by a phrase the same two papers
  share.

Only co-assertion edges are affected; structural (category/author) edges do not depend
on temperature.

Because `dive` reads the persisted assertions, run `diver extract` on the papers
you care about first — a paper with no extracted assertions won't appear as a
`dive` seed. (For plain abstract search, use `diver find`.)

```sh
diver extract 2301.00001                 # persist this paper's assertions
diver dive attention                     # explore (default temperature 0.5)
diver dive "machine translation"         # a phrase concept
diver dive attention --temperature 0.0   # only the most distinctive links
diver dive attention --temperature 1.0   # every shared concept, minus words a shared phrase covers
```

## Claim extraction (`diver extract`)

`diver extract <arxiv-id>` asks a model to read the stored paper's abstract and
extract the factual claims it makes, each with a supporting quote, as **structured
output**. A claim is only kept if its quote is **grounded** in the abstract
(hallucinated claims are dropped), and every claim passes the typestate validation
gate before it is shown.

The extractor is **agent-agnostic**: it speaks two compiled provider *shapes* and
selects one from hot-loadable config, so the same grounded, gated pipeline runs on
Claude, OpenAI/Codex, Grok, or a local model:

- **`anthropic`** — Anthropic Messages API, a forced `record_claims` tool.
- **`openai`** — OpenAI-compatible Chat Completions with a `json_schema`
  `response_format`. Covers OpenAI, Grok, and local llama.cpp/Ollama/vLLM servers —
  including **[Animus_Ferric](https://github.com/crussella0129/Animus_Ferric)** via
  `ferric server up` (an OpenAI-compatible server on `127.0.0.1:8080`).

### Providers config

Providers are defined in `providers.json` (at your platform config dir under
`diver/`, or point `DIVER_PROVIDERS_CONFIG` at any path). API keys are **never**
stored in the file — each provider names an `api_key_env`. The active provider is
the file's `"active"`, overridable with `DIVER_PROVIDER`. Edits take effect on the
next run (hot-loadable); a front-end embedding `diver-core` can also build an
extractor directly from a `ProviderConfig`.

```json
{ "active": "claude",
  "providers": {
    "claude": { "shape": "anthropic", "base_url": "https://api.anthropic.com", "model": "claude-opus-5", "api_key_env": "ANTHROPIC_API_KEY" },
    "openai": { "shape": "openai",    "base_url": "https://api.openai.com",     "model": "gpt-4o",       "api_key_env": "OPENAI_API_KEY" },
    "grok":   { "shape": "openai",    "base_url": "https://api.x.ai",           "model": "grok-2",       "api_key_env": "XAI_API_KEY" },
    "animus": { "shape": "openai",    "base_url": "http://127.0.0.1:8080",       "model": "your-model.gguf", "api_key_env": "ANIMUS_API_KEY" } } }
```

With **no** `providers.json`, extraction falls back to today's behavior: the
`anthropic` shape from `ANTHROPIC_API_KEY` / `DIVER_MODEL` / `ANTHROPIC_BASE_URL`.
Pass **`--deterministic`** to use the offline sentence-splitter instead of any API
(no key, no network, no cost).

```sh
export ANTHROPIC_API_KEY=sk-ant-...
diver extract 2301.00001                 # active provider (default: anthropic env)
DIVER_PROVIDER=animus diver extract 2301.00001   # local Animus_Ferric model
diver extract 2301.00001 --deterministic # offline, no API call
diver assertions 2301.00001              # show the stored assertions
```

`diver extract` **persists** the supported assertions it produces (idempotently
per paper+version — re-extracting replaces the prior set), so `diver assertions`
reads them back without re-running extraction. Only validated assertions are
stored: `Store::save_assertions` accepts a `&[Assertion<Supported>]`, so the
database can only ever hold claims that passed the validation gate.

### Building a corpus

`diver extract --all` extracts every stored paper, so building a real dive corpus is
`collect` → `extract --all` → `dive`. With `--deterministic` it needs no API key or
network for the extraction step:

```sh
diver collect "attention transformer neural machine translation" --max-results 6
diver extract --all --deterministic      # extract every ingested paper (offline)
diver dive attention                     # explore the weighted knowledge graph
```

The arXiv client is rate-limited to **one request every 3 seconds** (arXiv's requested
politeness), so repeated `collect`/`ingest` calls automatically space themselves and won't
trip arXiv's `429` throttle. `dive`, `extract --deterministic`, and the other local commands
make no network calls at all.

### Corpus location (`DIVER_DB`)

By default the corpus lives at the platform data directory — `~/.local/share/diver/diver.db`
on Linux, `%APPDATA%\diver\diver.db` on Windows. Set **`DIVER_DB`** to keep a corpus
somewhere else, which is how you build a scratch or fixture corpus without disturbing your
main one:

```sh
DIVER_DB=/tmp/scratch/diver.db diver collect "denoising diffusion" --max-results 5
DIVER_DB=/tmp/scratch/diver.db diver extract --all --deterministic
DIVER_DB=/tmp/scratch/diver.db diver dive diffusion
```

Missing parent directories are created. An **unset or empty** `DIVER_DB` selects the
default — `DIVER_DB=` is deliberately treated as unset, because SQLite reads an empty
filename as a private temporary database that would be discarded on exit.

> **Warning:** `DIVER_DB` silently redirects the corpus. A stray value left exported in
> your shell will make `list`, `find`, and `dive` look empty — the papers are not lost,
> you are simply pointed at a different database. Check the variable before concluding a
> corpus has disappeared.

## Database compatibility

> **Warning:** If you have a `diver.db` created before Sprint 5, you must delete it before running the new binary. The schema changed from a single `source_facts` table to `papers` + `paper_versions`. The binary will recreate the schema automatically on first run.
>
> - Linux/macOS: `rm -rf ~/.local/share/diver/`
> - Windows: delete `%APPDATA%\diver\`
