use std::path::PathBuf;
use akton::prelude::*;
use tracing::{instrument, trace};
use crate::commits::Commits;

#[akton_message]
pub(crate) struct ResponseCommit {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) commits: Commits,
}

impl ResponseCommit {
    /// Squashes all commits into a single string.
    ///
    /// # Returns
    /// A string representing the squashed commits.
    #[instrument(skip(self))]
    pub(crate) fn squash_commits(&self) -> String {
        // Event: Squashing Commits
        // Description: Squashing all commits into a single string.
        // Context: Path and number of commits.
        trace!(path = ?self.path, commit_count = self.commits.commits.len(), "Squashing all commits into a single string.");

        let mut iter = self.commits.commits.iter();
        let first_commit = iter.next().unwrap();
        let mut squashed_commit = format!(
            "{}\n\n{}",
            first_commit.commit.heading,
            first_commit.commit.description
        );

        for commit in iter {
            squashed_commit.push_str(&format!(
                "\n\n{}: {}",
                commit.commit.heading,
                commit.commit.description
            ));
        }

        // Event: Commits Squashed
        // Description: All commits have been squashed into a single string.
        // Context: Squashed commit string.
        trace!(squashed_commit = squashed_commit, "All commits have been squashed into a single string.");

        squashed_commit
    }
}

