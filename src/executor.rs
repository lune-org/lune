use std::{env, ops::ControlFlow, process::ExitCode};

use lune::Lune;

use anyhow::Result;
use num_traits::{FromBytes, ToBytes};
use tokio::fs::read as read_to_vec;

// The signature which separates indicates the presence of bytecode to execute
// If a binary contains this magic signature as the last 8 bytes, that must mean
//  it is a standalone binary
pub const MAGIC: &[u8; 8] = b"cr3sc3nt";

/// Utility struct to parse and generate bytes to the META chunk of standalone binaries.
#[derive(Debug, Clone)]
pub struct MetaChunk {
    /// Compiled lua bytecode of the entrypoint script.
    pub bytecode: Vec<u8>,
    /// Offset to the the beginning of the bytecode from the start of the lune binary.
    pub bytecode_offset: Option<u64>,
    /// Number of files present, currently unused. **For future use**.
    pub file_count: Option<u64>,
}

impl MetaChunk {
    /// Creates an emtpy `MetaChunk` instance.
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            bytecode_offset: None,
            file_count: None,
        }
    }

    /// Builder method to include the bytecode, **mandatory** before build.
    pub fn with_bytecode(&mut self, bytecode: Vec<u8>) -> Self {
        self.bytecode = bytecode;

        self.clone()
    }

    /// Builder method to include the bytecode offset, **mandatory** before build.
    pub fn with_bytecode_offset(&mut self, offset: u64) -> Self {
        self.bytecode_offset = Some(offset);

        self.clone()
    }

    /// Builder method to include the file count, **mandatory** before build.

    pub fn with_file_count(&mut self, count: u64) -> Self {
        self.file_count = Some(count);

        self.clone()
    }

    /// Builds the final `Vec` of bytes, based on the endianness specified.
    pub fn build(self, endianness: &str) -> Vec<u8> {
        match endianness {
            "big" => self.to_be_bytes(),
            "little" => self.to_le_bytes(),
            &_ => panic!("unexpected endianness"),
        }
    }

    /// Internal method which implements endian independent bytecode discovery logic.
    fn from_bytes(bytes: &[u8], int_handler: fn([u8; 8]) -> u64) -> Result<Self> {
        let mut bytecode_offset = 0;
        let mut bytecode_size = 0;

        // standalone binary structure (reversed, 8 bytes per field)
        // [0] => magic signature
        // ----------------
        // -- META Chunk --
        // [1] => file count
        // [2] => bytecode size
        // [3] => bytecode offset
        // ----------------
        // -- MISC Chunk --
        // [4..n] => bytecode (variable size)
        // ----------------
        // NOTE: All integers are 8 byte, padded, unsigned & 64 bit (u64's).

        // The rchunks will have unequally sized sections in the beginning
        // but that doesn't matter to us because we don't need anything past the
        // middle chunks where the bytecode is stored
        bytes
            .rchunks(MAGIC.len())
            .enumerate()
            .try_for_each(|(idx, chunk)| {
                if bytecode_offset != 0 && bytecode_size != 0 {
                    return ControlFlow::Break(());
                }

                if idx == 0 && chunk != MAGIC {
                    // Binary is guaranteed to be standalone, we've confirmed this before
                    unreachable!("expected proper magic signature for standalone binary")
                }

                if idx == 3 {
                    bytecode_offset = int_handler(chunk.try_into().unwrap());
                }

                if idx == 2 {
                    bytecode_size = int_handler(chunk.try_into().unwrap());
                }

                ControlFlow::Continue(())
            });

        Ok(Self {
            bytecode: bytes[usize::try_from(bytecode_offset)?
                ..usize::try_from(bytecode_offset + bytecode_size)?]
                .to_vec(),
            bytecode_offset: Some(bytecode_offset),
            file_count: Some(1),
        })
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
        Self::from_bytes(bytes, u64::from_be_bytes).unwrap()
    }

    fn from_le_bytes(bytes: &Self::Bytes) -> Self {
        Self::from_bytes(bytes, u64::from_le_bytes).unwrap()
    }
}

/**
    Returns information about whether the execution environment is standalone
    or not, the standalone binary signature, and the contents of the binary.
*/
pub async fn check_env() -> (bool, Vec<u8>) {
    // Read the current lune binary to memory
    let bin = if let Ok(contents) = read_to_vec(
        env::current_exe().expect("failed to get path to current running lune executable"),
    )
    .await
    {
        contents
    } else {
        Vec::new()
    };

    let is_standalone =
        !bin.is_empty() && bin[bin.len() - MAGIC.len()..bin.len()] == MAGIC.to_vec();

    (is_standalone, bin)
}

/**
    Discovers, loads and executes the bytecode contained in a standalone binary.
*/
pub async fn run_standalone(bin: Vec<u8>) -> Result<ExitCode> {
    // If we were able to retrieve the required metadata, we load
    // and execute the bytecode
    let MetaChunk { bytecode, .. } = MetaChunk::from_le_bytes(&bin);

    // Skip the first argument which is the path to current executable
    let args = env::args().skip(1).collect::<Vec<_>>();

    let result = Lune::new()
        .with_args(args)
        .run("STANDALONE", bytecode)
        .await;

    Ok(match result {
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
        Ok(code) => code,
    })
}
