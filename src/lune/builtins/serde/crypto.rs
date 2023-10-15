use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use base64::{engine::general_purpose as Base64, Engine as _};
use digest::Digest as _;
use mlua::prelude::*;

// TODO: Proper error handling, remove unwraps

macro_rules! impl_hash_algo {
    ($($algo:ident: $Type:ty),*) => {
        #[derive(Clone)]
        pub enum CryptoAlgo {
            $(
                $algo(Box<$Type>),
            )*
        }

        impl CryptoAlgo {
            pub fn update(&mut self, data: impl AsRef<[u8]>) {
                match self {
                    $(
                        Self::$algo(hasher) => hasher.update(data),
                    )*
                }
            }

            pub fn digest(&mut self, encoding: EncodingKind) -> Result<String> {
                let computed = match self {
                    $(
                        Self::$algo(hasher) => hasher.clone().finalize_reset().to_vec(),
                    )*
                };

                match encoding {
                    EncodingKind::Utf8 => String::from_utf8(computed).map_err(anyhow::Error::from),
                    EncodingKind::Base64 => Ok(Base64::STANDARD.encode(computed)),
                    EncodingKind::Hex => Ok(hex::encode(&computed)),
                }
            }
        }

        impl Crypto {
            $(
                paste::item! {
                    pub fn [<$algo:snake:lower>]<T: ToString>(content: Option<T>) -> Self {
                        let constructed = Self {
                            algo: Arc::new(Mutex::new(CryptoAlgo::$algo(Box::new($Type::new())))),
                        };

                        match content {
                            Some(inner) => constructed.update(inner.to_string()).clone(),
                            None => constructed,
                        }
                    }
                }
            )*
        }
    }
}

// Macro call creates the CryptoAlgo enum and implementations for it
// It also adds a method corresponding to the enum in the `Crypto` struct
impl_hash_algo! {
    Sha1: sha1::Sha1,
    Sha256: sha2::Sha256,
    Sha512: sha2::Sha512,
    Md5: md5::Md5,
    Blake2s256: blake2::Blake2s256,
    Blake2b512: blake2::Blake2b512,
    Sha3_256: sha3::Sha3_256,
    Sha3_512: sha3::Sha3_512
}

#[derive(Clone)]
pub struct Crypto {
    algo: Arc<Mutex<CryptoAlgo>>,
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

trait CryptoResult {
    fn update(&self, content: impl AsRef<[u8]>) -> &Self;
    fn digest(&self, encoding: EncodingKind) -> Result<String>;
}

impl CryptoResult for Crypto {
    fn update(&self, content: impl AsRef<[u8]>) -> &Crypto {
        (self.algo.lock().unwrap()).update(content);

        self
    }

    fn digest(&self, encoding: EncodingKind) -> Result<String> {
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
