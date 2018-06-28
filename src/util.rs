use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

pub enum Either<I, J> {
    Left(I),
    Right(J),
}

impl<'r, I, J> Responder<'r> for Either<I, J> where I: Responder<'r>, J: Responder<'r> {
    fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
        match self {
            Either::Left(responder) => responder.respond_to(request),
            Either::Right(responder) => responder.respond_to(request),
        }
    }
}