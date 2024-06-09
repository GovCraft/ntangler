use std::path::PathBuf;

use akton::prelude::*;

use crate::commits::Commits;

#[akton_message]
pub(crate) struct ResponseCommit {
    pub(crate) path: PathBuf,
    pub(crate) commits: Commits,
}

impl ResponseCommit {
    // Function to squash commits
    pub(crate) fn squash_commits(&self, response_commit: &ResponseCommit) -> String {
        let mut iter = response_commit.commits.commits.iter();
        let first_commit = iter.next().unwrap();

        let mut squashed_commit = format!(
            "{}\n\n{}",
            first_commit.commit.heading, first_commit.commit.description
        );

        for commit in iter {
            squashed_commit.push_str(&format!(
                "\n\n{}: {}",
                commit.commit.heading, commit.commit.description
            ));
        }

        squashed_commit
    }
}

