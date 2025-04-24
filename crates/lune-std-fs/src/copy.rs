use std::collections::VecDeque;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use async_fs as fs;
use futures_lite::prelude::*;
use mlua::prelude::*;

use super::options::FsWriteOptions;

pub struct CopyContents {
    // Vec<(relative depth, path)>
    pub dirs: Vec<(usize, PathBuf)>,
    pub files: Vec<(usize, PathBuf)>,
}

async fn get_contents_at(root: PathBuf, _: FsWriteOptions) -> LuaResult<CopyContents> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    let mut queue = VecDeque::new();

    let normalized_root = fs::canonicalize(&root).await.map_err(|e| {
        LuaError::RuntimeError(format!("Failed to canonicalize root directory path\n{e}"))
    })?;

    // Push initial children of the root path into the queue
    let mut reader = fs::read_dir(&normalized_root).await?;
    while let Some(entry) = reader.try_next().await? {
        queue.push_back((1, entry.path()));
    }

    // Go through the current queue, pushing to it
    // when we find any new descendant directories
    // FUTURE: Try to do async reading here concurrently to speed it up a bit
    while let Some((current_depth, current_path)) = queue.pop_front() {
        let meta = fs::metadata(&current_path).await?;
        if meta.is_symlink() {
            return Err(LuaError::RuntimeError(format!(
                "Symlinks are not yet supported, encountered at path '{}'",
                current_path.display()
            )));
        } else if meta.is_dir() {
            // FUTURE: Add an option in FsWriteOptions for max depth and limit it here
            let mut entries = fs::read_dir(&current_path).await?;
            while let Some(entry) = entries.try_next().await? {
                queue.push_back((current_depth + 1, entry.path()));
            }
            dirs.push((current_depth, current_path));
        } else {
            files.push((current_depth, current_path));
        }
    }

    // Ensure that all directory and file paths are relative to the root path
    // SAFETY: Since we only ever push dirs and files relative to the root, unwrap is safe
    for (_, dir) in &mut dirs {
        *dir = dir.strip_prefix(&normalized_root).unwrap().to_path_buf();
    }
    for (_, file) in &mut files {
        *file = file.strip_prefix(&normalized_root).unwrap().to_path_buf();
    }

    // FUTURE: Deduplicate paths such that these directories:
    // - foo/
    // - foo/bar/
    // - foo/bar/baz/
    // turn into a single foo/bar/baz/ and let create_dir_all do the heavy lifting

    Ok(CopyContents { dirs, files })
}

async fn ensure_no_dir_exists(path: impl AsRef<Path>) -> LuaResult<()> {
    let path = path.as_ref();
    match fs::metadata(&path).await {
        Ok(meta) if meta.is_dir() => Err(LuaError::RuntimeError(format!(
            "A directory already exists at the path '{}'",
            path.display()
        ))),
        _ => Ok(()),
    }
}

async fn ensure_no_file_exists(path: impl AsRef<Path>) -> LuaResult<()> {
    let path = path.as_ref();
    match fs::metadata(&path).await {
        Ok(meta) if meta.is_file() => Err(LuaError::RuntimeError(format!(
            "A file already exists at the path '{}'",
            path.display()
        ))),
        _ => Ok(()),
    }
}

pub async fn copy(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    options: FsWriteOptions,
) -> LuaResult<()> {
    let source = source.as_ref();
    let target = target.as_ref();

    // Check if we got a file or directory - we will handle them differently below
    let (is_dir, is_file) = match fs::metadata(&source).await {
        Ok(meta) => (meta.is_dir(), meta.is_file()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err(LuaError::RuntimeError(format!(
                "No file or directory exists at the path '{}'",
                source.display()
            )))
        }
        Err(e) => return Err(e.into()),
    };
    if !is_file && !is_dir {
        return Err(LuaError::RuntimeError(format!(
            "The given path '{}' is not a file or a directory",
            source.display()
        )));
    }

    // Perform copying:
    //
    // 1. If we are not allowed to overwrite, make sure nothing exists at the target path
    // 2. If we are allowed to overwrite, remove any previous entry at the path
    // 3. Write all directories first
    // 4. Write all files

    if !options.overwrite {
        if is_file {
            ensure_no_file_exists(target).await?;
        } else if is_dir {
            ensure_no_dir_exists(target).await?;
        }
    }

    if is_file {
        fs::copy(source, target).await?;
    } else if is_dir {
        let contents = get_contents_at(source.to_path_buf(), options).await?;

        if options.overwrite {
            let (is_dir, is_file) = match fs::metadata(&target).await {
                Ok(meta) => (meta.is_dir(), meta.is_file()),
                Err(e) if e.kind() == ErrorKind::NotFound => (false, false),
                Err(e) => return Err(e.into()),
            };
            if is_dir {
                fs::remove_dir_all(target).await?;
            } else if is_file {
                fs::remove_file(target).await?;
            }
        }

        fs::create_dir_all(target).await?;

        // FUTURE: Write dirs / files concurrently
        // to potentially speed these operations up
        for (_, dir) in &contents.dirs {
            fs::create_dir_all(target.join(dir)).await?;
        }
        for (_, file) in &contents.files {
            fs::copy(source.join(file), target.join(file)).await?;
        }
    }

    Ok(())
}
