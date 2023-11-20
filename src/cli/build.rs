use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::fs;

use anyhow::Result;
use mlua::Compiler as LuaCompiler;

pub async fn build_standalone<T: AsRef<Path> + Into<PathBuf>>(
    output_path: T,
    code: impl AsRef<[u8]>,
) -> Result<()> {
    // First, we read the contents of the lune interpreter as our starting point
    let mut patched_bin = fs::read(env::current_exe()?).await?;

    // The signature which separates indicates the presence of bytecode to execute
    // If a binary contains this signature, that must mean it is a standalone binar
    let signature: Vec<u8> = vec![0x12, 0xed, 0x93, 0x14, 0x28];

    // Append the signature to the base binary
    for byte in signature {
        patched_bin.push(byte);
    }

    // Compile luau input into bytecode
    let mut bytecode = LuaCompiler::new()
        .set_optimization_level(2)
        .set_coverage_level(0)
        .set_debug_level(0)
        .compile(code);

    // Append compiled bytecode to binary and finalize
    patched_bin.append(&mut bytecode);

    // Write the compiled binary to file
    fs::write(output_path, patched_bin).await?;

    Ok(())
}
