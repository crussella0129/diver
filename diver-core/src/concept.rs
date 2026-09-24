//! The vocabulary layer: how claim text is tokenized and which words count.
//!
//! Two stoplists classify every token. *Common* words (general English, function
//! words, web/URL tokens) are never concepts and break phrases. *Filler* words
//! (generic research vocabulary such as `model` or `method`) are never concepts on
//! their own but may head a phrase. Everything else is *content*.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Common words: never concepts, and they break phrases.
const COMMON_WORDS: &str = include_str!("stopwords_common.txt");
/// Research filler: never concepts alone, but may be a phrase's head.
const FILLER_WORDS: &str = include_str!("stopwords_filler.txt");

/// The words listed in a stoplist file, skipping `#` comment lines.
fn listed_words(file: &str) -> impl Iterator<Item = &str> {
    file.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(str::split_whitespace)
}

/// Words excluded from significant terms: the union of both stoplists — common
/// English, generic research filler (`model`, `results`, `method`, `propose`,
/// `existing`, …), near-function words, and web/URL tokens (`https`, `github`).
/// Domain terms (`attention`, `transformer`, `diffusion`, `convolutional`,
/// `translation`, `neural`, …) are intentionally absent, so `dive` links papers by
/// distinctive shared vocabulary, not filler. IDF weights the surviving terms; it
/// cannot do this job alone because a generic-but-corpus-rare word (e.g. `eight`,
/// df 2) still scores a high weight. Built once into a `HashSet` for O(1) membership.
static STOPWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    listed_words(COMMON_WORDS)
        .chain(listed_words(FILLER_WORDS))
        .collect()
});

/// Extract a claim's significant terms: alphanumeric tokens, lowercased, at least
/// 3 chars long, containing at least one letter, excluding [`STOPWORDS`].
/// Punctuation and case are ignored; pure-number tokens (e.g. "100", "2023") are
/// dropped so shared figures do not spuriously link papers, while mixed tokens
/// ("gpt3", "h100") survive.
pub fn significant_terms(claim: &str) -> Vec<String> {
    claim
        .split(|c: char| !c.is_alphanumeric())
        .filter(|tok| !tok.is_empty())
        .map(|tok| tok.to_lowercase())
        .filter(|tok| {
            tok.chars().count() >= 3
                && tok.chars().any(|c| c.is_alphabetic())
                && !STOPWORDS.contains(tok.as_str())
        })
        .collect()
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
}
