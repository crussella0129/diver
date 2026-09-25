//! Concept identity: how claim text becomes concepts.
//!
//! Two stoplists classify every token. *Common* words (general English, function
//! words, web/URL tokens) are never concepts and break phrases. *Filler* words
//! (generic research vocabulary such as `model` or `method`) are never concepts on
//! their own but may head a phrase. Everything else is *content*.
//!
//! Classification is by **folded** form: a token's category is the category of its
//! plural-folded key against the folded stoplists, common taking precedence, so every
//! inflection of a word is treated alike (`state` and `states` are both filler).
//!
//! [`form_concepts`] turns `(paper, claim)` pairs into a deterministic
//! [`ConceptSet`]: one `Term` concept per folded content word, and one `Phrase`
//! concept per two-word phrase shared by at least two papers. Every concept keeps
//! its observed surface forms and the claims and papers that formed it. No model is
//! involved; the same claims always yield the same concepts.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::LazyLock;

/// Bump whenever the formation rules in this module change in code. Persisted
/// concepts are rebuilt when this, the stoplists' digest, or the stored claims change.
pub const CONCEPT_FORMATION_VERSION: u32 = 2;

/// Common words: never concepts, and they break phrases.
const COMMON_WORDS: &str = include_str!("stopwords_common.txt");
/// Research filler: never concepts alone, but may be a phrase's head.
const FILLER_WORDS: &str = include_str!("stopwords_filler.txt");

/// Characters that end a clause. A phrase never spans one. Hyphens and slashes are
/// deliberately absent, so `encoder-decoder` forms `encoder decoder`.
const CLAUSE_BREAKS: &[char] = &[
    '.', ',', ';', ':', '!', '?', '(', ')', '[', ']', '{', '}', '"',
];

/// The words listed in a stoplist file, skipping `#` comment lines.
fn listed_words(file: &str) -> impl Iterator<Item = &str> {
    file.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(str::split_whitespace)
}

/// Folded common words. Built once for O(1) membership.
static COMMON: LazyLock<HashSet<String>> =
    LazyLock::new(|| listed_words(COMMON_WORDS).map(fold).collect());
/// Folded filler words. Built once for O(1) membership.
static FILLER: LazyLock<HashSet<String>> =
    LazyLock::new(|| listed_words(FILLER_WORDS).map(fold).collect());

/// How a token participates in concept formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenCategory {
    /// Never a concept; breaks a phrase.
    Common,
    /// Never a concept alone; may be a phrase's head (second word).
    Filler,
    /// A concept word.
    Content,
}

/// Fold a lowercased token across plural inflection. The first matching rule wins:
/// longer than 4 and ending in `ies` → `y` (`families` → `family`); longer than 4
/// and ending in `sses` → drop `es` (`classes` → `class`); ending in `ics` →
/// unchanged, so derivational pairs such as `semantics`/`semantic` stay apart;
/// longer than 3 and ending in `s` but not `ss`, `us`, or `is` → drop the `s`.
/// Anything else is returned unchanged. A suffix heuristic by design: it prefers a
/// missed merge (`bias`/`biases`) over a wrong one.
pub fn fold(token: &str) -> String {
    let n = token.chars().count();
    if n > 4 && token.ends_with("ies") {
        return format!("{}y", &token[..token.len() - 3]);
    }
    if n > 4 && token.ends_with("sses") {
        return token[..token.len() - 2].to_string();
    }
    if token.ends_with("ics") {
        return token.to_string();
    }
    if n > 3
        && token.ends_with('s')
        && !token.ends_with("ss")
        && !token.ends_with("us")
        && !token.ends_with("is")
    {
        return token[..token.len() - 1].to_string();
    }
    token.to_string()
}

/// Nouns ending in `ly` that are not adverbs, so [`is_adverb`] must not treat them as
/// filler.
const LY_NOUNS: &[&str] = &[
    "anomaly",
    "assembly",
    "butterfly",
    "family",
    "monopoly",
    "multiply",
    "supply",
];

/// Adverbs (`typically`, `previously`, `efficiently`) are never concepts: in abstract
/// prose they carry emphasis, not content, yet a stoplist can never enumerate them
/// all. A folded key of 6+ characters ending in `ly` is an adverb unless it is one of
/// the few `ly` nouns in [`LY_NOUNS`].
fn is_adverb(key: &str) -> bool {
    key.len() >= 6 && key.ends_with("ly") && !LY_NOUNS.contains(&key)
}

/// The category of a lowercased token: the category of its folded key, common
/// taking precedence over filler; adverbs are filler.
pub fn category(token: &str) -> TokenCategory {
    let key = fold(token);
    if COMMON.contains(&key) {
        TokenCategory::Common
    } else if FILLER.contains(&key) || is_adverb(&key) {
        TokenCategory::Filler
    } else {
        TokenCategory::Content
    }
}

/// A token can take part in a concept only if it is at least 3 characters long and
/// contains a letter, so figures (`100`, `2023`) and fragments never link papers.
fn passes_token_rule(token: &str) -> bool {
    token.chars().count() >= 3 && token.chars().any(char::is_alphabetic)
}

/// Alphanumeric runs, lowercased.
fn tokens(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|tok| !tok.is_empty())
        .map(str::to_lowercase)
}

/// Extract a claim's significant terms: alphanumeric tokens, lowercased, at least
/// 3 chars long, containing at least one letter, whose folded form is neither a
/// common nor a filler word. Punctuation and case are ignored; pure-number tokens
/// (e.g. "100", "2023") are dropped so shared figures do not spuriously link papers,
/// while mixed tokens ("gpt3", "h100") survive. Returns surface forms, unfolded.
pub fn significant_terms(claim: &str) -> Vec<String> {
    tokens(claim)
        .filter(|tok| passes_token_rule(tok) && category(tok) == TokenCategory::Content)
        .collect()
}

/// Two-word phrase candidates in a claim, as `(folded id, surface form)`. A pair of
/// adjacent tokens in one clause forms a candidate when both pass the token rule,
/// the first is content, and the second is content or filler.
fn phrase_candidates(claim: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for clause in claim.split(CLAUSE_BREAKS) {
        let toks: Vec<String> = tokens(clause).collect();
        for pair in toks.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            if !(passes_token_rule(a) && passes_token_rule(b)) {
                continue;
            }
            if category(a) != TokenCategory::Content || category(b) == TokenCategory::Common {
                continue;
            }
            out.push((format!("{} {}", fold(a), fold(b)), format!("{a} {b}")));
        }
    }
    out
}

/// Normalize a user query into a concept id: tokenize, drop common words and tokens
/// failing the token rule, fold the rest, and join with single spaces
/// (`"The Diffusion Models"` → `"diffusion model"`). `None` when nothing remains.
pub fn query_key(query: &str) -> Option<String> {
    let parts: Vec<String> = tokens(query)
        .filter(|tok| passes_token_rule(tok) && category(tok) != TokenCategory::Common)
        .map(|tok| fold(&tok))
        .collect();
    (!parts.is_empty()).then(|| parts.join(" "))
}

/// Whether a concept is a single word or a two-word phrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConceptKind {
    Term,
    Phrase,
}

impl ConceptKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ConceptKind::Term => "term",
            ConceptKind::Phrase => "phrase",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "term" => Some(ConceptKind::Term),
            "phrase" => Some(ConceptKind::Phrase),
            _ => None,
        }
    }
}

/// A formed concept. `id` is the folded key (stable across rebuilds and corpus
/// growth); `label` is the most frequent surface form, ties broken by the
/// lexicographically smallest. `claims` are indices into the input of
/// [`form_concepts`]; `papers` are the papers those claims belong to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concept {
    pub id: String,
    pub label: String,
    pub kind: ConceptKind,
    pub forms: BTreeMap<String, usize>,
    pub claims: BTreeSet<usize>,
    pub papers: BTreeSet<String>,
}

impl Concept {
    fn new(id: String, kind: ConceptKind) -> Self {
        Self {
            id,
            label: String::new(),
            kind,
            forms: BTreeMap::new(),
            claims: BTreeSet::new(),
            papers: BTreeSet::new(),
        }
    }

    fn observe(&mut self, surface: String, claim: usize, paper: &str) {
        *self.forms.entry(surface).or_insert(0) += 1;
        self.claims.insert(claim);
        self.papers.insert(paper.to_string());
    }

    /// For a phrase, its two word ids; for a term, its own id.
    pub fn words(&self) -> impl Iterator<Item = &str> {
        self.id.split(' ')
    }
}

/// Every concept formed from a set of claims, keyed by id.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConceptSet {
    pub concepts: BTreeMap<String, Concept>,
}

impl ConceptSet {
    pub fn get(&self, id: &str) -> Option<&Concept> {
        self.concepts.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Concept> {
        self.concepts.values()
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }
}

/// Form concepts from `(arxiv_id, claim)` pairs. Deterministic: ordered collections
/// throughout, so the same claims always produce the same set, and input order only
/// affects the recorded claim indices.
pub fn form_concepts(claims: &[(String, String)]) -> ConceptSet {
    let mut concepts: BTreeMap<String, Concept> = BTreeMap::new();

    for (idx, (paper, claim)) in claims.iter().enumerate() {
        for term in significant_terms(claim) {
            concepts
                .entry(fold(&term))
                .or_insert_with_key(|id| Concept::new(id.clone(), ConceptKind::Term))
                .observe(term, idx, paper);
        }
        for (id, surface) in phrase_candidates(claim) {
            concepts
                .entry(id)
                .or_insert_with_key(|id| Concept::new(id.clone(), ConceptKind::Phrase))
                .observe(surface, idx, paper);
        }
    }

    // Recurrence is the evidence a word pair is a unit: a phrase needs two papers.
    concepts.retain(|_, c| c.kind == ConceptKind::Term || c.papers.len() >= 2);

    for concept in concepts.values_mut() {
        concept.label = best_form(&concept.forms);
    }

    ConceptSet { concepts }
}

/// The most frequent form; ties go to the lexicographically smallest, which is the
/// first one a `BTreeMap` yields.
fn best_form(forms: &BTreeMap<String, usize>) -> String {
    let mut best: Option<(&String, usize)> = None;
    for (form, &count) in forms {
        if best.is_none_or(|(_, n)| count > n) {
            best = Some((form, count));
        }
    }
    best.map(|(form, _)| form.clone()).unwrap_or_default()
}

/// FNV-1a over a byte slice, continuing from `hash`.
fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// A deterministic digest of two stoplists' words. Whitespace and line endings do
/// not matter; adding, removing, or moving a word between the lists does.
pub fn digest_words(common: &str, filler: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for word in listed_words(common) {
        hash = fnv1a(hash, word.as_bytes());
        hash = fnv1a(hash, &[0]);
    }
    hash = fnv1a(hash, &[1]);
    for word in listed_words(filler) {
        hash = fnv1a(hash, word.as_bytes());
        hash = fnv1a(hash, &[0]);
    }
    hash
}

/// The digest of the embedded stoplists; part of the persisted-concept fingerprint.
pub fn rules_digest() -> u64 {
    digest_words(COMMON_WORDS, FILLER_WORDS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_significant_terms() {
        // 'the' and generic filler ('improves') stopped; domain terms + acronym kept.
        assert_eq!(
            significant_terms("Attention improves the RNN accuracy!"),
            vec!["attention", "rnn", "accuracy"],
            "'the'/'improves' stopped; punctuation/case ignored; 3-char acronym kept"
        );
        // Two-char tokens and stopwords are dropped.
        assert!(significant_terms("AI is ML").is_empty());
        // Pure-number tokens are dropped (no letter); 'data'/'with' stopped; 'gpt3' kept.
        assert_eq!(
            significant_terms("Trained for 100 epochs on 2023 data with GPT3."),
            vec!["epochs", "gpt3"],
            "'100'/'2023' dropped as numbers; 'trained'/'data'/'with' stopped; 'gpt3' kept"
        );
    }

    #[test]
    fn test_significant_terms_stoplist() {
        // Generic English, research filler, and web tokens are all dropped.
        let noise = significant_terms(
            "The model shows existing results between multiple https github.com repos",
        );
        for w in [
            "the", "model", "shows", "existing", "results", "between", "multiple", "https",
            "github", "com",
        ] {
            assert!(
                !noise.contains(&w.to_string()),
                "'{w}' should be stopped: {noise:?}"
            );
        }
        // Distinctive domain terms survive.
        assert_eq!(
            significant_terms("attention convolutional diffusion transformer translation bleu"),
            vec![
                "attention",
                "convolutional",
                "diffusion",
                "transformer",
                "translation",
                "bleu"
            ],
        );
    }

    fn claims(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect()
    }

    fn phrase_ids(claim: &str) -> Vec<String> {
        phrase_candidates(claim)
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    }

    #[test]
    fn test_stoplist_categories() {
        let common: HashSet<&str> = listed_words(COMMON_WORDS).collect();
        let filler: HashSet<&str> = listed_words(FILLER_WORDS).collect();
        assert!(common.is_disjoint(&filler), "the two stoplists overlap");
        for w in [
            "https", "http", "www", "github", "com", "propose", "provides", "recent", "show",
            "simple", "widely", "work", "works", "the", "of", "with",
        ] {
            assert_eq!(category(w), TokenCategory::Common, "{w} should be common");
        }
        for w in ["model", "mechanism", "learning", "score"] {
            assert_eq!(category(w), TokenCategory::Filler, "{w} should be filler");
        }
        // Drive repair: adverbs are filler by rule; a few `ly` nouns are not adverbs.
        for w in ["typically", "previously", "efficiently", "highly"] {
            assert_eq!(category(w), TokenCategory::Filler, "adverb {w}");
        }
        for w in ["anomaly", "anomalies", "family", "families", "assembly"] {
            assert_eq!(category(w), TokenCategory::Content, "noun {w}");
        }
    }

    #[test]
    fn test_fold() {
        for (plural, singular) in [
            ("networks", "network"),
            ("rnns", "rnn"),
            ("families", "family"),
            ("classes", "class"),
            ("studies", "study"),
        ] {
            assert_eq!(fold(plural), singular, "{plural}");
        }
        // Precedence: `ies` before `s` (not `familie`), `sses` before `s` (not `classe`).
        assert_ne!(fold("families"), "familie");
        assert_ne!(fold("classes"), "classe");
        for unchanged in [
            "analysis",
            "corpus",
            "process",
            "loss",
            "gas",
            "attention",
            "semantics",
            "metrics",
        ] {
            assert_eq!(fold(unchanged), unchanged);
        }
    }

    #[test]
    fn test_folded_classification_consistent() {
        for (a, b) in [
            ("state", "states"),
            ("detail", "details"),
            ("input", "inputs"),
            ("part", "parts"),
            ("effect", "effects"),
            ("condition", "conditions"),
            ("network", "networks"),
        ] {
            assert_eq!(category(a), category(b), "{a} vs {b}");
        }
    }

    #[test]
    fn test_phrase_rule() {
        assert_eq!(phrase_ids("a language model"), vec!["language model"]);
        assert_eq!(
            phrase_ids("machine translation"),
            vec!["machine translation"]
        );
        assert_eq!(phrase_ids("an encoder-decoder"), vec!["encoder decoder"]);
        // Each input has a break between every pair of words, so none forms a phrase.
        for (broken, why) in [
            ("model establishes", "filler cannot modify"),
            ("attention of translation", "common word"),
            ("project page https", "common and web tokens"),
            ("gpt 4 model", "short numeric token"),
            ("50 layers", "numeric token"),
            ("translation, attention", "punctuation"),
        ] {
            assert!(phrase_ids(broken).is_empty(), "{broken:?}: {why}");
        }
    }

    #[test]
    fn test_concept_formation() {
        // Inflections are one term concept, carrying every surface form.
        let set = form_concepts(&claims(&[
            ("A", "Neural networks learn."),
            ("B", "A network generalizes."),
        ]));
        let network = set.get("network").expect("network concept");
        assert_eq!(network.kind, ConceptKind::Term);
        assert_eq!(network.forms.get("networks"), Some(&1));
        assert_eq!(network.forms.get("network"), Some(&1));
        assert_eq!(network.papers.len(), 2);

        // A phrase needs two papers, however often one paper repeats it.
        let one = form_concepts(&claims(&[(
            "A",
            "Denoising diffusion works. Denoising diffusion again.",
        )]));
        assert!(one.get("denoising diffusion").is_none());
        let two = form_concepts(&claims(&[
            ("A", "Denoising diffusion works."),
            ("B", "We study denoising diffusion."),
        ]));
        assert_eq!(
            two.get("denoising diffusion").map(|c| c.kind),
            Some(ConceptKind::Phrase)
        );

        // A phrase is distinct from its words; no derivational or head merge.
        let set = form_concepts(&claims(&[
            (
                "A",
                "Machine translation improves. The attention mechanism helps.",
            ),
            (
                "B",
                "Machine translation scales. An attention mechanism is attentional.",
            ),
        ]));
        for id in [
            "machine translation",
            "machine",
            "translation",
            "attention",
            "attentional",
            "attention mechanism",
        ] {
            assert!(set.get(id).is_some(), "missing concept {id}");
        }

        // Label: most frequent form, ties to the lexicographically smallest.
        let set = form_concepts(&claims(&[
            ("A", "Networks. Networks. Networks. Network."),
            ("B", "Graphs. Graph."),
        ]));
        assert_eq!(set.get("network").unwrap().label, "networks");
        assert_eq!(set.get("graph").unwrap().label, "graph");
    }

    #[test]
    fn test_formation_invariants() {
        let input = claims(&[
            ("P1", "The hidden state grows."),
            ("P2", "A hidden state decays."),
            ("P3", "Each hidden state resets."),
            ("P4", "Two hidden states merge."),
            ("P5", "Hidden states split."),
            ("P6", "Machine translation improves."),
            ("P7", "Machine translation scales."),
            ("P8", "Translation drifts."),
        ]);
        let set = form_concepts(&input);

        // A phrase's papers are a subset of each constituent term's papers.
        for phrase in set.iter().filter(|c| c.kind == ConceptKind::Phrase) {
            for word in phrase.words() {
                if category(word) == TokenCategory::Content {
                    let term = set.get(word).expect("constituent term exists");
                    assert!(phrase.papers.is_subset(&term.papers), "{}", phrase.id);
                }
            }
        }

        // Deterministic, and independent of input order once indices are remapped.
        let again = form_concepts(&input);
        assert_eq!(again, set);
        assert_eq!(format!("{again:?}"), format!("{set:?}"));
        let n = input.len();
        let reversed: Vec<_> = input.iter().rev().cloned().collect();
        let mut rset = form_concepts(&reversed);
        for c in rset.concepts.values_mut() {
            c.claims = c.claims.iter().map(|i| n - 1 - i).collect();
        }
        assert_eq!(rset, set);

        // Traceable: each concept records exactly the claims and papers containing it.
        for concept in set.iter() {
            for (i, (paper, claim)) in input.iter().enumerate() {
                let contains = match concept.kind {
                    ConceptKind::Term => significant_terms(claim)
                        .iter()
                        .any(|t| fold(t) == concept.id),
                    ConceptKind::Phrase => phrase_ids(claim).contains(&concept.id),
                };
                assert_eq!(
                    concept.claims.contains(&i),
                    contains,
                    "{} / {i}",
                    concept.id
                );
                if contains {
                    assert!(concept.papers.contains(paper));
                }
            }
        }
    }

    #[test]
    fn test_query_key() {
        assert_eq!(
            query_key("The Diffusion Models").as_deref(),
            Some("diffusion model")
        );
        assert_eq!(query_key("NETWORKS").as_deref(), Some("network"));
        for empty in ["", "the of", "50", "   "] {
            assert_eq!(query_key(empty), None, "{empty:?}");
        }
    }

    #[test]
    fn test_digest_words() {
        let base = digest_words("a b\r\nc", "x");
        assert_eq!(base, digest_words("a  b\nc", "x"), "whitespace-insensitive");
        assert_eq!(
            base,
            digest_words("# a comment\na b c", "x"),
            "comments ignored"
        );
        assert_ne!(base, digest_words("a b c d", "x"), "added word");
        assert_ne!(base, digest_words("a b", "x"), "removed word");
        assert_ne!(base, digest_words("a b", "c x"), "word moved between lists");
    }
}
