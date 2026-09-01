# Sprint 13 End-to-End Tests

- **Tested head:** `74474dd70531b36456465ee96f894a124b66acc2`
- **Status:** possible (offline)

## Executed (manual, offline)

- **`diver dive --help` lists the flag.** Output contains
  `--temperature <TEMPERATURE>` with `[default: 0.5]` and the permissiveness
  description. **pass** (AC4)
- **Out-of-range rejected with non-zero exit.** `diver dive attention
  --temperature 1.5` →
  `error: invalid value '1.5' for '--temperature <TEMPERATURE>': temperature must
  be in [0.0, 1.0], got \`1.5\`` and **exit code 2**. **pass** (AC4 negative path)
- **Valid value runs.** `diver dive attention --temperature 0.5` → exit code 0;
  `--temperature 0.0` runs and prints the `Dive: attention` view (no-data path
  unchanged when no papers assert). **pass**

## Coverage note

The populated co-assertion path *with weighting and a real low-vs-high temperature
edge set* is covered deterministically at the library level by
`test_coassertion_temperature_pipeline` (integration), which runs the exact
`all_claims → compute_coassertion_relations(temperature) → build_dive` flow the
`dive` handler runs. A full binary run over a seeded on-disk DB remains out of the
automated suite — unlocked by a future fixture-DB E2E harness (consistent with the
INT-0012 / INT-0013 deferral); the handler change is a one-line thread-through of
the validated `temperature`.
