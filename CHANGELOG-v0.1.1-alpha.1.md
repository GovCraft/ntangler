- # CHANGELOG

  ## [0.1.1] - 2024-06-08

  This release introduces significant improvements to the ntangler application, including the addition of a broker integration, enhanced actor handling, and the renaming of the project from ginja to tangler. These changes aim to streamline the workflow for developers and enhance the efficiency of the application.

  ### Features

  #### General

  - **Rename project from ginja to tangler and add new message types**: The project is now officially named "tangler," reflecting its core functionality and purpose more accurately.

  #### Actors

  - **Add broker integration and improve actor handling**: Integrates a broker system to manage actor interactions more efficiently.
  - **Add repository watcher actor and enhance AI actor with broker**: Introduces a repository watcher actor to monitor changes and enhances the AI actor with broker capabilities for better commit message generation.
  - **Add broker actor and enhance repository actor with commit handling**: Implements a broker actor and improves the repository actor's ability to handle commits.
  - **Enhance RepositoryActor and add NotifyChange handling**: Improves the RepositoryActor by adding change notification handling.
  - **Improve RepositoryActor branch handling and add tests**: Enhances branch handling within the RepositoryActor and includes tests to ensure reliability.
  - **Add RepositoryActor and integrate with GinjaActor**: Integrates the new RepositoryActor with the existing GinjaActor for seamless operation.
  - **Enhance actor structs with Akton framework**: Utilizes the Akton framework to enhance actor structures, improving overall performance.
  - **Add actor system and configuration handling**: Introduces a comprehensive actor system and configuration management for better control and customization.

  #### AI Actor

  - **Add AI actor for commit message generation**: Incorporates an AI actor specifically designed to generate commit messages, reducing the manual effort required by developers.

  #### Configuration

  - **Add repository configuration and main application logic**: Adds configuration settings for repositories and the main logic of the application.
  - **Add cargo configuration file**: Introduces a Cargo configuration file to manage dependencies and build settings.

  ### Fixes

  - **Enhance repo watcher and tangler actor for better change detection**: Improves the sensitivity and accuracy of the repository watcher and tangler actor for detecting changes.
  - **Refactor broker actor and message handling**: Refactors the broker actor to streamline message handling and improve efficiency.
  - **Update version number in Cargo.toml**: Bumps the version from 0.1.0 to 0.1.1 to reflect the new release.

  ### Chores

  - **Update .gitignore to include mock repositories**: Adds mock repositories to the .gitignore file for better project management.
  - **Add Cargo.toml file with initial dependencies**: Introduces a Cargo.toml file with the initial set of dependencies.
  - **Add .gitignore file**: Adds a .gitignore file to manage ignored files and directories.

  [0.1.1]: https://github.com/GovCraft/ntangler/releases/tag/0.1.1