use akton::prelude::{akton_message, Arn};
use crate::models::{ RepositoryEvent, TangledCommit, TangledRepository, TanglerCommit};
use derive_new::new;

#[akton_message]
pub(crate) struct GitRepositoryEvent {
    repository_info: TangledRepository,
    commit_step:
    commit: TangledCommit,
}

impl RepositoryEvent for GitRepositoryEvent {
    fn get_repo_info(&self) -> TangledRepository {
        self.repository_info.clone()
    }

    fn get_commit_step(&self) -> CommitStep {
        self.commit_step.clone()
    }

    fn get_commit(&self) -> &TangledCommit {
        &self.commit
    }
}

impl GitRepositoryEvent {
    pub fn new(repo_info: TangledRepository, commit_step:  commit: TangledCommit) -> Self {
        GitRepositoryEvent { repository_info: repo_info, commit_step, commit }
    }
}