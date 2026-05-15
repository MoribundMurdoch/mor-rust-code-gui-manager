use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{parse_file, Item};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleVisibility {
    Private,
    Public,
    PublicCrate,
}

impl ModuleVisibility {
    pub fn declaration_prefix(&self) -> &'static str {
        match self {
            Self::Private => "mod",
            Self::Public => "pub mod",
            Self::PublicCrate => "pub(crate) mod",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Private => "mod",
            Self::Public => "pub mod",
            Self::PublicCrate => "pub(crate) mod",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterModuleRequest {
    pub target_file: PathBuf,
    pub module_name: String,
    pub visibility: ModuleVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterModulePreview {
    pub target_file: PathBuf,
    pub module_name: String,
    pub declaration: String,
    pub already_declared: bool,
    pub before: String,
    pub after: String,
    pub preview_diff: String,
}

impl RegisterModulePreview {
    pub fn summary(&self) -> String {
        if self.already_declared {
            format!(
                "[PREVIEW] Module '{}' is already declared in {}.",
                self.module_name,
                self.target_file.display()
            )
        } else {
            format!(
                "[PREVIEW] Would add '{}' to {}.",
                self.declaration.trim(),
                self.target_file.display()
            )
        }
    }
}

pub fn module_name_from_file_path(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Could not derive module name from selected file path")?;

    if stem == "mod" || stem == "lib" || stem == "main" {
        anyhow::bail!(
            "Selected file '{}' is a crate/module root, not a child module file",
            stem
        );
    }

    Ok(stem.to_string())
}

pub fn guess_target_file_for_module(selected_file: &Path) -> PathBuf {
    let parent = selected_file.parent().unwrap_or_else(|| Path::new("."));

    let sibling_mod = parent.join("mod.rs");
    if sibling_mod.exists() {
        return sibling_mod;
    }

    let src_lib = find_src_root_candidate(selected_file, "lib.rs");
    if src_lib.exists() {
        return src_lib;
    }

    let src_main = find_src_root_candidate(selected_file, "main.rs");
    if src_main.exists() {
        return src_main;
    }

    sibling_mod
}

fn find_src_root_candidate(selected_file: &Path, root_file: &str) -> PathBuf {
    for ancestor in selected_file.ancestors() {
        if ancestor.file_name().and_then(|s| s.to_str()) == Some("src") {
            return ancestor.join(root_file);
        }
    }

    selected_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(root_file)
}

pub fn build_register_module_preview(
    request: RegisterModuleRequest,
) -> Result<RegisterModulePreview> {
    let target_file = request.target_file;
    let module_name = request.module_name.trim().to_string();

    validate_module_name(&module_name)?;

    let declaration = format!(
        "{} {};\n",
        request.visibility.declaration_prefix(),
        module_name
    );

    let before = fs::read_to_string(&target_file)
        .with_context(|| format!("Could not read target file {}", target_file.display()))?;

    let already_declared = module_is_declared(&before, &module_name)?;

    let after = if already_declared {
        before.clone()
    } else {
        append_declaration(&before, &declaration)
    };

    let preview_diff = if already_declared {
        format!(
            "No changes.\n\n{} already contains a module declaration for '{}'.",
            target_file.display(),
            module_name
        )
    } else {
        format!(
            "--- {}\n+++ {}\n@@\n+ {}",
            target_file.display(),
            target_file.display(),
            declaration
        )
    };

    Ok(RegisterModulePreview {
        target_file,
        module_name,
        declaration,
        already_declared,
        before,
        after,
        preview_diff,
    })
}

pub fn apply_register_module_preview(preview: &RegisterModulePreview) -> Result<String> {
    if preview.already_declared {
        return Ok(format!(
            "[VERIFIED] Module '{}' is already declared in {}.",
            preview.module_name,
            preview.target_file.display()
        ));
    }

    fs::write(&preview.target_file, &preview.after)
        .with_context(|| format!("Could not write {}", preview.target_file.display()))?;

    Ok(format!(
        "[UPDATE] Added '{}' to {}.",
        preview.declaration.trim(),
        preview.target_file.display()
    ))
}

fn validate_module_name(module_name: &str) -> Result<()> {
    if module_name.is_empty() {
        anyhow::bail!("Module name cannot be empty");
    }

    let mut chars = module_name.chars();

    let first = chars.next().unwrap();
    if !(first == '_' || first.is_ascii_alphabetic()) {
        anyhow::bail!("Module name must start with a letter or underscore");
    }

    for c in chars {
        if !(c == '_' || c.is_ascii_alphanumeric()) {
            anyhow::bail!(
                "Module name '{}' contains invalid character '{}'",
                module_name,
                c
            );
        }
    }

    Ok(())
}

fn module_is_declared(source: &str, module_name: &str) -> Result<bool> {
    let ast = parse_file(source)?;

    for item in ast.items {
        if let Item::Mod(item_mod) = item {
            if item_mod.ident == module_name {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn append_declaration(source: &str, declaration: &str) -> String {
    let mut output = source.trim_end().to_string();

    if !output.is_empty() {
        output.push('\n');
    }

    output.push_str(declaration);
    output
}
