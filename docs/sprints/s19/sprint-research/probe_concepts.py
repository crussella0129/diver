"""Read-only probe of the real Diver corpus, for Sprint 19 (INT-0020) research.

Opens the database with mode=ro so nothing can be written. Replicates the
significant_terms tokenizer from diver-core/src/graph.rs and measures:
  1. substring-match false positives for dive queries (the LIKE '%q%' behaviour)
  2. singular/plural variants a substring query misses in one direction
  3. candidate two-word phrases under a function-word / filler split
"""

import os
import re
import sqlite3
import sys
from collections import defaultdict

DB = os.path.join(os.environ["APPDATA"], "diver", "diver.db")
STOP = sys.argv[1]

words = open(STOP, encoding="utf-8").read().split()
# stopwords.txt has two uncommented alphabetical blocks: common English first,
# then research filler starting at "abstract". Split there.
split_at = words.index("abstract")
common, filler = set(words[:split_at]), set(words[split_at:])
stop = common | filler


def tokens(text):
    return [t.lower() for t in re.split(r"[^0-9A-Za-z]+", text) if t]


def ok(t):
    return len(t) >= 3 and any(c.isalpha() for c in t)


con = sqlite3.connect(f"file:{DB}?mode=ro", uri=True)
rows = con.execute(
    "SELECT p.arxiv_id, a.claim FROM assertions a JOIN papers p ON p.id = a.paper_id"
).fetchall()
papers = sorted({r[0] for r in rows})
print(f"papers with claims: {len(papers)}   claims: {len(rows)}")
print(f"stoplist: {len(common)} common + {len(filler)} filler = {len(stop)}")

# ---- 1. substring false positives -------------------------------------------
print("\n== 1. substring (LIKE) matches vs whole-token matches ==")
for q in ["attention", "translation", "art", "net", "network", "transformer",
          "transformers", "diffusion", "model", "gan", "rnn", "bleu", "image"]:
    sub = [(p, c) for p, c in rows if q in c.lower()]
    tok = [(p, c) for p, c in rows if q in tokens(c)]
    fp = [c for p, c in sub if q not in tokens(c)]
    ex = sorted({w for _, c in sub for w in tokens(c) if q in w and w != q})
    print(f"  {q!r:15} substring={len(sub):3} exact-token={len(tok):3} "
          f"substring-only={len(fp):3}  via: {ex[:6]}")

# ---- 2. plural variants -------------------------------------------------------
print("\n== 2. singular/plural pairs both present in the corpus ==")
df = defaultdict(set)
for p, c in rows:
    for t in tokens(c):
        if ok(t):
            df[t].add(p)
pairs = []
for t in df:
    if t.endswith("s") and not t.endswith(("ss", "us", "is")) and t[:-1] in df:
        pairs.append((t[:-1], t, len(df[t[:-1]]), len(df[t]), len(df[t[:-1]] | df[t])))
for sing, plur, a, b, u in sorted(pairs, key=lambda x: -x[4]):
    tag = "(stopped)" if sing in stop or plur in stop else ""
    print(f"  {sing:18} df={a:2}  {plur:20} df={b:2}  union df={u:2} {tag}")
print(f"  -> {len(pairs)} pairs")

# ---- 3. phrase candidates -------------------------------------------------------
print("\n== 3. adjacent two-word phrases, df >= 2 papers ==")


def phrases(text, rule):
    ts = tokens(text)
    out = []
    for a, b in zip(ts, ts[1:]):
        if not (ok(a) and ok(b)):
            continue
        if rule == "strict" and (a in stop or b in stop):
            continue
        if rule == "split":
            if a in common or b in common:        # function/common words break phrases
                continue
            if a in filler and b in filler:       # need at least one content word
                continue
        out.append(f"{a} {b}")
    return out


for rule in ["strict", "split"]:
    pdf = defaultdict(set)
    for p, c in rows:
        for ph in phrases(c, rule):
            pdf[ph].add(p)
    shared = sorted(((ph, len(ps)) for ph, ps in pdf.items() if len(ps) >= 2),
                    key=lambda x: (-x[1], x[0]))
    print(f"\n  rule={rule}: {len(pdf)} distinct phrases, {len(shared)} with df>=2")
    for ph, n in shared[:40]:
        print(f"    {n:2}  {ph}")

# ---- 4. recommended rule: filler may HEAD a phrase but not modify it; web tokens
#         break phrases; plural folding on every token ---------------------------
print("\n== 4. rule=head (+web tokens break, +plural folding) ==")
WEB = {"https", "http", "www", "github", "com"}


def fold(t):
    if len(t) > 4 and t.endswith("ies"):
        return t[:-3] + "y"
    if len(t) > 4 and t.endswith("sses"):
        return t[:-2]
    if len(t) > 3 and t.endswith("s") and not t.endswith(("ss", "us", "is")):
        return t[:-1]
    return t


def head_phrases(text):
    ts = tokens(text)
    out = []
    for a, b in zip(ts, ts[1:]):
        if not (ok(a) and ok(b)):
            continue
        if a in common or b in common or a in WEB or b in WEB:
            continue
        if a in filler:          # filler may be the head (b), never the modifier (a)
            continue
        out.append((f"{fold(a)} {fold(b)}", f"{a} {b}"))
    return out


pdf, forms = defaultdict(set), defaultdict(set)
for p, c in rows:
    for key, surface in head_phrases(c):
        pdf[key].add(p)
        forms[key].add(surface)
shared = sorted(((k, len(ps)) for k, ps in pdf.items() if len(ps) >= 2), key=lambda x: (-x[1], x[0]))
print(f"  {len(pdf)} distinct phrase keys, {len(shared)} with df>=2")
for k, n in shared:
    print(f"    {n:2}  {k:28} forms={sorted(forms[k])}")

# ---- 5. unigram concepts under folding ------------------------------------------
udf, uforms = defaultdict(set), defaultdict(set)
for p, c in rows:
    for t in tokens(c):
        if ok(t) and t not in stop:
            udf[fold(t)].add(p)
            uforms[fold(t)].add(t)
merged = {k: v for k, v in uforms.items() if len(v) > 1}
print(f"\n== 5. unigram concepts: {len(udf)} keys; {len(merged)} fold >1 surface form ==")
for k in sorted(merged, key=lambda k: -len(udf[k])):
    print(f"    df={len(udf[k]):2}  {k:16} <- {sorted(merged[k])}")
