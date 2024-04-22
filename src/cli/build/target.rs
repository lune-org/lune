use std::{env::consts::ARCH, fmt, path::PathBuf, str::FromStr};

use directories::BaseDirs;
use once_cell::sync::Lazy;

const HOME_DIR: Lazy<PathBuf> = Lazy::new(|| {
    BaseDirs::new()
        .expect("could not find home directory")
        .home_dir()
        .to_path_buf()
});

pub const CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| HOME_DIR.join(".lune").join("target"));

/**
    A target operating system supported by Lune
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTargetOS {
    Windows,
    Linux,
    MacOS,
}

impl BuildTargetOS {
    fn current_system() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            "macos" => Self::MacOS,
            _ => panic!("unsupported target OS"),
        }
    }

    fn exe_extension(self) -> &'static str {
        // NOTE: We can't use the constants from std since
        // they are only accessible for the current target
        match self {
            Self::Windows => "exe",
            _ => "",
        }
    }

    fn exe_suffix(self) -> &'static str {
        match self {
            Self::Windows => ".exe",
            _ => "",
        }
    }
}

impl fmt::Display for BuildTargetOS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Windows => write!(f, "windows"),
            Self::Linux => write!(f, "linux"),
            Self::MacOS => write!(f, "macos"),
        }
    }
}

impl FromStr for BuildTargetOS {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "win" | "windows" => Ok(Self::Windows),
            "linux" => Ok(Self::Linux),
            "mac" | "macos" | "darwin" => Ok(Self::MacOS),
            _ => Err("invalid target OS"),
        }
    }
}

/**
    A target architecture supported by Lune
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTargetArch {
    X86_64,
    Aarch64,
}

impl BuildTargetArch {
    fn current_system() -> Self {
        match ARCH {
            "x86_64" => Self::X86_64,
            "aarch64" => Self::Aarch64,
            _ => panic!("unsupported target architecture"),
        }
    }
}

impl fmt::Display for BuildTargetArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X86_64 => write!(f, "x86_64"),
            Self::Aarch64 => write!(f, "aarch64"),
        }
    }
}

impl FromStr for BuildTargetArch {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "x86_64" | "x64" => Ok(Self::X86_64),
            "aarch64" | "arm64" => Ok(Self::Aarch64),
            _ => Err("invalid target architecture"),
        }
    }
}

/**
    A full target description that Lune supports (OS + Arch)

    This is used to determine the target to build for standalone binaries,
    and to download the correct base executable for cross-compilation.

    The target may be parsed from and displayed in the form `os-arch`.
    Examples of valid targets are:

    - `linux-aarch64`
    - `linux-x86_64`
    - `macos-aarch64`
    - `macos-x86_64`
    - `windows-x86_64`
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildTarget {
    pub os: BuildTargetOS,
    pub arch: BuildTargetArch,
}

impl BuildTarget {
    pub fn current_system() -> Self {
        Self {
            os: BuildTargetOS::current_system(),
            arch: BuildTargetArch::current_system(),
        }
    }

    pub fn is_current_system(&self) -> bool {
        self.os == BuildTargetOS::current_system() && self.arch == BuildTargetArch::current_system()
    }

    pub fn exe_extension(&self) -> &'static str {
        self.os.exe_extension()
    }

    pub fn exe_suffix(&self) -> &'static str {
        self.os.exe_suffix()
    }

    pub fn cache_path(&self) -> PathBuf {
        CACHE_DIR.join(format!("{self}{}", self.os.exe_extension()))
    }
}

impl fmt::Display for BuildTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.os, self.arch)
    }
}

impl FromStr for BuildTarget {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s
            .split_once('-')
            .ok_or("target must be in the form `os-arch`")?;

        let os = left.parse()?;
        let arch = right.parse()?;

        Ok(Self { os, arch })
    }
}
