use std::{
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

use anyhow::Result;
use mlua::Compiler as LuaCompiler;

pub async fn build_standalone<T: AsRef<Path> + Into<PathBuf>>(
    output_path: T,
    code: impl AsRef<[u8]>,
) -> Result<ExitCode> {
    // First, we read the contents of the lune interpreter as our starting point
    let mut patched_bin = fs::read(env::current_exe()?).await?;
    let base_bin_offset = u64::try_from(patched_bin.len())?;

    // The signature which separates indicates the presence of bytecode to execute
    // If a binary contains this signature, that must mean it is a standalone binary
    let signature: Vec<u8> = vec![0x4f, 0x3e, 0xf8, 0x41, 0xc3, 0x3a, 0x52, 0x16];

    // Compile luau input into bytecode
    let bytecode = LuaCompiler::new()
        .set_optimization_level(2)
        .set_coverage_level(0)
        .set_debug_level(0)
        .compile(code);

    patched_bin.append(&mut bytecode.clone());

    let mut meta = base_bin_offset.to_ne_bytes().to_vec();

    // Include metadata in the META chunk, each field is 8 bytes
    meta.append(&mut (bytecode.len() as u64).to_ne_bytes().to_vec()); // Size of bytecode, used to calculate end offset at runtime
    meta.append(&mut 1_u64.to_ne_bytes().to_vec()); // Number of files, padded with null bytes

    patched_bin.append(&mut meta);

    // Append the signature to the base binary
    patched_bin.append(&mut signature.clone());

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

    Ok(ExitCode::SUCCESS)
}
