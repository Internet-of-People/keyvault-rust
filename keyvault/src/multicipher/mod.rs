//! A type-erased version of [`AsymmetricCrypto`] and [`KeyDerivationCrypto`]. Serialized versions
//! of crypto concepts, like [`KeyId`], [`PublicKey`], [`PrivateKey`], [`Signature`],
//! [`ExtendedPrivateKey`] and [`ExtendedPublicKey`] can be all deserialized into
//! their [`MultiCipher`] versions.
//! This allows multiple cryptographic algorithms to co-exist in a software, which is needed
//! during migration of a single software to a new cryptography, or which is the status quo in
//! larger software ecosystems.
//!
//! [`MultiCipher`] can be thought of a variant of multiple incompatible cipher suits, which are
//! strongly typed, but are chosen at run-time.
//!
//! [`MultiCipher`]: struct.MultiCipher.html
//! [`AsymmetricCrypto`]: ../trait.AsymmetricCrypto.html
//! [`KeyDerivationCrypto`]: ../trait.KeyDerivationCrypto.html
//! [`KeyId`]: ../trait.AsymmetricCrypto.html#associatedtype.KeyId
//! [`PublicKey`]: ../trait.PublicKey.html
//! [`PrivateKey`]: ../trait.PrivateKey.html
//! [`Signature`]: ../trait.AsymmetricCrypto.html#associatedtype.Signature
//! [`ExtendedPrivateKey`]: ../trait.ExtendedPrivateKey.html
//! [`ExtendedPublicKey`]: ../trait.ExtendedPublicKey.html

macro_rules! e {
    (variant) => {
        CipherSuite::Ed25519
    };
    (id) => {
        EdKeyId
    };
    (pk) => {
        EdPublicKey
    };
    (sk) => {
        EdPrivateKey
    };
    (sig) => {
        EdSignature
    };
}

macro_rules! s {
    (variant) => {
        CipherSuite::Secp256k1
    };
    (id) => {
        SecpKeyId
    };
    (pk) => {
        SecpPublicKey
    };
    (sk) => {
        SecpPrivateKey
    };
    (sig) => {
        SecpSignature
    };
}

macro_rules! erased_type {
    ($(#[$meta:meta])* $v:vis struct $type:ident {}) => {
        $(#[$meta])*
        $v struct $type {
            #[allow(dead_code)]
            pub(super) suite: CipherSuite,
            #[allow(dead_code)]
            pub(super) erased: Box<dyn Any + Send + Sync>,
        }

        impl $type {
            /// Returns the cipher suite of the multicipher object
            pub fn suite(&self) -> CipherSuite {
                self.suite
            }
        }
    };
}

macro_rules! reify {
    ($suite:ident, $type:tt, $x:expr) => {{
        assert!($x.suite == $suite!(variant));
        $x.erased.downcast_ref::<$suite!($type)>().unwrap()
    }};
}

macro_rules! erase {
    ($suite:ident, $type:ident, $result:expr) => {
        $type { suite: $suite!(variant), erased: Box::new($result) as Box<dyn Any + Send + Sync> }
    };
}

macro_rules! visit_fac {
    ($left:ident($suite:expr) => $callback:ident($self_:tt)) => {
        visit_fac!($left($suite) => $callback($self_,))
    };
    ($left:ident($suite:expr) => $callback:ident($self_:tt, $($args:tt)*)) => {
        match $suite {
            $left!(e) => visit_fac!(@case e $callback $self_ [ $($args),* ]),
            $left!(s) => visit_fac!(@case s $callback $self_ [ $($args),* ]),
            _ => bail!("Unknown crypto suite suite '{}'", $suite),
        }
    };
    (@case $suite:ident $callback:ident $self_:tt [ ]) => {
        $callback!($suite, $self_)
    };
    (@case $suite:ident $callback:ident $self_:tt [ $($args:tt),* ]) => {
        $callback!($suite, $self_, $($args)*)
    };
}

macro_rules! visit {
    ($callback:ident($self_:tt)) => {
        visit!($callback($self_,))
    };
    ($callback:ident($self_:tt, $($args:tt)*) ) => {
        match $self_.suite {
            e!(variant) => visit!(@case e $callback $self_ [ $($args),* ]),
            s!(variant) => visit!(@case s $callback $self_ [ $($args),* ]),
        }
    };
    (@case $suite:ident $callback:ident $self_:tt [ ]) => {
        $callback!($suite, $self_)
    };
    (@case $suite:ident $callback:ident $self_:tt [ $($args:tt),* ]) => {
        $callback!($suite, $self_, $($args)*)
    };
}

mod id;
mod pk;
mod sig;
mod sk;

use super::*;

use std::any::Any;
use std::hash::Hash;
use std::hash::Hasher;

use crate::ed25519::{EdKeyId, EdPrivateKey, EdPublicKey, EdSignature};
use crate::secp256k1::{SecpKeyId, SecpPrivateKey, SecpPublicKey, SecpSignature};
use crate::{AsymmetricCrypto, PrivateKey, PublicKey};

pub use id::MKeyId;
pub use pk::MPublicKey;
pub use sig::MSignature;
pub use sk::MPrivateKey;

/// A suite type that is used to keep the type-safety of the erased types in [`multicipher`]
///
/// [`multicipher`]: index.html
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum CipherSuite {
    /// The object tagged with this variant belongs to the [`ed25519`] module
    ///
    /// [`ed25519`]: ../ed25519/index.html
    Ed25519,
    /// The object tagged with this variant belongs to the [`secp256k1`] module
    ///
    /// [`secp256k1`]: ../secp256k1/index.html
    Secp256k1,
}

#[derive(Clone, Debug)]
/// See the [module-level description](index.html).
pub struct MultiCipher;

impl AsymmetricCrypto for MultiCipher {
    type KeyId = MKeyId;
    type PublicKey = MPublicKey;
    type PrivateKey = MPrivateKey;
    type Signature = MSignature;
}

#[derive(Serialize, Deserialize)]
struct ErasedBytes {
    #[serde(rename = "s")]
    suite: u8,
    #[serde(rename = "v", with = "serde_bytes")]
    value: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let _cipher = MultiCipher {};
    }
}
