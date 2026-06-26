use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

use crate::trace::SessionStep;

pub(crate) fn validate_session_arrival_rate(sessions_per_second: f64) -> Result<()> {
    if sessions_per_second.is_finite() && sessions_per_second > 0.0 {
        Ok(())
    } else {
        Err(anyhow!(
            "--session-arrival-rate must be finite and greater than 0"
        ))
    }
}

pub(crate) fn apply_session_arrival_rate(
    sessions: &mut BTreeMap<String, Vec<SessionStep>>,
    sessions_per_second: f64,
) -> Result<()> {
    validate_session_arrival_rate(sessions_per_second)?;
    let spacing_ms = 1000.0 / sessions_per_second;

    for (session_ordinal, steps) in sessions.values_mut().enumerate() {
        let arrival_time = session_ordinal as f64 * spacing_ms;
        for step in steps {
            step.arrival_time = arrival_time;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(session_id: &str, round_idx: usize) -> SessionStep {
        SessionStep {
            session_id: session_id.to_string(),
            arrival_time: 99.0,
            round_idx,
            prefix_len: 0,
            input_len: 1,
            output_len: 1,
            tool_wait_after_ms: 0.0,
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn rate_1_assigns_0_1000_2000_ms() {
        let mut sessions = BTreeMap::from([
            ("a".to_string(), vec![step("a", 0)]),
            ("b".to_string(), vec![step("b", 0)]),
            ("c".to_string(), vec![step("c", 0)]),
        ]);

        apply_session_arrival_rate(&mut sessions, 1.0).unwrap();

        assert_close(sessions["a"][0].arrival_time, 0.0);
        assert_close(sessions["b"][0].arrival_time, 1000.0);
        assert_close(sessions["c"][0].arrival_time, 2000.0);
    }

    #[test]
    fn rate_2_5_assigns_0_400_800_ms() {
        let mut sessions = BTreeMap::from([
            ("a".to_string(), vec![step("a", 0)]),
            ("b".to_string(), vec![step("b", 0)]),
            ("c".to_string(), vec![step("c", 0)]),
        ]);

        apply_session_arrival_rate(&mut sessions, 2.5).unwrap();

        assert_close(sessions["a"][0].arrival_time, 0.0);
        assert_close(sessions["b"][0].arrival_time, 400.0);
        assert_close(sessions["c"][0].arrival_time, 800.0);
    }

    #[test]
    fn invalid_zero_rate_rejected() {
        assert!(validate_session_arrival_rate(0.0).is_err());
    }

    #[test]
    fn invalid_nan_rate_rejected() {
        assert!(validate_session_arrival_rate(f64::NAN).is_err());
    }

    #[test]
    fn all_steps_in_session_receive_same_generated_arrival_time() {
        let mut sessions = BTreeMap::from([
            ("a".to_string(), vec![step("a", 0), step("a", 1)]),
            ("b".to_string(), vec![step("b", 0), step("b", 1)]),
        ]);

        apply_session_arrival_rate(&mut sessions, 2.0).unwrap();

        assert_close(sessions["a"][0].arrival_time, 0.0);
        assert_close(sessions["a"][1].arrival_time, 0.0);
        assert_close(sessions["b"][0].arrival_time, 500.0);
        assert_close(sessions["b"][1].arrival_time, 500.0);
    }
}
