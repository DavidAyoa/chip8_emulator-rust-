# Chip-8 Emulator (Rust)

A small Chip-8 emulator written in Rust. This project was created while learning Rust in October 2024 as a hands-on exercise to better understand systems programming, emulation basics, and Rust's ownership model.

## About

Chip-8 is a simple interpreted programming language used on some vintage computers and calculators. This emulator implements the Chip-8 instruction set and provides a basic environment to load and run Chip-8 ROMs.

## Features

- Implements core Chip-8 instructions
- Pixel-based display rendering
- Keyboard input handling for Chip-8 key layout
- Load and run Chip-8 ROM files

(Implementation details and exact feature set may vary — see the source code for the current state.)

## Status

Learning project / experimental. Functionality implemented while learning Rust; expect rough edges and room for improvement. Created October 2025.

## Requirements

- Rust toolchain (stable) — see rustup.rs to install
- Any additional native libraries required by dependencies (if the project uses SDL2 or other native libs, install them via your platform package manager)

## Build & Run

From the repository root:

1. Build in debug:

   cargo build

2. Run with a ROM file (if the emulator accepts a ROM path as an argument):

   cargo run --release -- path/to/rom.ch8

If the repository uses a different runner or has additional flags, consult the source (main.rs or Cargo.toml) for exact usage.

## Typical Controls

Chip-8 programs expect a 16-key hex keyboard. Typical PC mappings used by many emulators:

```
Original Chip-8 keypad       Keyboard
1 2 3 C                      1 2 3 4
4 5 6 D       =>             Q W E R
7 8 9 E                      A S D F
A 0 B F                      Z X C V
```

Adjust mappings if the emulator uses a different layout.

## Contributing

This is primarily a personal learning project. Contributions welcome — open an issue or PR if you have improvements, bug fixes, or suggestions.

## License

Include a license if you want others to reuse the code. For a personal project, consider MIT, Apache-2.0, or another license. Add a LICENSE file to the repository.

## Notes

- This README is a starting point. If you want, tell me any specifics (dependencies like sdl2, exact run arguments, supported ROMs, or a preferred license) and I will update the README accordingly and push the change.

---

Made by jadwd — learning Rust (Oct 2025).
