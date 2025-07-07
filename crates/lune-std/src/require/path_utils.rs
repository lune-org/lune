use std::path::{Component, Path, PathBuf};

use lune_utils::path::clean_path;

/**
    Appends the given extension to the path.

    Does not replace or modify any existing extension(s).
*/
pub(crate) fn append_extension(path: &Path, ext: &str) -> PathBuf {
    match path.extension() {
        None => path.with_extension(ext),
        Some(curr_ext) => {
            let mut new_ext = curr_ext.to_os_string();
            new_ext.push(".");
            new_ext.push(ext);
            path.with_extension(new_ext)
        }
    }
}

/**
    Normalizes the given relative path.

    This will clean the path, removing any redundant components,
    and ensure that it has a leading "current dir" (`./`) component.
*/
pub(crate) fn relative_path_normalize(path: &Path) -> PathBuf {
    let path = clean_path(path);

    let mut it = path.components().peekable();
    if it.peek().is_none_or(|c| matches!(c, Component::Normal(..))) {
        std::iter::once(Component::CurDir)
            .chain(path.components())
            .collect()
    } else {
        path
    }
}

/**
    Pops the relative path up to the parent directory, pushing "parent dir" (`..`)
    components to the front of the path when it no longer has any normal components.

    This means that unlike [`PathBuf::pop`], this function may be called an arbitrary
    number of times, and represent parent folders without first canonicalizing paths.
*/
pub(crate) fn relative_path_parent(rel: &mut PathBuf) {
    if rel.as_os_str() == Component::CurDir.as_os_str() {
        *rel = PathBuf::from(Component::ParentDir.as_os_str());
    } else if rel.components().all(|c| c == Component::ParentDir) {
        rel.push(Component::ParentDir.as_os_str());
    } else {
        rel.pop();
    }
}
