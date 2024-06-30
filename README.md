<img style="margin-left: auto;margin-right: auto;" src="https://images.ntangler.ai/images/logo_mark@0.75x.png" alt="description" width="75" height="75">

# ntangler
#### feat(workflow): never write a commit message again


ntangler is an AI-powered tool that automatically generates meaningful commit messages, allowing developers to focus on coding without breaking their flow.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/Govcraft/ntangler/actions/workflows/release.yml/badge.svg)](https://travis-ci.org/yourusername/ntangler)

## Features

- 🧠 AI-powered analysis of code changes
- ✍️ Automatic generation of [Conventional Commit](https://www.conventionalcommits.org/en/v1.0.0/#specification) messages
- 🔄 Seamless integration with local Git workflow (branches, worktrees)
- 🎨 Customizable to match team commit styles
- ⚡ Boosts productivity by eliminating context switching
- 🌐 Compatible with any LLM via HTTP server

## Installation

### Current Alpha Version

#### Using Cargo

```bash
cargo install ntangler
```

#### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/Govcraft/ntangler.git
   ```
2. Navigate to the project directory:
   ```bash
   cd ntangler
   ```
3. Build and install:
   ```bash
   cargo build --release
   cargo install --path .
   ```

### Future Availability

In future releases, ntangler will be available through popular package managers for easier installation.

## Configuration

Create a `config.toml` file in the XDG config directory (typically `~/.config/ntangler/config.toml` on Unix-like systems):

```toml
[[repositories]]
path = "/path/to/your/repo"
nickname = "my-project"

[[repositories]]
path = "/path/to/another/repo"
nickname = "another-project"

[[repositories]]
path = "/path/to/another/repo"
nickname = "another-project"
```

Set an $NTANGLER_ENDPOINT environment variable that points to any HTTP server that accepts POST requests with a JSON body containing the diff and returns a JSON response with the commit message structure.

Example JSON response structure:

```json
{
  "commit_type": "feat",
  "scope": "user-auth",
  "description": "add user authentication functionality",
  "body": "Implement JWT-based authentication for user login and registration.\n\n- Add login endpoint\n- Add registration endpoint\n- Implement JWT token generation and validation",
  "is_breaking": false,
  "footers": [
    {"token": "Reviewed-by", "value": "Alice"}
  ],
  "semver_impact": "MINOR"
}
```

For all available options, refer to the [Configuration Guide](https://ntangler.ai/docs/configuration).

## Usage

Once configured, ntangler runs in the background, watching the specified repositories:

1. Start ntangler:
   ```bash
   ntangler
   ```

2. Work on projects as usual, saving files as needed.

3. ntangler automatically generates local commits each time a file is saved.

## Upcoming Features

- Installation from popular package managers.         
- CLI commands for enhanced control and interaction
- Setup wizard for easier initial configuration
- And more!

## Documentation

For full documentation, visit [ntangler.ai/docs](https://ntangler.ai/docs).

## About

ntangler was created to solve the common developer frustration of writing commit messages. To learn more about the motivation behind ntangler and its impact on developer productivity, check out the [About file](ABOUT.md).

## Contributing

Contributions are welcome! Please see the [Contributing Guide](CONTRIBUTING.md) for more details.

## Support

For support, please open an issue on the [GitHub issue tracker](https://github.com/Govcraft/ntangler/issues).

## License

ntangler is [MIT licensed](LICENSE-MIT.md).