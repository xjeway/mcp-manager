/// Security/risk placeholder module.
/// Frontend performs initial risk classification for current MVP.
pub fn is_high_risk_condition(err: &str) -> bool {
    err.contains("permission") || err.contains("invalid")
}
