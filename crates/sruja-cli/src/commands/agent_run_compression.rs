use super::StepObservation;

pub const DEFAULT_TOKEN_BUDGET_THRESHOLD: usize = 8000;
const CHARS_PER_TOKEN: usize = 4;
pub const DEFAULT_MAX_COMPRESSED_OUTPUT_LEN: usize = 120;
const MAX_COMPRESSED_OUTPUT_LEN: usize = DEFAULT_MAX_COMPRESSED_OUTPUT_LEN;

pub fn estimate_tokens(observations: &[StepObservation]) -> usize {
    observations
        .iter()
        .map(|o| {
            (o.step_id.len() + o.status.len() + o.stdout.len() + o.stderr.len())
                / CHARS_PER_TOKEN
        })
        .sum()
}

fn compress_single(obs: &StepObservation) -> StepObservation {
    let compressed_stdout = if obs.stdout.len() > MAX_COMPRESSED_OUTPUT_LEN {
        let first_line = obs.stdout.lines().next().unwrap_or("");
        if first_line.len() > MAX_COMPRESSED_OUTPUT_LEN {
            format!("{}...", &first_line[..MAX_COMPRESSED_OUTPUT_LEN])
        } else {
            format!("{} [+{} chars compressed]", first_line, obs.stdout.len())
        }
    } else {
        obs.stdout.clone()
    };

    let compressed_stderr = if obs.stderr.len() > MAX_COMPRESSED_OUTPUT_LEN {
        let first_line = obs.stderr.lines().next().unwrap_or("");
        if first_line.len() > MAX_COMPRESSED_OUTPUT_LEN {
            format!("{}...", &first_line[..MAX_COMPRESSED_OUTPUT_LEN])
        } else {
            format!("{} [+{} chars compressed]", first_line, obs.stderr.len())
        }
    } else {
        obs.stderr.clone()
    };

    StepObservation {
        step_id: obs.step_id.clone(),
        status: obs.status.clone(),
        exit_code: obs.exit_code,
        stdout: compressed_stdout,
        stderr: compressed_stderr,
        elapsed_ms: obs.elapsed_ms,
        content_hash: obs.content_hash.clone(),
    }
}

#[cfg(test)]
pub fn compress_if_needed(observations: &mut [StepObservation], keep_recent: usize) {
    compress_if_needed_with_threshold(
        observations,
        keep_recent,
        DEFAULT_TOKEN_BUDGET_THRESHOLD,
    );
}

pub fn compress_if_needed_with_threshold(
    observations: &mut [StepObservation],
    keep_recent: usize,
    threshold: usize,
) {
    if estimate_tokens(observations) <= threshold {
        return;
    }

    let total = observations.len();
    if total <= keep_recent {
        return;
    }

    let compress_count = total - keep_recent;
    for obs in observations.iter_mut().take(compress_count) {
        *obs = compress_single(obs);
    }
}
