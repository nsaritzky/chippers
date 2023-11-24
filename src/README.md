# Rust CHIP-8 Emulator

This project is a CHIP-8 emulator written in Rust. CHIP-8 is a simple interpreted programming language, originally developed in the mid-1970s for use on microcomputers. This emulator allows you to run CHIP-8 programs on modern hardware.

## Features

- Emulates CHIP-8 instruction set
- Monochrome 64x32 pixel display
- Keyboard input for CHIP-8 hexadecimal keypad
- Configurable execution speed
- Utilizes SDL2 for graphics rendering and event handling

## Prerequisites

Before you run the emulator, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/learn/get-started)
- [SDL2](https://www.libsdl.org/download-2.0.php)

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/nsaritzky/chippers
   ```
2. Navigate to the cloned directory:
   ```
   cd chippers
   ```
3. Build the project:
   ```
   cargo build --release
   ```

## Usage

To run the emulator, use the following command:

```
cargo run --release [path to CHIP-8 program]
```

Replace `[path to CHIP-8 program]` with the path to your CHIP-8 program file.

## Controls

The original CHIP-8 used a hexadecimal keypad. The keys are mapped to your keyboard as follows:

| CHIP-8 Keypad | Keyboard Keys |
| ------------- | ------------- |
| 1 2 3 C       | 1 2 3 4       |
| 4 5 6 D       | Q W E R       |
| 7 8 9 E       | A S D F       |
| A 0 B F       | Z X C V       |

## Contributing

Contributions to this project are welcome. Please feel free to fork the repository and submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
