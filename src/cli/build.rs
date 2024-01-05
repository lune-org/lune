use console::Style;
use itertools::Itertools;
use num_traits::{FromBytes, ToBytes};
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

/// Utility struct to parse and generate bytes to the META chunk of standalone binaries.
#[derive(Debug, Clone)]
pub struct MetaChunk {
    /// Compiled lua bytecode of the entrypoint script.
    bytecode: Vec<u8>,
    /// Offset to the the beginning of the bytecode from the start of the lune binary.
    bytecode_offset: Option<u64>,
    /// Number of files present, currently unused. **For future use**.
    file_count: Option<u64>,
}

impl MetaChunk {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            bytecode_offset: None,
            file_count: None,
        }
    }

    pub fn with_bytecode(&mut self, bytecode: Vec<u8>) -> Self {
        self.bytecode = bytecode;

        self.clone()
    }

    pub fn with_bytecode_offset(&mut self, offset: u64) -> Self {
        self.bytecode_offset = Some(offset);

        self.clone()
    }

    pub fn with_file_count(&mut self, count: u64) -> Self {
        self.file_count = Some(count);

        self.clone()
    }

    pub fn build(self, endianness: &str) -> Vec<u8> {
        match endianness {
            "big" => self.to_be_bytes(),
            "little" => self.to_le_bytes(),
            &_ => panic!("unexpected endianness"),
        }
    }
}

impl Default for MetaChunk {
    fn default() -> Self {
        Self {
            bytecode: Vec::new(),
            bytecode_offset: Some(0),
            file_count: Some(1),
        }
    }
}

impl ToBytes for MetaChunk {
    type Bytes = Vec<u8>;

    fn to_be_bytes(&self) -> Self::Bytes {
        // We start with the bytecode offset as the first field already filled in
        let mut tmp = self.bytecode_offset.unwrap().to_be_bytes().to_vec();

        // NOTE: The order of the fields here are reversed, which is on purpose
        tmp.extend(self.bytecode.len().to_be_bytes());
        tmp.extend(self.file_count.unwrap().to_be_bytes());

        tmp
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        // We start with the bytecode offset as the first field already filled in
        let mut tmp = self.bytecode_offset.unwrap().to_le_bytes().to_vec();

        // NOTE: The order of the fields here are reversed, which is on purpose
        tmp.extend(self.bytecode.len().to_le_bytes());
        tmp.extend(self.file_count.unwrap().to_le_bytes());

        tmp
    }
}

impl FromBytes for MetaChunk {
    type Bytes = Vec<u8>;

    fn from_be_bytes(bytes: &Self::Bytes) -> Self {
        todo!()
    }

    fn from_le_bytes(bytes: &Self::Bytes) -> Self {
        todo!()
    }
}

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

    let meta = MetaChunk::new()
        .with_bytecode(bytecode)
        .with_bytecode_offset(base_bin_offset)
        .with_file_count(1_u64); // Start with the base bytecode offset

    // Include metadata in the META chunk, each field is 8 bytes
    patched_bin.extend(meta.build("little"));

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
