use std::path::PathBuf;

use derive_new::new;

/// Represents a successful commit message with its details.
#[derive(new, Default, Debug, Clone)]
pub(crate) struct GenerationStarted {
    pub(crate) target_file: PathBuf,
    pub(crate) repository_nickname: String,
}
