use academy_core_auth_contracts::MockAuthService;
use academy_core_session_contracts::{SessionGetCurrentError, SessionService};
use academy_demo::{session::FOO_1, user::FOO};
use academy_models::auth::{AuthError, AuthenticateError};
use academy_persistence_contracts::{session::MockSessionRepository, MockDatabase};
use academy_utils::assert_matches;

use crate::{tests::Sut, SessionServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(false);

    let session_repo = MockSessionRepository::new().with_get(FOO_1.id, Some(FOO_1.clone()));

    let sut = SessionServiceImpl {
        auth,
        db,
        session_repo,
        ..Sut::default()
    };

    // Act
    let result = sut.get_current_session("token").await;

    // Assert
    assert_eq!(result.unwrap(), *FOO_1);
}

#[tokio::test]
async fn unauthenticated() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(None);

    let sut = SessionServiceImpl {
        auth,
        ..Sut::default()
    };

    // Act
    let result = sut.get_current_session("token").await;

    // Assert
    assert_matches!(
        result,
        Err(SessionGetCurrentError::Auth(AuthError::Authenticate(
            AuthenticateError::InvalidToken
        )))
    );
}
