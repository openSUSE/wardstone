//! Validate cryptographic primitives against the [NIST Special
//! Publication 800-57 Part 1 Revision 5 standard].
//!
//! # Safety
//!
//! This module contains functions that use raw pointers as arguments
//! for reading and writing data. However, this is only for the C API
//! that is exposed to interact with safe Rust equivalents. The C API is
//! essentially a wrapper around the Rust function to maintain
//! consistency with existing conventions.
//!
//! Checks against null dereferences are made in which the function will
//! return `-1` if the argument is required.
//!
//! [NIST Special Publication 800-57 Part 1 Revision 5 standard]: https://doi.org/10.6028/NIST.SP.800-57pt1r5

use std::collections::HashSet;
use std::ffi::c_int;

use lazy_static::lazy_static;

use crate::context::Context;
use crate::primitives::ffc::{Ffc, FFC_15360_512, FFC_2048_224, FFC_3072_256, FFC_7680_384};
use crate::primitives::hash::{
  Hash, SHA1, SHA224, SHA256, SHA384, SHA3_224, SHA3_256, SHA3_384, SHA3_512, SHA512, SHA512_224,
  SHA512_256,
};
use crate::primitives::symmetric::{Symmetric, AES128, AES192, AES256, TDEA2, TDEA3};

const CUTOFF_YEAR: u16 = 2031;
const CUTOFF_YEAR_3TDEA: u16 = 2023;

lazy_static! {
  static ref SPECIFIED_HASH: HashSet<u16> = {
    let mut s = HashSet::new();
    s.insert(SHA1.id);
    s.insert(SHA224.id);
    s.insert(SHA256.id);
    s.insert(SHA384.id);
    s.insert(SHA3_224.id);
    s.insert(SHA3_256.id);
    s.insert(SHA3_384.id);
    s.insert(SHA3_512.id);
    s.insert(SHA512.id);
    s.insert(SHA512_224.id);
    s.insert(SHA512_256.id);
    s
  };
  static ref SPECIFIED_SYMMETRIC: HashSet<u16> = {
    let mut s = HashSet::new();
    s.insert(TDEA2.id);
    s.insert(TDEA3.id);
    s.insert(AES128.id);
    s.insert(AES192.id);
    s.insert(AES256.id);
    s
  };
}

/// Validates a finite field cryptography primitive function examples
/// which include DSA and key establishment algorithms such as
/// Diffie-Hellman and MQV according to page 54-55 of the standard.
///
/// If the key is not compliant then `Err` will contain the recommended
/// primitive that one should use instead.
///
/// If the key is compliant but the context specifies a higher security
/// level, `Ok` will also hold the recommended primitive with the
/// desired security level.
///
/// **Note:** Unlike other functions in this module, this will return a
/// generic structure that specifies minimum private and public key
/// sizes.
///
/// # Example
///
/// The following illustrates a call to validate a compliant key.
///
/// ```
/// use wardstone::context::Context;
/// use wardstone::primitives::ffc::FFC_2048_224;
/// use wardstone::standards::nist;
///
/// let ctx = Context::default();
/// assert_eq!(nist::validate_ffc(&ctx, &FFC_2048_224), Ok(FFC_2048_224));
pub fn validate_ffc(ctx: &Context, key: &Ffc) -> Result<Ffc, Ffc> {
  match key {
    Ffc {
      l: ..=2047,
      n: ..=223,
    } => Err(FFC_2048_224),
    Ffc { l: 2048, n: 224 } => {
      if ctx.year() > CUTOFF_YEAR {
        Err(FFC_3072_256)
      } else {
        Ok(FFC_2048_224)
      }
    },
    Ffc {
      l: 2049..=3072,
      n: 225..=256,
    } => Ok(FFC_3072_256),
    Ffc {
      l: 3073..=7680,
      n: 257..=384,
    } => Ok(FFC_7680_384),
    Ffc {
      l: 7681..,
      n: 385..,
    } => Ok(FFC_15360_512),
    _ => Err(FFC_2048_224),
  }
}

/// Validates a hash function according to page 56 of the standard. The
/// reference is made with regards to applications that require
/// collision resistance such as digital signatures.
///
/// For applications that primarily require pre-image resistance such as
/// message authentication codes (MACs), key derivation functions
/// (KDFs), and random bit generation use
/// [`validate_hash_based`](crate::standards::nist::validate_hash_based).
///
///
/// If the hash function is not compliant then `Err` will contain the
/// recommended primitive that one should use instead.
///
/// If the hash function is compliant but the context specifies a higher
/// security level, `Ok` will also hold the recommended primitive with
/// the desired security level.
///
/// **Note:** that this means an alternative might be suggested for a
/// compliant hash functions with a similar security level in which a
/// switch to the recommended primitive would likely be unwarranted. For
/// example, when evaluating compliance for the `SHA3-256`, a
/// recommendation to use `SHA256` will be made but switching to this as
/// a result is likely unnecessary.
///
/// **Caution:** The default recommendation is from the SHA2 family.
/// While this is safe for most use cases, it is generally not
/// recommended for hashing secrets given its lack of resistance against
/// length extension attacks.
///
/// # Example
///
/// The following illustrates a call to validate a non-compliant hash
/// function.
///
/// ```
/// use wardstone::context::Context;
/// use wardstone::primitives::hash::{SHA1, SHA224};
/// use wardstone::standards::nist;
///
/// let ctx = Context::default();
/// assert_eq!(nist::validate_hash(&ctx, &SHA1), Err(SHA224));
pub fn validate_hash(ctx: &Context, hash: &Hash) -> Result<Hash, Hash> {
  if SPECIFIED_HASH.contains(&hash.id) {
    let security = ctx.security().max(hash.collision_resistance());
    match security {
      ..=111 => {
        if ctx.year() > CUTOFF_YEAR {
          Err(SHA256)
        } else {
          Err(SHA224)
        }
      },
      112 => {
        if ctx.year() > CUTOFF_YEAR {
          Err(SHA256)
        } else {
          Ok(SHA224)
        }
      },
      113..=128 => Ok(SHA256),
      129..=192 => Ok(SHA384),
      193.. => Ok(SHA512),
    }
  } else {
    Err(SHA256)
  }
}

/// Validates a hash function according to page 56 of the standard. The
/// reference is made with regards to applications that primarily
/// require pre-image resistance such as message authentication codes
/// (MACs), key derivation functions (KDFs), and random bit generation.
///
/// For applications that require collision resistance such digital
/// signatures use
/// [`validate_hash`](crate::standards::nist::validate_hash).
///
/// If the hash function is not compliant then `Err` will contain the
/// recommended primitive that one should use instead.
///
/// If the hash function is compliant but the context specifies a higher
/// security level, `Ok` will also hold the recommended primitive with
/// the desired security level.
///
/// **Note:** that this means an alternative might be suggested for a
/// compliant hash functions with a similar security level in which a
/// switch to the recommended primitive would likely be unwarranted. For
/// example, when evaluating compliance for the `SHA3-256`, a
/// recommendation to use `SHA256` will be made but switching to this as
/// a result is likely unnecessary.
///
/// **Caution:** The default recommendation is from the SHA2 family.
/// While this is safe for most use cases, it is generally not
/// recommended for hashing secrets given its lack of resistance against
/// length extension attacks.
///
/// # Example
///
/// The following illustrates a call to validate a non-compliant hash
/// function.
///
/// ```
/// use wardstone::context::Context;
/// use wardstone::primitives::hash::{SHA1, SHA224};
/// use wardstone::standards::nist;
///
/// let ctx = Context::default();
/// assert_eq!(nist::validate_hash_based(&ctx, &SHA1), Err(SHA224));
pub fn validate_hash_based(ctx: &Context, hash: &Hash) -> Result<Hash, Hash> {
  if SPECIFIED_HASH.contains(&hash.id) {
    let security = ctx.security().max(hash.pre_image_resistance());
    match security {
      ..=111 => Err(SHA224),
      112..=127 => {
        if ctx.year() > CUTOFF_YEAR {
          Err(SHA224)
        } else {
          Ok(SHA224)
        }
      },
      128..=224 => Ok(SHA224),
      225..=256 => Ok(SHA256),
      257..=394 => Ok(SHA384),
      395.. => Ok(SHA512),
    }
  } else {
    Err(SHA224)
  }
}

/// Validates a symmetric key primitive according to pages 54-55 of the
/// standard.
///
/// If the key is not compliant then `Err` will contain the recommended
/// primitive that one should use instead.
///
/// If the hash function is compliant but the context specifies a higher
/// security level, `Ok` will also hold the recommended primitive with
/// the desired security level.
///
/// # Example
///
/// The following illustrates a call to validate a three-key Triple DES
/// key (which is deprecated through the year 2023).
///
/// ```
/// use wardstone::context::Context;
/// use wardstone::primitives::symmetric::{AES128, TDEA3};
/// use wardstone::standards::nist;
///
/// let ctx = Context::default();
/// assert_eq!(nist::validate_symmetric(&ctx, &TDEA3), Ok(AES128));
/// ```
pub fn validate_symmetric(ctx: &Context, key: &Symmetric) -> Result<Symmetric, Symmetric> {
  if SPECIFIED_SYMMETRIC.contains(&key.id) {
    match key.security {
      ..=111 => Err(AES128),
      112 => {
        // See SP 800-131Ar2 p. 7.
        let cutoff = if key.id == TDEA3.id {
          CUTOFF_YEAR_3TDEA
        } else {
          CUTOFF_YEAR
        };
        if ctx.year() > cutoff {
          Err(AES128)
        } else {
          Ok(AES128)
        }
      },
      113..=128 => Ok(AES128),
      129..=192 => Ok(AES192),
      193.. => Ok(AES256),
    }
  } else {
    Err(AES128)
  }
}

// This function abstracts a call to a Rust function `f` and returns a
// result following C error handling conventions.
unsafe fn c_call<T>(
  f: fn(&Context, &T) -> Result<T, T>,
  ctx: *const Context,
  primitive: *const T,
  alternative: *mut T,
) -> c_int {
  if ctx.is_null() || primitive.is_null() {
    return -1;
  }

  let (recommendation, is_compliant) = match f(ctx.as_ref().unwrap(), primitive.as_ref().unwrap()) {
    Ok(recommendation) => (recommendation, true),
    Err(recommendation) => (recommendation, false),
  };

  if !alternative.is_null() {
    *alternative = recommendation;
  }

  is_compliant as c_int
}

/// Validates a finite field cryptography primitive function examples
/// which include DSA and key establishment algorithms such as
/// Diffie-Hellman and MQV according to page 54-55 of the standard.
///
/// If the key is not compliant then `struct ws_ffc*` will point to the
/// recommended primitive that one should use instead.
///
/// If the key is compliant but the context specifies a higher security
/// level, `struct ws_ffc` will also point to the recommended primitive
/// with the desired security level.
///
/// The function returns 1 if the key is compliant, 0 if it is not, and
/// -1 if an error occurs as a result of a missing or invalid argument.
///
/// **Note:** Unlike other functions in this module, this will return a
/// generic structure that specifies minimum private and public key
/// sizes.
///
/// # Safety
///
/// See module documentation for comment on safety.
#[no_mangle]
pub unsafe extern "C" fn ws_nist_validate_ffc(
  ctx: *const Context,
  key: *const Ffc,
  alternative: *mut Ffc,
) -> c_int {
  c_call(validate_ffc, ctx, key, alternative)
}

/// Validates a hash function according to page 56 of the standard. The
/// reference is made with regards to applications that require
/// collision resistance such as digital signatures.
///
/// For applications that primarily require pre-image resistance such as
/// message authentication codes (MACs), key derivation functions
/// (KDFs), and random bit generation use `ws_validate_hash_based`.
///
/// If the hash function is not compliant then `struct ws_hash*
/// alternative` will point to the recommended primitive that one should
/// use instead.
///
/// If the hash function is compliant but the context specifies a higher
/// security level, `struct ws_hash*` will also point to the recommended
/// primitive with the desired security level.
///
/// The function returns 1 if the hash function is compliant, 0 if it is
/// not, and -1 if an error occurs as a result of a missing or invalid
/// argument.
///
/// **Note:** that this means an alternative might be suggested for a
/// compliant hash functions with a similar security level in which a
/// switch to the recommended primitive would likely be unwarranted. For
/// example, when evaluating compliance for the `SHA3-256`, a
/// recommendation to use `SHA256` will be made but this likely
/// unnecessary.
///
/// **Caution:** The default recommendation is from the SHA2 family.
/// While this is safe for most use cases, it is generally not
/// recommended for hashing secrets given its lack of resistance against
/// length extension attacks.
///
/// # Safety
///
/// See module documentation for comment on safety.
#[no_mangle]
pub unsafe extern "C" fn ws_nist_validate_hash(
  ctx: *const Context,
  hash: *const Hash,
  alternative: *mut Hash,
) -> c_int {
  c_call(validate_hash, ctx, hash, alternative)
}

/// Validates a hash function according to page 56 of the standard. The
/// reference is made with regards to applications that primarily
/// require pre-image resistance such as message authentication codes
/// (MACs), key derivation functions (KDFs), and random bit generation.
///
/// For applications that require collision resistance such digital
/// signatures use `ws_nist_validate_hash`.
///
/// If the hash function is not compliant then
/// `struct ws_hash* alternative` will point to the recommended
/// primitive that one should use instead.
///
/// If the hash function is compliant but the context specifies a higher
/// security level, `struct ws_hash*` will also point to the recommended
/// primitive with the desired security level.
///
/// The function returns 1 if the hash function is compliant, 0 if it is
/// not, and -1 if an error occurs as a result of a missing or invalid
/// argument.
///
/// **Note:** that this means an alternative might be suggested for a
/// compliant hash functions with a similar security level in which a
/// switch to the recommended primitive would likely be unwarranted. For
/// example, when evaluating compliance for the `SHA3-256`, a
/// recommendation to use `SHA256` will be made but this likely
/// unnecessary.
///
/// **Caution:** The default recommendation is from the SHA2 family.
/// While this is safe for most use cases, it is generally not
/// recommended for hashing secrets given its lack of resistance against
/// length extension attacks.
///
/// # Safety
///
/// See module documentation for comment on safety.
#[no_mangle]
pub unsafe extern "C" fn ws_nist_validate_hash_based(
  ctx: *const Context,
  hash: *const Hash,
  alternative: *mut Hash,
) -> c_int {
  c_call(validate_hash_based, ctx, hash, alternative)
}

/// Validates a symmetric key primitive according to pages 54-55 of the
/// standard.
///
/// If the key is not compliant then `struct ws_symmetric* alternative`
/// will point to the recommended primitive that one should use instead.
///
/// If the symmetric key is compliant but the context specifies a higher
/// security level, `struct ws_symmetric*` will also point to the
/// recommended primitive with the desired security level.
///
/// The function returns 1 if the key is compliant, 0 if it is not, and
/// -1 if an error occurs as a result of a missing or invalid argument.
///
/// # Safety
///
/// See module documentation for comment on safety.
#[no_mangle]
pub unsafe extern "C" fn ws_nist_validate_symmetric(
  ctx: *const Context,
  key: *const Symmetric,
  alternative: *mut Symmetric,
) -> c_int {
  c_call(validate_symmetric, ctx, key, alternative)
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
  use super::*;
  use crate::primitives::{ffc::*, hash::*};

  macro_rules! test_case {
    ($name:ident, $func:ident, $input:expr, $want:expr) => {
      #[test]
      fn $name() {
        let ctx = Context::default();
        assert_eq!($func(&ctx, $input), $want);
      }
    };
  }

  test_case!(ffc_1024_160, validate_ffc, &FFC_1024_160, Err(FFC_2048_224));
  test_case!(ffc_2048_224, validate_ffc, &FFC_2048_224, Ok(FFC_2048_224));
  test_case!(ffc_3072_256, validate_ffc, &FFC_3072_256, Ok(FFC_3072_256));
  test_case!(ffc_7680_384, validate_ffc, &FFC_7680_384, Ok(FFC_7680_384));
  test_case!(ffc_15360_512, validate_ffc, &FFC_15360_512, Ok(FFC_15360_512));

  test_case!(blake2b_256_collision_resistance, validate_hash, &BLAKE2b_256, Err(SHA256));
  test_case!(blake2b_384_collision_resistance, validate_hash, &BLAKE2b_384, Err(SHA256));
  test_case!(blake2b_512_collision_resistance, validate_hash, &BLAKE2b_512, Err(SHA256));
  test_case!(blake2s_256_collision_resistance, validate_hash, &BLAKE2s_256, Err(SHA256));
  test_case!(md4_collision_resistance, validate_hash, &MD4, Err(SHA256));
  test_case!(md5_collision_resistance, validate_hash, &MD5, Err(SHA256));
  test_case!(ripemd160_collision_resistance, validate_hash, &RIPEMD160, Err(SHA256));
  test_case!(sha1_collision_resistance, validate_hash, &SHA1, Err(SHA224));
  test_case!(sha224_collision_resistance, validate_hash, &SHA224, Ok(SHA224));
  test_case!(sha256_collision_resistance, validate_hash, &SHA256, Ok(SHA256));
  test_case!(sha384_collision_resistance, validate_hash, &SHA384, Ok(SHA384));
  test_case!(sha3_224_collision_resistance, validate_hash, &SHA3_224, Ok(SHA224));
  test_case!(sha3_256_collision_resistance, validate_hash, &SHA3_256, Ok(SHA256));
  test_case!(sha3_384_collision_resistance, validate_hash, &SHA3_384, Ok(SHA384));
  test_case!(sha3_512_collision_resistance, validate_hash, &SHA3_512, Ok(SHA512));
  test_case!(sha512_collision_resistance, validate_hash, &SHA512, Ok(SHA512));
  test_case!(sha512_224_collision_resistance, validate_hash, &SHA512_224, Ok(SHA224));
  test_case!(sha512_256_collision_resistance, validate_hash, &SHA512_256, Ok(SHA256));
  test_case!(shake128_collision_resistance, validate_hash, &SHAKE128, Err(SHA256));
  test_case!(shake256_collision_resistance, validate_hash, &SHAKE256, Err(SHA256));

  test_case!(blake2b_256_pre_image_resistance, validate_hash_based, &BLAKE2b_256, Err(SHA224));
  test_case!(blake2b_384_pre_image_resistance, validate_hash_based, &BLAKE2b_384, Err(SHA224));
  test_case!(blake2b_512_pre_image_resistance, validate_hash_based, &BLAKE2b_512, Err(SHA224));
  test_case!(blake2s_256_pre_image_resistance, validate_hash_based, &BLAKE2s_256, Err(SHA224));
  test_case!(md4_pre_image_resistance, validate_hash_based, &MD4, Err(SHA224));
  test_case!(md5_pre_image_resistance, validate_hash_based, &MD5, Err(SHA224));
  test_case!(ripemd160_pre_image_resistance, validate_hash_based, &RIPEMD160, Err(SHA224));
  test_case!(sha1_pre_image_resistance, validate_hash_based, &SHA1, Err(SHA224));
  test_case!(sha224_pre_image_resistance, validate_hash_based, &SHA224, Ok(SHA224));
  test_case!(sha256_pre_image_resistance, validate_hash_based, &SHA256, Ok(SHA256));
  test_case!(sha384_pre_image_resistance, validate_hash_based, &SHA384, Ok(SHA384));
  test_case!(sha3_224_pre_image_resistance, validate_hash_based, &SHA3_224, Ok(SHA224));
  test_case!(sha3_256_pre_image_resistance, validate_hash_based, &SHA3_256, Ok(SHA256));
  test_case!(sha3_384_pre_image_resistance, validate_hash_based, &SHA3_384, Ok(SHA384));
  test_case!(sha3_512_pre_image_resistance, validate_hash_based, &SHA3_512, Ok(SHA512));
  test_case!(sha512_pre_image_resistance, validate_hash_based, &SHA512, Ok(SHA512));
  test_case!(sha512_224_pre_image_resistance, validate_hash_based, &SHA512_224, Ok(SHA224));
  test_case!(sha512_256_pre_image_resistance, validate_hash_based, &SHA512_256, Ok(SHA256));
  test_case!(shake128_pre_image_resistance, validate_hash_based, &SHAKE128, Err(SHA224));
  test_case!(shake256_pre_image_resistance, validate_hash_based, &SHAKE256, Err(SHA224));

  test_case!(two_key_tdea, validate_symmetric, &TDEA2, Err(AES128));
  test_case!(three_key_tdea, validate_symmetric, &TDEA3, Ok(AES128));
  test_case!(aes128, validate_symmetric, &AES128, Ok(AES128));
  test_case!(aes192, validate_symmetric, &AES192, Ok(AES192));
  test_case!(aes256, validate_symmetric, &AES256, Ok(AES256));
}
