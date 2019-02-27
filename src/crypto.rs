use crate::db::models::Account;
use failure::Error;
use openssl::rsa::Rsa;

const KEYSIZE: u32 = 4096;

pub struct DERKeypair {
    pub public:  Vec<u8>,
    pub private: Vec<u8>,
}

pub fn generate_keypair() -> Result<DERKeypair, Error> {
    let keypair = Rsa::generate(KEYSIZE)?;
    Ok(DERKeypair {
        public:  keypair.public_key_to_der()?,
        private: keypair.private_key_to_der()?,
    })
}

pub trait HasPublicKey {
    fn public_key_pem(&self) -> Result<String, Error>;
}

impl HasPublicKey for Account {
    fn public_key_pem(&self) -> Result<String, Error> {
        Ok(String::from_utf8(
            Rsa::public_key_from_der(&self.pubkey)?.public_key_to_pem()?,
        )?)
    }
}
