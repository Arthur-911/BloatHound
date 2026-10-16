# BloatHound

A command-line tool written in Rust to audit, inspect, and safely remove disposable build artifacts and caches across developer workspaces.

Unlike standard disk cleaning scripts that delete files indiscriminately, BloatHound inspects project structure, manifest files (`package.json`, `Cargo.toml`, `requirements.txt`), and file modification timestamps to distinguish between harmless build artifacts and active user data.

---

## Installation

### Pre-built Binaries

Download pre-compiled binaries for Windows, Linux, or macOS from the [Releases](https://github.com/Arthur-911/BloatHound/releases) page.

Extract the archive and place `bloathound` (or `bloathound.exe`) in your system's `PATH`.

### From Source

Ensure you have Rust and Cargo installed:

```bash
git clone https://github.com/Arthur-911/BloatHound.git
cd BloatHound
cargo install --path .
```

The `bloathound` binary will be installed to your Cargo bin directory and available globally in your path.

---

## Features

- **Fast Directory Traversal:** Uses multi-threaded directory walking with recursive pruning, skipping the internal contents of detected artifact folders to scan entire drives in seconds.
- **Structural Dependency Analysis:** Checks whether an artifact directory is connected to a recognized build manifest, or if it is an orphaned directory left behind from deleted projects.
- **Safety Guards for User Files:** Inspects folders for untracked user files (`.env`, configuration files, databases) and aborts deletion if custom files are detected.
- **Source Code Reference Checking:** Scans project source files to verify whether code directly imports from output folders before classifying them as disposable.
- **Lifecycle Analytics:** Extracts true file creation, access, and modification timestamps directly from the filesystem.
- **System Protection:** Hardcoded blocklists prevent scanning or modifying operating system roots, system directories, or `.git` repositories.
- **Intelligent Size Sorting & Paging:** Discovered artifacts are automatically sorted by size descending so the largest space consumers appear at `# 1`. Supports `--limit <N>` and `--all` to prevent terminal buffer overflow.
- **Execution Diagnostics & Timers:** Measures and displays exact run durations (e.g. `Scan time = 52ms`) across all operations.
- **Machine-Readable JSON Output:** Full JSON serialization via `--json` for integration with CI/CD pipelines, shell scripts, and monitoring tools.
- **Compact & High Performance:** Sub-megabyte binary with zero-allocation pattern dispatch.

---

## Supported Ecosystems

| Ecosystem | Detected Targets | Regeneration Command |
| :--- | :--- | :--- |
| Node.js / Web | `node_modules`, `.next`, `.nuxt`, `.turbo` | `npm install` / `npm run build` |
| Python | `.venv`, `venv`, `__pycache__`, `.pytest_cache` | `pip install -r requirements.txt` |
| Rust | `target` | `cargo build` |
| Java / Gradle | `.gradle`, `build` | `gradle build` |
| Bundlers | `dist`, `out` | `npm run build` |
| .NET / C# | `bin`, `obj` | `dotnet build` |

---

## Usage

### 1. Instant Scan

Scan the current directory:

```bash
bloathound
```

Scan a specific workspace or drive:

```bash
bloathound scan "C:\path\to\projects"
```

After scanning, the tool displays an overview table with sizes and timestamps, followed by an optional prompt to open any detected folder directly in your system file manager.

### 2. Dependency and Integrity Analysis

Analyze whether detected folders are linked to project manifests and if deleting them would impact the project:

```bash
bloathound analyze
```

Scan and analyze an external folder:

```bash
bloathound analyze "C:\path\to\projects"
```

The analysis checks:
- Manifest Link: Whether a valid manifest exists to regenerate the folder.
- Custom Files: Whether sensitive non-compiler files (`.env`, `.db`) are present.
- Source Imports: Whether sibling source code hard-references the directory.
- Verdict: Classifies each target as `[SAFE TO DELETE]` or `[MATTERS - KEEP]`.

### 3. Direct Folder Navigation

Open a project folder directly in File Explorer:

```bash
bloathound open "C:\path\to\project\target"
```

### 4. File-Level Timestamp Inspection

View the individual files inside detected directories with their creation and modification dates:

```bash
bloathound scan . --detailed
```

Inspect a specific directory's largest files:

```bash
bloathound inspect "path/to/node_modules" --limit 20
```

### 5. Safe Cleanup (Dry-Run and Live)

Preview what would be deleted without touching files:

```bash
bloathound clean . --dry-run
```

Execute an interactive cleanup with confirmation prompts:

```bash
bloathound clean .
```

### 6. Machine-Readable JSON Output

Export clean, parseable JSON for shell scripts, cron jobs, and automation:

```bash
bloathound scan . --json
```

Filter outputs with `jq`:

```bash
# List all detected paths and their byte sizes
bloathound scan . --json | jq '.artifacts[] | {path: .path, bytes: .size_bytes}'

# Check project analysis in JSON
bloathound analyze . --json | jq '.items[] | {path: .artifact.path, safe: (.analysis.is_needed | not)}'
```

Include detailed file breakdown in JSON output:

```bash
bloathound scan . --json --detailed --max-files 10
```

---

## Testing

Run the integration test suite:

```bash
cargo test
```

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

Maintained by [Arthur](https://github.com/Arthur-911).
