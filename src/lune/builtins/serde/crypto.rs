use crate::lune::builtins::{
    FromLua, Lua, LuaError, LuaResult, LuaUserData, LuaUserDataMethods, LuaValue,
};
use anyhow::Result;
use base64::{engine::general_purpose as Base64, Engine as _};
use ring::digest::{self, digest, Digest as RingDigest};

// TODO: Proper error handling, remove unwraps

#[derive(Debug, Clone, Copy)]
pub struct Crypto;
#[derive(Debug, Clone)]
pub struct CryptoResult<T, C>
where
    T: AsRef<[u8]>,
    C: AsRef<[u8]>,
{
    algo: CryptoAlgo,
    content: Option<T>,
    computed: Option<C>,
}

#[derive(Clone, Debug)]
pub enum CryptoAlgo {
    Sha1,
    Sha256,
    Sha512,
    // We shouldn't be able to Pass Hmac(Hmac), would there be a way to limit this?
    Hmac(Box<CryptoAlgo>),
    Md5,
}

impl Crypto {
    pub fn sha1<T: ToString>(content: Option<T>) -> CryptoResult<String, RingDigest> {
        let content = content.map(|data| data.to_string());

        CryptoResult {
            algo: CryptoAlgo::Sha1,
            content,
            computed: None,
        }
    }

    pub fn sha256<T: ToString>(content: Option<T>) -> CryptoResult<String, RingDigest> {
        let content = content.map(|data| data.to_string());

        CryptoResult {
            algo: CryptoAlgo::Sha256,
            content,
            computed: None,
        }
    }

    pub fn sha512<T: ToString>(content: Option<T>) -> CryptoResult<String, RingDigest> {
        let content = content.map(|data| data.to_string());

        CryptoResult {
            algo: CryptoAlgo::Sha512,
            content,
            computed: None,
        }
    }

    pub fn hmac<T: ToString>(
        content: Option<T>,
        algo: CryptoAlgo,
    ) -> CryptoResult<String, ring::hmac::Tag> {
        let content = content.map(|data| data.to_string());

        CryptoResult {
            algo: CryptoAlgo::Hmac(Box::new(algo)),
            content,
            computed: None,
        }
    }
}

trait FromCryptoAlgo {
    fn from_crypto_algo(value: CryptoAlgo) -> &'static Self;
}

impl FromCryptoAlgo for ring::digest::Algorithm {
    fn from_crypto_algo(value: CryptoAlgo) -> &'static Self {
        match &value {
            CryptoAlgo::Sha256 => &digest::SHA256,
            CryptoAlgo::Sha512 => &digest::SHA512,
            CryptoAlgo::Sha1 => &digest::SHA1_FOR_LEGACY_USE_ONLY,
            _ => panic!(),
        }
    }
}

impl From<CryptoAlgo> for ring::hmac::Algorithm {
    fn from(value: CryptoAlgo) -> Self {
        let val: ring::hmac::Algorithm = match value {
            CryptoAlgo::Hmac(algo) => match *algo {
                CryptoAlgo::Sha256 => ring::hmac::HMAC_SHA256,
                CryptoAlgo::Sha512 => ring::hmac::HMAC_SHA512,
                CryptoAlgo::Hmac(_) => panic!("Hmac(Hmac) is not allowed!"),
                // FIXME: We're match MD5 to SHA1 here, should fix
                CryptoAlgo::Sha1 => ring::hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
                CryptoAlgo::Md5 => todo!(),
            },
            _ => panic!("invalid type"),
        };

        val
    }
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

impl From<i32> for EncodingKind {
    fn from(value: i32) -> Self {
        Self::from(value as usize)
    }
}

impl From<f64> for EncodingKind {
    fn from(value: f64) -> Self {
        Self::from(value as usize)
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
            LuaValue::Integer(int) => Ok(EncodingKind::from(int)),
            LuaValue::Number(num) => Ok(EncodingKind::from(num)),
            LuaValue::String(str) => Ok(EncodingKind::from(str.to_string_lossy().to_string())),

            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "EncodingKind",
                message: Some("value must be a an Integer, Number or String".to_string()),
            }),
        }
    }
}

// Note that compute and digest declared here are identical to those of the below implementation
// Quite a bit of boilerplate, is there any way to avoid this without using derive macros?
// impl<T: AsRef<[u8]>, C: AsRef<[u8]>> CryptoResult<T, C> {
//     fn update(&mut self, content: String) -> Self {}
// }

impl CryptoResult<String, Vec<u8>> {
    pub fn update(&mut self, content: String) -> Self {
        self.content = Some(content);

        (*self).to_owned()
    }

    pub fn compute(&mut self) -> Self {
        let content = match &self.content {
            Some(inner) => inner.to_owned(),
            None => "".to_string(),
        };

        match self.algo {
            CryptoAlgo::Md5 => self.computed = Some(md5::compute(content).to_vec()),
            _ => panic!("Invalid implementation"),
        };

        (*self).to_owned()
    }

    pub fn digest(&self, encoding: EncodingKind) -> Result<String> {
        let computed = self.computed.clone().ok_or(anyhow::Error::msg(
            "compute the hash first before trying to obtain a digest",
        ))?;

        match encoding {
            EncodingKind::Utf8 => String::from_utf8(computed.to_vec()).map_err(anyhow::Error::from),
            EncodingKind::Base64 => Ok(Base64::STANDARD.encode(computed)),
            EncodingKind::Hex => Ok(hex::encode(computed.to_vec())),
        }
    }
}

impl CryptoResult<String, ring::hmac::Tag> {
    pub fn update(&mut self, content: String) -> Self {
        self.content = Some(content);

        (*self).to_owned()
    }

    fn compute(&mut self) -> Self {
        let content = match &self.content {
            Some(inner) => inner.to_owned(),
            None => "".to_string(),
        };

        match self.algo {
            CryptoAlgo::Hmac(_) => {
                let rng = ring::rand::SystemRandom::new();
                let key =
                    ring::hmac::Key::generate(ring::hmac::Algorithm::from(self.algo.clone()), &rng)
                        .expect("failed to generate random key");

                // we should probably return the key to the user too

                self.computed = Some(ring::hmac::sign(&key, content.as_bytes()));
            }
            _ => panic!("Invalid implementation"),
        };

        (*self).to_owned()
    }

    pub fn digest(&self, encoding: EncodingKind) -> Result<String> {
        let computed = self.computed.ok_or(anyhow::Error::msg(
            "compute the hash first before trying to obtain a digest",
        ))?;

        match encoding {
            EncodingKind::Utf8 => {
                String::from_utf8(computed.as_ref().to_vec()).map_err(anyhow::Error::from)
            }
            EncodingKind::Base64 => Ok(Base64::STANDARD.encode(computed)),
            EncodingKind::Hex => Ok(hex::encode(computed.as_ref())),
        }
    }
}

impl CryptoResult<String, RingDigest> {
    pub fn update(&mut self, content: String) -> Self {
        self.content = Some(content);

        (*self).to_owned()
    }

    pub fn compute(&mut self) -> Self {
        let content = match &self.content {
            Some(inner) => inner.to_owned(),
            None => "".to_string(),
        };

        match self.algo {
            CryptoAlgo::Sha256 | CryptoAlgo::Sha512 | CryptoAlgo::Sha1 => {
                self.computed = Some(digest(
                    ring::digest::Algorithm::from_crypto_algo(self.algo.clone()),
                    content.as_bytes(),
                ))
            }
            _ => unreachable!(),
        };

        (*self).to_owned()
    }

    pub fn digest(&self, encoding: EncodingKind) -> Result<String> {
        let computed = self.computed.ok_or(anyhow::Error::msg(
            "compute the hash first before trying to obtain a digest",
        ))?;

        match encoding {
            EncodingKind::Utf8 => {
                String::from_utf8(computed.as_ref().to_vec()).map_err(anyhow::Error::from)
            }
            EncodingKind::Base64 => Ok(Base64::STANDARD.encode(computed)),
            EncodingKind::Hex => Ok(hex::encode(computed.as_ref())),
        }
    }
}

trait HasComputedValue {
    fn update(&mut self, value: String) -> Self;
    fn compute(&mut self) -> Self;
    fn digest(&self, encoding: EncodingKind) -> Result<String>;
}
impl HasComputedValue for CryptoResult<String, Vec<u8>> {
    fn update(&mut self, content: String) -> Self {
        self.update(content)
    }

    fn compute(&mut self) -> Self {
        self.compute()
    }

    fn digest(&self, encoding: EncodingKind) -> Result<String> {
        self.digest(encoding)
    }
}
impl HasComputedValue for CryptoResult<String, RingDigest> {
    fn update(&mut self, content: String) -> Self {
        self.update(content)
    }

    fn compute(&mut self) -> Self {
        self.compute()
    }

    fn digest(&self, encoding: EncodingKind) -> Result<String> {
        self.digest(encoding)
    }
}
impl HasComputedValue for CryptoResult<String, ring::hmac::Tag> {
    fn update(&mut self, content: String) -> Self {
        self.update(content)
    }

    fn compute(&mut self) -> Self {
        self.compute()
    }

    fn digest(&self, encoding: EncodingKind) -> Result<String> {
        self.digest(encoding)
    }
}

fn register_methods<
    'lua,
    C: LuaUserData + HasComputedValue + 'static,
    M: LuaUserDataMethods<'lua, C>,
>(
    methods: &mut M,
) {
    methods.add_method_mut("update", |_, this, value| Ok(this.update(value)));
    methods.add_method_mut("compute", |_, this, ()| Ok(this.compute()));
    methods.add_method("digest", |_, this, encoding| {
        this.digest(encoding)
            .map_err(|_| mlua::Error::external("whoopsie!"))
    });
}

impl LuaUserData for CryptoResult<String, Vec<u8>> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        register_methods(methods);
    }
}

impl LuaUserData for CryptoResult<String, RingDigest> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        register_methods(methods);
    }
}

impl LuaUserData for CryptoResult<String, ring::hmac::Tag> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        register_methods(methods);
    }
}
