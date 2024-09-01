use academy_core_user_contracts::{
    commands::request_password_reset_email::MockUserRequestPasswordResetEmailCommandService,
    UserRequestPasswordResetError, UserService,
};
use academy_demo::user::FOO;
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};
use academy_shared_contracts::captcha::{CaptchaCheckError, MockCaptchaService};
use academy_utils::assert_matches;

use crate::{tests::Sut, UserServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let db = MockDatabase::build(false);

    let captcha = MockCaptchaService::new().with_check(Some("resp"), Ok(()));

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));

    let user_request_password_reset_email = MockUserRequestPasswordResetEmailCommandService::new()
        .with_invoke(FOO.user.id, FOO.user.email.clone().unwrap());

    let sut = UserServiceImpl {
        db,
        captcha,
        user_repo,
        user_request_password_reset_email,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_password_reset(
            FOO.user.email.clone().unwrap(),
            Some("resp".try_into().unwrap()),
        )
        .await;

    // Assert
    result.unwrap();
}

#[tokio::test]
async fn invalid_captcha_response() {
    // Arrange
    let captcha =
        MockCaptchaService::new().with_check(Some("resp"), Err(CaptchaCheckError::Failed));

    let sut = UserServiceImpl {
        captcha,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_password_reset(
            FOO.user.email.clone().unwrap(),
            Some("resp".try_into().unwrap()),
        )
        .await;

    // Assert
    assert_matches!(result, Err(UserRequestPasswordResetError::Recaptcha));
}

#[tokio::test]
async fn user_not_found() {
    // Arrange
    let db = MockDatabase::build(false);

    let captcha = MockCaptchaService::new().with_check(None, Ok(()));

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), None);

    let sut = UserServiceImpl {
        db,
        captcha,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_password_reset(FOO.user.email.clone().unwrap(), None)
        .await;

    // Assert
    result.unwrap();
}
