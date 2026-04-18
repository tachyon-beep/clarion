//! `LlmProvider` trait stub.
//!
//! WP6 (summary-cache + prompt dispatch) fills this out. Sprint 1 defines
//! the hook point so the trait has a stable import path from day one.
//! `NoopProvider` panics if its `name()` is called — Sprint 1 has no
//! code path that legitimately calls it, so panic is a louder bug signal
//! than a silent default.

pub trait LlmProvider: Send + Sync {
    /// Human-readable provider identifier.
    fn name(&self) -> &str;
}

/// Stub provider used in Sprint 1 code paths that take a provider
/// argument. Calling `name()` panics — if you see this panic, something
/// in the WP1 code is reaching for a real provider before WP6 lands.
pub struct NoopProvider;

impl LlmProvider for NoopProvider {
    /// Always panics.
    ///
    /// # Panics
    ///
    /// `NoopProvider` is a Sprint-1 stub; any call to `name()` indicates
    /// `WP1` code is reaching for a real provider before `WP6` lands.
    fn name(&self) -> &str {
        panic!("NoopProvider::name called — WP6 should have replaced this by now")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_provider_implements_trait() {
        fn assert_trait<T: LlmProvider>(_: &T) {}
        let p = NoopProvider;
        assert_trait(&p);
    }

    #[test]
    #[should_panic(expected = "NoopProvider::name called")]
    fn noop_provider_panics_on_name() {
        let p = NoopProvider;
        let _ = p.name();
    }
}
