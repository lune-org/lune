use anyhow::Result;

#[allow(clippy::unnecessary_wraps)]
pub fn generate_from_type_definitions(contents: &str) -> Result<String> {
    Ok(format!(
        "--> Lune v{}\n\n{}",
        env!("CARGO_PKG_VERSION"),
        contents
    ))
}
