use console::Style;
use std::{env, path::Path, process::ExitCode};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

use anyhow::Result;
use mlua::Compiler as LuaCompiler;

// The signature which separates indicates the presence of bytecode to execute
// If a binary contains this magic signature as the last 8 bytes, that must mean
//  it is a standalone binary
pub const MAGIC: &[u8; 8] = b"cr3sc3nt";

/**
    Compiles and embeds the bytecode of a requested lua file to form a standalone binary,
    then writes it to an output file, with the required permissions.
*/
#[allow(clippy::similar_names)]
pub async fn build_standalone<T: AsRef<Path>>(
    script_path: String,
    output_path: T,
    code: impl AsRef<[u8]>,
) -> Result<ExitCode> {
    let log_output_path = output_path.as_ref().display();

    let prefix_style = Style::new().green().bold();
    let compile_prefix = prefix_style.apply_to("Compile");
    let bytecode_prefix = prefix_style.apply_to("Bytecode");
    let write_prefix = prefix_style.apply_to("Write");
    let compiled_prefix = prefix_style.apply_to("Compiled");

    println!("{compile_prefix} {script_path}");

    // First, we read the contents of the lune interpreter as our starting point
    let mut patched_bin = fs::read(env::current_exe()?).await?;
    let base_bin_offset = u64::try_from(patched_bin.len())?;

    // Compile luau input into bytecode
    let bytecode = LuaCompiler::new()
        .set_optimization_level(2)
        .set_coverage_level(0)
        .set_debug_level(0)
        .compile(code);

    println!("  {bytecode_prefix} {script_path}");

    patched_bin.extend(&bytecode);

    let mut meta = base_bin_offset.to_ne_bytes().to_vec(); // Start with the base bytecode offset

    // Include metadata in the META chunk, each field is 8 bytes
    meta.extend((bytecode.len() as u64).to_ne_bytes()); // Size of bytecode, used to calculate end offset at runtime
    meta.extend(1_u64.to_ne_bytes()); // Number of files, padded with null bytes - for future use

    patched_bin.extend(meta);

    // Append the magic signature to the base binary
    patched_bin.extend(MAGIC);

    // Write the compiled binary to file
    #[cfg(target_family = "unix")]
    OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o770) // read, write and execute permissions for user and group
        .open(&output_path)
        .await?
        .write_all(&patched_bin)
        .await?;

    #[cfg(target_family = "windows")]
    fs::write(&output_path, &patched_bin).await?;

    println!("  {write_prefix} {log_output_path}");

    println!("{compiled_prefix} {log_output_path}");

    Ok(ExitCode::SUCCESS)
}
