use std::path::PathBuf;

/**

adds extension to path without replacing it's current extensions

### Example

appending `.luau` to `path/path.config` will return `path/path.config.luau`

 */
pub fn append_extension(path: impl Into<PathBuf>, ext: &'static str) -> PathBuf {
    let mut new: PathBuf = path.into();
    match new.extension() {
        // FUTURE: There's probably a better way to do this than converting to a lossy string
        Some(e) => new.set_extension(format!("{}.{ext}", e.to_string_lossy())),
        None => new.set_extension(ext),
    };
    new
}
