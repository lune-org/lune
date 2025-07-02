use std::{
    collections::VecDeque,
    path::{Component, Path, PathBuf},
};

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

    let mut components = path.components().collect::<VecDeque<_>>();
    if matches!(components.front(), None | Some(Component::Normal(..))) {
        components.push_front(Component::CurDir);
    }

    components.into_iter().collect()
}

/**
    Pops the relative path up to the parent directory, pushing "parent dir" (`..`)
    components to the front of the path when it no longer has any normal components.

    This means that unlike [`PathBuf::pop`], this function may be called an arbitrary
    number of times, and represent parent folders without first canonicalizing paths.
*/
pub(crate) fn relative_path_parent(rel: &mut PathBuf) {
    // If our relative path becomes empty, we should keep traversing it,
    // but we need to do so by appending the special "parent dir" component,
    // which is normally represented by ".."
    if rel.components().count() == 1 && rel.components().next().unwrap() == Component::CurDir {
        rel.pop();
        rel.push(Component::ParentDir);
    } else if rel.components().all(|c| matches!(c, Component::ParentDir)) {
        rel.push(Component::ParentDir);
    } else {
        rel.pop();
    }
}
