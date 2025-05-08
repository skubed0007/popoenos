# Popoen

Popoen is a minimal operating system written in Rust. It features a custom shell, keyboard driver, and basic utilities, making it a great starting point for learning OS development.

## Features

- **Custom Shell**: A simple shell that supports commands like `pofetch`, `clear`, and `exit`.
- **Keyboard Driver**: Handles PS/2 keyboard input with support for scancode decoding.
- **System Information**: The `pofetch` command displays ASCII art and OS version information.
- **VGA Text Mode**: Provides basic text output with color formatting.

## Getting Started

### Prerequisites

- Rust nightly toolchain
- `bootimage` installed (`cargo install bootimage`)
- QEMU or a similar emulator for testing

### Building the Project

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd popoen
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Create a bootable image:
   ```bash
   cargo bootimage
   ```

### Running the OS

Run the OS in QEMU:
```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-popoen.bin
```

## Usage

### Shell Commands

- `pofetch`: Displays system information and ASCII art.
- `clear`: Clears the screen.
- `exit`: Exits the shell.

### Keyboard Input

The keyboard driver supports basic input and handles special keys like Backspace and Enter.

## Project Structure

- `src/main.rs`: Entry point of the OS.
- `src/driver/`: Contains the keyboard and shell drivers.
- `src/apps/`: Includes applications like `pofetch`.
- `src/polib/`: Utility library for printing and macros.

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the [Writing an OS in Rust](https://os.phil-opp.com/) series.
- Uses the `bootloader` crate for bootstrapping.

---

Feel free to contribute or report issues to improve this project!