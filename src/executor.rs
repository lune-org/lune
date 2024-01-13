use std::{env, process::ExitCode};

use crate::cli::build::{MetaChunk, MAGIC};
use lune::Lune;

use anyhow::Result;
use num_traits::FromBytes;
use tokio::fs::read as read_to_vec;

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
