use super::agent_run_compression;
use super::StepObservation;

fn make_obs(id: &str, stdout_len: usize) -> StepObservation {
    StepObservation {
        step_id: id.to_string(),
        status: "ok".to_string(),
        exit_code: Some(0),
        stdout: "x".repeat(stdout_len),
        stderr: String::new(),
        elapsed_ms: 100,
        content_hash: None,
    }
}

#[test]
fn apply_success_allows_ok_and_skipped() {
    use super::agent_apply_verification_success;
    assert!(agent_apply_verification_success(&[
        StepObservation {
            step_id: "a".into(),
            status: "ok".into(),
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            elapsed_ms: 1,
            content_hash: None,
        },
        StepObservation {
            step_id: "b".into(),
            status: "skipped".into(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            elapsed_ms: 0,
            content_hash: None,
        },
    ]));
    assert!(!agent_apply_verification_success(&[StepObservation {
        step_id: "c".into(),
        status: "error".into(),
        exit_code: Some(1),
        stdout: String::new(),
        stderr: String::new(),
        elapsed_ms: 1,
        content_hash: None,
    }]));
}

#[test]
fn compress_noop_under_threshold() {
    let mut obs = vec![make_obs("step_1", 100), make_obs("step_2", 100)];
    let original_len: usize = obs.iter().map(|o| o.stdout.len()).sum();
    agent_run_compression::compress_if_needed(&mut obs, 1);
    let after_len: usize = obs.iter().map(|o| o.stdout.len()).sum();
    assert_eq!(
        original_len, after_len,
        "Should not compress under threshold"
    );
}

#[test]
fn compress_reduces_older_observations() {
    let mut obs: Vec<StepObservation> = (0..10)
        .map(|i| make_obs(&format!("step_{}", i), 4000))
        .collect();
    agent_run_compression::compress_if_needed(&mut obs, 2);

    for i in 0..8 {
        assert!(
            obs[i].stdout.len() < 4000,
            "Older observation {} should be compressed, got len {}",
            i,
            obs[i].stdout.len()
        );
    }
    assert_eq!(obs[8].stdout.len(), 4000, "Recent observations preserved");
    assert_eq!(obs[9].stdout.len(), 4000, "Recent observations preserved");
}

#[test]
fn compute_observation_hash_is_stable_and_sensitive() {
    let h1 = super::compute_observation_hash("cargo test", "ok", Some(0), "pass", "", 42);
    let h2 = super::compute_observation_hash("cargo test", "ok", Some(0), "pass", "", 42);
    let h3 = super::compute_observation_hash("cargo test", "error", Some(1), "pass", "", 42);
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
    assert_eq!(h1.len(), 64);
}

#[test]
fn compress_preserves_status_and_exit_code() {
    let mut obs = vec![
        StepObservation {
            step_id: "failing".to_string(),
            status: "error".to_string(),
            exit_code: Some(1),
            stdout: "x".repeat(5000),
            stderr: "error: something failed\ndetails...".to_string(),
            elapsed_ms: 200,
            content_hash: None,
        },
        make_obs("recent", 100),
    ];
    agent_run_compression::compress_if_needed(&mut obs, 1);
    assert_eq!(obs[0].status, "error");
    assert_eq!(obs[0].exit_code, Some(1));
    assert_eq!(obs[0].step_id, "failing");
}
