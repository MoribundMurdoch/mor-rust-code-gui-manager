use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WikiLookupKind {
    CommonDocs,
    ProjectWiki,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WikiLookup {
    pub label: String,
    pub url: String,
    pub kind: WikiLookupKind,
}

impl WikiLookup {
    pub fn common(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
            kind: WikiLookupKind::CommonDocs,
        }
    }

    pub fn project(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
            kind: WikiLookupKind::ProjectWiki,
        }
    }
}

pub fn open_url(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

pub fn common_file_wiki_lookup(path: &Path) -> Option<WikiLookup> {
    let file_name = path.file_name()?.to_str()?;

    match file_name {
        "Cargo.toml" => Some(WikiLookup::common(
            "Search Rust/File Wiki: Cargo.toml",
            "https://doc.rust-lang.org/cargo/reference/manifest.html",
        )),
        "Cargo.lock" => Some(WikiLookup::common(
            "Search Rust/File Wiki: Cargo.lock",
            "https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html",
        )),
        "build.rs" => Some(WikiLookup::common(
            "Search Rust/File Wiki: build.rs",
            "https://doc.rust-lang.org/cargo/reference/build-scripts.html",
        )),
        "lib.rs" => Some(WikiLookup::common(
            "Search Rust/File Wiki: lib.rs",
            "https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html",
        )),
        "main.rs" => Some(WikiLookup::common(
            "Search Rust/File Wiki: main.rs",
            "https://doc.rust-lang.org/book/ch01-02-hello-world.html",
        )),
        "mod.rs" => Some(WikiLookup::common(
            "Search Rust/File Wiki: mod.rs",
            "https://doc.rust-lang.org/reference/items/modules.html",
        )),
        ".gitignore" => Some(WikiLookup::common(
            "Search Rust/File Wiki: .gitignore",
            "https://git-scm.com/docs/gitignore",
        )),
        "README.md" | "README" => Some(WikiLookup::common(
            "Search Rust/File Wiki: README.md",
            "https://www.markdownguide.org/basic-syntax/",
        )),
        _ => extension_wiki_lookup(path),
    }
}

fn extension_wiki_lookup(path: &Path) -> Option<WikiLookup> {
    let extension = path.extension()?.to_str()?;

    match extension {
        "rs" => Some(WikiLookup::common(
            "Search Rust/File Wiki: Rust source file",
            "https://doc.rust-lang.org/reference/items/modules.html",
        )),
        "toml" => Some(WikiLookup::common(
            "Search Rust/File Wiki: TOML",
            "https://toml.io/en/",
        )),
        "md" => Some(WikiLookup::common(
            "Search Rust/File Wiki: Markdown",
            "https://www.markdownguide.org/basic-syntax/",
        )),
        "css" => Some(WikiLookup::common(
            "Search Rust/File Wiki: CSS",
            "https://developer.mozilla.org/en-US/docs/Web/CSS",
        )),
        "json" => Some(WikiLookup::common(
            "Search Rust/File Wiki: JSON",
            "https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting/JSON",
        )),
        _ => None,
    }
}

pub fn project_wiki_lookup(topic: &str) -> Option<WikiLookup> {
    let wiki_base = "https://github.com/MoribundMurdoch/mor-rust-code-gui-manager/wiki";

    let slug = match topic {
        "Inspect Mode" => "Inspect-Mode",
        "Automate Mode" => "Automate-Mode",
        "Safety Switch" => "Safety-Switch",
        "Node File Manager" => "Node-File-Manager",
        "Hierarchy" => "Hierarchy-View",
        "Dependency" => "Dependency-View",
        "Heatmap" => "Heatmap-View",
        "Types" => "Types-View",
        "Calls" => "Calls-View",
        "Register Module" => "Register-Module-Automation",
        "Preview" => "Preview",
        "Apply" => "Apply",
        "FileSystem Sync Log" => "FileSystem-Sync-Log",
        _ => return None,
    };

    Some(WikiLookup::project(
        format!("Search Project Wiki: {}", topic),
        format!("{}/{}", wiki_base, slug),
    ))
}

pub fn log_and_open_wiki(
    logger: &dioxus::prelude::Coroutine<String>,
    lookup: WikiLookup,
) {
    match open_url(&lookup.url) {
        Ok(_) => logger.send(format!("[WIKI] {}", lookup.label)),
        Err(e) => logger.send(format!(
            "[ERROR] Failed to open wiki lookup for {}: {}",
            lookup.label, e
        )),
    }
}

