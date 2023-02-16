use std::{fmt::Write, path::PathBuf};

use anyhow::{Context, Result};

use tokio::fs::{create_dir_all, write};

use super::doc::DocumentationVisitor;
use super::GENERATED_COMMENT_TAG;

pub async fn generate_from_type_definitions(contents: &str) -> Result<()> {
    let visitor = DocumentationVisitor::from_definitions(contents)?;
    // Create the wiki dir at the repo root
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .unwrap();
    create_dir_all(&root.join("wiki"))
        .await
        .context("Failed to create wiki dir")?;
    for global in &visitor.globals {
        // Create the dir for this global
        let global_dir_path = root.join("wiki").join("api-reference").join(&global.0);
        create_dir_all(&global_dir_path)
            .await
            .context("Failed to create doc dir for global")?;
        // Create the markdown docs file for this global
        let mut contents = String::new();
        writeln!(contents, "<!-- {GENERATED_COMMENT_TAG} -->\n")?;
        writeln!(contents, "# **{}**\n", global.0)?;
        writeln!(contents, "{}\n", global.1.documentation)?;
        if !global.1.code_sample.is_empty() {
            writeln!(contents, "{}", global.1.code_sample)?;
        }
        let funcs = visitor
            .functions
            .iter()
            .filter(|f| f.1.global_name == global.0)
            .collect::<Vec<_>>();
        if !funcs.is_empty() {
            writeln!(contents, "## Functions\n")?;
            for func in funcs {
                writeln!(contents, "### {}\n", func.0)?;
                writeln!(contents, "{}\n", func.1.documentation)?;
                if !func.1.code_sample.is_empty() {
                    writeln!(contents, "{}", func.1.code_sample)?;
                }
            }
        }
        // Write the file in the dir, with the same
        // name as the dir to create an "index" page
        write(&global_dir_path.join(format!("{}.md", &global.0)), contents)
            .await
            .context("Failed to create doc file for global")?;
    }
    Ok(())
}
