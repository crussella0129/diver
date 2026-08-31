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
| `diver list` | List all ingested papers |

> **Note:** `diver dive` is reserved for a future sprint — graph traversal is not yet implemented.

## Claim extraction (`diver extract`)

`diver extract <arxiv-id>` asks Claude to read the stored paper's abstract and
extract the factual claims it makes, each with a supporting quote. A claim is
only kept if its quote is **grounded** in the abstract (hallucinated claims are
dropped), and every claim passes the typestate validation gate before it is
shown.

- Requires **`ANTHROPIC_API_KEY`** in the environment.
- The model defaults to `claude-opus-5`; override it with **`DIVER_MODEL`**
  (e.g. `DIVER_MODEL=claude-haiku-4-5` for a cheaper run).
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
