use crate::errors::ApiError;
use actix_web::http::header::Header;
use actix_web::HttpRequest;
use actix_web_httpauth::headers::authorization;
use anyhow::Context;
use itertools::Itertools;
use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::{decode, decode_header, jwk, Algorithm, DecodingKey, Validation};
use lazy_static::lazy_static;
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::configuration::config;
use subscribers::contracts::find_subscriber::SubscriberExternalId;

#[derive(Deserialize)]
struct AuthSettings {
    jwks: String,
    authorities: String,
    audiences: String,
}

#[derive(Deserialize)]
struct Settings {
    auth: AuthSettings,
}

lazy_static! {
    static ref SETTINGS: AuthSettings = config::<Settings>()
        .expect("Failed to unwrap settings")
        .auth;
    static ref JWKS: String = SETTINGS.jwks.to_owned();
    static ref AUTHORITIES: Vec<String> = SETTINGS
        .authorities
        .split(',')
        .map(|audience| audience.trim().to_owned())
        .collect_vec();
    static ref JWKS_SET: jwk::JwkSet = {
        let uri = &format!("https://{}/{}", JWKS.as_str(), ".well-known/jwks.json");
        let res: serde_json::Value = ureq::get(uri)
            .call()
            .unwrap_or_else(|_| panic!("Could not fetch the JWKS from {}", &uri))
            .into_json()
            .expect("Parsed into json");
        serde_json::from_value(res).unwrap()
    };
    static ref AUDIENCES: Vec<String> = SETTINGS
        .audiences
        .split(',')
        .map(|audience| audience.trim().to_owned())
        .collect_vec();
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: Vec<String>,
    exp: usize,
    iat: usize,
    iss: String,
    sub: String,
    permissions: Vec<String>,
}

#[derive(Debug)]
pub struct AuthenticatedUserInfo {
    #[allow(dead_code)]
    permissions: Vec<String>,
    pub(crate) external_id: SubscriberExternalId,
}

impl TryFrom<&HttpRequest> for AuthenticatedUserInfo {
    type Error = ApiError;

    fn try_from(req: &HttpRequest) -> Result<Self, Self::Error> {
        let token = authorization::Authorization::<authorization::Bearer>::parse(req)
            .context("Failed to extract bearer token")
            .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?;

        let token = token.as_ref().token().to_string();
        let header = decode_header(&token)
            .context("Failed to decode header")
            .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?;

        let kid = header.kid.ok_or_else(|| {
            ApiError::Unauthorized("Token does not have a `kid` header field".to_string())
        })?;

        let jwk = JWKS_SET.find(&kid).ok_or_else(|| {
            ApiError::Unauthorized("No matching JWK found for the given kid".to_string())
        })?;

        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                .context("Failed to get DecodingKey from rsa_components")
                .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?,
            _ => {
                return Err(ApiError::Unauthorized(
                    "Algorithm should be RSA".to_string(),
                ))
            }
        };

        let validation = {
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_issuer(
                &AUTHORITIES
                    .iter()
                    .map(|a| format!("https://{a}/"))
                    .collect::<Vec<_>>(),
            );
            validation.set_audience(&AUDIENCES);
            validation
        };

        let validated_token = decode::<Claims>(&token, &decoding_key, &validation)
            .context("Failed to decode token")
            .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?;

        let claims = validated_token.claims;

        let external_id = claims
            .sub
            .clone()
            .try_into()
            .map_err(ApiError::Unauthorized)?;

        Ok(AuthenticatedUserInfo {
            permissions: claims.permissions,
            external_id,
        })
    }
}
