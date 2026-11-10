# <div align="center">

  <img src="icon.png" width="120" alt="Blaze Logo" style="border-radius: 50%;">
  <h1>Blaze Programming Language</h1>

  <p><strong>A high-performance, statically typed systems programming language engineered for bare-metal execution speed and pristine developer ergonomics.</strong></p>

  <p>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-orange.svg" alt="License: MIT"></a>
    <a href="#"><img src="https://img.shields.io/badge/Version-0.0.1-brightgreen.svg" alt="Version"></a>
    <a href="#"><img src="https://img.shields.io/badge/Rust-Edition%202021-orange.svg" alt="Rust Edition"></a>
    <a href="#"><img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-blue.svg" alt="Platform"></a>
  </p>
</div>

---

## 🚀 Overview

Blaze is a modern, statically typed systems programming language designed to bridge the gap between high-level developer experience and raw native execution speed. It combines modern functional and procedural control structures with a robust, multi-stage compiler pipeline that targets optimized native machine code via C and clang/gcc `-O3` optimization flags.

---

## ✨ Key Features

- **Static Type Safety:** Catch type errors at compile time with a rigorous semantic analyzer supporting integers (`int`, `int8`, `int32`, `int64`, `uint`, `uint8`, `uint32`, `uint64`), floating-point numbers (`float32`, `float64`), booleans, strings, characters, and typed arrays.
- **Blazing Native Performance:** Bypasses virtual machines, garbage collectors, and dynamic interpreter overhead entirely. In rigorous stress tests (such as computing over 740 million Collatz hailstone iterations), natively compiled Blaze binaries execute up to **100x faster** than standard Python implementations.
- **Expressive & Modern Syntax:** Features clean, highly readable control flows including `if / elif / else` branching ladders, pattern matching via `match / case / default`, `while` loops, and `for / range` iterators.
- **Dual Execution Modes:** Run scripts instantly for rapid prototyping using the built-in tree-walking interpreter, or transpile and compile them into standalone native binaries for production performance.
- **First-Class Developer Tooling:** Complete support for local Visual Studio Code syntax highlighting, intelligent auto-indentation, smart bracket pairing, auto-formatting on save, custom flame file icons, and native file icon themes.

---

## ⚙️ Compiler Architecture

Blaze is built entirely from scratch in Rust. Its modular architecture is divided into five core sequential phases:

```
Source Code (.blz)
  --> Lexer (src/lexer.rs)
  --> Parser (src/parser.rs)
  --> Semantic Analyzer (src/semantic.rs)
  --> Native Codegen (src/codegen.rs)
  --> Executable Binary (via cc -O3)
```

1. **Lexer (`src/lexer.rs`):** Ingests raw `.blz` source text as a character stream, stripping out whitespace and comments while tokenizing keywords, type declarations, literals, identifiers, and operators.
2. **Parser (`src/parser.rs`):** Implements a recursive descent parser that transforms flat token streams into a strict Abstract Syntax Tree (AST), ensuring precise operator precedence across logical (`and`, `or`, `not`), relational, and arithmetic expressions.
3. **Semantic Analyzer (`src/semantic.rs`):** Walks the AST prior to code generation to enforce strict lexical scoping, constant mutability (`let` vs `const`), function return types, and binary expression type checking.
4. **Native Code Generator (`src/codegen.rs`):** Transpiles validated AST nodes into clean, standards-compliant C code, automatically invoking the system C compiler (`cc`) with aggressive `-O3` optimization flags to build standalone executables.
5. **Interpreter (`src/interpreter.rs`):** Provides a robust tree-walking execution environment equipped with environment scope stacks, built-in I/O (`print`, `input`), and runtime type safety for immediate scripting support.

---

## 📁 Project Directory Structure

```text
blaze/
├── Cargo.toml
├── LICENSE
├── src/
│   ├── main.rs          # CLI entry point (interprets or compiles)
│   ├── ast.rs           # AST node definitions, Tokens, and Types
│   ├── lexer.rs         # Lexical analyzer
│   ├── parser.rs        # Recursive descent parser
│   ├── semantic.rs      # Static type checker and scope analyzer
│   ├── codegen.rs       # C-backend native code generator
│   └── interpreter.rs   # Tree-walking runtime interpreter
└── blaze-vscode/        # VS Code extension folder
    ├── package.json     # Extension manifest and contributions
    ├── extension.js     # Auto-formatter and extension logic
    ├── icon.png         # Marketplace branding logo
    ├── blaze-icon.svg   # Custom file icon for .blz files
    ├── blaze-icon-theme.json # File icon theme mapping
    ├── language-configuration.json # Indentation & bracket rules
    └── syntaxes/
        └── blaze.tmLanguage.json # TextMate syntax highlighting grammar
```

---

## 📦 Detailed Installation & Setup Instructions

### Prerequisites

Before installing Blaze, ensure your system has the following tools installed:

- **Rust Toolchain (Cargo & rustc):** Required to build the Blaze compiler. Install via [rustup.rs](https://rustup.rs/).
- **System C Compiler:** Clang or GCC (`cc`) must be installed in your system `PATH` to handle native code compilation (`-O3`).

### Step 1: Clone and Build the Blaze Compiler

Clone your repository, navigate to the project directory, and compile the compiler binary in release mode:

```bash
git clone https://github.com/thevoidop/blaze
cd blaze
cargo build --release
```

### Step 2: Install Blaze Globally via Cargo

To make the `blaze` command accessible globally from any terminal window:

```bash
cargo install --path .
```

Verify the installation:

```bash
blaze
```

---

## 💻 VS Code Extension Installation (`blaze-vscode`)

The `blaze-vscode` directory resides directly inside your project repository alongside `src/`. To integrate syntax highlighting, auto-formatting, and custom flame icons into Visual Studio Code:

### 1. Link the Extension Locally via Symlink

On macOS and Linux, create a symbolic link from your project's `blaze-vscode` folder directly into VS Code's extensions directory so that any edits you make update live:

```bash
ln -s "$(pwd)/blaze-vscode" ~/.vscode/extensions/local.blaze-vscode
```

### 2. Reload VS Code

1. Open Visual Studio Code.
2. Open the Command Palette (`Cmd + Shift + P` on macOS or `Ctrl + Shift + P` on Windows/Linux).
3. Type and run: **Developer: Reload Window**.

### 3. Activate File Icons

1. Open the Command Palette again (`Cmd + Shift + P`).
2. Type and select: **Preferences: File Icon Theme**.
3. Choose **Blaze File Icons** to enable the custom flame logo for all `.blz` files.

---

## 🛠️ Usage Guide

### 1. Run Scripts via the Interpreter

For rapid prototyping and script execution without compiling binaries:

```bash
blaze example.blz
```

### 2. Compile to Native Binaries

To transpile your Blaze code to C, compile it with `-O3` optimizations, and generate a standalone native executable:

```bash
blaze build bench.blz
./bench
```

### 3. Automated Formatting

Open any `.blz` file in VS Code and press **`Shift + Alt + F`** (or enable **Format on Save** in your editor settings). The built-in extension formatter will automatically clean up indentation, match braces, and maintain clean trailing newlines.

---

## 📝 Code Example (`example.blz`)

```javascript
const int32 MAX_ATTEMPTS = 3;
const float64 TAX_RATE = 0.18;

// Function with typed parameters, default argument, and return type
fn add(a: int, b: int = 10) -> int {
    return a + b;
}

// Function using if / elif / else and logical operators
fn grade(score: int) -> string {
    if score >= 90 and score <= 100 {
        return "A";
    } elif score >= 75 {
        return "B";
    } elif score >= 50 or score == 49 {
        return "C";
    } else {
        return "F";
    }
}

// Function using for/range, continue, break, arrays, and .len()
fn sum_valid(values: int) -> int {
    let numbers = [5, -3, 12, 0, 40, 99, 7];
    let total = 0;

    for i in range(numbers.len()) {
        let n = numbers[i];

        if n < 0 {
            continue;            // skip negatives
        }
        if n == 99 {
            break;               // stop at 99
        }
        total = total + n;
    }
    return total;
}

print("=== Blaze Full Example ===");
print(grade(82));
print(sum_valid());
print("=== Done ===");
```

---

## 📊 Performance Benchmark

In rigorous head-to-head benchmarking executing an unoptimizable Collatz hailstone sequence computing over **740 million inner-loop arithmetic and branching operations**, Blaze outperforms Python by nearly **100x**:

| Metric                   | Blaze (Native Binary via `-O3`) | Python 3.12 |
| ------------------------ | ------------------------------- | ----------- |
| **Total Steps Computed** | 740,438,133                     | 740,438,133 |
| **User CPU Time**        | **0.55s**                       | 53.58s      |
| **Total Execution Time** | **0.905s**                      | 53.773s     |

---

## 🤝 Contributing

We welcome contributions from the community to make Blaze even faster, safer, and more expressive! Whether it's adding new features, optimizing the compiler pipeline, improving the VS Code extension, or fixing bugs, your help is appreciated.

### How to Contribute

1. **Fork the Repository** on GitHub (`thevoidop/blaze`).
2. **Create a Feature Branch** (`git checkout -b feature/amazing-feature`).
3. **Commit Your Changes** (`git commit -m 'Add amazing feature'`).
4. **Push to the Branch** (`git push origin feature/amazing-feature`).
5. **Open a Pull Request** describing your changes and test results.

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for details.
