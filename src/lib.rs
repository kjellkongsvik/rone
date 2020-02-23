use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, DecodingKey, Validation};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, State};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Decoding<'a> {
    pub hs256: DecodingKey<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
}

pub struct JWT(());

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for JWT {
    type Error = jsonwebtoken::errors::ErrorKind;

    async fn from_request(
        request: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let secret = request
            .guard::<State<Decoding>>()
            .await
            .unwrap()
            .hs256
            .clone();
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        match keys.len() {
            1 => match decode::<Claims>(
                &keys[0].trim_start_matches("Bearer "),
                &secret,
                &Validation::default(),
            ) {
                Ok(_) => Outcome::Success(JWT(())),
                Err(e) => Outcome::Failure((Status::Unauthorized, e.into_kind())),
            },
            _ => Outcome::Failure((Status::Unauthorized, ErrorKind::InvalidToken)),
        }
    }
}
