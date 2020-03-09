use jsonwebtoken::DecodingKey;
use reqwest::Error;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq)]
struct Oid {
    jwks_uri: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Keys {
    e: String,
    n: String,
    alg: String,
    kid: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Jwk {
    keys: Vec<Keys>,
}

fn keys<'a, T>(uri: &str) -> Result<T, Error>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    println!("AAAA {}", uri);
    Ok(reqwest::blocking::get(uri)?.json::<T>()?)
}

pub fn ten<'a>(uri: &str) -> Result<HashMap<String, DecodingKey<'a>>, Error> {
    let mut hm = HashMap::new();

    println!("YYYY {}", uri);
    let re = en(uri)?;
    println!("XXXX: {:?}", re);

    for r in re.iter() {
        hm.insert(
            r.kid.clone(),
            DecodingKey::from_rsa_components(
                Box::leak(r.n.clone().into_boxed_str()),
                Box::leak(r.e.clone().into_boxed_str()),
            ),
        );
    }
    Ok(hm)
}

fn en(uri: &str) -> Result<Vec<Keys>, Error> {
    Ok(keys::<Jwk>(&keys::<Oid>(uri)?.jwks_uri)?
        .keys
        .into_iter()
        .filter(|k| k.alg == "RS256")
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[test]
    fn e_n() -> Result<(), Error> {
        let disc = "/.well-known/openid-configuration";
        let some_uri = reqwest::Url::parse(&server_url())
            .unwrap()
            .join("/jwks")
            .unwrap();
        let disc_body = format!(r#"{{"jwks_uri": "{}"}}"#, some_uri);
        let jwk_body = r#"
            {{
                "keys": [
                    {{  "alg": "RS256",
                        "kty": "RSA",
                        "e": "AQAB",
                        "n": "yeNlzlub94Y",
                        "kid": "NjVBRjY5MDlCMUIwNzU4RTA2QzZFMDQ4QzQ2MDAyQjVDNjk1RTM2Qg",
                    }}
                ]
            }}
            "#;

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

        let u = &reqwest::Url::parse(&server_url())
            .unwrap()
            .join(disc)
            .unwrap()
            .to_string();
        let t = ten(u).unwrap();
        assert_eq!(t.len(), 1);

        jwk_mock.assert();
        disc_mock.assert();

        Ok(())
    }
}
