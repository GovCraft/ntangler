use crate::models::signature::TangledSignature;
use crate::models::{Oid, TangledCommit, TangledRepository};
use akton::prelude::Arn;
use git2::Signature;
use std::fmt::Display;
use termcolor::ColorChoice;

pub(crate) trait ConsoleStyle: Display {
    fn determine_color_choice(&self, stream: atty::Stream) -> ColorChoice {
        if atty::is(stream) {
            if self.supports_truecolor() {
                ColorChoice::Always
            } else {
                ColorChoice::Auto
            }
        } else {
            ColorChoice::Never
        }
    }
    // Function to check if the terminal supports true color
    fn supports_truecolor(&self) -> bool {
        // This is a simple heuristic. Likely need a more robust check.
        std::env::var("COLORTERM").map_or(false, |colorterm| {
            colorterm == "truecolor" || colorterm == "24bit"
        })
    }
}

pub(crate) trait RepositoryEvent: Send + Sync + 'static {
    fn get_repo_info(&self) -> TangledRepository;
    // fn get_commit_step(&self) -> CommitStep;
    fn get_commit(&self) -> &TangledCommit;
    fn get_id(&self) -> &str;
    fn display(&self) -> &str;
}

pub(crate) trait TanglerCommit {
    fn get_oid(&self) -> &Oid;
    fn get_signature(&self) -> TangledSignature;

    fn get_summary(&self) -> Option<&str>;
    fn get_body(&self) -> Option<&str>;
}
