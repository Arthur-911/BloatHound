use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ArtifactPattern {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub recreate_cmd: &'static str,
}

pub const DEFAULT_PATTERNS: &[ArtifactPattern] = &[
    ArtifactPattern { name: "node_modules", category: "Node.js / JS", description: "Installed npm, yarn, or pnpm packages", recreate_cmd: "npm install" },
    ArtifactPattern { name: ".next", category: "Next.js", description: "Next.js build cache and outputs", recreate_cmd: "npm run build" },
    ArtifactPattern { name: ".nuxt", category: "Nuxt.js", description: "Nuxt build output", recreate_cmd: "npm run build" },
    ArtifactPattern { name: ".turbo", category: "Turborepo", description: "Turborepo build cache", recreate_cmd: "npx turbo build" },
    ArtifactPattern { name: ".venv", category: "Python", description: "Python virtual environment", recreate_cmd: "pip install -r requirements.txt" },
    ArtifactPattern { name: "venv", category: "Python", description: "Python virtual environment", recreate_cmd: "pip install -r requirements.txt" },
    ArtifactPattern { name: "__pycache__", category: "Python", description: "Compiled Python bytecode files", recreate_cmd: "auto-generated on python run" },
    ArtifactPattern { name: ".pytest_cache", category: "Python", description: "Pytest execution cache", recreate_cmd: "pytest" },
    ArtifactPattern { name: "target", category: "Rust", description: "Cargo build artifacts, dependencies, and binaries", recreate_cmd: "cargo build" },
    ArtifactPattern { name: ".gradle", category: "Java / Gradle", description: "Gradle cache directory", recreate_cmd: "gradle build" },
    ArtifactPattern { name: "build", category: "Java / Build", description: "Standard build output directory", recreate_cmd: "build script" },
    ArtifactPattern { name: "dist", category: "Bundler", description: "Compiled distribution files", recreate_cmd: "npm run build" },
    ArtifactPattern { name: "out", category: "Bundler", description: "Build output directory", recreate_cmd: "npm run build" },
    ArtifactPattern { name: "bin", category: ".NET", description: ".NET compiled binaries", recreate_cmd: "dotnet build" },
    ArtifactPattern { name: "obj", category: ".NET", description: ".NET intermediate object files", recreate_cmd: "dotnet build" },
];

#[inline]
pub fn find_pattern(name: &str) -> Option<&'static ArtifactPattern> {
    DEFAULT_PATTERNS.iter().find(|p| p.name == name)
}

pub fn get_default_patterns() -> HashMap<&'static str, ArtifactPattern> {
    DEFAULT_PATTERNS.iter().map(|p| (p.name, p.clone())).collect()
}

pub const PROTECTED_SYSTEM_DIRS: &[&str] = &[
    "windows",
    "program files",
    "program files (x86)",
    "programdata",
    "$recycle.bin",
    "system volume information",
    "proc",
    "sys",
    "dev",
    "etc",
    "boot",
    "sbin",
];

