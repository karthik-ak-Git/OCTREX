use crate::router::types::RoutingMode;

pub struct FallbackGuard;

impl FallbackGuard {
    /// Asserts invariant: Automatic cloud fallback is strictly forbidden.
    /// Returns true if fallback attempt is safe (i.e. blocked), or false if invalid.
    pub fn assert_no_automatic_fallback(
        original_mode: RoutingMode,
        fallback_target_online: bool,
    ) -> Result<(), String> {
        if fallback_target_online && original_mode != RoutingMode::OnlineOnly {
            Err(
                "FORBIDDEN: Automatic fallback to online execution is strictly prohibited. User explicit action and privacy re-evaluation is required."
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }
}
