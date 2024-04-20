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

/// A target operating system supported by Lune
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOS {
    Windows,
    Linux,
    MacOS,
}

impl TargetOS {
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

impl fmt::Display for TargetOS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Windows => write!(f, "windows"),
            Self::Linux => write!(f, "linux"),
            Self::MacOS => write!(f, "macos"),
        }
    }
}

impl FromStr for TargetOS {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "windows" => Ok(Self::Windows),
            "linux" => Ok(Self::Linux),
            "macos" => Ok(Self::MacOS),
            _ => Err("invalid target OS"),
        }
    }
}

/// A target architecture supported by Lune
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
    X86_64,
    Aarch64,
}

impl TargetArch {
    fn current_system() -> Self {
        match ARCH {
            "x86_64" => Self::X86_64,
            "aarch64" => Self::Aarch64,
            _ => panic!("unsupported target architecture"),
        }
    }
}

impl fmt::Display for TargetArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X86_64 => write!(f, "x86_64"),
            Self::Aarch64 => write!(f, "aarch64"),
        }
    }
}

impl FromStr for TargetArch {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "x86_64" | "x64" => Ok(Self::X86_64),
            "aarch64" | "arm64" => Ok(Self::Aarch64),
            _ => Err("invalid target architecture"),
        }
    }
}

/// A full target description for cross-compilation (OS + Arch)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub os: TargetOS,
    pub arch: TargetArch,
}

impl Target {
    pub fn current_system() -> Self {
        Self {
            os: TargetOS::current_system(),
            arch: TargetArch::current_system(),
        }
    }

    pub fn is_current_system(&self) -> bool {
        self.os == TargetOS::current_system() && self.arch == TargetArch::current_system()
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

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.os, self.arch)
    }
}

impl FromStr for Target {
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
