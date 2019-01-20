use failure::Error;
use openssl::pkey::Private;
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
