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

`diver extract <arxiv-id>` asks Claude to read the stored paper's abstract and
extract the factual claims it makes, each with a supporting quote. A claim is
only kept if its quote is **grounded** in the abstract (hallucinated claims are
dropped), and every claim passes the typestate validation gate before it is
shown.

- Requires **`ANTHROPIC_API_KEY`** in the environment.
- The model defaults to `claude-opus-5`; override it with **`DIVER_MODEL`**
  (e.g. `DIVER_MODEL=claude-haiku-4-5` for a cheaper run).
- The API root defaults to `https://api.anthropic.com`; override it with
  **`ANTHROPIC_BASE_URL`** to point at a proxy or a mock server (the request goes
  to `{base}/v1/messages`).
- Pass **`--deterministic`** to use the offline sentence-splitter instead of the
  API (no key, no network, no cost).

```sh
export ANTHROPIC_API_KEY=sk-ant-...
diver extract 2301.00001                 # LLM extraction (default), persists results
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
