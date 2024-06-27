use std::path::PathBuf;

use derive_new::new;

/// Represents a successful commit message with its details.
#[derive(new, Default, Debug, Clone)]
pub(crate) struct FileChangeDetected {
    pub(crate) path: PathBuf,
}
