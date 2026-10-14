# 🐕 BloatHound

> **A blazing-fast, safety-first developer artifact & cache purger with dependency analysis and file lifecycle tracking.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/Arthur-911/BloatHound)
[![GitHub stars](https://img.shields.io/github/stars/Arthur-911/BloatHound.svg?style=social)](https://github.com/Arthur-911/BloatHound)

Modern software development fills your SSD with gigabytes of disposable build artifacts: `node_modules` (300MB–1GB+ each), `.venv` virtual environments, Rust `target/` directories (up to 20GB), and Java/Gradle build outputs.

Written in **Rust**, **`BloatHound`** scans your workspace at native speed, calculates reclaimable space, analyzes project manifests (`package.json`, `Cargo.toml`), guards against sensitive custom files (`.env`, `.db`), and tracks file lifecycle timestamps (**When Made**, **Last Opened**, and **Last Modified**) before touching a single byte.

---

## ⚡ Quick Start

### Installation

Clone and install globally with Cargo:

```bash
git clone https://github.com/Arthur-911/BloatHound.git
cd BloatHound
cargo install --path .
```

Once installed, **`bloathound`** is available as a global command from any terminal on your computer!

---

## ✨ Key Features

- ⚡ **Native Rust Speed:** Built with Rust and `walkdir` to scan thousands of directories in milliseconds without freezing.
- 🔬 **Structural Dependency & Integrity Analysis:**
  - Verifies if an artifact folder is connected to an active manifest (`package.json`, `Cargo.toml`, `requirements.txt`).
  - Guards against deleting folders containing sensitive user files (`.env`, `.db`, `.sql`).
  - Checks if sibling code files actively import or reference the folder.
  - Clearly reports **`🟢 DOES NOT MATTER (SAFE TO DELETE)`** vs **`🔴 MATTERS (KEEP)`**.
- 🕒 **File Lifecycle Analytics:**
  - **When Made:** Creation timestamp (`st_birthtime` / `created`).
  - **Last Opened:** Access timestamp (`accessed`).
  - **Last Modified:** Modification write timestamp (`modified`).
- 📂 **Instant Explorer Jump:** Automatically displays full paths and lets you jump directly into any folder in Windows File Explorer by typing its row number.
- 🛡️ **Safety Guardrails:**
  - Built-in `--dry-run` simulation mode.
  - Automatically blocks system root paths (`C:\`, `C:\Windows`, `/usr`, `/etc`).
  - Resilient Windows deletion (clears read-only attributes on locked files).

---

## 📦 Supported Ecosystems

| Ecosystem | Detected Artifacts | Recreated With |
| :--- | :--- | :--- |
| **Node.js / JS / TS** | `node_modules`, `.next`, `.nuxt`, `.turbo` | `npm install` / `npm run build` |
| **Python** | `.venv`, `venv`, `__pycache__`, `.pytest_cache` | `pip install -r requirements.txt` |
| **Rust** | `target` | `cargo build` |
| **Java / Android** | `.gradle`, `build` | `gradle build` |
| **Bundlers & Web** | `dist`, `out` | `npm run build` |
| **.NET / C#** | `bin`, `obj` | `dotnet build` |

---

## 🛠️ Usage & Commands

### 1. Instant Scan & Jump to Folder
Scan the workspace and jump to any folder directly in Windows File Explorer:
```bash
bloathound
# or scan a specific directory
bloathound scan "C:\path\to\projects"
```
After scanning, simply type the row number (e.g., `1` or `5`) to immediately pop open that folder in File Explorer!

### 2. Structural Dependency & Necessity Analysis
Analyze whether folders are connected to project files and if deleting them matters:
```bash
bloathound analyze
# or
bloathound check "C:\path\to\projects"
```
Categorizes all folders into:
- 🟢 **DOES NOT MATTER (SAFE TO DELETE):** Recreatable cache connected to a manifest with 0 custom files.
- 🔴 **MATTERS (KEEP):** Contains sensitive user files (`.env`, `.db`) or is directly imported by project code.

### 3. Direct Jump to Any Folder
Open any project folder on your PC directly in File Explorer:
```bash
bloathound open "C:\path\to\project\target"
```

### 4. Detailed File Breakdown
View individual files inside the detected artifacts with their creation, access, and modified dates:
```bash
bloathound scan . --detailed
```

### 5. Deep-Inspect a Specific Artifact Folder
See a sorted list of the largest files inside an artifact folder:
```bash
bloathound inspect "path/to/project/node_modules" --limit 20
```

### 6. Safe Simulation (Dry-Run Mode)
Preview what would be deleted without touching any files on disk:
```bash
bloathound clean . --dry-run
```

---

## 🧪 Running Tests

Verify the codebase with the automated integration test suite:

```bash
cargo test
```

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Developed with ❤️ by [Arthur](https://github.com/Arthur-911).
