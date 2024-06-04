use academy_core_auth_contracts::{AuthResultExt, AuthService};
use academy_core_session_contracts::commands::create::SessionCreateCommandService;
use academy_core_user_contracts::{
    commands::{
        create::{UserCreateCommand, UserCreateCommandError, UserCreateCommandService},
        request_password_reset_email::UserRequestPasswordResetEmailCommandService,
        request_subscribe_newsletter_email::UserRequestSubscribeNewsletterEmailCommandService,
        request_verification_email::UserRequestVerificationEmailCommandService,
        reset_password::{UserResetPasswordCommandError, UserResetPasswordCommandService},
        update_admin::UserUpdateAdminCommandService,
        update_email::{UserUpdateEmailCommandError, UserUpdateEmailCommandService},
        update_enabled::UserUpdateEnabledCommandService,
        update_name::{
            UserUpdateNameCommandError, UserUpdateNameCommandService, UserUpdateNameRateLimitPolicy,
        },
        update_password::UserUpdatePasswordCommandService,
        verify_email::{UserVerifyEmailCommandError, UserVerifyEmailCommandService},
        verify_newsletter_subscription::{
            UserVerifyNewsletterSubscriptionCommandError,
            UserVerifyNewsletterSubscriptionCommandService,
        },
    },
    queries::list::{UserListQuery, UserListQueryService, UserListResult},
    PasswordUpdate, UserCreateError, UserCreateRequest, UserDeleteError, UserGetError,
    UserListError, UserRequestPasswordResetError, UserRequestVerificationEmailError,
    UserResetPasswordError, UserService, UserUpdateError, UserUpdateRequest, UserUpdateUserRequest,
    UserVerifyEmailError, UserVerifyNewsletterSubscriptionError,
};
use academy_di::Build;
use academy_models::{
    auth::Login,
    session::DeviceName,
    user::{UserComposite, UserId, UserIdOrSelf, UserPassword, UserPatchRef},
    VerificationCode,
};
use academy_persistence_contracts::{user::UserRepository, Database, Transaction};
use academy_utils::patch::{Patch, PatchValue};
use anyhow::anyhow;
use email_address::EmailAddress;

pub mod commands;
pub mod queries;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, Build)]
pub struct UserServiceImpl<
    Db,
    Auth,
    UserList,
    UserCreate,
    UserRequestSubscribeNewsletterEmail,
    UserUpdateName,
    UserUpdateEmail,
    UserUpdateAdmin,
    UserUpdateEnabled,
    UserUpdatePassword,
    UserVerifyNewsletterSubscription,
    UserRequestVerificationEmail,
    UserVerifyEmail,
    UserRequestPasswordResetEmail,
    UserResetPassword,
    SessionCreate,
    UserRepo,
> {
    db: Db,
    auth: Auth,
    user_list: UserList,
    user_create: UserCreate,
    user_request_subscribe_newsletter_email: UserRequestSubscribeNewsletterEmail,
    user_update_name: UserUpdateName,
    user_update_email: UserUpdateEmail,
    user_update_admin: UserUpdateAdmin,
    user_update_enabled: UserUpdateEnabled,
    user_update_password: UserUpdatePassword,
    user_verify_newsletter_subscription: UserVerifyNewsletterSubscription,
    user_request_verification_email: UserRequestVerificationEmail,
    user_verify_email: UserVerifyEmail,
    user_request_password_reset_email: UserRequestPasswordResetEmail,
    user_reset_password: UserResetPassword,
    session_create: SessionCreate,
    user_repo: UserRepo,
}

impl<
        Db,
        Auth,
        UserList,
        UserCreate,
        UserRequestSubscribeNewsletterEmail,
        UserUpdateName,
        UserUpdateEmail,
        UserUpdateAdmin,
        UserUpdateEnabled,
        UserUpdatePassword,
        UserVerifyNewsletterSubscription,
        UserRequestVerificationEmail,
        UserVerifyEmail,
        UserRequestPasswordResetEmail,
        UserResetPassword,
        SessionCreate,
        UserRepo,
    > UserService
    for UserServiceImpl<
        Db,
        Auth,
        UserList,
        UserCreate,
        UserRequestSubscribeNewsletterEmail,
        UserUpdateName,
        UserUpdateEmail,
        UserUpdateAdmin,
        UserUpdateEnabled,
        UserUpdatePassword,
        UserVerifyNewsletterSubscription,
        UserRequestVerificationEmail,
        UserVerifyEmail,
        UserRequestPasswordResetEmail,
        UserResetPassword,
        SessionCreate,
        UserRepo,
    >
where
    Db: Database,
    Auth: AuthService<Db::Transaction>,
    UserList: UserListQueryService<Db::Transaction>,
    UserCreate: UserCreateCommandService<Db::Transaction>,
    UserRequestSubscribeNewsletterEmail: UserRequestSubscribeNewsletterEmailCommandService,
    UserUpdateName: UserUpdateNameCommandService<Db::Transaction>,
    UserUpdateEmail: UserUpdateEmailCommandService<Db::Transaction>,
    UserUpdateAdmin: UserUpdateAdminCommandService<Db::Transaction>,
    UserUpdateEnabled: UserUpdateEnabledCommandService<Db::Transaction>,
    UserUpdatePassword: UserUpdatePasswordCommandService<Db::Transaction>,
    UserVerifyNewsletterSubscription:
        UserVerifyNewsletterSubscriptionCommandService<Db::Transaction>,
    UserRequestVerificationEmail: UserRequestVerificationEmailCommandService,
    UserVerifyEmail: UserVerifyEmailCommandService<Db::Transaction>,
    UserRequestPasswordResetEmail: UserRequestPasswordResetEmailCommandService,
    UserResetPassword: UserResetPasswordCommandService<Db::Transaction>,
    SessionCreate: SessionCreateCommandService<Db::Transaction>,
    UserRepo: UserRepository<Db::Transaction>,
{
    async fn list_users(
        &self,
        token: &str,
        query: UserListQuery,
    ) -> Result<UserListResult, UserListError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        auth.ensure_admin().map_auth_err()?;

        let mut txn = self.db.begin_transaction().await.unwrap();

        self.user_list
            .invoke(&mut txn, query)
            .await
            .map_err(Into::into)
    }

    async fn get_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> Result<UserComposite, UserGetError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await.unwrap();

        self.user_repo
            .get_composite(&mut txn, user_id)
            .await?
            .ok_or(UserGetError::NotFound)
    }

    async fn create_user(
        &self,
        request: UserCreateRequest,
        device_name: Option<DeviceName>,
    ) -> Result<Login, UserCreateError> {
        let mut txn = self.db.begin_transaction().await.unwrap();

        let cmd = UserCreateCommand {
            name: request.name,
            display_name: request.display_name,
            email: request.email,
            password: request.password,
            admin: false,
            enabled: true,
            email_verified: false,
        };

        let user = self
            .user_create
            .invoke(&mut txn, cmd)
            .await
            .map_err(|err| match err {
                UserCreateCommandError::NameConflict => UserCreateError::NameConflict,
                UserCreateCommandError::EmailConflict => UserCreateError::EmailConflict,
                UserCreateCommandError::Other(err) => err.into(),
            })?;

        let result = self
            .session_create
            .invoke(&mut txn, user, device_name, true)
            .await
            .map_err(UserCreateError::Other)?;

        txn.commit().await.unwrap();

        Ok(result)
    }

    async fn update_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        UserUpdateRequest {
            user:
                UserUpdateUserRequest {
                    name,
                    email,
                    email_verified,
                    password,
                    enabled,
                    admin,
                    newsletter,
                },
            profile: profile_update,
        }: UserUpdateRequest,
    ) -> Result<UserComposite, UserUpdateError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        let UserComposite {
            mut user,
            mut profile,
            details,
        } = self
            .user_repo
            .get_composite(&mut txn, user_id)
            .await?
            .ok_or(UserUpdateError::NotFound)?;

        let mut commit = false;

        let name = name.minimize(&user.name);
        let email = email.map(Some).minimize(&user.email);
        let email_verified =
            email_verified.minimize(&(user.email_verified && email.is_unchanged()));
        let enabled = enabled.minimize(&user.enabled);
        let admin = admin.minimize(&user.admin);
        let newsletter = newsletter.minimize(&user.newsletter);

        let profile_update = profile_update.minimize(&profile);

        if email_verified.is_update() || enabled.is_update() || admin.is_update() {
            auth.ensure_admin().map_auth_err()?;
        }

        if enabled == PatchValue::Update(false) && user_id == auth.user_id {
            return Err(UserUpdateError::CannotDisableSelf);
        }

        if admin.is_update() && user_id == auth.user_id {
            return Err(UserUpdateError::CannotDemoteSelf);
        }

        if profile_update.is_update() {
            self.user_repo
                .update_profile(&mut txn, user_id, profile_update.as_ref())
                .await?;
            profile = profile.update(profile_update);
            commit = true;
        }

        if let PatchValue::Update(name) = name {
            let rate_limit_policy = if auth.admin {
                UserUpdateNameRateLimitPolicy::Bypass
            } else {
                UserUpdateNameRateLimitPolicy::Enforce
            };
            user = self
                .user_update_name
                .invoke(&mut txn, user, name, rate_limit_policy)
                .await
                .map_err(|err| match err {
                    UserUpdateNameCommandError::Conflict => UserUpdateError::NameConflict,
                    UserUpdateNameCommandError::RateLimit { until } => {
                        UserUpdateError::NameChangeRateLimit { until }
                    }
                    UserUpdateNameCommandError::Other(err) => err.into(),
                })?;
            commit = true;
        }

        if email.is_update() || email_verified.is_update() {
            user.email_verified =
                email_verified.update(user.email_verified && email.is_unchanged());
            user.email = email.update(user.email);
            self.user_update_email
                .invoke(&mut txn, user_id, &user.email, user.email_verified)
                .await
                .map_err(|err| match err {
                    UserUpdateEmailCommandError::Conflict => UserUpdateError::EmailConflict,
                    UserUpdateEmailCommandError::Other(err) => err.into(),
                })?;
            commit = true;
        }

        if let PatchValue::Update(enabled) = enabled {
            self.user_update_enabled
                .invoke(&mut txn, user_id, enabled)
                .await?;
            user.enabled = enabled;
            commit = true;
        }

        if let PatchValue::Update(admin) = admin {
            self.user_update_admin
                .invoke(&mut txn, user_id, admin)
                .await?;
            user.admin = admin;
            commit = true;
        }

        match password {
            PatchValue::Update(PasswordUpdate::Remove) => {
                return Err(UserUpdateError::CannotRemovePassword)
            }
            PatchValue::Update(PasswordUpdate::Change(password)) => {
                self.user_update_password
                    .invoke(&mut txn, user_id, password)
                    .await?;
                commit = true;
            }
            PatchValue::Unchanged => (),
        }

        if let PatchValue::Update(newsletter) = newsletter {
            if newsletter && !auth.admin {
                let email = user.email.clone().ok_or(UserUpdateError::NoEmail)?;
                self.user_request_subscribe_newsletter_email
                    .invoke(user_id, email)
                    .await?;
            } else {
                user.newsletter = newsletter;
                self.user_repo
                    .update(
                        &mut txn,
                        user_id,
                        UserPatchRef::new().update_newsletter(&newsletter),
                    )
                    .await
                    .map_err(|err| UserUpdateError::Other(err.into()))?;
                commit = true;
            }
        }

        if commit {
            txn.commit().await?;
        }

        Ok(UserComposite {
            user,
            profile,
            details,
        })
    }

    async fn delete_user(&self, token: &str, user_id: UserIdOrSelf) -> Result<(), UserDeleteError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        self.auth
            .invalidate_access_tokens(&mut txn, user_id)
            .await?;

        if !self.user_repo.delete(&mut txn, user_id).await? {
            return Err(UserDeleteError::NotFound);
        }

        txn.commit().await?;

        Ok(())
    }

    async fn request_verification_email(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> Result<(), UserRequestVerificationEmailError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        let user_composite = self
            .user_repo
            .get_composite(&mut txn, user_id)
            .await?
            .ok_or(UserRequestVerificationEmailError::NotFound)?;

        if user_composite.user.email_verified {
            return Err(UserRequestVerificationEmailError::AlreadyVerified);
        }

        let email = user_composite
            .user
            .email
            .ok_or(UserRequestVerificationEmailError::NoEmail)?;

        self.user_request_verification_email.invoke(email).await?;

        Ok(())
    }

    async fn verify_email(&self, code: VerificationCode) -> Result<(), UserVerifyEmailError> {
        let mut txn = self.db.begin_transaction().await?;

        match self.user_verify_email.invoke(&mut txn, &code).await {
            Ok(_) => {
                txn.commit().await?;
                Ok(())
            }
            Err(UserVerifyEmailCommandError::AlreadyVerified) => Ok(()),
            Err(UserVerifyEmailCommandError::InvalidCode) => Err(UserVerifyEmailError::InvalidCode),
            Err(UserVerifyEmailCommandError::Other(err)) => Err(err.into()),
        }
    }

    async fn verify_newsletter_subscription(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        code: VerificationCode,
    ) -> Result<UserComposite, UserVerifyNewsletterSubscriptionError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        let mut user_composite = self
            .user_repo
            .get_composite(&mut txn, user_id)
            .await?
            .ok_or(UserVerifyNewsletterSubscriptionError::NotFound)?;

        if user_composite.user.newsletter {
            return Err(UserVerifyNewsletterSubscriptionError::AlreadySubscribed);
        }

        self.user_verify_newsletter_subscription
            .invoke(&mut txn, user_id, code)
            .await
            .map_err(|err| match err {
                UserVerifyNewsletterSubscriptionCommandError::InvalidCode => {
                    UserVerifyNewsletterSubscriptionError::InvalidCode
                }
                UserVerifyNewsletterSubscriptionCommandError::Other(err) => err.into(),
            })?;

        user_composite.user.newsletter = true;

        txn.commit().await?;

        Ok(user_composite)
    }

    async fn request_password_reset(
        &self,
        email: EmailAddress,
    ) -> Result<(), UserRequestPasswordResetError> {
        let mut txn = self.db.begin_transaction().await?;

        if let Some(user_composite) = self
            .user_repo
            .get_composite_by_email(&mut txn, &email)
            .await?
        {
            let email = user_composite.user.email.ok_or_else(|| {
                anyhow!(
                    "User {} fetched by email {} has no email address",
                    user_composite.user.id.hyphenated(),
                    email
                )
            })?;
            self.user_request_password_reset_email
                .invoke(user_composite.user.id, email)
                .await?;
        }

        Ok(())
    }

    async fn reset_password(
        &self,
        email: EmailAddress,
        code: VerificationCode,
        new_password: UserPassword,
    ) -> Result<UserComposite, UserResetPasswordError> {
        let mut txn = self.db.begin_transaction().await?;

        let user_composite = self
            .user_repo
            .get_composite_by_email(&mut txn, &email)
            .await?
            .ok_or(UserResetPasswordError::Failed)?;

        self.user_reset_password
            .invoke(&mut txn, user_composite.user.id, code, new_password)
            .await
            .map_err(|err| match err {
                UserResetPasswordCommandError::InvalidCode => UserResetPasswordError::Failed,
                UserResetPasswordCommandError::Other(err) => err.into(),
            })?;

        txn.commit().await?;

        Ok(user_composite)
    }
}

fn subscribe_newsletter_cache_key(user_id: UserId) -> String {
    format!("subscribe_newsletter_code:{}", user_id.hyphenated())
}

fn verification_cache_key(verification_code: &VerificationCode) -> String {
    format!("verification:{}", **verification_code)
}

fn reset_password_cache_key(user_id: UserId) -> String {
    format!("reset_password_code:{}", user_id.hyphenated())
}
