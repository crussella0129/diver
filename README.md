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
| `diver extract <arxiv-id>` | Extract supported assertions from a stored paper's abstract |
| `diver list` | List all ingested papers |

> **Note:** `diver dive` is reserved for a future sprint — graph traversal is not yet implemented.

## Database compatibility

> **Warning:** If you have a `diver.db` created before Sprint 5, you must delete it before running the new binary. The schema changed from a single `source_facts` table to `papers` + `paper_versions`. The binary will recreate the schema automatically on first run.
>
> - Linux/macOS: `rm -rf ~/.local/share/diver/`
> - Windows: delete `%APPDATA%\diver\`
