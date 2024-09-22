use std::sync::Arc;

use academy_auth_contracts::internal::AuthInternalAuthenticateError;
use academy_core_internal_contracts::{
    InternalGetUserByEmailError, InternalGetUserError, InternalService,
};
use academy_models::email_address::EmailAddress;
use aide::{
    axum::{routing, ApiRouter},
    transform::TransformOperation,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    docs::TransformOperationExt,
    errors::{
        error, internal_server_error, internal_server_error_docs, ApiError, InvalidTokenDetail,
        UserNotFoundDetail,
    },
    extractors::auth::{ApiToken, InternalApiToken},
    models::user::{ApiUser, PathUserId},
};

pub const TAG: &str = "Internal";

pub fn router(service: Arc<impl InternalService>) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route(
            "/auth/_internal/users/:user_id",
            routing::get_with(get_user, get_user_docs),
        )
        .api_route(
            "/auth/_internal/users/by_email/:email",
            routing::get_with(get_user_by_email, get_user_by_email_docs),
        )
        .with_state(service)
        .with_path_items(|op| op.tag(TAG))
}

async fn get_user(
    service: State<Arc<impl InternalService>>,
    token: ApiToken<InternalApiToken>,
    Path(PathUserId { user_id }): Path<PathUserId>,
) -> Response {
    match service.get_user(&token.0, user_id).await {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(InternalGetUserError::NotFound) => error(StatusCode::NOT_FOUND, UserNotFoundDetail),
        Err(InternalGetUserError::Auth(AuthInternalAuthenticateError::InvalidToken)) => {
            error(StatusCode::UNAUTHORIZED, InvalidTokenDetail)
        }
        Err(InternalGetUserError::Other(err)) => internal_server_error(err),
    }
}

fn get_user_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Return the user with the given id.")
        .add_response::<ApiUser>(StatusCode::OK, None)
        .add_response::<ApiError<UserNotFoundDetail>>(
            StatusCode::NOT_FOUND,
            "The user does not exist.",
        )
        .with(internal_auth_error_docs)
        .with(internal_server_error_docs)
}

#[derive(Deserialize, JsonSchema)]
struct GetUserByEmailPath {
    email: EmailAddress,
}

async fn get_user_by_email(
    service: State<Arc<impl InternalService>>,
    token: ApiToken<InternalApiToken>,
    Path(GetUserByEmailPath { email }): Path<GetUserByEmailPath>,
) -> Response {
    match service.get_user_by_email(&token.0, email).await {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(InternalGetUserByEmailError::NotFound) => {
            error(StatusCode::NOT_FOUND, UserNotFoundDetail)
        }
        Err(InternalGetUserByEmailError::Auth(err)) => internal_auth_error(err),
        Err(InternalGetUserByEmailError::Other(err)) => internal_server_error(err),
    }
}

fn get_user_by_email_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Return the user with the given email address.")
        .add_response::<ApiUser>(StatusCode::OK, None)
        .add_response::<ApiError<UserNotFoundDetail>>(
            StatusCode::NOT_FOUND,
            "The user does not exist.",
        )
        .with(internal_auth_error_docs)
        .with(internal_server_error_docs)
}

fn internal_auth_error(err: AuthInternalAuthenticateError) -> Response {
    match err {
        AuthInternalAuthenticateError::InvalidToken => {
            error(StatusCode::UNAUTHORIZED, InvalidTokenDetail)
        }
    }
}

fn internal_auth_error_docs(op: TransformOperation) -> TransformOperation {
    op.add_response::<ApiError<InvalidTokenDetail>>(
        StatusCode::UNAUTHORIZED,
        "The internal authentication token is invalid or has expired.",
    )
}
