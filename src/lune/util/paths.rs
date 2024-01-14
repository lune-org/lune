use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use path_clean::PathClean;

pub static CWD: Lazy<PathBuf> = Lazy::new(|| {
    let cwd = current_dir().expect("failed to find current working directory");
    dunce::canonicalize(cwd).expect("failed to canonicalize current working directory")
});

pub fn make_absolute_and_clean(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_relative() {
        CWD.join(path).clean()
    } else {
        path.clean()
    }
}
