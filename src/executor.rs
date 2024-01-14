use std::{env, process::ExitCode};

use lune::Runtime;

use anyhow::{bail, Result};
use tokio::fs;

const MAGIC: &[u8; 8] = b"cr3sc3nt";

/**
    Metadata for a standalone Lune executable. Can be used to
    discover and load the bytecode contained in a standalone binary.
*/
#[derive(Debug, Clone)]
pub struct MetaChunk {
    pub bytecode: Vec<u8>,
}

impl MetaChunk {
    /**
        Tries to read a standalone binary from the given bytes.
    */
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self> {
        let bytes = bytes.as_ref();
        if bytes.len() < 16 || !bytes.ends_with(MAGIC) {
            bail!("not a standalone binary")
        }

        // Extract bytecode size
        let bytecode_size_bytes = &bytes[bytes.len() - 16..bytes.len() - 8];
        let bytecode_size =
            usize::try_from(u64::from_be_bytes(bytecode_size_bytes.try_into().unwrap()))?;

        // Extract bytecode
        let bytecode = bytes[bytes.len() - 16 - bytecode_size..].to_vec();

        Ok(Self { bytecode })
    }

    /**
        Writes the metadata chunk to a byte vector, to later bet read using `from_bytes`.
    */
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.bytecode);
        bytes.extend_from_slice(&(self.bytecode.len() as u64).to_be_bytes());
        bytes.extend_from_slice(MAGIC);
        bytes
    }
}

/**
    Returns whether or not the currently executing Lune binary
    is a standalone binary, and if so, the bytes of the binary.
*/
pub async fn check_env() -> (bool, Vec<u8>) {
    let path = env::current_exe().expect("failed to get path to current running lune executable");
    let contents = fs::read(path).await.unwrap_or_default();
    let is_standalone = contents.ends_with(MAGIC);
    (is_standalone, contents)
}

/**
    Discovers, loads and executes the bytecode contained in a standalone binary.
*/
pub async fn run_standalone(patched_bin: impl AsRef<[u8]>) -> Result<ExitCode> {
    // The first argument is the path to the current executable
    let args = env::args().skip(1).collect::<Vec<_>>();
    let meta = MetaChunk::from_bytes(patched_bin).expect("must be a standalone binary");

    let result = Runtime::new()
        .with_args(args)
        .run("STANDALONE", meta.bytecode)
        .await;

    Ok(match result {
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
        Ok(code) => code,
    })
}
