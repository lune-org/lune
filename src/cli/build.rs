use std::{env, path::Path, process::ExitCode};

use anyhow::Result;
use console::style;
use mlua::Compiler as LuaCompiler;
use tokio::{fs, io::AsyncWriteExt as _};

use crate::executor::MetaChunk;

/**
    Compiles and embeds the bytecode of a given lua file to form a standalone
    binary, then writes it to an output file, with the required permissions.
*/
#[allow(clippy::similar_names)]
pub async fn build_standalone(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    source_code: impl AsRef<[u8]>,
) -> Result<ExitCode> {
    let input_path_displayed = input_path.as_ref().display();
    let output_path_displayed = output_path.as_ref().display();

    // First, we read the contents of the lune interpreter as our starting point
    println!(
        "Creating standalone binary using {}",
        style(input_path_displayed).green()
    );
    let mut patched_bin = fs::read(env::current_exe()?).await?;

    // Compile luau input into bytecode
    let bytecode = LuaCompiler::new()
        .set_optimization_level(2)
        .set_coverage_level(0)
        .set_debug_level(1)
        .compile(source_code);

    // Append the bytecode / metadata to the end
    let meta = MetaChunk { bytecode };
    patched_bin.extend_from_slice(&meta.to_bytes());

    // And finally write the patched binary to the output file
    println!(
        "Writing standalone binary to {}",
        style(output_path_displayed).blue()
    );
    write_executable_file_to(output_path, patched_bin).await?;

    Ok(ExitCode::SUCCESS)
}

async fn write_executable_file_to(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        options.mode(0o755); // Read & execute for all, write for owner
    }

    let mut file = options.open(path).await?;
    file.write_all(bytes.as_ref()).await?;

    Ok(())
}
