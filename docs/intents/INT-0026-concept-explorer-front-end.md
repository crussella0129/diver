# INT-0026 — Concept explorer front end

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0026
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

A read-only front end for exploring a Diver corpus by concept: search for a term, land on
its concept (forms, reach, narrower and broader concepts), read each paper's claims *with
their support quotes*, and move to related papers through the concepts they share. Every
screen should answer "why is this here?" by showing the shared concept or quote that put it
there — provenance visible by default, per the [epistemic charter](../README.md#epistemic-principles).

It binds only to the data contract of [[machine-readable-interface]] (INT-0025).

### Visual system (owner's direction)

The interface uses the project owner's house visual system: dark-only, high contrast,
system sans type, tight-tracked headings at regular weight, pill-shaped actions, generous
space, no decorative motion. The tokens, verbatim:

```css
:root {
  color-scheme: dark;
  --bg: #000; --surface: #111; --text: #fff; --muted: #b8b8b8; --rule: #333;
  --hover: #dedede; --pressed: #b8b8b8;
  --font: Arial, Helvetica, sans-serif;
  --text-sm: .875rem; --text-base: 1rem;
  --text-lead: clamp(1.125rem, 1.8vw, 1.375rem);
  --text-h2: clamp(1.5rem, 2.5vw, 2rem);
  --text-title: clamp(2.5rem, 5.8vw, 4.5rem);
  --leading: 1.6; --leading-title: 1.04;
  --tracking-title: -.045em; --tracking-wordmark: -.035em;
  --s1: .25rem; --s2: .5rem; --s3: .75rem; --s4: 1rem; --s5: 1.25rem; --s6: 1.5rem;
  --s8: 2rem; --s10: 2.5rem; --s12: 3rem; --s16: 4rem; --s20: 5rem;
  --width: 72rem; --gutter: clamp(1.25rem, 4vw, 3rem); --section-gap: clamp(3rem, 6vw, 5rem);
  --measure: 54ch; --radius: 1.5rem; --radius-media: 2rem;
  --stroke: 1px; --focus: 2px; --target: 2.75rem; --action-height: 3.25rem;
}
```

Rules that define the feel: headings weight 400 with `--tracking-title` and
`text-wrap: balance`; bold only for the wordmark and primary actions; body copy in
`--muted` at `--text-lead`, capped at `--measure`; the primary action a white pill
(`--text` background, `--bg` text, `--radius`, `--action-height`), hover `--hover`, active
`--pressed`; navigation as muted pills with the current item on `--surface`; cards and
panels on `--surface` with `1px solid var(--rule)` hairlines and `--radius-media` corners
(`corner-shape: squircle` where supported); a visible `--focus` outline offset by `--s1`;
selection inverted; no transitions, no webfonts.

Non-goals:
- Not a chatbot. No generated summary as the primary view; if one is ever added, it is a
  view over the concept graph, never a replacement (charter principle 10).
- No graph visualization of *typed* epistemic relations yet — they do not exist
  (INT-0021). A concept neighbourhood view is in scope; a claim-relation graph is not.
- No corpus mutation from the UI in the first version (see INT-0025).
- The front end does not name, link, or otherwise reference the source of its visual
  system.

## Acceptance criteria

1. Searching a term lands on its concept view or, when unresolved, on its suggestions
   and did-you-mean — the same results as `diver dive`.
2. The concept view shows label, forms, paper count, narrower/broader concepts, and each
   asserting paper's claims with their support quotes.
3. Related papers are grouped and ranked as in the CLI, each showing the shared concepts
   and weights that link it; category-only links are summarized, not listed.
4. Every rendered colour, type size, space and radius comes from the tokens above; there
   is no other palette.
5. Keyboard reachable, with visible focus; usable at 360 px width.
6. Works against the proving-ground corpora (480 and 2,068 papers) with each view
   rendered within a second of the API responding.

## Rationale

The Sprint 19 drive made the concept layer good enough to be worth seeing: at 480 papers,
the related papers for a language-modelling paper were Transformer-XL, Segatron and Primer,
linked by `wikitext`, `positional encoding`, `perplexity`. A terminal buries that under
hundreds of lines. A front end is the natural way to navigate it, and it will surface the
next quality problems faster than the CLI.

## Alternatives

- **Desktop shell first** (Electron, then Tauri, per the owner's general preference) —
  open: a local web UI served by INT-0025's `serve` is the smallest first step, and it can
  be wrapped in a desktop shell later without changing the UI. To be decided when this
  intent is scheduled.
- **A graph-first UI** (force-directed concept map) — deferred: at corpus scale the
  category and generic-concept edges make hairballs; a ranked, provenance-first list view
  is more honest about what the data supports today.

## Consequences

- Depends on INT-0025.
- The claim layer becomes visible to users. With deterministic extraction the "claims" are
  abstract sentences, which will look unfinished; LLM extraction (or a local model) becomes
  a user-visible quality lever.
- A second implementation language and toolchain enter the repository.

## Transition history
- 2026-09-24: created as `proposed` at the close of Sprint 19, recording the owner's
  visual-system direction and the readiness analysis that followed the proving-ground drive.
