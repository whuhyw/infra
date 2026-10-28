use gm_sm2::key::{Sm2Model, Sm2PublicKey};

use crate::{Error, Result};

pub(crate) struct Sm2CryptoService {
    public_key: Sm2PublicKey,
}

impl Sm2CryptoService {
    pub(crate) fn new(public_key_hex: &str) -> Result<Self> {
        let bytes =
            hex::decode(public_key_hex).map_err(|error| Error::PublicKey(error.to_string()))?;
        let public_key =
            Sm2PublicKey::new(&bytes).map_err(|error| Error::PublicKey(format!("{error:?}")))?;
        Ok(Self { public_key })
    }

    pub(crate) fn encrypt(&self, plaintext: &str) -> Result<String> {
        let ciphertext = self
            .public_key
            .encrypt(plaintext.as_bytes(), false, Sm2Model::C1C3C2)
            .map_err(|error| Error::PublicKey(format!("SM2 encryption failed: {error:?}")))?;
        Ok(hex::encode(ciphertext))
    }
}

#[cfg(test)]
mod tests {
    use super::Sm2CryptoService;

    #[test]
    fn encrypts_using_uncompressed_c1_c3_c2_format() {
        let crypto = Sm2CryptoService::new(
            "04b238b7d42c87a25e1a4eaddca81e8f33fd95773ccd471408e4195db62aa4085f6e0a3b695cad8d60acece1af348f534b7f72d312f2966b144248c9c590c930fc",
        )
        .unwrap();
        let ciphertext = crypto.encrypt("p1=test").unwrap();

        // Uncompressed C1 (65 bytes), SM3 digest (32 bytes), and encrypted payload.
        assert_eq!(ciphertext.len(), (65 + 32 + "p1=test".len()) * 2);
        assert!(ciphertext.starts_with("04"));
    }
}
