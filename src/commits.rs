use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct CommitDetails {
    pub(crate) heading: String,
    pub(crate) description: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Commit {
    pub(crate) commit: CommitDetails,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Commits {
    pub(crate) commits: Vec<Commit>,
}
