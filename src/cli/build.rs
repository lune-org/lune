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
    let base_bin_offset = u64::try_from(patched_bin.len())?;

    // The signature which separates indicates the presence of bytecode to execute
    // If a binary contains this signature, that must mean it is a standalone binar
    let signature: Vec<u8> = vec![0x4f, 0x3e, 0xf8, 0x41, 0xc3, 0x3a, 0x52, 0x16];

    // Compile luau input into bytecode
    let bytecode = LuaCompiler::new()
        .set_optimization_level(2)
        .set_coverage_level(0)
        .set_debug_level(0)
        .compile(code);

    println!("{}", bytecode.len());

    patched_bin.append(&mut bytecode.clone());

    let mut meta = base_bin_offset.to_ne_bytes().to_vec();

    // bytecode base size files signature
    // meta.append(&mut [0, 0, 0, 0].to_vec()); // 4 extra padding bytes after 4 byte u64
    meta.append(&mut (bytecode.len() as u64).to_ne_bytes().to_vec()); // FIXME: len is greater than u8::max
    meta.append(&mut 1_u64.to_ne_bytes().to_vec()); // Number of files, padded with null bytes
                                                    // meta.append(&mut [0, 0, 0, 0].to_vec()); // 4 extra padding bytes after 4 byte u32

    patched_bin.append(&mut meta);

    // Append the signature to the base binary
    for byte in signature {
        patched_bin.push(byte);
    }

    // Write the compiled binary to file
    fs::write(output_path, patched_bin).await?;

    Ok(())
}
