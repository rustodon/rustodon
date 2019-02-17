use base32::{decode, encode, Alphabet};

use rocket::http::{RawStr, Status};
use rocket::request::{FromParam, Request};
use rocket::response::{Responder, Response};

pub enum Either<I, J> {
    Left(I),
    Right(J),
}

impl<'r, I, J> Responder<'r> for Either<I, J>
where
    I: Responder<'r>,
    J: Responder<'r>,
{
    fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
        match self {
            Either::Left(responder) => responder.respond_to(request),
            Either::Right(responder) => responder.respond_to(request),
        }
    }
}

pub fn base32_to_u64(input: String) -> Result<u64, String> {
    let vec_bytes = match decode(Alphabet::Crockford, &input[..]) {
        Some(bytes) => bytes,
        None => return Err(format!("failed to decode '{}' from base32", input)),
    };

    if vec_bytes.len() != 8 {
        return Err(format!("'{}' did not decode with 8 bytes", input));
    }

    let mut bytes: [u8; 8] = [0; 8];

    for i in 0..8 {
        bytes[i] = vec_bytes[i];
    }

    Ok(u64::from_le_bytes(bytes))
}

pub fn u64_to_base32(input: u64) -> String {
    let mut bytes: [u8; 8] = [0; 8];
    for i in 0..8 {
        let mask: u64 = 0x00000000000000FF << (i * 8);
        let byte: u8 = ((input & mask) >> (i * 8)) as u8;

        bytes[i] = byte;
    }

    encode(Alphabet::Crockford, &bytes)
}

pub struct StatusID(pub u64);

impl<'r> FromParam<'r> for StatusID {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let status_id = match param.parse::<u64>() {
            Ok(status_id) => status_id,
            Err(_) => match base32_to_u64(param.to_string()) {
                Ok(status_id) => status_id,
                Err(_) => return Err(param),
            },
        };
        println!("status id: {}", status_id);
        Ok(StatusID(status_id))
    }
}
