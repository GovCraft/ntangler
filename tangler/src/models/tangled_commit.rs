use git2::Signature;
use crate::models::{Oid, TanglerCommit};
use derive_more::*;
use crate::models::signature::TangledSignature;

#[derive(Clone, Debug, )]
pub(crate) struct TangledCommit {
    oid: Oid,
    signature: TangledSignature,
    summary: Option<String>,
    body: Option<String>,
}

impl TanglerCommit for TangledCommit {
    fn get_oid(&self) -> &Oid {
        &self.oid
    }

    fn get_signature(&self) -> TangledSignature {
        self.signature.clone()
    }

    fn get_summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    fn get_body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl TangledCommit {
    pub fn new(oid: Oid, signature: TangledSignature, summary: Option<String>, body: Option<String>) -> Self {
        TangledCommit { oid, signature, summary, body }
    }

    // Additional methods for developing the commit through CommitSteps
    pub fn update_summary(&mut self, summary: String) {
        self.summary = Some(summary);
    }

    pub fn update_body(&mut self, body: String) {
        self.body = Some(body);
    }
}