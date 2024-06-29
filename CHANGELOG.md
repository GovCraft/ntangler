# CHANGELOG (ntangler)


<a name="v0.1.2-alpha.1"></a>
## [v0.1.2-alpha.1](https://github.com/GovCraft/ntangler/compare/v0.1.5-alpha.1...v0.1.2-alpha.1)

> 2024-06-29

### Chore

* bump version to 0.1.2 Updated Cargo.toml to increase the package version from 0.1.1 to 0.1.2. perf: increase channel capacity and debounce timeout Increased the channel capacity to 200 and debounce timeout to 1500ms for better performance in handling watch events. refactor: remove redundant broker clone in GitSentinel Removed duplicate broker clone in the async block within the Observe event handler. fix: correct polling interval in GitSentinel Changed the polling interval from 5 seconds to 3 seconds in the GitSentinel actor. style: add trace logging for initialization steps Added trace logging for broker actor initialization and error notification handler setup. refactor: remove unnecessary variable assignment in main function Removed an unnecessary variable assignment in the main function to simplify the code. docs: add comments for OpenAi generator activation Added comments to describe the activation of the OpenAi generator in the PooledActor implementation. fix: correct default branch name in config Changed default branch name from 'dogfood' to 'main' in the configuration file. fix: correct polling interval in GitSentinel Changed the polling interval from 5 seconds to 60 seconds in the GitSentinel implementation. style: improve error message for OpenAi generator activation Updated the error message in the OpenAi generator activation to provide more context. style: add warning log for displayed user error Added a warning log for displayed user errors in the Tangler actor. fix: correct error handling in checkout_branch function Improved error handling in the checkout_branch function by adding specific error messages for locking the repository mutex and finding the branch. style: improve error logging messages in repositories.rs Updated error messages for better clarity and consistency in the GitRepository implementation. style: improve error logging for missing repository Updated error message to provide more detailed information when the repository is not found. style: improve error message for repository activation failure Updated the error message in the GitRepository implementation to provide more user-friendly feedback when failing to activate RepositoryActor. feat: enhance error messages for repository operations Added detailed error messages for various git2::ErrorCode cases including NotFound, BareRepo, UnbornBranch, Unmerged, NotFastForward, Conflict, Auth, Certificate, MergeConflict, and IndexDirty.


<a name="v0.1.5-alpha.1"></a>
## [v0.1.5-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.0.3-alpha.1...v0.1.5-alpha.1)

> 2024-06-29


<a name="v2.0.3-alpha.1"></a>
## [v2.0.3-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.0.2-alpha.1...v2.0.3-alpha.1)

> 2024-06-29

### Chore

* **build:** bump version to 2.0.3

### Refactor

* **main:** improve config handling and shutdown


<a name="v2.0.2-alpha.1"></a>
## [v2.0.2-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.0.1-alpha.1...v2.0.2-alpha.1)

> 2024-06-29

### Chore

* **build:** bump version to 2.0.2
* **repo:** remove commented code

### Fix

* **ui:** correct column heading length calculation
* **ui:** corrects label text and styling

### Refactor

* **actors:** rename init to initialize


<a name="v2.0.1-alpha.1"></a>
## [v2.0.1-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.2.0-alpha.1...v2.0.1-alpha.1)

> 2024-06-29


<a name="v2.2.0-alpha.1"></a>
## [v2.2.0-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.1.0-alpha.1...v2.2.0-alpha.1)

> 2024-06-29

### Chore

* **build:** bump version to 2.2.0

### Feat

* **logging:** Add dynamic log path creation

### Fix

* **logging:** correct event matching

### Refactor

* **logging:** update notify watcher setup
* **logging:** Refactor logging setup and config path


<a name="v2.1.0-alpha.1"></a>
## [v2.1.0-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.0.4-alpha.1...v2.1.0-alpha.1)

> 2024-06-29

### Build

* **dependencies:** add tracing-appender dependency

### Chore

* **actors:** remove duplicate import
* **actors:** remove commented code
* **actors:** remove duplicate imports
* **build:** rename binary target
* **build:** bump version to 2.1.0
* **build:** update package version
* **build:** update package version
* **cleanup:** remove unused module
* **logging:** fix import formatting
* **messages:** remove unused import
* **messages:** remove unused import
* **messages:** uncomment DiffQueued use
* **models:** clean up unused imports

### Feat

* **logging:** Add logs path resolution
* **logging:** add file appender for logging
* **repo:** include unreadable files as untracked

### Fix

* **actors:** remove duplicate import
* **imports:** remove duplicate imports
* **imports:** remove duplicate imports
* **imports:** remove duplicate imports
* **logging:** correct logging directives
* **logging:** remove duplicate log directive
* **repositories:** fix file status filter
* **repositories:** fix Status import
* **ui:** correct spacing issue

### Refactor

* **actors:** remove commented code
* **actors:** remove duplicate imports
* **actors:** clean up commented code
* **config:** rename find_config_file_path to find_config_path
* **logging:** refactor tracing setup
* **logging:** change info to trace in Oid from
* **logging:** replace debug with trace
* **messages:** remove unused import
* **tests:** move tests module
* **tracing:** consolidate tracing setup functions


<a name="v2.0.4-alpha.1"></a>
## [v2.0.4-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.4.0-alpha.1...v2.0.4-alpha.1)

> 2024-06-29


<a name="v2.4.0-alpha.1"></a>
## [v2.4.0-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.3.3-alpha.1...v2.4.0-alpha.1)

> 2024-06-29

### Chore

* **actors:** remove duplicate imports
* **build:** bump version to 2.4.0
* **config:** remove duplicate imports
* **generators:** remove commented code
* **messages:** remove duplicate imports
* **messages:** remove duplicate derive_new import
* **messages:** remove duplicate derive_new import
* **messages:** remove duplicate import
* **messages:** remove unused imports
* **messages:** comment out unused code
* **models:** remove duplicate imports
* **models:** remove unused imports
* **ui:** remove unused imports

### Docs

* **generators:** fix typo in comment

### Feat

* **models:** add AsRef implementation for Footer

### Fix

* **ui:** handle fmt error in DescriptionTerminal
* **ui:** handle fmt error in FilenameTerminal
* **ui:** handle fmt error in OidTerminal
* **ui:** fix fmt function
* **ui:** correct fmt error handling

### Refactor

* **actors:** remove unused code
* **actors:** clean up imports and structs
* **actors:** remove duplicate imports
* **actors:** remove dead code
* **imports:** reorder import statements
* **imports:** remove duplicate imports
* **lib:** remove redundant assertions
* **main:** reformat code for readability
* **main:** clean up imports and code structure
* **messages:** remove unused imports
* **messages:** remove duplicate imports
* **messages:** remove duplicate import
* **models:** remove unused imports
* **models:** remove unused imports
* **models:** remove unused imports
* **models:** remove unused code and tests
* **models:** remove unused imports and modules
* **models:** remove unused imports
* **models:** remove unused imports
* **models:** remove redundant imports
* **models:** remove duplicate imports and struct
* **repository:** clean up imports and streamline code
* **ui:** improve error handling in ScopeTerminal
* **ui:** remove redundant code
* **ui:** remove duplicate import
* **ui:** refactor SemVerImpactTerminal fmt
* **ui:** remove redundant imports
* **ui:** simplify commit heading formatting
* **ui:** simplify terminal handling

### Style

* **repositories:** reformat repository actor code
* **ui:** remove duplicate import


<a name="v2.3.3-alpha.1"></a>
## [v2.3.3-alpha.1](https://github.com/GovCraft/ntangler/compare/v3.0.0-alpha.1...v2.3.3-alpha.1)

> 2024-06-29


<a name="v3.0.0-alpha.1"></a>
## [v3.0.0-alpha.1](https://github.com/GovCraft/ntangler/compare/v2.3.2-alpha.1...v3.0.0-alpha.1)

> 2024-06-29

### Chore

* **actors:** remove duplicate imports
* **build:** update Cargo.toml workspace members
* **build:** bump version to 2.4.0
* **build:** bump version to 2.3.3
* **config:** remove duplicate imports
* **generators:** remove commented code
* **imports:** remove unused import
* **messages:** remove duplicate derive_new import
* **messages:** remove duplicate imports
* **messages:** remove duplicate derive_new import
* **messages:** remove duplicate import
* **messages:** remove unused imports
* **messages:** comment out unused code
* **models:** reorder use statements
* **models:** remove duplicate imports
* **models:** remove unused imports
* **ui:** remove unused imports

### Docs

* update README clone and run instructions
* **generators:** fix typo in comment

### Feat

* **models:** add AsRef implementation for Footer

### Fix

* **models:** correct ARN root value
* **ui:** handle fmt error in DescriptionTerminal
* **ui:** handle fmt error in FilenameTerminal
* **ui:** handle fmt error in OidTerminal
* **ui:** fix fmt function
* **ui:** correct fmt error handling
* **ui:** correct column heading

### Refactor

* **actors:** remove dead code
* **actors:** remove unused code
* **actors:** remove duplicate imports
* **actors:** clean up imports and structs
* **config:** standardize config handling
* **imports:** remove duplicate imports
* **imports:** reorder import statements
* **lib:** remove redundant assertions
* **main:** clean up imports and code structure
* **main:** reformat code for readability
* **messages:** remove duplicate import
* **messages:** remove duplicate imports
* **messages:** remove unused imports
* **models:** remove unused imports and modules
* **models:** remove unused imports
* **models:** remove unused imports
* **models:** remove unused imports
* **models:** remove unused code and tests
* **models:** reorder use statements
* **models:** remove duplicate imports and struct
* **models:** remove redundant imports
* **models:** remove unused imports
* **models:** remove unused imports
* **repository:** clean up imports and streamline code
* **ui:** simplify terminal handling
* **ui:** remove redundant code
* **ui:** remove duplicate import
* **ui:** refactor SemVerImpactTerminal fmt
* **ui:** remove redundant imports
* **ui:** improve error handling in ScopeTerminal
* **ui:** simplify commit heading formatting

### Style

* **repositories:** reformat repository actor code
* **ui:** remove duplicate import

### BREAKING CHANGE


Major version update to 3.0.0.
BREAKING CHANGES: You have made changes that may break backward compatibility. According to Semantic Versioning (SemVer), this requires a major version update. Please verify and update your version number accordingly.

Renaming Tangler to Ntangler affects the public API and requires changes in any dependent code.
BREAKING CHANGES: You have made changes that may break backward compatibility. According to Semantic Versioning (SemVer), this requires a major version update. Please verify and update your version number accordingly.

The GitRepository struct and its initialization functions now require NtangledRepository instead of TangledRepository.
BREAKING CHANGES: You have made changes that may break backward compatibility. According to Semantic Versioning (SemVer), this requires a major version update. Please verify and update your version number accordingly.


<a name="v2.3.2-alpha.1"></a>
## [v2.3.2-alpha.1](https://github.com/GovCraft/ntangler/compare/v3.3.1-alpha.1...v2.3.2-alpha.1)

> 2024-06-29


<a name="v3.3.1-alpha.1"></a>
## [v3.3.1-alpha.1](https://github.com/GovCraft/ntangler/compare/v3.3.0-alpha.1...v3.3.1-alpha.1)

> 2024-06-29

### Chore

* **build:** bump version to 3.3.1

### Fix

* **actors:** fix duplicate async call

### Refactor

* **actors:** Refactor circuit breaker calls
* **generators:** extract async endpoint call


<a name="v3.3.0-alpha.1"></a>
## [v3.3.0-alpha.1](https://github.com/GovCraft/ntangler/compare/v3.1.1-alpha.1...v3.3.0-alpha.1)

> 2024-06-29

### Chore

* **actors:** remove commented-out code
* **build:** bump version to 3.3.0
* **build:** update package version

### Feat

* **actors:** add create_message_with_circuit_breaker function
* **generators:** add run stream creation with circuit breaker

### Refactor

* **actors:** remove redundant timeout block
* **imports:** remove duplicate import

### Style

* **imports:** remove duplicate import


<a name="v3.1.1-alpha.1"></a>
## [v3.1.1-alpha.1](https://github.com/GovCraft/ntangler/compare/v3.5.0-alpha.1...v3.1.1-alpha.1)

> 2024-06-29


<a name="v3.5.0-alpha.1"></a>
## [v3.5.0-alpha.1](https://github.com/GovCraft/ntangler/compare/v3.4.1-alpha.1...v3.5.0-alpha.1)

> 2024-06-29

### Chore

* **build:** bump version to 3.5.0

### Feat

* **logging:** Add log for DiffQueued event

### Refactor

* **actors:** improve logging details
* **logging:** improve logging messages
* **logging:** consolidate logging statements
* **logging:** improve logging messages

### Style

* **logging:** improve logging details


<a name="v3.4.1-alpha.1"></a>
## [v3.4.1-alpha.1](https://github.com/GovCraft/ntangler/compare/v3.4.0-alpha.1...v3.4.1-alpha.1)

> 2024-06-29

### Chore

* **build:** bump version to 3.4.1
* **error:** add error logging

### Fix

* **logging:** improve error logging
* **logging:** Improve error logging

### Refactor

* **logging:** update logging levels

### Style

* **logging:** improve logging messages


<a name="v3.4.0-alpha.1"></a>
## v3.4.0-alpha.1

> 2024-06-29

### Build

* **dependencies:** add tracing-appender dependency
* **dependencies:** add new dependencies
* **package:** bump version to 3.4.0

### Chore

* update tracing directives Added 'async_openai=trace' directive to the tracing configuration.
* add Cargo.toml file with initial dependencies
* add trace log for relative path in commit Added a trace log for the relative path of the committed file to improve debugging.
* update tracing directives for various modules Changed log levels for 'tangler::actors::sentinels' to info and 'async_openai' to trace.
* update .gitignore to include mock repositories
* add serde_json dependency Add serde_json version 1.0.117 to dependencies
* clean up commented code in tests Removed commented-out code in the test module for better readability.
* remove duplicate import of RepositoryConfig Removed the duplicate import of RepositoryConfig from tangler_config.rs.
* update config for repository settings Changed branch_name to 'dogfood' and updated api_url to 'no longer needed' in config.toml.
* remove unnecessary import in tangler_config.rs Removed an unused import statement in tangler_config.rs to clean up the code.
* update tracing directives for various actors Changed log level for 'tangler::actors::generators' to 'info' and added 'async_openai' with 'trace' level.
* remove duplicate use statement for OpenAi Removed the duplicate use statement for OpenAi in src/actors.rs.
* update gitignore patterns
* add .gitignore file
* update gitignore
* format commit message with heading and description Formatted the commit message to include both heading and description.
* update dependencies in Cargo.toml Added 'time' crate version 0.3.36 to dependencies.
* update api_url in config.toml Removed the old api_url and added the new api_url 'https://api.example.com/generate-commit-message'.
* bump version to 0.1.2 Updated Cargo.toml to increase the package version from 0.1.1 to 0.1.2. perf: increase channel capacity and debounce timeout Increased the channel capacity to 200 and debounce timeout to 1500ms for better performance in handling watch events. refactor: remove redundant broker clone in GitSentinel Removed duplicate broker clone in the async block within the Observe event handler. fix: correct polling interval in GitSentinel Changed the polling interval from 5 seconds to 3 seconds in the GitSentinel actor. style: add trace logging for initialization steps Added trace logging for broker actor initialization and error notification handler setup. refactor: remove unnecessary variable assignment in main function Removed an unnecessary variable assignment in the main function to simplify the code. docs: add comments for OpenAi generator activation Added comments to describe the activation of the OpenAi generator in the PooledActor implementation. fix: correct default branch name in config Changed default branch name from 'dogfood' to 'main' in the configuration file. fix: correct polling interval in GitSentinel Changed the polling interval from 5 seconds to 60 seconds in the GitSentinel implementation. style: improve error message for OpenAi generator activation Updated the error message in the OpenAi generator activation to provide more context. style: add warning log for displayed user error Added a warning log for displayed user errors in the Tangler actor. fix: correct error handling in checkout_branch function Improved error handling in the checkout_branch function by adding specific error messages for locking the repository mutex and finding the branch. style: improve error logging messages in repositories.rs Updated error messages for better clarity and consistency in the GitRepository implementation. style: improve error logging for missing repository Updated error message to provide more detailed information when the repository is not found. style: improve error message for repository activation failure Updated the error message in the GitRepository implementation to provide more user-friendly feedback when failing to activate RepositoryActor. feat: enhance error messages for repository operations Added detailed error messages for various git2::ErrorCode cases including NotFound, BareRepo, UnbornBranch, Unmerged, NotFastForward, Conflict, Auth, Certificate, MergeConflict, and IndexDirty.
* bump version to 0.1.2 Updated Cargo.toml to increase the package version from 0.1.1 to 0.1.2. perf: increase channel capacity and debounce timeout Increased the channel capacity to 200 and debounce timeout to 1500ms for better performance in handling watch events. refactor: remove redundant broker clone in GitSentinel Removed duplicate broker clone in the async block within the Observe event handler. fix: correct polling interval in GitSentinel Changed the polling interval from 5 seconds to 3 seconds in the GitSentinel actor. style: add trace logging for initialization steps Added trace logging for broker actor initialization and error notification handler setup. refactor: remove unnecessary variable assignment in main function Removed an unnecessary variable assignment in the main function to simplify the code. docs: add comments for OpenAi generator activation Added comments to describe the activation of the OpenAi generator in the PooledActor implementation. fix: correct default branch name in config Changed default branch name from 'dogfood' to 'main' in the configuration file. fix: correct polling interval in GitSentinel Changed the polling interval from 5 seconds to 60 seconds in the GitSentinel implementation. style: improve error message for OpenAi generator activation Updated the error message in the OpenAi generator activation to provide more context. style: add warning log for displayed user error Added a warning log for displayed user errors in the Tangler actor. fix: correct error handling in checkout_branch function Improved error handling in the checkout_branch function by adding specific error messages for locking the repository mutex and finding the branch. style: improve error logging messages in repositories.rs Updated error messages for better clarity and consistency in the GitRepository implementation. style: improve error logging for missing repository Updated error message to provide more detailed information when the repository is not found. style: improve error message for repository activation failure Updated the error message in the GitRepository implementation to provide more user-friendly feedback when failing to activate RepositoryActor. feat: enhance error messages for repository operations Added detailed error messages for various git2::ErrorCode cases including NotFound, BareRepo, UnbornBranch, Unmerged, NotFastForward, Conflict, Auth, Certificate, MergeConflict, and IndexDirty.
* remove unused imports
* **actors:** remove commented-out code
* **actors:** remove duplicate imports
* **actors:** remove duplicate import
* **actors:** remove duplicate imports
* **actors:** remove commented code
* **build:** update Cargo.toml for library
* **build:** bump version to 2.4.0
* **build:** update package version
* **build:** bump version to 2.2.0
* **build:** bump version to 2.1.0
* **build:** bump version to 2.3.3
* **build:** update Cargo.toml
* **build:** rename binary target
* **build:** update dependencies
* **build:** bump version to 3.3.1
* **build:** bump version to 3.3.0
* **build:** update package version
* **build:** update package version
* **build:** bump version to 3.1.1
* **build:** update library path
* **build:** update package version
* **build:** update package version
* **build:** bump version to 2.0.4
* **build:** update Cargo.toml dependencies
* **build:** update dependencies
* **build:** update library name
* **build:** update Cargo.toml workspace members
* **build:** rename package to ntangler
* **build:** bump version to 2.0.3
* **build:** bump version to 2.0.2
* **build:** add uuid dependency
* **build:** bump version to 2.0.1
* **build:** bump version to 2.3.2
* **build:** bump version to 0.1.5
* **build:** update package version
* **build:** update Cargo.toml configuration
* **build:** update Cargo.toml
* **build:** add workspace members
* **cleanup:** remove unused module
* **config:** remove duplicate imports
* **config:** remove duplicate use statements
* **dependencies:** remove unused imports
* **dependencies:** clean up imports
* **dependencies:** remove unused dependencies
* **deps:** remove duplicate imports
* **deps:** remove unused import
* **generators:** remove unused code
* **generators:** remove unused code
* **generators:** remove commented code
* **imports:** remove unused import
* **imports:** remove unused imports
* **imports:** remove duplicate import
* **logging:** update logging directives
* **logging:** fix import formatting
* **messages:** remove duplicate imports
* **messages:** remove duplicate derive_new import
* **messages:** remove unused import
* **messages:** comment out unused code
* **messages:** clean up unused imports
* **messages:** remove unused imports
* **messages:** remove unused import
* **messages:** uncomment DiffQueued use
* **messages:** reorder module imports
* **messages:** remove duplicate imports
* **messages:** remove duplicate import
* **messages:** remove duplicate derive_new import
* **models:** reorder use statements
* **models:** clean up unused imports
* **models:** reformat footer.rs file
* **models:** remove unused imports
* **models:** remove duplicate imports
* **models:** clean up commented imports
* **models:** uncomment modules
* **models:** remove duplicate imports
* **repo:** remove commented code
* **repo:** update .gitignore
* **styles:** remove unused styles
* **ui:** reorganize module imports
* **ui:** remove unused import
* **ui:** remove unused imports
* **ui:** remove unused module import
* **ui:** add new module imports

### Docs

* update README clone and run instructions
* update repository URL
* update error message for object not found Updated the error message for 'NotFound' error code to be more concise.
* update welcome message in main function Updated the welcome message in the main function to be more concise.
* update welcome message with stop instructions Updated the welcome message to include instructions on how to stop Tangler by pressing Ctrl+C.
* update welcome message in main.rs Updated the welcome message to provide more clarity on Tangler's functionality.
* update README with project description Added a description 'Auto commits using AI and Rust' to the README file.
* correct references from RepositoryWatcherActor to GitSentinel Updated comments and log messages to correctly refer to GitSentinel instead of RepositoryWatcherActor.
* add event descriptions for error handling in repository watcher Added comments to describe events for debounce error, watcher setup failure, and watcher start failure.
* add comments for event handling in RepositoryWatcherActor Added comments to describe the setup of the Watch handler and the debounce error handling in RepositoryWatcherActor.
* **generators:** fix typo in comment
* **readme:** rename Tangler to ntangler

### Feat

* add Poll message emission on file change Added the emission of the Poll message in the GitSentinel actor when a file change is detected.
* add support for subscriber IDs in Broker Modified the Broker struct to include subscriber IDs. Updated methods to handle subscriber IDs, including load_subscriber_futures and load_subscriber_future_by_id. Added debug logs for better traceability.
* add Poll message handling to GitRepository Added handling for Poll message in GitRepository to check for unstaged files and log them. Also included Status and StatusOptions from git2 crate.
* add polling mechanism for repository changes Replaced Watch event handler with Observe event handler.
* add cargo configuration file
* add function to squash commits Implemented a function in ResponseCommit to squash multiple commits into a single string representation.
* add warning log for empty diff in repository_actor.rs Added a warning log when no repository diffs are found to provide better diagnostics.
* add broker integration and improve actor handling
* add repository configuration and main application logic
* add repository watcher actor and enhance AI actor with broker
* add actor system and configuration handling
* add broker actor and enhance repository actor with commit handling
* rename project from ginja to tangler and add new message types
* **actors:** add RepositoryActor and integrate with GinjaActor
* **actors:** add call_ai_endpoint function
* **actors:** improve RepositoryActor branch handling and add tests
* **actors:** add create_message_with_circuit_breaker function
* **actors:** enhance RepositoryActor and add NotifyChange handling
* **actors:** enhance actor structs with Akton framework
* **actors:** add GenerationStarted message handling
* **ai_actor:** add AI actor for commit message generation
* **config:** add id to RepositoryConfig
* **generators:** add circuit breaker to create thread
* **generators:** add run stream creation with circuit breaker
* **logging:** add tracing directive
* **logging:** add file appender for logging
* **logging:** Add logs path resolution
* **logging:** Add dynamic log path creation
* **logging:** add debug log for modified files
* **messages:** add FinalizedCommit struct
* **messages:** add reply_address field
* **messages:** add GenerationStarted struct
* **messages:** add CommitEvent struct
* **messages:** add CommitPending struct
* **messages:** add CommitAuthoring struct
* **models:** add PendingCommit struct
* **models:** add generate_id function
* **models:** implement Ord for TimeStamp
* **models:** add filename to commit struct
* **models:** add Filename struct
* **models:** add AsRef implementation for Footer
* **repo:** include unreadable files as untracked
* **server:** integrate Mistral AI client
* **ui:** add terminal status display
* **ui:** add AppEvent struct and impls
* **ui:** add TAB_WIDTH constant

### Fix

* correct async task spawning method Replaced `tokio::task::spawn_local` with `tokio::spawn` for polling repository changes to ensure proper async task handling.
* correct error message handling in shutdown signal listener Updated error log message to properly handle the shutdown signal error.
* improve error message for shutdown signal failure Updated the error message when unable to listen for the shutdown signal to provide more context and user-friendly information.
* correct broker initialization in unit tests Replaced BrokerActor::init() with Broker::init() in unit tests to ensure proper broker initialization.
* enhance repo watcher and tangler actor for better change detection
* update version number in Cargo.toml Bump version from 0.1.0 to 0.1.1
* handle JSON parsing errors in ai_actor Added error handling for JSON parsing in ai_actor to prevent crashes on malformed JSON.
* correct log directive level in main.rs Changed log directive level from debug to trace for repository_actor in main.rs.
* update alert message for empty todo title Changed the alert message from 'Please enter a todo title.' to 'Todo title cannot be empty.'
* update variable naming in repository_config.rs Renamed variables in the test module of repository_config.rs for clarity and consistency. fix: update Repositories variable assignment in repository_config.rs Changed the variable assignment in the test module of repository_config.rs to match the updated Repositories struct.
* update tracing directives for repository_actor and ai_actor Changed the tracing directives for repository_actor and ai_actor to include both trace and debug levels.
* correct duplicate use statement for AiActor Removed duplicate use statement for AiActor from generators module.
* handle index.lock error during debounce Added specific handling to ignore 'index.lock not found' error during the debounce process.
* correct duplicate imports in repository_actor.rs Removed duplicate imports of DiffOptions, Error, and Repository from git2 and tracing.
* correct path canonicalization and relative path extraction Fixed the canonicalization of the event file path and extraction of the relative path by stripping the repository root prefix.
* ensure correct commit message format in repository_actor.rs Corrected the commit message format to include both heading and description in the commit.
* remove duplicate commits field in ResponseCommit struct Removed the duplicate 'commits' field in the 'ResponseCommit' struct to avoid compilation errors.
* remove duplicate import of Broker and Sentinel Removed duplicate import of Broker and Sentinel from tangler_actor.rs. Changed pool builder to use OpenAi instead of AiActor.
* remove redundant error log in shutdown handling Removed the redundant error log statement in the shutdown handling block to avoid duplicate messages.
* increase channel capacity and debounce timeout Increased the channel capacity to 200 and debounce timeout to 2000ms to handle more events efficiently.
* correct shutdown signal error message Changed the error message for shutdown signal from 'Couldn't listen for the shutdown signal' to 'Couldn't catch the shutdown signal' for clarity.
* correct duplicate import in git_repository.rs Removed duplicate import of RepositoryActor in unit_tests module.
* refactor broker actor and message handling
* correct struct field type in SubmitDiff Changed the type of the 'path' field in the SubmitDiff struct from PathBuf to String.
* remove duplicate .gitignore entry in WalkBuilder Removed the duplicate entry for .gitignore in the WalkBuilder configuration.
* correct module imports in actors.rs Fixed incorrect module imports by changing 'broker' to 'brokers' and 'sentinel' to 'sentinels'.
* correct import path for GitRepository Corrected the import path for GitRepository in the unit tests module.
* correct trace log id for OpenAi generator activation Changed trace log id from actor.key.value to context.key.value in OpenAi generator activation.
* correct logging directives in init_tracing function Changed logging directive for 'tangler::actors::repositories' from 'error' to 'info' and added missing 'error' directive for 'tangler::actors::generators'.
* correct struct field type in NotifyChange Changed the type of the 'path' field in the NotifyChange struct from PathBuf to String.
* remove duplicate activation call in GitSentinel Removed the duplicate call to actor.activate(None).await? in the GitSentinel implementation.
* correct tracing directives and imports Corrected duplicate import of 'error' from 'tracing'. Added missing 'Poll' message import. Updated tracing directives to set appropriate log levels and disable unnecessary logs.
* correct TanglerActor initialization in tests Replaced TanglerActor::init with Tangler::init in test setup.
* correct duplicate field in ResponseCommit struct Removed duplicate 'path' field from the ResponseCommit struct.
* correct Poll message handling and remove duplicate status check Fixed the Poll message handling by removing the duplicate status check and correcting the trace message for CheckoutBranch. Added subscription to broker for poll requests.
* emit commit message with id to broker Added 'id' to the ResponseCommit emission to the broker in the commit message handling logic.
* **actors:** correct polling interval
* **actors:** correct pool size calculation
* **actors:** fix duplicate async call
* **actors:** remove duplicate import
* **actors:** handle thread creation error
* **async:** fix duplicate method call
* **config:** correct config file path
* **config:** correct log path environment variable
* **generators:** remove redundant returns
* **imports:** remove duplicate imports
* **imports:** remove duplicate imports
* **imports:** remove duplicate imports
* **logging:** remove duplicate log directive
* **logging:** correct event matching
* **logging:** correct logging directives
* **messages:** fix struct declaration
* **messages:** fix timestamp initialization
* **models:** correct ARN root value
* **models:** ensure Oid is lowercase
* **repositories:** include untracked files in diff
* **repositories:** fix file status filter
* **repositories:** fix Status import
* **styles:** correct color definitions
* **tests:** correct repository paths and async task spawning Updated repository paths in tests to use parent directory. Changed tokio::spawn to tokio::task::spawn_local for polling repository changes.
* **ui:** correct spacing issue
* **ui:** correct semver style usage
* **ui:** correct copyright text formatting
* **ui:** remove duplicate imports
* **ui:** remove duplicate imports
* **ui:** correct column heading length calculation
* **ui:** corrects label text and styling
* **ui:** correct style usage
* **ui:** correct style usage
* **ui:** remove duplicate imports
* **ui:** correct color style usage
* **ui:** correct column heading
* **ui:** correct fmt error handling
* **ui:** fix fmt function
* **ui:** handle fmt error in OidTerminal
* **ui:** handle fmt error in FilenameTerminal
* **ui:** handle fmt error in DescriptionTerminal
* **ui:** correct tab width calculation

### Perf

* increase channel capacity for Watch actor Increased the channel capacity from default to 200 in the Watch actor setup to handle more messages efficiently.
* increase channel capacity and debounce timeout Increased channel capacity to 200 and debounce timeout to 1500ms for better performance.
* increase channel capacity and debounce timeout Increased the channel capacity from 100 to 200 and debounce timeout from 1000ms to 2000ms for better performance.

### Refactor

* replace RepositoryActor with GitRepository Replaced RepositoryActor with GitRepository in TanglerActor initialization and tests for better code clarity and consistency.
* rename BrokerActor to Broker Renamed struct BrokerActor to Broker and updated associated methods and references.
* improve diff generation and handling Refactored the diff generation logic to handle multiple modified files and improved error handling. Updated the Poll and Diff message handling to use async functions. Changed the path handling in commit logic to use canonical paths.
* remove duplicate event logging and unused variable Removed duplicate event logging for setting up the watcher and unused variable 'file' in change detection.
* rename Watch struct to Observe Renamed the Watch struct to Observe in observe.rs for better clarity.
* remove NotifyChange handling and improve logging Removed the NotifyChange message handling logic. Replaced info logs with trace logs for better granularity. Added Observe message handling for diff watchers.
* replace TanglerActor with Tangler Replaced TanglerActor with Tangler in main function, tests, and tracing directives for consistency and code simplification.
* rename TanglerActor to Tangler Renamed TanglerActor struct to Tangler and updated associated methods and comments accordingly.
* clone repo path before activation Cloned the repository path before activation to avoid borrowing issues.
* update squash_commits function to improve logging and formatting Removed unused parameter from squash_commits function. Added detailed logging for squashing commits. Fixed formatting issues in the squashed commit string.
* update calculate_id function for better tracing and documentation Added tracing instrumentation and detailed documentation to the calculate_id function. Fixed duplicate hasher update call. Added PartialEq to RepositoryConfig struct.
* clean up commented code in repository_actor Removed commented-out code and improved error messages in repository_actor.
* rename AiActor to OpenAi Renamed struct AiActor to OpenAi and updated related references in the initialize method and PooledActor implementation.
* simplify repository path handling Simplified the handling of repository path by removing redundant clone operations.
* remove redundant event comments Removed redundant event comments to improve code readability and maintainability.
* remove redundant clone calls Removed unnecessary clone calls for repository_path and repository_path_trace to improve code efficiency.
* replace RepositoryWatcherActor with Sentinel Replaced RepositoryWatcherActor with Sentinel in TanglerActor initialization.
* rename RepositoryActor to GitRepository Renamed the struct RepositoryActor to GitRepository and updated all references accordingly in repository.rs.
* rename RepositoryWatcherActor to Sentinel Renamed the RepositoryWatcherActor struct to Sentinel and updated corresponding references in the init function.
* replace Sentinel with GitSentinel for diff watcher initialization Replaced Sentinel with GitSentinel in TanglerActor for initializing a diff watcher actor.
* rename BrokerActor to Broker Renamed BrokerActor to Broker in tangler_actor.rs to improve code readability and consistency.
* **actors:** Refactor circuit breaker calls
* **actors:** remove dead code
* **actors:** refactor async function parameters
* **actors:** refactor OpenAi initialization
* **actors:** rename init to initialize
* **actors:** rename MySecret struct
* **actors:** clean up commented code
* **actors:** remove duplicate spawn call
* **actors:** remove redundant runtime creation
* **actors:** remove duplicate imports
* **actors:** clean up imports and structs
* **actors:** clean up repository actor
* **actors:** update event handling
* **actors:** clean up unused imports and comments
* **actors:** remove redundant timeout block
* **actors:** Replace CommitEvent with AppEvent
* **actors:** clean up repository actor code
* **actors:** remove duplicate imports
* **actors:** remove redundant imports
* **actors:** refactor Scribe actor
* **actors:** remove unused code
* **actors:** Refactor session_count and text formatting
* **actors:** remove commented code
* **actors:** reformat tangler actor init
* **async:** remove redundant task spawn
* **brokers:** replace debug with trace
* **code:** improve code formatting
* **config:** rename find_config_file_path to find_config_path
* **config:** standardize config handling
* **generators:** simplify circuit breaker call
* **generators:** refactor thread creation
* **generators:** refactor circuit breaker usage
* **generators:** extract async endpoint call
* **generators:** Refactor OpenAi actor
* **imports:** reorder import statements
* **imports:** remove duplicate imports
* **imports:** remove duplicate import
* **lib:** remove redundant assertions
* **logging:** update notify watcher setup
* **logging:** replace debug with trace
* **logging:** change info to trace in Oid from
* **logging:** refactor tracing setup
* **logging:** Refactor logging setup and config path
* **logging:** update tracing setup
* **main:** improve config handling and shutdown
* **main:** reformat code for readability
* **main:** clean up imports and code structure
* **messages:** remove unused import
* **messages:** remove unused imports
* **messages:** remove duplicate import
* **messages:** remove duplicate imports
* **models:** remove unused imports
* **models:** simplify oid formatting
* **models:** reorder use statements
* **models:** refactor TimeStamp struct
* **models:** remove unused imports and modules
* **models:** remove unused imports
* **models:** remove unused imports
* **models:** remove duplicate imports
* **models:** remove unused code and tests
* **models:** remove unused imports
* **models:** remove unused imports
* **models:** remove redundant imports
* **models:** reformat commit_message.rs
* **models:** remove duplicate imports and struct
* **models:** remove unused imports
* **models:** reformat timestamp module
* **models:** improve deserialization logic
* **repositories:** update repository event creation
* **repositories:** refactor async emit
* **repository:** reorganize imports and format code
* **repository:** clean up imports and streamline code
* **repository:** remove dead code and cleanup
* **styles:** rename and reorganize styles
* **tests:** move tests module
* **tests:** remove unnecessary comments and fix variable names Removed TODO comments and fixed variable names in test cases for better readability.
* **tracing:** consolidate tracing setup functions
* **traits:** Add bounds to RepositoryEvent trait
* **ui:** improve error handling in ScopeTerminal
* **ui:** remove redundant imports
* **ui:** remove unused imports
* **ui:** remove duplicate import
* **ui:** simplify terminal handling
* **ui:** update terminal event styling
* **ui:** remove commented code
* **ui:** refactor AppEvent struct and methods
* **ui:** simplify commit heading formatting
* **ui:** refactor SemVerImpactTerminal fmt
* **ui:** remove redundant code

### Style

* add missing newline at end of file Added a newline at the end of src/messages/broker_subscribe.rs to adhere to coding standards.
* improve shutdown signal error message Updated the error message for shutdown signal to reassure users that their code is safe.
* fix spacing issues in Cargo.toml Corrected spacing inconsistencies in the dependencies section of Cargo.toml.
* update log messages for better clarity Changed log message from 'Activating the AiActor' to 'Activating the OpenAi generator' for better clarity.
* fix inconsistent spacing in async calls Fixed inconsistent spacing in async calls to maintain code style consistency.
* add missing newline at end of file Added a newline at the end of src/messages/broker_unsubscribe.rs to adhere to coding standards.
* reorder use statements for better readability Reordered use statements in src/actors.rs for better readability and organization.
* add newline at end of file Added a newline at the end of src/messages/submit_diff.rs to comply with POSIX standards.
* organize imports in messages.rs Added missing import for checkout_branch and load_repo modules.
* improve logging format for commit messages Updated the logging format to include the number of commits in the log message.
* fix duplicate error message Removed duplicate error message in GitRepository implementation.
* update log message for repository change detection Updated the log message from 'Repository change detected' to 'Change in' for better clarity.
* add trace logging for better debugging Added trace logging alongside info logging for setting up Watch handler, setting up Watcher, and detecting repository changes. Increased channel capacity to 200.
* remove extra blank line Removed an extra blank line to improve code readability.
* reorder use statements for better readability Reordered use statements in src/actors.rs for better readability and maintainability.
* improve error messages for branch and repository access issues Updated error message for branch not found to be more concise.
* unify log message format for local commit Changed log message from 'local commit: {}' to 'Local commit: {:?}' for consistency.
* adjust tracing directives for various actors Changed tracing directive for 'tangler::actors::repository_watcher_actor' to 'debug' and 'async_openai' to 'trace'.
* improve readability of activation message Separated the activation message into two lines for better readability.
* remove trailing whitespace in unit_tests module Removed trailing whitespace in the unit_tests module to improve code readability.
* improve log message formatting in GitSentinel Updated the println! statement to use the format string for repo_name and branch_name variables.
* add title attribute to buttons for better accessibility Added title attributes to 'Delete', 'Add Todo', and 'Clear Completed' buttons to improve accessibility.
* improve shutdown messages for better clarity Updated shutdown messages to provide more detailed information:
* **actors:** fix spacing in function signature
* **actors:** fix formatting issues
* **actors:** rename CallAiEndpoint method
* **actors:** fix indentation in initialize
* **format:** reformat code for readability
* **imports:** remove duplicate import
* **main:** reformat code for readability
* **messages:** add newline at EOF
* **messages:** remove duplicate imports
* **messages:** fix newline at EOF
* **messages:** remove duplicate code
* **messages:** add missing comma
* **messages:** remove redundant newline
* **messages:** fix formatting issue
* **messages:** remove redundant newline
* **messages:** remove duplicate imports
* **messages:** add missing comma
* **messages:** add missing comma
* **messages:** remove duplicate imports
* **messages:** remove duplicate imports
* **messages:** remove duplicate imports
* **models:** reorder imports
* **models:** reorder imports
* **models:** reformat TangledCommit struct
* **models:** fix formatting issues
* **models:** reformat description.rs
* **models:** remove duplicate derive macro
* **models:** remove extra newline
* **models:** fix formatting issues
* **models:** reorder imports
* **repositories:** reformat repository actor code
* **terminal:** reformat code for readability
* **ui:** reformat code structure
* **ui:** reformat scope.rs file
* **ui:** reformat repository.rs
* **ui:** reformat is_breaking.rs
* **ui:** reformat semver_impact.rs
* **ui:** fix formatting issues
* **ui:** reorder mod imports
* **ui:** fix spacing issue
* **ui:** reorganize imports
* **ui:** remove extra blank lines
* **ui:** reformat description.rs
* **ui:** remove duplicate imports
* **ui:** remove duplicate imports
* **ui:** fix formatting and imports
* **ui:** remove duplicate import

### Test

* add test for detecting modified but unstaged files Added a new test 'test_poll_modified_unstaged_files' to check for modified but unstaged files in a repository. This includes setting up a bare repository, cloning it, creating and modifying a test file, and verifying the detection of changes.
* add debug information for diff generation Added additional debug information for diff generation in tests.
* add missing import for fs module in tests Added the missing import for the fs module in the tests module to ensure file operations work correctly.
* **repository_actor:** add unit test for squash_commits function Added a unit test to verify the functionality of the squash_commits function, ensuring it correctly combines multiple commits into a single string.

### BREAKING CHANGE


Major version update to 3.0.0.
BREAKING CHANGES: You have made changes that may break backward compatibility. According to Semantic Versioning (SemVer), this requires a major version update. Please verify and update your version number accordingly.

Renaming Tangler to Ntangler affects the public API and requires changes in any dependent code.
BREAKING CHANGES: You have made changes that may break backward compatibility. According to Semantic Versioning (SemVer), this requires a major version update. Please verify and update your version number accordingly.

The GitRepository struct and its initialization functions now require NtangledRepository instead of TangledRepository.
BREAKING CHANGES: You have made changes that may break backward compatibility. According to Semantic Versioning (SemVer), this requires a major version update. Please verify and update your version number accordingly.

Replaced old commit event handling with new repository and commit step handling logic.

Replaced CommitEventCategory::DiffGenerated with DiffQueued and refactored related logic.

Renamed 'Commit' to 'CommittedCommit' and updated event categories, which may affect existing event handling and message processing logic.

Updated commit event category from Commit to Posted. This affects all places where commit events are handled.

The Broker actor has been removed and replaced with the AktonReady system. This will require updates to any code that relied on the Broker actor.

`RepositoryWatcherActor` now requires a broker context for initialization.

The `RepositoryActor` and `TanglerActor` now require a broker context for initialization.

The project and binary names have been changed, which may affect existing scripts and documentation.

