use anyhow::Result;
use base64::{engine::general_purpose as Base64, Engine as _};
use ring::digest::{self, digest, Digest};

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
    Sha256,
    Sha512,
    // We shouldn't be able to Pass Hmac(Hmac), would there be a way to limit this?
    Hmac(Box<CryptoAlgo>),
    Md5,
}

impl Crypto {
    pub fn sha256<T>(content: Option<T>) -> CryptoResult<String, Digest>
    where
        T: ToString,
    {
        let content = content.map(|data| data.to_string());

        CryptoResult {
            algo: CryptoAlgo::Sha256,
            content,
            computed: None,
        }
    }

    pub fn sha512<T>(content: Option<T>) -> CryptoResult<String, Digest>
    where
        T: ToString,
    {
        let content = content.map(|data| data.to_string());

        CryptoResult {
            algo: CryptoAlgo::Sha512,
            content,
            computed: None,
        }
    }

    pub fn hmac<T>(content: Option<T>, algo: CryptoAlgo) -> CryptoResult<String, ring::hmac::Tag>
    where
        T: ToString,
    {
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

trait FromCryptoAlgoOwned {
    fn from_crypto_algo(value: CryptoAlgo) -> Self;
}

impl FromCryptoAlgo for ring::digest::Algorithm {
    fn from_crypto_algo(value: CryptoAlgo) -> &'static Self {
        match &value {
            CryptoAlgo::Sha256 => &digest::SHA256,
            CryptoAlgo::Sha512 => &digest::SHA512,
            _ => panic!(),
        }
    }
}

impl FromCryptoAlgoOwned for ring::hmac::Algorithm {
    fn from_crypto_algo(value: CryptoAlgo) -> Self {
        let val: ring::hmac::Algorithm = match value {
            CryptoAlgo::Hmac(algo) => match *algo {
                CryptoAlgo::Sha256 => ring::hmac::HMAC_SHA256,
                CryptoAlgo::Sha512 => ring::hmac::HMAC_SHA512,
                CryptoAlgo::Hmac(_) => panic!("Hmac(Hmac) is not allowed!"),
                CryptoAlgo::Md5 => ring::hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            },
            _ => panic!("invalid type"),
        };

        val
    }
}

pub enum EncodingKind {
    Utf8,
    Base64,
    Hex,
}

// Note that compute and digest declared here are identical to those of the below implementation
// Quite a bit of boilerplate, is there any way to avoid this without using derive macros?
impl CryptoResult<String, ring::hmac::Tag> {
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
            CryptoAlgo::Hmac(_) => {
                let rng = ring::rand::SystemRandom::new();
                let key = ring::hmac::Key::generate(
                    ring::hmac::Algorithm::from_crypto_algo(self.algo.clone()),
                    &rng,
                )
                .expect("failed to generate random key");

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

impl CryptoResult<String, Digest> {
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
            CryptoAlgo::Sha256 | CryptoAlgo::Sha512 => {
                self.computed = Some(digest(
                    ring::digest::Algorithm::from_crypto_algo(self.algo.clone()),
                    content.as_bytes(),
                ))
            }
            CryptoAlgo::Hmac(_) => unreachable!(),
            CryptoAlgo::Md5 => todo!(),
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

pub fn test() {
    println!(
        "{}",
        Crypto::sha256::<&str>(None /* or Some("some string!") */)
            .update("some string!".to_string())
            .compute()
            .digest(EncodingKind::Hex)
            .unwrap()
    );

    Crypto::hmac(Some("test"), CryptoAlgo::Sha256)
        .compute()
        .digest(EncodingKind::Base64)
        .unwrap();
}
