//! Typestate assertions — the epistemic gate.
//!
//! An [`Assertion`] carries a claim plus the [`Observation`]s that support it,
//! parameterized by a typestate marker. An [`Assertion<Supported>`] can be
//! produced **only** by [`Assertion::<Candidate>::validate`]; the `Supported`
//! variant has no public constructor. Downstream code that requires an
//! `Assertion<Supported>` therefore cannot be handed an unvalidated one — the
//! guarantee is enforced by the compiler, not a runtime check.

use std::marker::PhantomData;

use crate::observation::Observation;

/// Typestate marker: an unvalidated assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate;

/// Typestate marker: an assertion that passed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Supported;

/// A claim about the corpus together with its supporting observations. The
/// `State` parameter is `Candidate` before validation and `Supported` after.
#[derive(Debug, Clone)]
pub struct Assertion<State> {
    claim: String,
    support: Vec<Observation>,
    _state: PhantomData<State>,
}

impl Assertion<Candidate> {
    /// Build a candidate assertion from a claim and its supporting observations.
    pub fn new(claim: impl Into<String>, support: Vec<Observation>) -> Self {
        Self {
            claim: claim.into(),
            support,
            _state: PhantomData,
        }
    }

    /// Attempt to promote this candidate to [`Supported`]. This is the **only**
    /// constructor of `Assertion<Supported>`.
    ///
    /// Returns `Ok(Assertion<Supported>)` when the deterministic support rule is
    /// met, otherwise `Err` carrying the original candidate unchanged.
    pub fn validate(self) -> Result<Assertion<Supported>, Assertion<Candidate>> {
        if self.is_supported() {
            Ok(Assertion {
                claim: self.claim,
                support: self.support,
                _state: PhantomData,
            })
        } else {
            Err(self)
        }
    }

    /// The deterministic v1 support rule: a candidate is supported iff it has at
    /// least one supporting observation. Later sprints refine this rule without
    /// changing the typestate gate.
    fn is_supported(&self) -> bool {
        !self.support.is_empty()
    }
}

impl<State> Assertion<State> {
    /// The asserted claim text.
    pub fn claim(&self) -> &str {
        &self.claim
    }

    /// The observations supporting this assertion.
    pub fn support(&self) -> &[Observation] {
        &self.support
    }
}

/// Build one candidate assertion per observation: the claim is the observation's
/// text and its sole support is that observation. This is the deterministic v1
/// seeding; a later sprint may group or synthesize observations into richer
/// candidates without changing the [`Assertion::validate`] gate.
pub fn candidate_assertions(observations: &[Observation]) -> Vec<Assertion<Candidate>> {
    observations
        .iter()
        .map(|obs| Assertion::<Candidate>::new(obs.text(), vec![obs.clone()]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{ArxivId, ArxivVersion};

    fn obs(text: &str) -> Observation {
        Observation::new(ArxivId::new("2301.00001"), ArxivVersion(1), text)
    }

    #[test]
    fn test_validate_supported() {
        let candidate = Assertion::<Candidate>::new(
            "Attention improves translation accuracy.",
            vec![obs("Attention improves translation accuracy.")],
        );
        let supported = candidate.validate().expect("non-empty support validates");
        assert_eq!(
            supported.claim(),
            "Attention improves translation accuracy."
        );
        assert_eq!(supported.support().len(), 1);
    }

    #[test]
    fn test_validate_rejects_unsupported() {
        let candidate = Assertion::<Candidate>::new("Unsupported claim.", vec![]);
        let result = candidate.validate();
        assert!(result.is_err(), "empty support must not validate");
        // The rejected candidate is returned unchanged.
        let returned = result.unwrap_err();
        assert_eq!(returned.claim(), "Unsupported claim.");
        assert!(returned.support().is_empty());
    }

    #[test]
    fn test_candidate_assertions_from_observations() {
        let observations = vec![
            obs("First observation here."),
            obs("Second observation here."),
        ];
        let candidates = candidate_assertions(&observations);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].claim(), "First observation here.");
        assert_eq!(candidates[0].support().len(), 1);
        assert_eq!(candidates[1].claim(), "Second observation here.");
    }
}
