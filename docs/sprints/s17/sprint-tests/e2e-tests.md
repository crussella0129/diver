# Sprint 17 End-to-End Tests

- **Tested head:** `0aaa31cfeb486ccdee045905dc42f9fb7af24077`
- **Status:** possible (offline)

## Real-corpus validation (the AC3 evidence)
A probe over the persisted real 13-paper corpus (Python replica of `significant_terms` loading
the actual `stopwords.txt`) measured the effect on the terms that create co-assertion edges:

- **Terms shared by ≥2 papers: 189 → 92** (−51%).
- **Top of the list flipped from filler to technical:** now `transformer(8)`, `image(7)`,
  `neural(7)`, `translation(7)`, `attention(6)`, `diffusion(6)`, `machine(6)`, `bleu(4)`,
  `convolutional(4)`, `encoder(3)`, `decoder(3)` — the generic drivers (`model`, `data`,
  `results`, `training`, `existing`, `however`, `https`, `github`, `eight`, `literature`) are gone.

## Manual dive (offline, on the real corpus)
`diver dive "machine translation"` co-assertion edges are now all meaningful:
```
1706.03762 — co-asserts encoder (w=0.78)
1706.03762 — co-asserts decoder (w=0.78)
1706.03762 — co-asserts recurrent (w=0.78)
1706.03762 — co-asserts convolutional (w=0.63)
1706.03762 — co-asserts bleu (w=0.63)
1807.11605 — co-asserts attentive (w=0.78)
```
compared with the Sprint 16 noise (`co-asserts eight (w=1.00)`, `existing`, `literature`,
`https`), which no longer appears.
