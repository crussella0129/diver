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

`diver dive <concept>` traverses the **extracted knowledge graph**: it finds the
papers whose stored assertions mention the concept, shows the matching claims, and
lists the papers each is related to via deterministic edges:

- **shared category** or **shared author** (structural), and
- **co-assertion** — the papers' stored claims share a significant term, so `dive`
  links papers by *what they assert*, not only their metadata. Each co-assertion
  edge is weighted by the term's inverse document frequency across the corpus
  (rarer, more distinctive terms score higher), shown as `co-asserts <term> (w=…)`.

### Temperature (`--temperature`)

`diver dive <concept> --temperature <t>` tunes how permissive co-assertion linking
is, with `t` in `[0.0, 1.0]` (default **0.5**):

- **low** (→ 0.0) links papers only on rare, distinctive shared terms — a sparse,
  high-signal graph;
- **high** (→ 1.0) also links on common shared terms — a denser graph. `1.0`
  admits every shared term (the original unweighted behavior).

Only co-assertion edges are affected; structural (category/author) edges are always
shown.

Because `dive` reads the persisted assertions, run `diver extract` on the papers
you care about first — a paper with no extracted assertions won't appear as a
`dive` seed. (For plain abstract search, use `diver find`.)

```sh
diver extract 2301.00001            # persist this paper's assertions
diver dive attention                # explore (default temperature 0.5)
diver dive attention --temperature 0.0   # only the most distinctive links
diver dive attention --temperature 1.0   # every shared term links
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

## Database compatibility

> **Warning:** If you have a `diver.db` created before Sprint 5, you must delete it before running the new binary. The schema changed from a single `source_facts` table to `papers` + `paper_versions`. The binary will recreate the schema automatically on first run.
>
> - Linux/macOS: `rm -rf ~/.local/share/diver/`
> - Windows: delete `%APPDATA%\diver\`
