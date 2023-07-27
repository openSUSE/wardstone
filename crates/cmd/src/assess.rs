use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::ValueEnum;
use once_cell::sync::Lazy;
use openssl::nid::Nid;
use openssl::pkey::Id;
use openssl::x509::X509;
use wardstone_core::primitive::ecc::*;
use wardstone_core::primitive::hash::*;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Guide {
  /// The BSI TR-02102 series of technical guidelines.
  Bsi,
}

// Maintains a mapping of identifiers and their wardstone_core
// equivalents.
static ELLIPTIC_CURVES: Lazy<HashMap<&str, Ecc>> = Lazy::new(|| {
  let mut m = HashMap::new();
  m.insert("SM2", SM2);
  m.insert("brainpoolP160r1", BRAINPOOLP160R1);
  m.insert("brainpoolP160t1", BRAINPOOLP160T1);
  m.insert("brainpoolP192r1", BRAINPOOLP192R1);
  m.insert("brainpoolP192t1", BRAINPOOLP192T1);
  m.insert("brainpoolP224r1", BRAINPOOLP224R1);
  m.insert("brainpoolP224t1", BRAINPOOLP224T1);
  m.insert("brainpoolP256r1", BRAINPOOLP256R1);
  m.insert("brainpoolP256t1", BRAINPOOLP256T1);
  m.insert("brainpoolP320r1", BRAINPOOLP320R1);
  m.insert("brainpoolP320t1", BRAINPOOLP320T1);
  m.insert("brainpoolP384r1", BRAINPOOLP384R1);
  m.insert("brainpoolP384t1", BRAINPOOLP384T1);
  m.insert("brainpoolP512r1", BRAINPOOLP512R1);
  m.insert("brainpoolP512t1", BRAINPOOLP512T1);
  m.insert("c2pnb163v1", C2PNB163V1);
  m.insert("c2pnb163v2", C2PNB163V2);
  m.insert("c2pnb163v3", C2PNB163V3);
  // TODO:
  // m.insert("c2pnb176v1", C2PNB176V1);
  m.insert("c2pnb208w1", C2PNB208W1);
  m.insert("c2pnb272w1", C2PNB272W1);
  m.insert("c2pnb304w1", C2PNB304W1);
  m.insert("c2pnb368w1", C2PNB368W1);
  m.insert("c2tnb191v1", C2TNB191V1);
  m.insert("c2tnb191v2", C2TNB191V2);
  m.insert("c2tnb191v3", C2TNB191V3);
  m.insert("c2tnb239v1", C2TNB239V1);
  m.insert("c2tnb239v2", C2TNB239V2);
  m.insert("c2tnb239v3", C2TNB239V3);
  m.insert("c2tnb359v1", C2TNB359V1);
  m.insert("c2tnb431r1", C2TNB431R1);
  m.insert("ed25519", ED25519);
  m.insert("ed448", ED448);
  m.insert("prime192v1", PRIME192V1);
  m.insert("prime192v2", PRIME192V2);
  m.insert("prime192v3", PRIME192V3);
  m.insert("prime239v1", PRIME239V1);
  m.insert("prime239v2", PRIME239V2);
  m.insert("prime239v3", PRIME239V3);
  m.insert("prime256v1", PRIME256V1);
  m.insert("secp112r1", SECP112R1);
  m.insert("secp112r2", SECP112R2);
  m.insert("secp128r1", SECP128R1);
  m.insert("secp128r2", SECP128R2);
  // TODO:
  // m.insert("secp160k1", SECP160K1);
  m.insert("secp160r1", SECP160R1);
  m.insert("secp160r2", SECP160R2);
  m.insert("secp192k1", SECP192K1);
  m.insert("secp224k1", SECP224K1);
  m.insert("secp224r1", SECP224R1);
  m.insert("secp256k1", SECP256K1);
  m.insert("secp384r1", SECP384R1);
  m.insert("secp521r1", SECP521R1);
  m.insert("sect113r1", SECT113R1);
  m.insert("sect113r2", SECT113R2);
  m.insert("sect131r1", SECT131R1);
  m.insert("sect131r2", SECT131R2);
  m.insert("sect163k1", SECT163K1);
  m.insert("sect163r1", SECT163R1);
  m.insert("sect163r2", SECT163R2);
  m.insert("sect193r1", SECT193R1);
  m.insert("sect193r2", SECT193R2);
  m.insert("sect233k1", SECT233K1);
  m.insert("sect233r1", SECT233R1);
  m.insert("sect239k1", SECT239K1);
  // TODO:
  // m.insert("sect283k1", SECT283K1);
  m.insert("sect283r1", SECT283R1);
  m.insert("sect409k1", SECT409K1);
  m.insert("sect409r1", SECT409R1);
  m.insert("sect571k1", SECT571K1);
  m.insert("sect571r1", SECT571R1);
  m.insert("wap-wsg-idm-ecid-wtls1", WAP_WSG_IDM_ECID_WTLS1);
  m.insert("wap-wsg-idm-ecid-wtls10", WAP_WSG_IDM_ECID_WTLS10);
  m.insert("wap-wsg-idm-ecid-wtls11", WAP_WSG_IDM_ECID_WTLS11);
  m.insert("wap-wsg-idm-ecid-wtls12", WAP_WSG_IDM_ECID_WTLS12);
  m.insert("wap-wsg-idm-ecid-wtls3", WAP_WSG_IDM_ECID_WTLS3);
  m.insert("wap-wsg-idm-ecid-wtls4", WAP_WSG_IDM_ECID_WTLS4);
  m.insert("wap-wsg-idm-ecid-wtls5", WAP_WSG_IDM_ECID_WTLS5);
  m.insert("wap-wsg-idm-ecid-wtls6", WAP_WSG_IDM_ECID_WTLS6);
  m.insert("wap-wsg-idm-ecid-wtls7", WAP_WSG_IDM_ECID_WTLS7);
  m.insert("wap-wsg-idm-ecid-wtls8", WAP_WSG_IDM_ECID_WTLS8);
  m.insert("wap-wsg-idm-ecid-wtls9", WAP_WSG_IDM_ECID_WTLS9);
  m
});

struct Certificate(X509);

impl Certificate {
  pub fn from_file(path: &PathBuf) -> Certificate {
    let mut file = File::open(path).expect("open certificate");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("read file");
    let certificate = X509::from_pem(&bytes).expect("PEM encoded X509 certificate");
    Self(certificate)
  }

  // TODO: The return type could also be a generic type encompassing all
  // supported signature algorithms.
  pub fn key(&self) -> Option<&Ecc> {
    let public_key = self.0.public_key().expect("public key");
    match public_key.id() {
      Id::EC => {
        let key = public_key.ec_key().expect("elliptic curve key");
        let id = key.group().curve_name().expect("curve name");
        CORE_INSTANCES.get(&id)
      },
      _ => todo!(),
    }
  }
}

pub fn x509(path: &PathBuf, _against: &Guide) {
  let certificate = Certificate::from_file(path);
  let key = certificate.key();
  // TODO: Validate.
  println!("debug: validate key: {:?}", key)
}
