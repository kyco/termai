TermAI
======

**TermAI** is a terminal-based AI assistant developed in Rust. It leverages OpenAI's APIs to provide intelligent and
interactive experiences directly from your command line.

Features
--------

- **Interactive Terminal UI**: Utilize a responsive and user-friendly interface built with `ratatui` and `crossterm`.
- **Asynchronous Operations**: Powered by `tokio` for efficient asynchronous processing.
- **Local Configuration Storage**: Manage configurations using SQLite via `rusqlite`.
- **Command-Line Parsing**: Easy-to-use CLI with `clap`.
- **Colored Output**: Enhanced terminal output with `colored`.
- **Extensible Architecture**: Modular design allows for easy extensions and integrations.

Installation
------------

1. **Prerequisites**:
    - Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

2. **Clone the Repository**:
   ```sh
   git clone https://github.com/yourusername/termAI.git
   cd termAI
   ```

3. **Build the Project**:
   ```sh
   cargo build --release
   ```

4. **Run the Application**:
   ```sh
   ./target/release/termAI
   ```

Usage
-----

1. **Set OpenAI API Key**:
   ```sh
   ./termAI --chat-gpt-api-key YOUR_API_KEY
   ```

2. **Prompts**:
    - Use it directly in your terminal to chat with the AI assistant.
    - It's currently designed for 1 shot requests
   ```shell
    ./termAI "What is the capital of France?"
    ```
    - You can also provide a path as context for the AI to understand the context of the request.
   ```shell
    ./termAI "Create a README for this project" .
    ```
    - You can also provide a path as context for the AI to understand the context of the request.
   ```shell
   ./termAI "create mocks for this file" ./src/main.rs
   ```

3. **Commands**:
    - Use the following commands to interact with the application:
    ```shell
    ./termAI --help
    ```
    - To set the OpenAI API key:
    ```shell
    ./termAI --chat-gpt-api-key YOUR_API
    ```

4. **Piping**:
    - You can also pipe the output of other commands into **TermAI**:
    ```shell
    echo "What is the capital of France?" | ./termAI
    ```
    - Create git commit messages from diffs
   ```shell
   git diff | ./termAI "create a short git commit message"
   ```

Configuration
-------------

Configurations are stored locally using SQLite. You can view and modify settings using the provided CLI commands.

Contributing
------------

Contributions are welcome! Please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Commit your changes with clear messages.
4. Open a pull request describing your changes.

License
-------

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

Support
-------

For support or questions, please open an issue on the [GitHub Issues](https://github.com/yourusername/termAI/issues)
page.

---

Thank you for using **TermAI**! 🖥️✨