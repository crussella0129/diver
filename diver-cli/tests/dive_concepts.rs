//! `diver dive` end to end through the real binary against scratch corpora
//! (`DIVER_DB`, set with `Command::env` — no `set_var`).

use std::path::Path;
use std::process::Command;

use diver_core::assertion::{Assertion, Candidate};
use diver_core::id::{ArxivId, ArxivVersion};
use diver_core::observation::Observation;
use diver_core::store::Store;

fn save(db: &Path, paper: &str, claim: &str) {
    let store = Store::open_at(db).unwrap();
    let obs = Observation::new(ArxivId::new(paper), ArxivVersion(1), claim.to_lowercase());
    let supported = Assertion::<Candidate>::new(claim, vec![obs])
        .validate()
        .unwrap();
    store.save_assertions(paper, "v1", &[supported]).unwrap();
}

/// Run `diver <args>` against `db`, returning stdout with ANSI escapes removed.
fn diver(db: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_diver"))
        .args(args)
        .env("DIVER_DB", db)
        .output()
        .expect("the diver binary runs");
    assert!(
        out.status.success(),
        "diver {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    strip_ansi(&String::from_utf8_lossy(&out.stdout))
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn test_cli_dive_concepts() {
    let scratch = tempfile::tempdir().unwrap();
    let db = scratch.path().join("corpus.db");
    save(&db, "2301.00001", "Diffusion models generate images.");
    save(&db, "2302.00002", "A diffusion model denoises networks.");
    save(&db, "2303.00003", "Neural network training converges.");

    // Either inflection resolves; C only says `network`, which substring matching on
    // `networks` would miss.
    // Assert on the *result* set, not bare ids: an id can also appear in another paper's
    // related-papers line, which would let a missing result pass.
    let out = diver(&db, &["dive", "networks"]);
    assert!(out.contains("Dive: network (term, 2 papers)"), "{out}");
    assert!(
        out.contains("\u{2022} A diffusion model denoises networks."),
        "{out}"
    );
    assert!(
        out.contains("\u{2022} Neural network training converges."),
        "{out}"
    );
    assert!(
        out.contains("networks \u{00d7}1") && out.contains("network \u{00d7}1"),
        "{out}"
    );

    // A filler word is not a concept, but it points at the phrase it heads.
    let out = diver(&db, &["dive", "model"]);
    assert!(out.contains("'model' is not a concept"), "{out}");
    assert!(out.contains("diffusion model"), "{out}");

    // A paper added between two runs appears in the second, with no rebuild step.
    assert!(!diver(&db, &["dive", "diffusion"]).contains("2304.00004"));
    save(&db, "2304.00004", "Diffusion sampling accelerates.");
    assert!(diver(&db, &["dive", "diffusion"]).contains("2304.00004"));
}

#[test]
fn test_cli_dive_empty_and_help() {
    let scratch = tempfile::tempdir().unwrap();
    let db = scratch.path().join("empty.db");
    let out = diver(&db, &["dive", "anything"]);
    assert!(out.contains("diver extract"), "{out}");

    let help = diver(&db, &["dive", "--help"]);
    assert!(help.contains("resolved to a concept"), "{help}");
    assert!(help.contains("subsumed by a phrase"), "{help}");
}
