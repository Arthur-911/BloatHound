# 🧹 dev-cleaner (Rust)

> **A blazing-fast, safety-first developer artifact & cache purger with file lifecycle analytics.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

Modern software development fills your drive with gigabytes of recreatable build artifacts: `node_modules` (300MB–1GB+ each), `.venv` virtual environments, Rust `target/` directories (up to 20GB), and Java/Gradle build outputs.

Written in **Rust**, `dev-cleaner` scans your workspace at native speed, calculates reclaimable space, and gives you total transparency by tracking **when files were made (created)**, **when they were last opened (accessed)**, and **when they were last modified**—so you can safely purge abandoned projects without fear.

---

## ⚡ Instant Run (Zero Configuration)

To scan your workspace immediately without any arguments:

```bash
cargo run
```

Or on Windows, simply double-click **`scan.bat`**.

---

## ✨ Features

- ⚡ **Native Rust Speed:** Built with Rust and `walkdir` to scan thousands of directories in milliseconds without freezing.
- 🔍 **Recursive Pruning:** Discovers disposable artifacts instantly without descending inside 50,000 files in deep `node_modules`.
- 🕒 **File Lifecycle Analytics:**
  - **When Made:** Creation timestamp (`st_birthtime` / `created`).
  - **Last Opened:** Access timestamp (`accessed`).
  - **Last Modified:** Modification write timestamp (`modified`).
- 🔬 **Deep Inspection Mode:** Inspect individual files within any artifact folder to see largest space hogs and their exact timestamps.
- 🛡️ **Safety Guardrails:**
  - Built-in `--dry-run` simulation mode.
  - Automatically blocks system root paths (`C:\`, `C:\Windows`, `/usr`, `/etc`).
  - Resilient Windows deletion (clears read-only attributes on locked files).
- 🎨 **Terminal Dashboard:** Built with UTF-8 border tables, colored status panels, and humanized sizes (`450.2 MB`, `2.4 GB`).

---

## 📦 Supported Ecosystems

| Ecosystem | Detected Artifacts |
| :--- | :--- |
| **Node.js / JS / TS** | `node_modules`, `.next`, `.nuxt`, `.turbo` |
| **Python** | `.venv`, `venv`, `__pycache__`, `.pytest_cache` |
| **Rust** | `target` |
| **Java / Android** | `.gradle`, `build` |
| **Bundlers & Web** | `dist`, `out` |
| **.NET / C#** | `bin`, `obj` |

---

## 🛠️ Usage & Commands

### 1. Instant Scan & Jump to Folder
Scan the workspace and jump to any folder directly in Windows File Explorer:
```bash
dev-cleaner
# or
dev-cleaner scan "C:\path\to\projects"
```
After scanning, simply type the row number (e.g., `1` or `5`) to immediately pop open that folder in File Explorer!

### 2. Intelligent Necessity Audit
Scan and evaluate whether folders are needed or safe to delete based on activity recency:
```bash
dev-cleaner audit
# or
dev-cleaner check "C:\path\to\projects"
```
Categorizes all folders into:
- 🔴 **NOT NEEDED (Safe to Delete):** Untouched for > 30 days. Tells you how to recreate it anytime.
- 🟡 **REVIEW:** Inactive for 14–30 days.
- 🟢 **KEEP / ACTIVE:** Active within 14 days. Avoids re-downloading packages for active projects.

### 3. Direct Open Command
Open any project folder on your PC directly:
```bash
dev-cleaner open "C:\path\to\project\target"
```

### 4. Detailed File Breakdown
View individual files inside the detected artifacts with their creation, access, and modified dates:
```bash
dev-cleaner scan . --detailed
```

### 5. Deep-Inspect a Specific Artifact Folder
See a sorted list of the largest files inside an artifact folder:
```bash
dev-cleaner inspect "path/to/project/node_modules" --limit 20
```

### 5. Safe Simulation (Dry-Run Mode)
Preview what would be deleted without touching any files on disk:
```bash
dev-cleaner clean . --dry-run
```

### 5. Live Cleaning
Prompts for confirmation before safely purging selected artifact directories:
```bash
cargo run -- clean .
```

---

## 🧪 Running Tests

Verify the codebase with the automated integration test suite:

```bash
cargo test
```

---

## 📦 Standalone Binary Compilation

To compile a single, zero-dependency `.exe` binary that you can copy to your desktop or PATH:

```bash
cargo build --release
```
The compiled executable is ready at:  
`target/release/dev-cleaner.exe`

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
