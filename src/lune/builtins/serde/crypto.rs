use std::sync::Arc;

use anyhow::Result;
use base64::{engine::general_purpose as Base64, Engine as _};
use digest::Digest as _;
use mlua::prelude::*;
use std::sync::Mutex;

// TODO: Proper error handling, remove unwraps

#[derive(Clone)]
pub struct Crypto {
    algo: Arc<Mutex<CryptoAlgo>>,
}

#[derive(Clone)]
pub enum CryptoAlgo {
    Sha1(Box<sha1::Sha1>),
    Sha256(Box<sha2::Sha256>),
    Sha512(Box<sha2::Sha512>),
    // Blake2(Box<T>),
    // Md5(Box<T>),
}

#[derive(PartialOrd, PartialEq, Ord, Eq)]
pub enum EncodingKind {
    Utf8,
    Base64,
    Hex,
}

impl From<usize> for EncodingKind {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Utf8,
            1 => Self::Base64,
            2 => Self::Hex,
            _ => panic!("invalid value"),
        }
    }
}

impl From<String> for EncodingKind {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "utf8" => Self::Utf8,
            "base64" => Self::Base64,
            "hex" => Self::Hex,
            &_ => panic!("invalid value"),
        }
    }
}

impl FromLua<'_> for EncodingKind {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(int) => Ok(EncodingKind::from(int as usize)),
            LuaValue::Number(num) => Ok(EncodingKind::from(num as usize)),
            LuaValue::String(str) => Ok(EncodingKind::from(str.to_string_lossy().to_string())),

            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "EncodingKind",
                message: Some("value must be a an Integer, Number or String".to_string()),
            }),
        }
    }
}

impl CryptoAlgo {
    // TODO: Replace boilerplate using a macro

    pub fn update(&mut self, content: impl AsRef<[u8]>) {
        match self {
            CryptoAlgo::Sha1(hasher) => hasher.update(content),
            CryptoAlgo::Sha256(hasher) => hasher.update(content),
            CryptoAlgo::Sha512(hasher) => hasher.update(content),
        };
    }

    pub fn digest(&mut self, encoding: EncodingKind) -> Result<String> {
        let computed: Vec<u8> = match self {
            CryptoAlgo::Sha1(hasher) => hasher.clone().finalize().to_vec(),
            CryptoAlgo::Sha256(hasher) => hasher.clone().finalize().to_vec(),
            CryptoAlgo::Sha512(hasher) => hasher.clone().finalize().to_vec(),
        };

        match encoding {
            EncodingKind::Utf8 => String::from_utf8(computed).map_err(anyhow::Error::from),
            EncodingKind::Base64 => Ok(Base64::STANDARD.encode(computed)),
            EncodingKind::Hex => Ok(hex::encode(&computed)),
        }
    }
}

impl Crypto {
    pub fn sha1<T: ToString>(content: Option<T>) -> Crypto {
        let constructed = Self {
            algo: Arc::new(Mutex::new(CryptoAlgo::Sha1(Box::new(sha1::Sha1::new())))),
        };

        match content {
            Some(inner) => constructed.update(inner.to_string()).clone(),
            None => constructed,
        }
    }

    pub fn sha256<T: ToString>(content: Option<T>) -> Crypto {
        let constructed = Self {
            algo: Arc::new(Mutex::new(CryptoAlgo::Sha256(
                Box::new(sha2::Sha256::new()),
            ))),
        };

        match content {
            Some(inner) => constructed.update(inner.to_string()).clone(),
            None => constructed,
        }
    }

    pub fn sha512<T: ToString>(content: Option<T>) -> Crypto {
        let constructed = Self {
            algo: Arc::new(Mutex::new(CryptoAlgo::Sha512(
                Box::new(sha2::Sha512::new()),
            ))),
        };

        match content {
            Some(inner) => constructed.update(inner.to_string()).clone(),
            None => constructed,
        }
    }

    pub fn update(&self, content: impl AsRef<[u8]>) -> &Crypto {
        (self.algo.lock().unwrap()).update(content);

        self
    }

    pub fn digest(&self, encoding: EncodingKind) -> Result<String> {
        (*self.algo.lock().unwrap()).digest(encoding)
    }
}

impl LuaUserData for Crypto {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("digest", |_, this, encoding| {
            this.digest(encoding).map_err(mlua::Error::runtime)
        });

        methods.add_method("update", |_, this, content: String| {
            Ok(this.update(content).clone())
        });
    }
}
