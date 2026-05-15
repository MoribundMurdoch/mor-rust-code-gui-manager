use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::{Item, parse_file};

/// Safely injects a `pub mod <name>;` declaration into the appropriate mod.rs file
pub fn inject_pub_mod(new_file_path: &Path) -> Result<String> {
    // Extract the exact name of the new file (e.g., "controls" from "controls.rs")
    let file_stem = new_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid file nomenclature")?;

    // We strictly ignore root architectural files
    if file_stem == "mod" || file_stem == "main" || file_stem == "lib" {
        return Ok(format!(
            "[SKIP] Bypassing root structural file: {}.rs",
            file_stem
        ));
    }

    let parent_dir = new_file_path
        .parent()
        .context("File possesses no parent directory")?;
    let mod_rs_path = parent_dir.join("mod.rs");

    // Case 1: No mod.rs exists in this folder. We generate a fresh one.
    if !mod_rs_path.exists() {
        fs::write(&mod_rs_path, format!("pub mod {};\n", file_stem))?;
        return Ok(format!(
            "[ARCHIVE] Generated new mod.rs and injected: pub mod {};",
            file_stem
        ));
    }

    // Case 2: mod.rs already exists. We parse the AST to ensure we don't duplicate the entry.
    let content = fs::read_to_string(&mod_rs_path)?;
    let ast = parse_file(&content)?;

    let mut is_declared = false;
    for item in ast.items {
        if let Item::Mod(item_mod) = item {
            // Check if the identifier exactly matches our file name
            if item_mod.ident == file_stem {
                is_declared = true;
                break;
            }
        }
    }

    // If the AST confirms it is not there, we carefully append it to the bottom.
    if !is_declared {
        let mut new_content = content.trim_end().to_string();
        if !new_content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(&format!("pub mod {};\n", file_stem));
        fs::write(&mod_rs_path, new_content)?;
        return Ok(format!(
            "[UPDATE] Appended 'pub mod {};' to existing hierarchy.",
            file_stem
        ));
    }

    Ok(format!(
        "[VERIFIED] Module '{}' is already cataloged.",
        file_stem
    ))
}
