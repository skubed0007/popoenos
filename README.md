# Popoen

Popoen is a minimal operating system written in Rust that implements a basic but functional kernel with essential features. It provides a foundation for learning OS development concepts through practical implementation.

## Features

- **Custom Shell**: A simple shell with support for:
  - File operations (`ls`, `cat`, `touch`)
  - File content manipulation (`>>` for appending text)
  - System commands (`clear`, `exit`)
  - System information (`pofetch`)
- **Keyboard Driver**: Advanced PS/2 keyboard support with:
  - Full scancode set 2 implementation
  - Modifier key support (Shift, Caps Lock, Ctrl, Alt)
  - Special key handling (Function keys, Arrow keys, etc.)
  - Extended scancode support
- **VGA Text Mode Interface**:
  - 80x25 text mode display
  - Color support with ANSI-style formatting
  - Cursor management
  - Scrolling support
- **Basic File System**:
  - Inode-based structure
  - Support for basic file operations
  - Directory entry management
  - Block device abstraction

## Getting Started

### Prerequisites

- Rust nightly toolchain
- `bootimage` installed (`cargo install bootimage`)
- QEMU or similar emulator for testing
- `rust-src` component (`rustup component add rust-src`)

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

- `ls`: List files in the current directory
- `cat <filename>`: Display contents of a file
- `touch <filename>`: Create a new empty file
- `>> <filename> <text>`: Append text to a file
- `clear`: Clear the screen
- `pofetch`: Display system information with ASCII art
- `exit`: Exit the shell

### File System Operations

The file system supports basic operations through an inode-based structure with:
- Direct block pointers
- File modes and permissions
- Directory entries with name-to-inode mapping

### Project Structure

- `src/`
  - `main.rs`: Kernel entry point and initialization
  - `driver/`
    - `keyboard.rs`: PS/2 keyboard driver implementation
    - `shell.rs`: Interactive shell implementation
  - `fs/`
    - `structure.rs`: File system data structures
    - `utils.rs`: File system utility functions
  - `apps/`
    - `pofetch.rs`: System information display
  - `polib/`
    - `print.rs`: VGA text mode interface
    - `macros.rs`: Utility macros for printing

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the [Writing an OS in Rust](https://os.phil-opp.com/) series
- Uses the `bootloader` crate for bootstrapping
- Built with Rust's no-std ecosystem for bare metal development

---

Feel free to contribute or report issues to improve this project!