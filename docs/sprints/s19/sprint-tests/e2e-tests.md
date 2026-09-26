# Sprint 19 End-to-End Tests

- **Tested head:** `d08eaa61cbccb32a468e72b06b3588874a638637` (production code identical to `f5c215f`; later commits changed
  only tests and documentation). The verification section below was re-run against the
  release binary built from that production code (logged in the proving
  ground as `log/drive-head.txt`, `log/battery-head.txt`, `log/equiv-head.txt`). The
  *walls* section describes what earlier binaries did, which is why they were repaired;
  each row names the commit that changed it.
- **Status:** possible — and the primary evidence for this sprint.
- **Method (user direction, 2026-09-24):** operate first, test after. The real release
  binary was driven against real arXiv corpora in an isolated proving ground until it
  stopped finding defects that could be repaired within this intent (residual limits are
  listed under Findings); the hermetic CLI tests below were written only afterwards.

## Proving-ground drive (primary evidence)

### Environment
- `C:\Users\charl\diver-proving\` — outside the repository. Scratch corpora
  (`corpora/*.db`, via `DIVER_DB`), a probe crate linking `diver-core` directly
  (`probe/`), drive scripts (`build-corpus.sh`, `grow-corpus.sh`, `battery.sh`) and
  logs (`log/`). The user's real corpus was only ever **copied** (`real13.db`), never
  opened in place.
- Corpora: `real13` (the user's 13 papers), `multi` (480 papers: transformer LMs,
  diffusion, RL, GNNs, quantum error correction, gravitational waves, zeta/primes,
  protein structure, causal inference, cuprates), `big` (2,068 papers: `multi` plus
  deep learning, quantum computing, cosmology, genome sequencing, number theory,
  combinatorics). Extraction: deterministic sentence splitter — no LLM key is present
  in this environment (see Findings).

### Per-task boundary checks (T-1901 – T-1906)
| Task | Driven how | Observed |
|------|-----------|----------|
| T-1901 | one-off union check against the pre-split file | union 525 = original 525, disjoint |
| T-1902 | probe `formation` on `real13` | 522 concepts (497 terms, 25 phrases — matching research); 0 subset-invariant violations; twice-equal and reversed-equal-after-remap true |
| T-1903 | probe `resolve`/`fresh` on a `real13` copy | upgrade path builds tables and concepts on first read (7.6 ms); `networks`/`network` → same 6 papers; `gan`/`net`/`art` unresolved; a saved claim resolves on the next read, and stops resolving after removal |
| T-1904 | probe `edges` on `real13` | edges t0/0.5/1 = 60/189/305, monotonic; 0 subsumption violations; the `hidden state`/`hidden states` counterexample monotonic |
| T-1905 | release binary on a `real13` copy | header/forms/narrower correct; `dive model` suggests phrases; empty corpus → extract hint; `--help` states the new semantics; live `ingest` + `extract` of 2006.11239 between two dives appears in the second (2 → 3 papers) |
| T-1906 | reading | see [documentation-review.md](documentation-review.md) |

### Walls hit, and the repairs made (observed on pre-repair binaries)
| # | Wall (corpus) | Evidence | Repair (commit) |
|---|---------------|----------|-----------------|
| 1 | **Related lists unusable** (480) | every node listed 700+ related papers; the first ten always `shared category cs.LG`; bare ids; 540–1,300 lines per dive | group per paper and rank concepts → authors → category; titles; top 5; category-only links counted, never listed (`f1f3341`) |
| 2 | **Weak multi-word suggestions** (480) | `quantum error correction` offered `error (77)`, `quantum (59)` before `error correction` | any-token fallback ranks by words covered (`f1f3341`) |
| 3 | **Misspellings dead-end** (480) | `atention` → nothing | edit-distance "did you mean" (`f1f3341`) |
| 4 | **Generic words as concepts → false edges** (480) | `wide range` (w = 0.61) linked 17 unrelated papers at the default temperature; `typically`, `analyze`, `incorporate` … among 1,122 mid-frequency terms | adverb rule (`…ly`, noun allowlist) + 236 generic filler words from the evidence; formation version 2 (`8cd1cc1`) |
| 5 | **Scale: past INT-0024's 2 s trigger** (2,068) | dive 3.5–6 s; 12.3 s at t = 1.0. Profile: `build_dive` 9.5 s (277 nodes × 3.2M edges), co-assertion 2 s (all 2.1M pairs) | index edges by endpoint; compute only seed-touching pairs; pre-filter concepts by the gate (`ecf760f`) → **0.6–1.5 s** |
| 6 | Cosmetic: query echo and no-content queries | an 800-char query echoed verbatim; `the` said "no concept contains it" | echo truncated at 80 chars; "no searchable words" message (`f1f3341`) |

### Verification of the repairs (re-run on the head binary, `f5c215f`)
- **Repair 5 is output-identical.** The committed `test_coassertion_matches_naive_reference`
  compares the pre-filtering implementation with an unoptimized reference as ordered
  `Vec`s at five temperatures, and `test_touching_variants_match_full` proves scoped ==
  full ∩ seeds (both mutation-checked). On real data (`log/equiv-head.txt`, `multi`):
  full == reference on 1,319 / 69,131 / 173,102 edges at t = 0 / 0.5 / 1, and scoped ==
  full ∩ seeds for co-assertion and structural edges (15,871 of 28,140).
- **Repair 5 timing** (`big`, 2,068 papers): `dive qubit` 628 ms, `attention` 582 ms,
  `neural networks` 852 ms, `deep learning` 880 ms, `dark energy --temperature 1.0`
  1,461 ms — all under INT-0024's 2 s trigger (before: 3.5–6 s; 12.3 s at t = 1.0).
- **Repair 4 kept the domain vocabulary**: `entropy`, `graph`, `network`, `gradient`,
  `noise`, `qubit`, `ligo`, `family`, `anomaly` remain concepts; `typically`,
  `analyze` do not. The stoplist edit rebuilt persisted concepts automatically on the next
  read (fingerprint digest `bc71…` → `72cd…`) — a live test of plan critique C-005.
- **Repair 1, before/after** (`dive attention`, 480 papers): 539 lines of category noise
  → 364 lines whose first related papers share `wikitext (0.83)`, `positional encoding
  (0.83)`, `lstm (0.80)`, `perplexity (0.75)`, `self attention (0.71)`.
- **Concurrency** (`race.sh`, `log/race-head.txt`): the stored fingerprint of a corpus copy
  was forced to `stale`, then six `diver dive` processes were started at once, so all
  contended for the rebuild. All exit 0; one distinct output; afterwards the fingerprint
  is the current `v2:525eed6c17a434dc:3352:3352`, with 6,285 concepts, 34,876 links and
  0 orphan links.
- **Adversarial battery** (71 queries on `multi`, head binary): every exit 0; slowest
  316 ms; SQL-injection text inert; Unicode (`α`, `Schrödinger`, `état`) handled;
  `self-attention` ≡ `self attention`; `LLM` ≡ `LLMs`; `large language models` →
  `language models`; `cuprates` → `cuprate`; `policy gradients` → `policy gradient`;
  `atention` → did you mean `attention`; `quantum error correction` → `error correction`,
  `quantum error` first. (Earlier runs on pre-repair binaries are `battery-1.txt` and
  `battery-2.txt`; they are not the evidence for this head.)

### Findings recorded, not repaired
- **Deterministic claims are sentences, not claims.** A 10-claim sample includes "The
  main aim of this paper is to investigate…" and LaTeX fragments. Concept formation is
  robust to this; the *epistemic* layer is not doing epistemic work without LLM
  extraction (no key in this environment). Carried to the readiness analysis.
- **Generic vocabulary still leaks, reduced but not removed.** In the head binary's
  `dive attention` on 480 papers, top related-paper lines still name generic words
  (`hybrid (0.75)`, `configurations (0.69)`, `open source (0.69)`). Repair 4 cut this
  class — it did not eliminate it, and a stoplist cannot. Recorded in INT-0020.
- **Word concepts are polysemous.** `code` (150 papers) spans genetic code,
  error-correcting codes and source code; phrases disambiguate (`surface code`: 5 of 5
  QEC papers). Recorded in INT-0020.
- **Known fold limits confirmed**: `bias` (23 papers) and `biases` (6) are separate;
  `news` folds to common `new`. Recorded in INT-0020.
- **arXiv politeness across processes**: the client's 3 s throttle is per process, so
  back-to-back `collect` invocations are not spaced. The drive scripts sleep 4 s; worth a
  persistent throttle later.

## Hermetic CLI tests (`diver-cli/tests/dive_concepts.rs`)
- `test_cli_dive_concepts` — **pass**. Seeds A/B/C through `diver_core` into a scratch
  `DIVER_DB`, runs the real binary: `dive networks` reports `Dive: network (term, 2 papers)`
  with both surface forms and prints both B's and C's claims as results (C only says
  `network`, which substring matching missed — asserted on the claim bullets, not bare
  ids, which also appear in related-paper lines); `dive model` says it is not a
  concept and offers `diffusion model`; a paper D added between two `dive diffusion` runs
  appears in the second, with no rebuild step.
- `test_cli_dive_empty_and_help` — **pass**. An empty corpus gives the `diver extract`
  hint; `dive --help` describes concept resolution and the refined `t = 1.0`.

## Still not-yet-possible
- Evaluation-quality scoring of concepts and relations against a gold set — unlocked by
  [INT-0022](../../../intents/INT-0022-relation-evaluation-harness.md), unchanged.
