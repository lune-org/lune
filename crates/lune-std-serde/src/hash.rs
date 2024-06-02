use bstr::BString;
use md5::Md5;
use mlua::prelude::*;

use blake3::Hasher as Blake3;
use sha1::Sha1;
use sha2::{Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};

pub struct HashOptions {
    algorithm: HashAlgorithm,
    message: BString,
    secret: Option<BString>,
    // seed: Option<BString>,
}

#[derive(Debug, Clone, Copy)]
enum HashAlgorithm {
    Md5,
    Sha1,
    // SHA-2 variants
    Sha2_224,
    Sha2_256,
    Sha2_384,
    Sha2_512,
    // SHA-3 variants
    Sha3_224,
    Sha3_256,
    Sha3_384,
    Sha3_512,
    // Blake3
    Blake3,
}

impl HashAlgorithm {
    pub fn list_all_as_string() -> String {
        [
            "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3-224", "sha3-256",
            "sha3-384", "sha3-512", "blake3",
        ]
        .join(", ")
    }
}

impl HashOptions {
    /**
        Computes the hash for the `message` using whatever `algorithm` is
        contained within this struct.
    */
    #[inline]
    #[must_use = "hashing a message is useless without using the resulting hash"]
    pub fn hash(self) -> Vec<u8> {
        use digest::Digest;

        let message = self.message;
        match self.algorithm {
            HashAlgorithm::Md5 => Md5::digest(message).to_vec(),
            HashAlgorithm::Sha1 => Sha1::digest(message).to_vec(),
            HashAlgorithm::Sha2_224 => Sha224::digest(message).to_vec(),
            HashAlgorithm::Sha2_256 => Sha256::digest(message).to_vec(),
            HashAlgorithm::Sha2_384 => Sha384::digest(message).to_vec(),
            HashAlgorithm::Sha2_512 => Sha512::digest(message).to_vec(),

            HashAlgorithm::Sha3_224 => Sha3_224::digest(message).to_vec(),
            HashAlgorithm::Sha3_256 => Sha3_256::digest(message).to_vec(),
            HashAlgorithm::Sha3_384 => Sha3_384::digest(message).to_vec(),
            HashAlgorithm::Sha3_512 => Sha3_512::digest(message).to_vec(),

            HashAlgorithm::Blake3 => Blake3::digest(message).to_vec(),
        }
    }

    /**
        Computes the HMAC for the `message` using whatever `algorithm` and
        `secret` are contained within this struct.

        # Errors

        If the `secret` is not provided or is otherwise invalid.
    */
    #[inline]
    pub fn hmac(self) -> LuaResult<Vec<u8>> {
        use hmac::{Hmac, Mac, SimpleHmac};

        let secret = self
            .secret
            .ok_or_else(|| LuaError::FromLuaConversionError {
                from: "nil",
                to: "string or buffer",
                message: Some("Argument #3 missing or nil".to_string()),
            })?;

        /*
            These macros exist to remove what would ultimately be dozens of
            repeating lines. Essentially, there's several step to processing
            HMacs, which expands into the 3 lines you see below. However,
            the Hmac struct is specialized towards eager block-based processes.
            In order to support anything else, like blake3, there's a second
            type named `SimpleHmac`. This results in duplicate macros like
            there are below.
        */
        macro_rules! hmac {
            ($Type:ty) => {{
                let mut mac: Hmac<$Type> = Hmac::new_from_slice(&secret).into_lua_err()?;
                mac.update(&self.message);
                Ok(mac.finalize().into_bytes().to_vec())
            }};
        }
        macro_rules! hmac_no_blocks {
            ($Type:ty) => {{
                let mut mac: SimpleHmac<$Type> =
                    SimpleHmac::new_from_slice(&secret).into_lua_err()?;
                mac.update(&self.message);
                Ok(mac.finalize().into_bytes().to_vec())
            }};
        }

        match self.algorithm {
            HashAlgorithm::Md5 => hmac!(Md5),
            HashAlgorithm::Sha1 => hmac!(Sha1),

            HashAlgorithm::Sha2_224 => hmac!(Sha224),
            HashAlgorithm::Sha2_256 => hmac!(Sha256),
            HashAlgorithm::Sha2_384 => hmac!(Sha384),
            HashAlgorithm::Sha2_512 => hmac!(Sha512),

            HashAlgorithm::Sha3_224 => hmac!(Sha3_224),
            HashAlgorithm::Sha3_256 => hmac!(Sha3_256),
            HashAlgorithm::Sha3_384 => hmac!(Sha3_384),
            HashAlgorithm::Sha3_512 => hmac!(Sha3_512),

            HashAlgorithm::Blake3 => hmac_no_blocks!(Blake3),
        }
    }
}

impl<'lua> FromLua<'lua> for HashAlgorithm {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::String(str) = value {
            /*
                Casing tends to vary for algorithms, so rather than force
                people to remember it we'll just accept any casing.
            */
            let str = str.to_str()?.to_ascii_lowercase();
            match str.as_str() {
                "md5" => Ok(Self::Md5),
                "sha1" => Ok(Self::Sha1),

                "sha224" => Ok(Self::Sha2_224),
                "sha256" => Ok(Self::Sha2_256),
                "sha384" => Ok(Self::Sha2_384),
                "sha512" => Ok(Self::Sha2_512),

                "sha3-224" => Ok(Self::Sha3_224),
                "sha3-256" => Ok(Self::Sha3_256),
                "sha3-384" => Ok(Self::Sha3_384),
                "sha3-512" => Ok(Self::Sha3_512),

                "blake3" => Ok(Self::Blake3),

                _ => Err(LuaError::FromLuaConversionError {
                    from: "string",
                    to: "HashAlgorithm",
                    message: Some(format!(
                        "Invalid hashing algorithm '{str}', valid kinds are:\n{}",
                        HashAlgorithm::list_all_as_string()
                    )),
                }),
            }
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "HashAlgorithm",
                message: None,
            })
        }
    }
}

impl<'lua> FromLuaMulti<'lua> for HashOptions {
    fn from_lua_multi(mut values: LuaMultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let algorithm = values
            .pop_front()
            .map(|value| HashAlgorithm::from_lua(value, lua))
            .transpose()?
            .ok_or_else(|| LuaError::FromLuaConversionError {
                from: "nil",
                to: "HashAlgorithm",
                message: Some("Argument #1 missing or nil".to_string()),
            })?;
        let message = values
            .pop_front()
            .map(|value| BString::from_lua(value, lua))
            .transpose()?
            .ok_or_else(|| LuaError::FromLuaConversionError {
                from: "nil",
                to: "string or buffer",
                message: Some("Argument #2 missing or nil".to_string()),
            })?;
        let secret = values
            .pop_front()
            .map(|value| BString::from_lua(value, lua))
            .transpose()?;
        // let seed = values
        //     .pop_front()
        //     .map(|value| BString::from_lua(value, lua))
        //     .transpose()?;

        Ok(HashOptions {
            algorithm,
            message,
            secret,
            // seed,
        })
    }
}
