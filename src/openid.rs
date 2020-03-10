use reqwest::Error;
use serde::Deserialize;

pub fn get_rsa_components<'a>(uri: &str) -> Result<Jwk, Error> {
    Ok(get_json::<Jwk>(&get_json::<Oid>(uri)?.jwks_uri)?)
}

#[derive(Deserialize)]
struct Oid {
    jwks_uri: String,
}

#[derive(Deserialize)]
pub struct Keys {
    pub alg: String,
    pub e: String,
    pub n: String,
    pub kid: String,
}

#[derive(Deserialize)]
pub struct Jwk {
    pub keys: Vec<Keys>,
}

fn get_json<'a, T>(uri: &str) -> Result<T, Error>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    Ok(reqwest::blocking::get(uri)?.json::<T>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[test]
    fn jku() -> Result<(), Error> {
        let disc = "/.well-known/openid-configuration";
        let some_uri = reqwest::Url::parse(&server_url())
            .unwrap()
            .join("/jwks")
            .unwrap();
        let disc_body = format!(r#"{{"jwks_uri": "{}"}}"#, some_uri);
        let jwk_body = r#" { "keys": [ {
                        "alg": "RS256",
                        "e": "AQAB",
                        "n": "actually a big int base 64 encoded as a string",
                        "kid": "N" } ] } "#;

        serde_json::from_str::<Jwk>(&jwk_body).unwrap();

        let disc_mock = mock("GET", disc)
            .with_header("content-type", "application/json")
            .with_body(disc_body)
            .expect(1)
            .create();

        let jwk_mock = mock("GET", some_uri.path())
            .with_header("content-type", "application/json")
            .with_body(jwk_body)
            .expect(1)
            .create();

        assert_eq!(
            get_rsa_components(&(server_url() + disc))?
                .keys
                .first()
                .expect("1 key")
                .alg,
            "RS256"
        );

        jwk_mock.assert();
        disc_mock.assert();

        Ok(())
    }
}
