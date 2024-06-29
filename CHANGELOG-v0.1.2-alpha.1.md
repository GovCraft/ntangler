# Changelog

## [0.1.2] - 2024-06-09

This release introduces significant performance improvements, enhanced error handling, and several bug fixes. Key changes include increased channel capacity, refined logging, and improved repository handling.

### Features
- **repository**: Add detailed error messages for various git operations, enhancing user feedback and debugging capabilities.
- **broker**: Add support for subscriber IDs, improving traceability and management of subscribers.
- **repository**: Implement a new polling mechanism for repository changes, replacing the Watch event handler.
- **repository**: Add a warning log for empty diffs to aid diagnostics.

### Fixes
- **config**: Correct the default branch name from 'dogfood' to 'main'.
- **repository**: Correct polling intervals and error handling in the GitSentinel and checkout_branch functions.
- **repository**: Correct struct field types in SubmitDiff and NotifyChange for better type safety.
- **repository**: Handle index.lock errors during debounce processes.
- **repository**: Correct path canonicalization and relative path extraction.
- **tests**: Ensure proper async task handling and correct repository paths.

### Performance Improvements
- **repository**: Increase channel capacity to 200 and debounce timeout to 1500ms for better event handling.
- **repository**: Simplify repository path handling by removing redundant clone operations.

### Refactors
- **repository**: Rename Watch struct to Observe for clarity.
- **repository**: Remove redundant broker clone and variable assignments to simplify the code.
- **repository**: Replace RepositoryActor with GitRepository for better code clarity.
- **repository**: Replace TanglerActor with Tangler for consistency.
- **logging**: Improve log message formats and unify logging directives across modules.

### Documentation
- **comments**: Add comments for OpenAi generator activation and repository watcher event handling.
- **README**: Update README with a project description.

### Style
- **logging**: Add trace logging for initialization steps and better error logging messages.
- **code**: Reorder use statements and remove trailing whitespace for better readability.

### Tests
- **repository**: Add tests for detecting modified but unstaged files.
- **repository**: Add unit tests for the squash_commits function to ensure correct commit combination.

### Chores
- **dependencies**: Update dependencies in Cargo.toml, including adding the 'time' crate.
- **config**: Update api_url in config.toml and remove unnecessary imports.