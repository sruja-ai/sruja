//! Trajectory execution boundary for MaTTS.
//!
//! This trait is intentionally small: the agent crate defines the contract and
//! outcome types, while the CLI provides concrete executors (e.g. git worktrees).

use crate::matts::TrajectoryOutcome;
use std::future::Future;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait TrajectoryExecutor {
    type Error;

    /// Execute a single trajectory attempt and return its outcome.
    fn run_trajectory<'a>(
        &'a self,
        trajectory_id: &'a str,
    ) -> BoxFuture<'a, Result<TrajectoryOutcome, Self::Error>>;

    /// Execute N trajectories (default sequential) and return all outcomes.
    fn run_n<'a>(&'a self, n: usize) -> BoxFuture<'a, Result<Vec<TrajectoryOutcome>, Self::Error>>
    where
        Self: Send + Sync,
    {
        Box::pin(async move {
            let mut out = Vec::new();
            for i in 0..n {
                let id = format!("t{}", i + 1);
                out.push(self.run_trajectory(&id).await?);
            }
            Ok(out)
        })
    }
}
