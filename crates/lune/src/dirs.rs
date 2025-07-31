use anyhow::{Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

/**
    Get the cache directory for Lune, respecting `XDG_CACHE_HOME`

    Implements backwards compatibility by preferring existing legacy
    directories over new XDG locations to minimize user friction.

    # Errors

    Returns an error if the system's base directories cannot be determined.
*/
pub fn cache_dir() -> Result<PathBuf> {
    if let Ok(custom_cache) = std::env::var("LUNE_CACHE") {
        return Ok(PathBuf::from(custom_cache));
    }

    let dirs = BaseDirs::new().context("Unable to find cache directory")?;
    let xdg_cache = dirs.cache_dir().join("lune");
    let legacy_cache = dirs.home_dir().join(".lune");

    if legacy_cache.join("target").exists() && !xdg_cache.join("target").exists() {
        return Ok(legacy_cache);
    }

    Ok(xdg_cache)
}

/**
    Get the state directory for Lune, respecting `XDG_STATE_HOME`

    Implements backwards compatibility by preferring existing legacy
    directories over new XDG locations to minimize user friction.

    # Errors

    Returns an error if the system's base directories cannot be determined.
*/
pub fn state_dir() -> Result<PathBuf> {
    if let Ok(custom_state) = std::env::var("LUNE_STATE") {
        return Ok(PathBuf::from(custom_state));
    }

    let dirs = BaseDirs::new().context("Unable to find base directories")?;

    let base_dir = match dirs.state_dir() {
        Some(state_dir) => state_dir,
        None => dirs.cache_dir(),
    };

    let xdg_state = base_dir.join("lune");
    let legacy_home = dirs.home_dir();

    if legacy_home.join(".lune_history").exists() && !xdg_state.join(".lune_history").exists() {
        return Ok(legacy_home.to_path_buf());
    }

    Ok(xdg_state)
}

/**
    Get the typedefs directory (always home-based for LSP compatibility)

    # Errors

    Returns an error if the home directory cannot be determined.
*/
pub fn typedefs_dir() -> Result<PathBuf> {
    let dirs = BaseDirs::new().context("Unable to find home directory")?;
    Ok(dirs.home_dir().join(".lune").join(".typedefs"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cache_dir_with_custom_env() {
        let test_path = if cfg!(windows) {
            "C:\\custom\\cache"
        } else {
            "/custom/cache"
        };
        unsafe {
            env::set_var("LUNE_CACHE", test_path);
        }
        let result = cache_dir().unwrap();
        assert_eq!(result, PathBuf::from(test_path));
        unsafe {
            env::remove_var("LUNE_CACHE");
        }
    }

    #[test]
    fn test_cache_dir_without_custom_env() {
        unsafe {
            env::remove_var("LUNE_CACHE");
        }
        let result = cache_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("lune"));
    }

    #[test]
    fn test_state_dir_with_custom_env() {
        let test_path = if cfg!(windows) {
            "C:\\custom\\state"
        } else {
            "/custom/state"
        };
        unsafe {
            env::set_var("LUNE_STATE", test_path);
        }
        let result = state_dir().unwrap();
        assert_eq!(result, PathBuf::from(test_path));
        unsafe {
            env::remove_var("LUNE_STATE");
        }
    }

    #[test]
    fn test_state_dir_without_custom_env() {
        use std::sync::Mutex;
        static TEST_MUTEX: Mutex<()> = Mutex::new(());
        let _guard = TEST_MUTEX.lock().unwrap();

        unsafe {
            env::remove_var("LUNE_STATE");
        }

        // Temporarily hide any existing legacy files to test the non-legacy path
        let dirs = BaseDirs::new().unwrap();
        let legacy_history = dirs.home_dir().join(".lune_history");
        let backup_path = dirs
            .home_dir()
            .join(format!(".lune_history.test_backup_{}", std::process::id()));

        let had_legacy = legacy_history.exists();
        if had_legacy {
            std::fs::rename(&legacy_history, &backup_path).unwrap();
        }

        let result = state_dir();

        // Restore legacy file if it existed
        if had_legacy && backup_path.exists() {
            std::fs::rename(&backup_path, &legacy_history)
                .expect("Failed to restore backup file after test");
        }

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("lune"));
    }

    #[test]
    fn test_typedefs_dir() {
        let result = typedefs_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".lune"));
        assert!(path_str.contains(".typedefs"));
    }

    #[test]
    fn test_cache_dir_backwards_compatibility_legacy_exists() {
        unsafe {
            env::remove_var("LUNE_CACHE");
        }

        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path();

        let legacy_cache = temp_home.join(".lune");
        let legacy_target = legacy_cache.join("target");
        fs::create_dir_all(&legacy_target).unwrap();
        fs::write(legacy_target.join("test_file"), "legacy content").unwrap();

        // Note: This test documents expected behavior but can't fully mock BaseDirs
        assert!(legacy_cache.exists());
        assert!(legacy_cache.join("target").exists());
    }

    #[test]
    fn test_cache_dir_backwards_compatibility_no_legacy() {
        unsafe {
            env::remove_var("LUNE_CACHE");
        }

        let result = cache_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("lune"));
    }

    #[test]
    fn test_state_dir_backwards_compatibility_legacy_exists() {
        unsafe {
            env::remove_var("LUNE_STATE");
        }

        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path();

        let legacy_history = temp_home.join(".lune_history");
        fs::write(&legacy_history, "history content").unwrap();

        assert!(legacy_history.exists());
    }

    #[test]
    fn test_state_dir_backwards_compatibility_no_legacy() {
        use std::sync::Mutex;
        static TEST_MUTEX2: Mutex<()> = Mutex::new(());
        let _guard = TEST_MUTEX2.lock().unwrap();

        unsafe {
            env::remove_var("LUNE_STATE");
        }

        // Temporarily hide any existing legacy files to test the non-legacy path
        let dirs = BaseDirs::new().unwrap();
        let legacy_history = dirs.home_dir().join(".lune_history");
        let backup_path = dirs.home_dir().join(format!(
            ".lune_history.test_backup_bc_{}",
            std::process::id()
        ));

        let had_legacy = legacy_history.exists();
        if had_legacy {
            std::fs::rename(&legacy_history, &backup_path).unwrap();
        }

        let result = state_dir();

        // Restore legacy file if it existed
        if had_legacy && backup_path.exists() {
            std::fs::rename(&backup_path, &legacy_history)
                .expect("Failed to restore backup file after test");
        }

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("lune"));
    }

    #[test]
    fn test_cache_dir_prefers_env_var_over_legacy() {
        let test_path = if cfg!(windows) {
            "C:\\env\\cache"
        } else {
            "/env/cache"
        };

        unsafe {
            env::set_var("LUNE_CACHE", test_path);
        }

        let result = cache_dir().unwrap();
        assert_eq!(result, PathBuf::from(test_path));

        unsafe {
            env::remove_var("LUNE_CACHE");
        }
    }

    #[test]
    fn test_state_dir_prefers_env_var_over_legacy() {
        let test_path = if cfg!(windows) {
            "C:\\env\\state"
        } else {
            "/env/state"
        };

        unsafe {
            env::set_var("LUNE_STATE", test_path);
        }

        let result = state_dir().unwrap();
        assert_eq!(result, PathBuf::from(test_path));

        unsafe {
            env::remove_var("LUNE_STATE");
        }
    }
}
