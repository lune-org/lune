use anyhow::{Context, Result};
use etcetera::{BaseStrategy, choose_base_strategy};
use std::path::PathBuf;

/**
    Get the cache directory for Lune, respecting `XDG_CACHE_HOME`

    # Errors

    Returns an error if the system's base directories cannot be determined.
*/
pub fn cache_dir() -> Result<PathBuf> {
    if let Ok(custom_cache) = std::env::var("LUNE_CACHE") {
        return Ok(PathBuf::from(custom_cache));
    }

    let strategy = choose_base_strategy().context("Unable to find cache directory")?;
    Ok(strategy.cache_dir().join("lune"))
}

/**
    Get the state directory for Lune, respecting `XDG_STATE_HOME`

    # Errors

    Returns an error if the system's base directories cannot be determined.
*/
pub fn state_dir() -> Result<PathBuf> {
    if let Ok(custom_state) = std::env::var("LUNE_STATE") {
        return Ok(PathBuf::from(custom_state));
    }

    let strategy = choose_base_strategy().context("Unable to find base directories")?;

    // Try to get state_dir, fall back to cache_dir on platforms that don't support it
    let base_dir = match strategy.state_dir() {
        Some(state_dir) => state_dir,
        None => {
            // Fall back to cache directory if state directory is not supported
            strategy.cache_dir()
        }
    };

    Ok(base_dir.join("lune"))
}

/**
    Get the typedefs directory (always home-based for LSP compatibility)

    # Errors

    Returns an error if the home directory cannot be determined.
*/
pub fn typedefs_dir() -> Result<PathBuf> {
    let strategy = choose_base_strategy().context("Unable to find home directory")?;
    Ok(strategy.home_dir().join(".lune").join(".typedefs"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

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
        unsafe {
            env::remove_var("LUNE_STATE");
        }
        let result = state_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("lune"));
    }

    #[test]
    fn test_typedefs_dir() {
        let result = typedefs_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        // Check that the path contains both .lune and .typedefs components
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".lune"));
        assert!(path_str.contains(".typedefs"));
    }
}
