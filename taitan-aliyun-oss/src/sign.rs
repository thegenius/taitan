// use super::request::HttpRequest;
// use super::utils::{canonical, trim};
// use hmac::{Hmac, Mac};
// use reqwest::Body;
// use sha2::{Digest, Sha256};
// use std::borrow::Cow;

// type HmacSha256 = Hmac<Sha256>;

// pub struct Signer {}

// impl Signer {
//     pub fn authorization<'a>(
//         access_id: impl Into<Cow<'a, str>>,
//         secret: impl Into<Cow<'a, str>>,
//         signed_headers: impl Into<Cow<'a, str>>,
//         canonic_request: impl Into<Cow<'a, str>>,
//     ) -> String {
//         let signature = Signer::sign(secret, canonic_request);
//         format!(
//             "Authorization:{} Credential={},SignedHeaders={},Signature={}",
//             "ACS3-HMAC-SHA256",
//             access_id.into().as_ref(),
//             signed_headers.into().as_ref(),
//             signature
//         )
//     }

//     fn sign<'a>(
//         secret: impl Into<Cow<'a, str>>,
//         canonic_request: impl Into<Cow<'a, str>>,
//     ) -> String {
//         let sign_str = Signer::sign_string(canonic_request);
//         let mut hasher = HmacSha256::new_from_slice(secret.into().as_bytes())
//             .expect("HMAC can take key of any size");
//         hasher.update(sign_str.as_bytes());
//         let result = hasher.finalize().into_bytes();
//         hex::encode(result)
//     }

//     fn sign_string<'a>(canonic_request: impl Into<Cow<'a, str>>) -> String {
//         let mut hasher = Sha256::new();
//         hasher.update(canonic_request.into().as_bytes());
//         let hashed = hasher.finalize();
//         let hashed_canoical_req = hex::encode(hashed);
//         format!("ACS3-HMAC-SHA256\n{}", hashed_canoical_req)
//     }
// }

// pub struct HashedRequestPayload {
//     pub data: String,
// }

// impl HashedRequestPayload {
//     // HexEncode(Hash(payload))
//     // Hash must use SHA256
//     pub fn encode(body: impl Into<Body>) -> String {
//         let body: Body = body.into();
//         match body.as_bytes() {
//             None => String::default(),
//             Some(payload) => {
//                 let mut hasher = Sha256::new();
//                 hasher.update(payload);
//                 let hashed_payload = hasher.finalize();
//                 hex::encode(hashed_payload)
//             }
//         }
//     }
// }
// impl<T: Into<Body>> From<Option<T>> for HashedRequestPayload {
//     fn from(value: Option<T>) -> Self {
//         Self {
//             data: value.map_or(String::default(), Self::encode),
//         }
//     }
// }

// pub struct CanonicalRequest<'a> {
//     http_method: String,
//     canonical_uri: String,
//     canonical_query: String,
//     canonical_headers: CanonicalHeaders<'a>,
//     signed_headers: String,
//     hashed_payload: HashedRequestPayload,
// }

// impl<'a> CanonicalRequest<'a> {
//     pub fn format(&self) -> String {
//         format!("")
//     }
// }

// impl<'a, T: Into<Body>> From<HttpRequest<T>> for CanonicalRequest<'a> {
//     fn from(value: HttpRequest<T>) -> Self {
//         let payload: HashedRequestPayload = value.body.into();
//     }
// }
