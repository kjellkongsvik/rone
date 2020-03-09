use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, DecodingKey, Validation};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Decoding<'a> {
    pub hs256: DecodingKey<'a>,
    pub rs256: HashMap<String, DecodingKey<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
}

pub struct JWT(());

fn bearer(r: &Request) -> Option<String> {
    let k: Vec<_> = r.headers().get("Authorization").collect();
    if k.len() == 1 {
        let mut it = k[0].split(' ');
        if Some("Bearer") == it.next() {
            return it.next().map(|t| t.into());
        }
    }
    None
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for JWT {
    type Error = jsonwebtoken::errors::ErrorKind;

    async fn from_request(
        request: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let token = match bearer(request) {
            Some(t) => t,
            None => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    ErrorKind::InvalidToken,
                ))
            }
        };

        let decoding_key = match request.guard::<State<Decoding>>().await {
            Outcome::Success(s) => s.hs256.to_owned(),
            _ => {
                return Outcome::Failure((
                    Status::InternalServerError,
                    ErrorKind::__Nonexhaustive,
                ))
            }
        };

        match decode::<Claims>(&token, &decoding_key, &Validation::default()) {
            Ok(_) => Outcome::Success(JWT(())),
            Err(e) => Outcome::Failure((Status::Unauthorized, e.into_kind())),
        }
    }
}
