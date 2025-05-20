// use rustls::{Certificate, PrivateKey, ServerConfig};
// use rustls_pemfile::{certs, rsa_private_keys};
// fn gen_tls_config() {
//
//     let cert_chain: Vec<Certificate> = certs(cert_file)
//         .unwrap()
//         .into_iter()
//         .map(Certificate)
//         .collect();
//
//     let mut keys: Vec<PrivateKey> = rsa_private_keys(key_file)
//         .unwrap()
//         .into_iter()
//         .map(PrivateKey)
//         .collect();
//
//     let config = ServerConfig::builder()
//         .with_safe_defaults()
//         .with_no_client_auth()
//         .with_single_cert(cert_chain, keys.remove(0))
//         .expect("bad certificate/key");
//
//     let config = Arc::new(config);
// }