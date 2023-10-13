use std::sync::Arc;

use crate::lune::builtins::{
    FromLua, Lua, LuaError, LuaResult, LuaUserData, LuaUserDataMethods, LuaValue,
};
use anyhow::Result;
use base64::{engine::general_purpose as Base64, Engine as _};
use digest::DynDigest;
use sha1::Digest as _;
// use ring::digest::{self, digest, Digest as RingDigest};
use std::sync::Mutex;

// TODO: Proper error handling, remove unwraps

// #[derive(Debug, Clone, Copy)]
// pub struct Crypto;
#[derive(Clone)]
pub struct Crypto {
    algo: Arc<Mutex<CryptoAlgo<(dyn digest::DynDigest + 'static)>>>,
}

#[derive(Clone)]
pub enum CryptoAlgo<T: ?Sized> {
    Sha1(Box<T>),
    Sha256(Box<T>),
    Sha512(Box<T>),
    Blake2(Box<T>),
    Md5(Box<T>),
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

impl CryptoAlgo<dyn DynDigest> {
    pub fn get_hasher(self) -> &'static dyn DynDigest {
        // TODO: Replace boilerplate using a macro

        match self {
            CryptoAlgo::Sha1(hasher) => &*hasher,
            CryptoAlgo::Sha256(hasher) => &*hasher,
            CryptoAlgo::Sha512(hasher) => &*hasher,
            CryptoAlgo::Blake2(hasher) => &*hasher,
            CryptoAlgo::Md5(hasher) => &*hasher,
        }
    }
}

impl Crypto {
    pub fn sha1<T: ToString>(content: Option<T>) -> Crypto {
        let content = content.map(|data| data.to_string());

        Self {
            algo: Arc::new(Mutex::new(CryptoAlgo::Sha1(DynDigest::box_clone(
                &sha1::Sha1::new(),
            )))),
        }
    }

    pub fn sha256<T: ToString>(content: Option<T>) -> Crypto {
        let content = content.map(|data| data.to_string());

        Self {
            algo: Arc::new(Mutex::new(CryptoAlgo::Sha256(DynDigest::box_clone(
                &sha2::Sha256::new(),
            )))),
        }
    }

    pub fn sha512<T: ToString>(content: Option<T>) -> Crypto {
        let content = content.map(|data| data.to_string());

        Self {
            algo: Arc::new(Mutex::new(CryptoAlgo::Sha512(DynDigest::box_clone(
                &sha2::Sha512::new(),
            )))),
        }
    }

    pub fn update(&self, content: impl AsRef<[u8]>) -> &Crypto {
        let mut binding = (*self.algo.lock().unwrap()).get_hasher().box_clone();
        let hasher = binding.as_mut();

        hasher.update(content.as_ref());

        self
    }

    pub fn digest(&self, encoding: EncodingKind) -> Result<String> {
        let algo = *self.algo.lock().unwrap();
        let hasher = algo.get_hasher();

        let computed = &*hasher.finalize_reset();

        match encoding {
            EncodingKind::Utf8 => String::from_utf8(computed.to_vec()).map_err(anyhow::Error::from),
            EncodingKind::Base64 => Ok(Base64::STANDARD.encode(computed)),
            EncodingKind::Hex => Ok(hex::encode::<&[u8]>(computed.as_ref())),
        }
    }
}

// impl FromLua<'_> for Crypto {
//     fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
//         if !value.is_table() {
//             return Err(LuaError::FromLuaConversionError {
//                 from: value.type_name(),
//                 to: "Crypto",
//                 message: Some("value must be a table".to_string()),
//             });
//         };

//         let value = value.as_table().unwrap();
//         let values = Self {
//             algo: value.get("value")?,
//         };

//         Ok(values)
//     }
// }

impl LuaUserData for &'static Crypto {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "update",
            |_, this, content: String| Ok(this.update(content)),
        );
    }
}

impl LuaUserData for Crypto {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("digest", |_, this, encoding| {
            this.digest(encoding)
                .map_err(|_| mlua::Error::external("whoopsie!"))
        });
    }
}
