use std::time::Duration;

use academy_core_auth_contracts::{AuthService, AuthenticateByRefreshTokenError};
use academy_demo::{session::FOO_1, SHA256HASH1};
use academy_persistence_contracts::session::MockSessionRepository;
use academy_shared_contracts::{hash::MockHashService, time::MockTimeService};
use academy_utils::assert_matches;

use crate::{tests::Sut, AuthServiceConfig, AuthServiceImpl};

#[tokio::test]
async fn authenticate_by_refresh_token_ok() {
    // Arrange
    let config = AuthServiceConfig::default();

    let hash = MockHashService::new()
        .with_sha256("the refresh token".to_owned().into_bytes(), *SHA256HASH1);

    let time = MockTimeService::new()
        .with_now(FOO_1.updated_at + config.refresh_token_ttl - Duration::from_secs(1));

    let session_repo = MockSessionRepository::new()
        .with_get_by_refresh_token_hash((*SHA256HASH1).into(), Some(FOO_1.clone()));

    let sut = AuthServiceImpl {
        hash,
        time,
        session_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .authenticate_by_refresh_token(&mut (), "the refresh token")
        .await;

    // Assert
    assert_eq!(result.unwrap(), FOO_1.id);
}

#[tokio::test]
async fn authenticate_by_refresh_token_invalid() {
    // Arrange
    let hash = MockHashService::new()
        .with_sha256("the refresh token".to_owned().into_bytes(), *SHA256HASH1);

    let session_repo =
        MockSessionRepository::new().with_get_by_refresh_token_hash((*SHA256HASH1).into(), None);

    let sut = AuthServiceImpl {
        hash,
        session_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .authenticate_by_refresh_token(&mut (), "the refresh token")
        .await;

    // Assert
    assert_matches!(result, Err(AuthenticateByRefreshTokenError::Invalid));
}

#[tokio::test]
async fn authenticate_by_refresh_token_expired() {
    // Arrange
    let config = AuthServiceConfig::default();

    let hash = MockHashService::new()
        .with_sha256("the refresh token".to_owned().into_bytes(), *SHA256HASH1);

    let time = MockTimeService::new()
        .with_now(FOO_1.updated_at + config.refresh_token_ttl + Duration::from_secs(2));

    let session_repo = MockSessionRepository::new()
        .with_get_by_refresh_token_hash((*SHA256HASH1).into(), Some(FOO_1.clone()));

    let sut = AuthServiceImpl {
        hash,
        time,
        session_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .authenticate_by_refresh_token(&mut (), "the refresh token")
        .await;

    // Assert
    assert_matches!(result, Err(AuthenticateByRefreshTokenError::Expired(x)) if *x == FOO_1.id);
}
