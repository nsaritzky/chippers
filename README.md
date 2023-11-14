# Chippers

## Overview

This project is a Chip-8 emulator written in Rust using SDL. Chip-8 is an interpreted programming language developed in the mid-1970s, used on some microcomputers and graphing calculators. This emulator aims to replicate the Chip-8 environment, allowing users to run Chip-8 programs and games.

## Features

- **Accurate Emulation:** Faithfully emulates the Chip-8 instruction set and behavior.
- **Cross-Platform:** Runs on multiple operating systems, leveraging Rust's cross-platform capabilities.

## Installation

To install this emulator, ensure you have Rust installed on your system. You can download Rust from [the official website](https://www.rust-lang.org/).

```bash
# Clone the repository
git clone https://github.com/nsaritzky/chippers

# Navigate to the project directory
cd chippers

# Build the project
cargo build --release
```

## Usage

After installation, you can run the emulator with:

```bash
cargo run -- [path-to-chip-8-program]
```

Replace `[path-to-chip-8-program]` with the path to a Chip-8 ROM file you wish to run.

## License

This project is licensed under the [MIT License](LICENSE.md) - see the LICENSE.md file for details.

## Contact

For any inquiries or to report issues, please open an issue on the GitHub repository or contact me at mail@requirenathan.com.
