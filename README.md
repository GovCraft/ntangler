# ntangler

Welcome to ntangler! ntangler is an innovative AI-driven command line application designed to automate the commit process for software developers. By leveraging advanced language models, ntangler generates clean, concise commit messages that adhere to the Conventional Commits policy and makes commits automatically as developers save their work. This reduces the manual effort involved in the commit process and minimizes the context switching that typically disrupts a developer’s workflow.

## Key Features

- **Automated Commits:** ntangler automatically commits changes as developers save their files, generating commit messages using AI to ensure consistency and adherence to the Conventional Commits policy.
- **Context Switching Reduction:** By automating the commit process, ntangler minimizes interruptions, allowing developers to maintain their focus and productivity.

## Getting Started

### Prerequisites

- **Rust:** Ensure you have Rust installed. You can download it from [rust-lang.org](https://www.rust-lang.org/).
- **Git:** Make sure Git is installed on your machine. You can download it from [git-scm.com](https://git-scm.com/).

### Installation

1. **Clone the Repository:**
    ```sh
    git clone https://github.com/GovCraft/ntangler.git
    cd ntangler
    ```

2. **Build the Project:**
    ```sh
    cargo build --release
    ```

3. **Run ntangler:**
    ```sh
    ./target/release/ntangler
    ```

## Usage

When you start ntangler, it will automatically begin monitoring your repository and making commits to your local branch. Just code—commits are taken care of.

- **To stop ntangler, press:** `Ctrl+C`
