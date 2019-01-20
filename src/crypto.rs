use failure::Error;
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use types::crypto::DERKeypair;

const KEYSIZE: u32 = 4096;

pub fn generate_keypair() -> Result<DERKeypair, Error> {
    let keypair = Rsa::generate(KEYSIZE)?;
    Ok(DERKeypair {
        public:  keypair.public_key_to_der()?,
        private: keypair.private_key_to_der()?,
    })
}
