use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ArtifactPattern {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub recreate_cmd: &'static str,
}

pub fn get_default_patterns() -> HashMap<&'static str, ArtifactPattern> {
    let mut patterns = HashMap::new();

    // Node.js & Web
    patterns.insert(
        "node_modules",
        ArtifactPattern {
            name: "node_modules",
            category: "Node.js / JS",
            description: "Installed npm, yarn, or pnpm packages",
            recreate_cmd: "npm install",
        },
    );
    patterns.insert(
        ".next",
        ArtifactPattern {
            name: ".next",
            category: "Next.js",
            description: "Next.js build cache and outputs",
            recreate_cmd: "npm run build",
        },
    );
    patterns.insert(
        ".nuxt",
        ArtifactPattern {
            name: ".nuxt",
            category: "Nuxt.js",
            description: "Nuxt build output",
            recreate_cmd: "npm run build",
        },
    );
    patterns.insert(
        ".turbo",
        ArtifactPattern {
            name: ".turbo",
            category: "Turborepo",
            description: "Turborepo build cache",
            recreate_cmd: "npx turbo build",
        },
    );

    // Python
    patterns.insert(
        ".venv",
        ArtifactPattern {
            name: ".venv",
            category: "Python",
            description: "Python virtual environment",
            recreate_cmd: "pip install -r requirements.txt",
        },
    );
    patterns.insert(
        "venv",
        ArtifactPattern {
            name: "venv",
            category: "Python",
            description: "Python virtual environment",
            recreate_cmd: "pip install -r requirements.txt",
        },
    );
    patterns.insert(
        "__pycache__",
        ArtifactPattern {
            name: "__pycache__",
            category: "Python",
            description: "Compiled Python bytecode files",
            recreate_cmd: "auto-generated on python run",
        },
    );
    patterns.insert(
        ".pytest_cache",
        ArtifactPattern {
            name: ".pytest_cache",
            category: "Python",
            description: "Pytest execution cache",
            recreate_cmd: "pytest",
        },
    );

    // Rust
    patterns.insert(
        "target",
        ArtifactPattern {
            name: "target",
            category: "Rust",
            description: "Cargo build artifacts, dependencies, and binaries",
            recreate_cmd: "cargo build",
        },
    );

    // Java / Build
    patterns.insert(
        ".gradle",
        ArtifactPattern {
            name: ".gradle",
            category: "Java / Gradle",
            description: "Gradle cache directory",
            recreate_cmd: "gradle build",
        },
    );
    patterns.insert(
        "build",
        ArtifactPattern {
            name: "build",
            category: "Java / Build",
            description: "Standard build output directory",
            recreate_cmd: "build script",
        },
    );

    // Bundlers
    patterns.insert(
        "dist",
        ArtifactPattern {
            name: "dist",
            category: "Bundler",
            description: "Compiled distribution files",
            recreate_cmd: "npm run build",
        },
    );
    patterns.insert(
        "out",
        ArtifactPattern {
            name: "out",
            category: "Bundler",
            description: "Build output directory",
            recreate_cmd: "npm run build",
        },
    );

    // .NET
    patterns.insert(
        "bin",
        ArtifactPattern {
            name: "bin",
            category: ".NET",
            description: ".NET compiled binaries",
            recreate_cmd: "dotnet build",
        },
    );
    patterns.insert(
        "obj",
        ArtifactPattern {
            name: "obj",
            category: ".NET",
            description: ".NET intermediate object files",
            recreate_cmd: "dotnet build",
        },
    );

    patterns
}

pub const PROTECTED_SYSTEM_DIRS: &[&str] = &[
    "windows",
    "program files",
    "program files (x86)",
    "programdata",
    "$recycle.bin",
    "system volume information",
    "appdata",
    "proc",
    "sys",
    "dev",
    "etc",
    "usr",
    "var",
    "boot",
];
