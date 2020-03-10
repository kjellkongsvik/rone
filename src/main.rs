#![feature(proc_macro_hygiene, decl_macro)]
use jsonwebtoken::DecodingKey;
use jwt::{Decoding, JWT};
use rocket::fairing::AdHoc;
use rocket::{get, routes, Rocket};
use std::collections::HashMap;
use std::env;

mod jwt;
mod openid;

#[macro_use]
extern crate lazy_static;

#[get("/")]
fn index(_jwt: JWT) -> String {
    "".to_owned()
}

fn rocket(
    secret_key: &'static [u8],
    rsa_keys: Option<&'static HashMap<String, DecodingKey<'static>>>,
) -> Rocket {
    rocket::ignite()
        .mount("/", routes![index])
        .attach(AdHoc::on_attach("DecodingKey", move |r| {
            Ok(r.manage(Decoding {
                hs256: Some(DecodingKey::from_secret(secret_key)),
                rs256: rsa_keys,
            }))
        }))
}

fn main() -> Result<(), rocket::error::Error> {
    lazy_static! {
        static ref SECRET_KEY: String =
            env::var("SECRET_KEY").expect("SECRET_KEY in env");
        static ref RSA_COMPONENTS: HashMap<String, DecodingKey<'static>> =
            openid::get_rsa_components(
                &env::var("AUTHSERVER").expect("AUTHSERVER in env")
            )
            .unwrap()
            .keys
            .into_iter()
            .filter(|k| k.alg == "RS256")
            .fold(HashMap::new(), |mut hm, r| {
                hm.insert(
                    r.kid.clone(),
                    DecodingKey::from_rsa_components(
                        Box::leak(r.n.clone().into_boxed_str()),
                        Box::leak(r.e.clone().into_boxed_str()),
                    ),
                );
                hm
            });
    }
    rocket(SECRET_KEY.as_ref(), Some(&RSA_COMPONENTS)).launch()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::Header as jwtHeader;
    use jsonwebtoken::{encode, EncodingKey};
    use jwt::Claims;
    use rocket::http::{Header, Status};
    use rocket::local::Client;

    fn secret_key() -> &'static [u8] {
        lazy_static! {
            static ref SECRET_KEY: String = "very_secret".to_string();
        }
        SECRET_KEY.as_ref()
    }

    fn jwt() -> String {
        let my_claims = Claims {
            exp: 10_000_000_000,
        };
        encode(
            &jwtHeader::default(),
            &my_claims,
            &EncodingKey::from_secret(secret_key()),
        )
        .unwrap()
    }

    #[rocket::async_test]
    async fn test_401() {
        let client = Client::new(rocket(secret_key(), None)).unwrap();
        let response = client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn test_200() {
        let client = Client::new(rocket(secret_key(), None)).unwrap();
        let response = client
            .get("/")
            .header(Header::new("Authorization", "Bearer ".to_owned() + &jwt()))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }
}
