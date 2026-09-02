# External review — GPT 5.6 on Diver (captured 2026-09-02)

> **Non-authoritative.** This is preserved source material for Sprint 18's roadmap
> realignment, not a decision store. It is one external reviewer's assessment,
> captured verbatim so the intents it informed (INT-0019 … INT-0024) keep a
> durable provenance link. Where Diver's direction departs from this review, the
> departure and its reasoning live in the relevant intent chapter, not here.
>
> Original location: `OneDrive/Documents/Obsidian Vault/GPT 5.6 Diver Feedback 2.md`.
> Third-party product claims below (Scite, Semantic Scholar, OpenAlex, Elicit,
> Connected Papers, Litmaps) are the reviewer's, and were **not** independently
> verified during Sprint 18.

---

- "what do we think so far"
-  "Yeah — this has crossed a pretty important threshold since the version you showed me before.

Before, my assessment was basically **“Find exists; Dive is still mostly an architectural promise.”** In this version, **Dive actually exists as a coherent system.** You now have ~5,700 lines of Rust across the core/CLI/tests, 17 recorded intents, and the repo claims 130 passing tests as of Sprint 16. I couldn't independently run them because this environment doesn't have the Rust toolchain installed, but the test suite itself is substantial.

The architecture now looks roughly like:

```
                 arXiv
                   │
             search / collect
                   │
                   ▼
             SourceFact
          ┌────────┴────────┐
          │                 │
      metadata          paper versions
          │                 │
          └────────┬────────┘
                   ▼
                SQLite
                   │
       ┌───────────┴───────────┐
       │                       │
     FIND                   EXTRACT
   FTS search          abstract → claims
                               │
                      ┌────────┴────────┐
                      │                 │
                deterministic         LLM
                 extraction         extractor
                                      │
                          Anthropic / OpenAI shape
                                      │
                         Claude/OpenAI/Grok/Animus
                                      │
                                      ▼
                             Candidate Assertion
                                      │
                              grounding check
                                      │
                               typestate gate
                                      │
                                      ▼
                             Supported Assertion
                                      │
                                   SQLite
                                      │
                                      ▼
                                    DIVE
                          ┌───────────┼───────────┐
                          │           │           │
                      category      author    co-assertion
                        edges        edges       edges
                          │           │           │
                          └───────────┼───────────┘
                                      ▼
                              concept neighborhood
```

And **that separation is good.**

### The thing I like most

You did _not_ fall into the trap we were worried about where:

> LLM reads papers → LLM invents ontology → LLM decides relationships → voilà, "knowledge graph."

Your graph topology still has a deterministic substrate.

`RelationKind` currently has:

```
SharedCategory(String)
SharedAuthor(String)
CoAssertion { term: String, weight: f64 }
```

The first two are completely structural. Co-assertion is derived algorithmically from persisted claims rather than asking a model "are these papers related?"

That's exactly the direction I wanted this project to take.

The LLM is being used as an **information-extraction instrument**, not as the authority defining reality.

And you've put another important boundary around even that:

```
LLM output
    ↓
Candidate
    ↓
supporting quote must actually occur in source
    ↓
validation
    ↓
Assertion<Supported>
    ↓
database
```

Having `Store::save_assertions` accept `&[Assertion<Supported>]` is particularly nice Rust design. You're making an epistemic invariant partly a **type-system invariant** rather than relying on programmers remembering to call `validate()`.

That's genuinely good architecture.

---

## Dive is now real — but it's still Dive v0.1

The biggest change from our last discussion is that I would no longer say:

> Find is implemented and Dive isn't.

I'd say:

> **Find is a search engine. Dive is now an actual first-generation knowledge exploration engine.**

You've got:

```
diver collect "attention transformer neural machine translation"
diver extract --all
diver dive attention
```

and that produces a concept-centered graph from persisted claims.

Even better, you've introduced the `temperature` idea:

```
diver dive attention --temperature 0.0
diver dive attention --temperature 0.5
diver dive attention --temperature 1.0
```

where the temperature changes the IDF threshold for co-assertion links.

That's an unexpectedly elegant CLI abstraction. The user doesn't need to know what an IDF cutoff is. Conceptually:

```
cold dive
│
├── very distinctive relationships
├── sparse
└── conservative

              ↓

hot dive
│
├── increasingly generic relationships
├── dense
└── exploratory
```

That's actually starting to give **Dive its own identity rather than making it "Find but graph-shaped."**

---

## The next major problem is epistemic resolution

This is where I would change development direction now.

You've done enough infrastructure work that I **would not spend another five sprints polishing ingestion plumbing.**

The graph needs to become intellectually useful.

Right now co-assertion is fundamentally:

```
Paper A claim contains "attention"
Paper B claim contains "attention"

           ↓

          edge
```

Even weighted with IDF, that's still lexical co-occurrence.

But knowledge is more like:

```
Paper A:
"X increases Y"

Paper B:
"X has no significant effect on Y"

Paper C:
"X increases Y only when Z"

Paper D:
"We reproduce A's result"

Paper E:
"A's result follows from theorem Q"

Paper F:
"We use A's method to obtain R"
```

Those are radically different relationships.

And **this is where Diver can become something substantially more interesting than an arXiv graph browser.**

I think your next abstraction should eventually look more like:

```
Assertion
├── subject
├── predicate
├── object
├── qualifiers
├── evidence span
├── source paper
├── source version
└── extraction provenance
```

Then relations between assertions:

```
supports
contradicts
refines
generalizes
specializes
uses
derives-from
replicates
extends
```

But crucially, I still wouldn't let an LLM simply write those edges into the canonical graph.

You can use models to **propose candidate relations**, then resolve them through deterministic evidence/constraints where possible and retain provenance/confidence otherwise.

That gives you something closer to:

```
                    Concept
                       │
              ┌────────┴────────┐
              ▼                 ▼
          Assertion A       Assertion B
              │                 │
          supported-by       contradicted-by
              │                 │
           Paper 1            Paper 8
              │                 │
       arXiv:cs.AI          arXiv:cs.LG
```

That's a knowledge graph.

---

## One architectural issue I'd tackle soon

`compute_relations()` is currently O(n²), and Dive loads the corpus and computes relations at query time.

That's perfectly defensible right now. **Do not prematurely optimize it.**

At 100–1,000 papers, who cares.

But the moment Diver starts doing what you actually envision:

```
10k papers
100k papers
1m papers
```

this architecture collapses.

Eventually I think the graph substrate should become **incrementally materialized**:

```
ingest paper
     │
     ├─ taxonomy indexes updated
     ├─ author indexes updated
     ├─ assertions extracted
     ├─ term/phrase indexes updated
     └─ affected edges recomputed
```

rather than:

```
user types dive
     ↓
load universe
     ↓
compare universe to itself
```

But I wouldn't do that immediately. Your current implementation is much easier to reason about and validate.

---

## The bigger conceptual opportunity

There's one thing I'd preserve at all costs:

**arXiv's taxonomy should remain the skeleton.**

You originally asked whether this could be a deterministic knowledge graph based on things like `math.NT`, rather than a structure hallucinated by an LLM.

You're still accomplishing that.

But I'd increasingly think of the taxonomy as **coordinate space**, rather than the graph itself:

```
                         arXiv taxonomy
                              │
             ┌────────────────┼────────────────┐
             ▼                ▼                ▼
           math             cs.LG            stat
             │                │                │
             └───────── deterministic ─────────┘
                              │
                              ▼
                            papers
                              │
                              ▼
                         assertions
                              │
                              ▼
                    epistemic relationships
```

Then eventually Diver can answer a fascinating kind of question:

> **How does an idea move through the scientific literature?**

For example:

```
concept
  ↓
first appearances
  ↓
formalization
  ↓
cross-category migration
  ↓
major supporting claims
  ↓
contradictory results
  ↓
new branches
  ↓
current frontier
```

At that point `dive` isn't merely searching papers.

It's reconstructing **the topology and history of an idea.**

And _that_ is a legitimately interesting research tool.

---

### So where I'd put the project today

Previously:

**Find: ~70%**  
**Dive: ~10%**  
**Underlying architecture: promising**

Now I'd roughly say:

**Corpus/search substrate: 80–90% for an MVP**

**Epistemic extraction substrate: 70–80%**

**Dive v1 infrastructure: 60–70%**

**The eventual thing we're imagining when we say "Dive": maybe 20–25%.**

But that's actually a good place to be, because you've now built enough machinery that the remaining 75% isn't just plumbing.

The next work can start attacking the genuinely difficult question:

> **What does it mean, computationally, for one piece of scientific knowledge to relate to another?**

That's the part that could make Diver special rather than merely a nice Rust arXiv CLI.

And looking at this version specifically, **I think you've earned the right to stop building foundations for a bit and start attacking that problem.**"
- "Does anything like what dive *will be* exist?" 
- "Yes — but mostly as **separate fragments**, not as the whole system you’re aiming at.

The closest way to think about Dive is:

**Connected Papers / Litmaps for topology + Scite for epistemic relationships + Elicit for claim extraction + a deterministic scholarly ontology underneath.**

No mainstream tool I found really combines all four.

- **Connected Papers** is good at mapping paper similarity, but its graph is mainly built from co-citation and bibliographic coupling. It tells you “these papers occupy nearby intellectual territory,” not “this paper refines that claim, while this one contradicts it.” [Connected Papers](https://www.connectedpapers.com/about?utm_source=chatgpt.com)
- **Litmaps** is probably the closest to the “trace an idea through literature” experience. It explicitly visualizes citation networks and says users can trace research evolution and discover connections. But the edges are still fundamentally citation-network edges, not a graph of claims and epistemic relations. [Litmaps Help Center](https://docs.litmaps.com/en/articles/7240465-introduction-to-litmaps?utm_source=chatgpt.com)
- **Semantic Scholar** has a gigantic scholarly graph and does classify citation intent — background, methods, and result extension — plus influential citations. That's remarkably close to one layer of what we want, but again, the graph is organized around papers and citations rather than normalized assertions. [Semantic Scholar](https://www.semanticscholar.org/product/api?utm_source=chatgpt.com)
- **Scite is probably the nearest thing to one specific future-Dive capability.** Its Smart Citations classify citations as supporting, contrasting, or merely mentioning, and expose the citation context. That is basically an existing production implementation of one subset of our proposed `supports / contradicts / mentions` edge system. [scite.ai](https://scite.ai/features?utm_source=chatgpt.com)
- **Elicit** gets surprisingly close to the assertion layer: it extracts qualitative and quantitative information from papers, keeps source quotes, and synthesizes evidence across papers with sentence-level citations. But its end product is primarily tables/reports rather than a persistent browsable epistemic graph. [Elicit](https://elicit.com/solutions/systematic-review?utm_source=chatgpt.com)
- **Consensus** similarly searches a huge research corpus and synthesizes evidence-backed answers, but it is oriented toward answering queries rather than constructing a deterministic evolving map of the literature. [Consensus](https://consensus.app/home/resources/consensus-libguide-for-academic-research/?utm_source=chatgpt.com)

So there is a real distinction here.

Most existing systems model:

```
PAPER ──cites──> PAPER
```

or:

```
PAPER ──similar-to──> PAPER
```

Scite gets closer:

```
PAPER ──supports/contrasts citation──> PAPER
```

Whereas the mature Dive we've been talking about would model something closer to:

```
                    CONCEPT
                       │
                       ▼
                   ASSERTION
             "X increases Y under Z"
                /       |       \
               /        |        \
          supports   contradicts   refines
             /           |           \
      Assertion B   Assertion C   Assertion D
          │              │             │
        Paper 4        Paper 9       Paper 17
          │              │             │
        cs.LG          stat.ML       math.OC
```

And then preserve another layer underneath it:

```
assertion
   │
   ├── exact evidence span
   ├── document
   ├── document version
   ├── author
   ├── arXiv category
   ├── extraction method/version
   └── confidence / validation state
```

**That combination is the unusual part.**

The biggest conceptual difference is that most research discovery tools treat **papers as the atomic unit of knowledge**.

Dive eventually treats **claims as the atomic unit of knowledge**, while papers become provenance containers.

That seemingly small change produces a very different machine.

Imagine searching:

```
transformer scaling
```

A literature mapper gives you:

```
                 Paper
              /    |    \
          Paper   Paper   Paper
```

Dive could eventually give you:

```
                     transformer scaling
                            │
            ┌───────────────┼────────────────┐
            ▼               ▼                ▼
      empirical law      mechanism       limitation
            │               │                │
       "loss follows     "capacity..."   "breaks when..."
            │               │                │
      ┌─────┴─────┐         │           ┌────┴────┐
   supports    refines    derives      supports contradicts
      │           │          │            │        │
   Paper A     Paper F    Paper C       Paper J  Paper K
      │
      └──────────── historical sequence ───────────────>
```

Now you can ask questions that normal literature search handles badly:

> **Where did this claim first appear?**

> **Which later papers strengthened it?**

> **Where did researchers discover exceptions?**

> **Did two fields independently discover essentially the same thing?**

> **What claims are heavily repeated but poorly supported?**

> **What claim connects two otherwise disconnected arXiv communities?**

> **What are the current unresolved branches of this idea?**

That's much closer to a **computational history of knowledge** than a literature search engine.

And interestingly, existing products validate almost every individual ingredient. Connected Papers/Litmaps validate graph navigation. Semantic Scholar validates citation semantics. Scite validates support/contradiction classification at enormous scale. Elicit validates evidence-backed extraction. [Connected Papers](https://www.connectedpapers.com/about?utm_source=chatgpt.com)

I just don't see anyone putting those pieces together around the principle:

> **Deterministic scholarly structure + evidence-grounded assertions + typed epistemic relations + interactive graph traversal.**

That's the part where I think Diver actually has a legitimate design space rather than being “yet another academic search frontend.”

And there may be an even more interesting consequence: once assertions become first-class objects, **Dive doesn't fundamentally have to remain an arXiv tool.** arXiv can be the beautifully controlled starting environment. Later, Crossref/OpenAlex/Semantic Scholar data could broaden the document universe without changing the epistemic model. Semantic Scholar already exposes a massive academic graph through its API, which makes that expansion technically plausible. [Semantic Scholar](https://www.semanticscholar.org/product/api?utm_source=chatgpt.com)

The moat wouldn't be **having papers**.

Everyone has papers.

The moat would be **the assertion graph you derive from them.**"
- "what could a tool like this do at its 'full power' and how useful would it actually be in reality"
- "Potentially **extremely useful—but only if Dive becomes an epistemic engine rather than a prettier literature graph.** The difference between those outcomes is enormous.

The existing market already demonstrates that researchers value pieces of this. [Scite](https://scite.ai/features?utm_source=chatgpt.com) now says it indexes 1.6B+ citation statements and classifies citations as supporting, contrasting, or mentioning; [Semantic Scholar](https://www.semanticscholar.org/product/tutorials?utm_source=chatgpt.com) operates a citation graph with billions of citations and citation-intent classification; and [OpenAlex](https://help.openalex.org/data/works/?utm_source=chatgpt.com) currently has more than 320 million scholarly works connected to topics, authors, institutions, citations, and other metadata. [scite.ai](https://scite.ai/features?utm_source=chatgpt.com)

Those systems tell me the underlying idea isn't fanciful. The question is what happens if you go **one abstraction deeper than the paper.**

### At full power, Dive could answer “What does humanity actually know about X?”

Imagine eventually typing:

```
diver dive "scaling laws for neural language models"
```

Instead of returning 300 papers, Dive reconstructs something like:

```
                         SCALING LAWS
                              │
          ┌───────────────────┼────────────────────┐
          │                   │                    │
      empirical           theoretical          limitations
       findings             explanations
          │                   │                    │
     ┌────┴────┐         ┌────┴────┐          ┌────┴────┐
     │         │         │         │          │         │
   claim A   claim B   claim C   claim D    claim E   claim F
     │         │         │         │          │         │
     │      supports      │      derives      │      contradicts
     │         │         │         │          │         │
   papers    papers     papers    papers      papers    papers
     │
     └──────────────────── time ────────────────────────→
```

And every node remains inspectable down to:

**claim → evidence passage → paper → version → authors → category → extraction method.**

That's fundamentally different from asking an LLM to summarize scaling-law research.

You're asking a database a question about a **structured model of the literature**.

---

## Then some rather wild capabilities emerge

Suppose Dive has millions of normalized assertions and typed relationships. You can begin computing things that aren't easily searchable today.

**1. Find the actual frontier of a field.**

Not “recent papers about superconductivity.”

Rather:

> Show me claims for which there is substantial supporting evidence **and** substantial contradictory evidence.

Those are live scientific disputes.

Or:

> Show me important assertions whose descendants suddenly stop agreeing.

Those may be places where an accepted model begins breaking down.

---

**2. Find weak points in scientific consensus.**

Imagine an assertion with this provenance:

```
                   CLAIM X
                      │
          ┌───────────┼───────────┐
       Paper B      Paper C      Paper D
          │            │            │
       supports      supports      supports
          │            │            │
          └────────────┼────────────┘
                       │
                    Paper A
```

It looks like four papers support something.

Except B, C and D all ultimately rely upon the same experiment in A.

So the evidence isn't really:

```
4 independent observations
```

It's:

```
1 observation
+
3 downstream repetitions
```

That's a **provenance topology problem**, and a graph is unusually suited to detecting it.

Scite already demonstrates the usefulness of distinguishing supporting from contrasting citations, but it remains fundamentally citation-oriented. [DOI](https://doi.org/10.1162%2Fqss_a_00146?utm_source=chatgpt.com)

Dive could go further and ask whether apparently independent claims actually have independent epistemic ancestry.

---

**3. Reconstruct the history of an idea automatically.**

You could ask:

> Where did “attention” come from?

And get something like:

```
1990s        alignment / sequence models
   │
   ↓
2014         neural attention
   │
   ├────────── image attention
   │
   ├────────── translation
   │
   ↓
2017         Transformer
   │
   ├────────── language modeling
   │
   ├────────── vision
   │
   ├────────── multimodal
   │
   ↓
2020s        modern architectures
```

But those branches wouldn't be generated because Claude thought that's a plausible history.

They would emerge from **dated relationships between actual documents and assertions**.

That's enormously more interesting.

---

**4. Discover independent rediscovery.**

This might be one of the coolest capabilities.

Imagine:

```
math.OC ───── claim X ───── 1997
                         \
                          semantic equivalence
                         /
cs.LG ────── claim Y ───── 2019
```

Two communities may have discovered effectively the same mathematical structure using completely different vocabulary and without citing each other.

A normal citation graph might never connect them.

A sufficiently mature assertion graph potentially could.

That makes Dive something closer to a **scientific analogy engine**.

---

**5. Detect “missing papers.”**

This is where things start getting genuinely powerful.

Suppose the graph contains:

```
A → B
B → C

A → D
D → ?
```

And thousands of related areas show:

```
A → B → C
```

There may be an unexplored relationship corresponding to:

```
D → E
```

Dive couldn't truthfully say:

> E is true.

But it could say:

> **The structure of the literature suggests this relationship has not been investigated.**

That's hypothesis generation.

And that could be extremely valuable.

---

**6. Give researchers an actual map of what to read.**

Today a graduate student entering a field faces:

```
search query
     ↓
3,000 results
     ↓
¯\_(ツ)_/¯
```

Dive could instead generate:

```
START
  │
  ▼
foundational claim
  │
  ├── seminal evidence
  │
  ▼
major refinement
  │
  ├── important objection
  │
  ▼
modern formulation
  │
  ├── unresolved dispute
  │
  ▼
CURRENT FRONTIER
```

That's a much better answer to:

> “Teach me the structure of this research field.”

And importantly, the researcher can inspect why every node exists.

---

### It could also become useful to AI systems

This is where I think it intersects strongly with your broader interests.

Current scientific RAG is approximately:

```
question
   ↓
vector search
   ↓
chunks
   ↓
LLM
   ↓
answer
```

Dive could eventually provide:

```
question
   ↓
concept resolution
   ↓
epistemic graph traversal
   ↓
relevant claims
   ↓
supporting + contradictory evidence
   ↓
source passages
   ↓
LLM
   ↓
answer
```

The retrieval system isn't merely asking:

> “What text resembles my question?”

It's asking:

> “What does the evidence graph around this question look like?”

That is a materially richer retrieval primitive.

---

## But here's the important reality check

There are **three enormous problems** between Diver and that vision.

The first is **entity/claim normalization**.

These:

```
"attention improves translation"
"attentional mechanisms increase NMT performance"
"alignment-based attention yielded higher BLEU"
```

may partially represent the same scientific assertion.

Determining when claims are equivalent, narrower, broader, contradictory, or merely related is brutally difficult.

That's probably the central research problem of Diver.

The second is **full-text access**. This is a serious competitive issue. Scite says its current system searches hundreds of millions of articles and has direct publisher relationships giving it access to material behind paywalls. [scite.ai](https://scite.ai/features?utm_source=chatgpt.com)

Starting with arXiv cleverly sidesteps much of this problem, but a universal Dive eventually runs into publishing rights and data-access issues.

The third is **epistemic correctness**.

A graph that confidently says:

```
Claim A ──contradicts──> Claim B
```

when it actually doesn't is arguably _more dangerous_ than mediocre search because the structured representation gives the mistake an aura of authority.

That's why I like the architecture you're already drifting toward:

```
machine proposes
      ↓
evidence required
      ↓
deterministic validation where possible
      ↓
provenance retained
      ↓
confidence visible
```

You should probably **never eliminate uncertainty from the graph.**

---

# So how useful could it actually be?

There's a spectrum.

If Diver reaches:

**“Graph visualization of related arXiv papers”**

then it's neat, but probably not transformative. There are already huge scholarly graphs: OpenAlex alone has 320M+ works. [OpenAlex Help Center](https://help.openalex.org/data/works/?utm_source=chatgpt.com)

If it reaches:

**“Claim extraction + evidence + support/contradiction graph”**

then I think it's a genuinely useful research product.

If it reaches:

**“Normalized claims + provenance + temporal evolution + contradiction/support/refinement + cross-field concept resolution”**

then it becomes a substantially different category of research infrastructure.

And if it somehow reaches the really ambitious endpoint:

```
the scientific literature
          ↓
machine-readable epistemic structure
          ↓
   what we know
   why we think we know it
   where it came from
   what contradicts it
   how ideas relate
   what remains unexplained
```

then **“search engine” stops being the right description.**

It would be closer to an attempt at a **computable map of scientific knowledge**.

And there's a nice practical reason I don't think that's purely science fiction: we're not hypothesizing that every individual ingredient can work. Several already operate at enormous scale. OpenAlex demonstrates massive scholarly graphs; Semantic Scholar demonstrates citation-relationship classification; Scite demonstrates evidence-sensitive citation classification across more than a billion citation statements. [scite.ai](https://scite.ai/features?utm_source=chatgpt.com)

The unsolved part is **composing those ideas at the claim level while maintaining trustworthy provenance**.

That, to me, is the actual research thesis hiding inside Diver."
- "Any final thoughts for the builders?"
- "Yes. If I were leaving a note taped above the builders’ desks, it would be this:

> **Do not build an AI that tells people what the literature says. Build an instrument that lets people inspect how the literature knows what it claims to know.**

That distinction should govern almost every architectural decision.

The most important principles I'd preserve are:

- **Claims are first-class objects. Papers are provenance.** The paper should eventually become the container from which assertions, evidence, methods, qualifications, relationships, authorship, and history emerge.
- **Never confuse extraction with truth.** An LLM saying “this passage asserts X” creates a _candidate assertion_. It doesn't establish X.
- **Never throw provenance away.** Ideally you can click any graph edge all the way down to the actual passage that justified its existence. `concept → assertion → relation → evidence → source → version`.
- **Represent disagreement rather than resolving it.** If five papers support X and three contradict X, don't have the model decide which side “wins.” Preserve the structure and let other machinery characterize the evidence.
- **Uncertainty belongs in the data model.** Ambiguous equivalence, weak evidence, extraction confidence, disputed relations, and unresolved identity should remain representable rather than being prematurely collapsed.
- **Keep the deterministic skeleton.** arXiv categories, dates, authorship, citations, identifiers, versions and other externally defined facts are extraordinarily valuable precisely because they weren't hallucinated by Diver.
- **LLMs should propose semantic structure, not secretly become the database.** Every time you're tempted to solve something with `"ask Claude what these are related to"`, ask whether you're destroying the property that makes Diver interesting.
- **Don't optimize scale prematurely.** A trustworthy graph of 10,000 papers would teach you much more than an epistemically dubious graph of 100 million.
- **Evaluate relations, not just extraction.** Eventually you'll need gold-standard datasets for `equivalent`, `supports`, `contradicts`, `refines`, `generalizes`, etc. That's probably where some of the actual research work begins.
- **Keep Find boring. Make Dive weird.** Find should be an excellent, predictable retrieval tool. Dive is where the novel interaction belongs: temperature, paths through concepts, competing branches, intellectual ancestry, cross-disciplinary bridges, unresolved questions.

And I'd add one warning.

### Don't accidentally turn Dive into a chatbot.

There will be enormous temptation to eventually make the interface:

```
> What does the literature say about X?

[beautiful AI-generated paragraph]
```

You can certainly offer that interface someday.

But that shouldn't be the _product_.

The interesting object is underneath:

```
                         X
                         │
            ┌────────────┼────────────┐
            │            │            │
        supports      disputes      extends
            │            │            │
         Claim A      Claim B       Claim C
          /   \          │          /     \
        P1     P2       P3        P4       P5
        │       │        │         │        │
     evidence evidence evidence  evidence evidence
```

The generated paragraph should be **a view over that object**.

Today we're extremely good at generating plausible prose about a pile of retrieved documents. What we don't have nearly as much of is durable, inspectable, computationally queryable representations of **why the documents collectively imply the things we say they imply.**

That's Diver's opportunity.

### And I'd give the builders one north-star test

Every few sprints, take a research question none of you know much about.

Give one person Google Scholar/arXiv and an afternoon.

Give another person Diver and an afternoon.

Then ask each to draw:

> **What are the major claims in this area, what evidence supports them, where do researchers disagree, how did the ideas develop, and what questions appear unresolved?**

Don't primarily measure whether Diver found more papers.

Measure whether the Diver user developed a **better mental model of the field**.

If that consistently happens, you're building something valuable.

And eventually the truly ambitious benchmark becomes:

> **Can Diver show a knowledgeable researcher a relationship in their own field that they didn't already know about—and can it show enough provenance that they can verify it themselves?**

The first time it does that legitimately, I'd consider that a major milestone. Because at that point Diver has gone from organizing existing knowledge for you to becoming an **instrument for discovering structure within knowledge**.

That's the version I'd build toward."