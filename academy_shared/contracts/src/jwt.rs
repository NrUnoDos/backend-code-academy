use std::{fmt::Debug, time::Duration};

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait JwtService: Send + Sync + 'static {
    /// Sign a JWT with the given data and time to live.
    ///
    /// `data` must serialize to a map (JSON object), which may not contain the
    /// `exp` key.
    fn sign<T: Serialize + Debug + 'static, S: From<String> + Debug + 'static>(
        &self,
        data: T,
        ttl: Duration,
    ) -> anyhow::Result<S>;

    /// Verify the signature of the given JWT, deserialize its payload and
    /// ensure the JWT has not expired yet.
    fn verify<S: AsRef<str> + Debug + 'static, T: DeserializeOwned + Debug + 'static>(
        &self,
        jwt: &S,
    ) -> Result<T, VerifyJwtError<T>>;
}

#[derive(Debug, Error)]
pub enum VerifyJwtError<T> {
    #[error("JWT has already expired (data: {0})")]
    Expired(T),
    #[error("Invalid JWT")]
    Invalid,
}

#[cfg(feature = "mock")]
impl MockJwtService {
    pub fn with_sign<
        T: Debug + PartialEq + Serialize + Send + 'static,
        S: From<String> + Debug + Send + 'static,
    >(
        mut self,
        data: T,
        ttl: Duration,
        result: anyhow::Result<S>,
    ) -> Self {
        self.expect_sign()
            .once()
            .with(mockall::predicate::eq(data), mockall::predicate::eq(ttl))
            .return_once(|_, _| result);
        self
    }

    pub fn with_verify<
        S: AsRef<str> + Debug + PartialEq + Send + 'static,
        T: DeserializeOwned + Debug + Send + 'static,
    >(
        mut self,
        jwt: S,
        result: Result<T, VerifyJwtError<T>>,
    ) -> Self {
        self.expect_verify()
            .once()
            .with(mockall::predicate::eq(jwt))
            .return_once(|_| result);
        self
    }
}
