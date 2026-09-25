# INT-0025 — A machine-readable interface to find and dive

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0025
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Everything Diver computes reaches the outside world as `println!` text built in
`display.rs`. A person at a terminal can read it; nothing else can use it. A front end,
an agent, or a script would have to scrape terminal output.

Expose the same results as structured data, in two layers:

- **`--json` on the read commands** (`find`, `dive`, `list`, `inspect`, `assertions`):
  the exact result the text view renders, serialized, with a documented and versioned
  shape. The text view becomes one renderer of that value, not the place it is built.
- **A local HTTP API** (`diver serve`, loopback only by default) over the same values, so a
  long-lived process keeps the store open and a UI need not spawn a process per query.

The dive result carries everything the text view shows and more it currently drops: the
resolved concept (id, label, kind, forms, paper count), linked concepts, each asserting
paper with its claims **and their support quotes**, and grouped related papers with every
shared concept and weight — not only the top five.

Non-goals:
- No write operations over HTTP in the first version (`collect`, `ingest`, `extract` stay
  CLI-only), so exposing the API never lets a page mutate the corpus.
- No authentication beyond binding to loopback; not a network service.
- No new computation: the API serves exactly what the CLI computes.

## Acceptance criteria

1. `diver dive <term> --json` emits one JSON document for resolved and unresolved terms
   alike (a discriminated result), including support quotes and all related papers with
   their shared concepts and weights.
2. `find`, `list`, `inspect`, and `assertions` accept `--json`.
3. The JSON shapes are versioned (`"schema": 1`) and documented in the README.
4. `diver serve` answers the same queries over HTTP on `127.0.0.1` and returns documents
   identical to the `--json` output for the same input.
5. The text views are rendered from the same values, so text and JSON cannot drift.

## Rationale

The Sprint 19 proving-ground drive showed the concept layer is worth looking at. A
front end ([[concept-explorer-front-end]], INT-0026) is the next step, but it should bind to
a stable data contract rather than to terminal text. This is also the architecture
already intended for the project: a Rust engine with a separate human interface. And it
gives agents a retrieval primitive — "what does the evidence graph around this concept
look like" — rather than text to parse.

## Alternatives

- **Front end parses CLI text** — rejected: brittle, and it loses what the text view drops.
- **Front end links `diver-core` directly** (e.g. a Tauri command layer) — viable later,
  and it would still want these serializable values; defer until the front-end shell is
  chosen.
- **HTTP only, no `--json`** — rejected: `--json` is the cheapest agent and script
  interface, and the HTTP layer can reuse its values.

## Consequences

- `display.rs` splits into result-building and rendering; the CLI gains a `serde`-shaped
  result layer.
- A new binary surface (`serve`) with its own dependency (likely `axum`, matching the
  planned `diver-server` crate).
- The JSON shape becomes a compatibility promise, which is why it is versioned.

## Transition history
- 2026-09-24: created as `proposed` at the close of Sprint 19, as the prerequisite for a
  front end identified by the readiness analysis that followed the proving-ground drive.
