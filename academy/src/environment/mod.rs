use std::{collections::HashMap, sync::Arc};

use academy_config::Config;
use academy_core_auth_impl::AuthServiceConfig;
use academy_core_contact_impl::ContactServiceConfig;
use academy_core_health_impl::HealthServiceConfig;
use academy_core_oauth2_impl::OAuth2ServiceConfig;
use academy_core_session_impl::SessionServiceConfig;
use academy_core_user_impl::commands::{
    request_password_reset_email::UserRequestPasswordResetEmailCommandServiceConfig,
    request_subscribe_newsletter_email::UserRequestSubscribeNewsletterEmailCommandServiceConfig,
    request_verification_email::UserRequestVerificationEmailCommandServiceConfig,
    update_name::UserUpdateNameCommandServiceConfig,
};
use academy_di::provider;
use academy_extern_impl::recaptcha::RecaptchaApiServiceConfig;
use academy_models::oauth2::OAuth2Provider;
use academy_shared_impl::{
    captcha::{CaptchaServiceConfig, RecaptchaCaptchaServiceConfig},
    jwt::JwtServiceConfig,
    totp::TotpServiceConfig,
};
use types::{Cache, Database, Email};

pub mod types;

provider! {
    pub Provider {
        database: Database,
        cache: Cache,
        email: Email,
        ..config: ConfigProvider {
            AuthServiceConfig,
            JwtServiceConfig,
            UserUpdateNameCommandServiceConfig,
            HealthServiceConfig,
            UserRequestSubscribeNewsletterEmailCommandServiceConfig,
            ContactServiceConfig,
            UserRequestVerificationEmailCommandServiceConfig,
            UserRequestPasswordResetEmailCommandServiceConfig,
            TotpServiceConfig,
            Arc<CaptchaServiceConfig>,
            Arc<RecaptchaApiServiceConfig>,
            SessionServiceConfig,
            OAuth2ServiceConfig,
        }
    }
}

impl Provider {
    pub fn new(config: ConfigProvider, database: Database, cache: Cache, email: Email) -> Self {
        Self {
            _state: Default::default(),
            database,
            cache,
            email,
            config,
        }
    }
}

provider! {
    pub ConfigProvider {
        auth_service_config: AuthServiceConfig,
        jwt_service_config: JwtServiceConfig,
        user_update_name_command_service_config: UserUpdateNameCommandServiceConfig,
        health_service_config: HealthServiceConfig,
        user_request_subscribe_newsletter_email_command_service_config: UserRequestSubscribeNewsletterEmailCommandServiceConfig,
        contact_service_config: ContactServiceConfig,
        user_request_verification_email_command_service_config: UserRequestVerificationEmailCommandServiceConfig,
        user_request_password_reset_email_command_service_config: UserRequestPasswordResetEmailCommandServiceConfig,
        totp_service_config: TotpServiceConfig,
        captcha_service_config: Arc<CaptchaServiceConfig>,
        recaptcha_api_service_config: Arc<RecaptchaApiServiceConfig>,
        session_service_config: SessionServiceConfig,
        oauth2_service_config: OAuth2ServiceConfig,
    }
}

impl ConfigProvider {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let auth_service_config = AuthServiceConfig {
            access_token_ttl: config.session.access_token_ttl.into(),
            refresh_token_ttl: config.session.refresh_token_ttl.into(),
            refresh_token_length: config.session.refresh_token_length,
        };
        let jwt_service_config = JwtServiceConfig::new(&config.jwt.secret)?;
        let user_update_name_command_service_config = UserUpdateNameCommandServiceConfig {
            name_change_rate_limit: config.user.name_change_rate_limit.into(),
        };
        let health_service_config = HealthServiceConfig {
            cache_ttl: config.health.cache_ttl.into(),
        };
        let user_request_subscribe_newsletter_email_command_service_config =
            UserRequestSubscribeNewsletterEmailCommandServiceConfig {
                redirect_url: config.user.newsletter_redirect_url.clone().into(),
                verification_code_ttl: config.user.newsletter_code_ttl.into(),
            };
        let contact_service_config = ContactServiceConfig {
            email: config.contact.email.clone().into(),
        };
        let user_request_verification_email_command_service_config =
            UserRequestVerificationEmailCommandServiceConfig {
                redirect_url: config.user.verification_redirect_url.clone().into(),
                verification_code_ttl: config.user.verification_code_ttl.into(),
            };
        let user_request_password_reset_email_command_service_config =
            UserRequestPasswordResetEmailCommandServiceConfig {
                redirect_url: config.user.password_reset_redirect_url.clone().into(),
                verification_code_ttl: config.user.password_reset_code_ttl.into(),
            };
        let totp_service_config = TotpServiceConfig {
            secret_length: config.totp.secret_length,
        };

        let captcha_service_config = match config.recaptcha.as_ref() {
            Some(recaptcha) => CaptchaServiceConfig::Recaptcha(RecaptchaCaptchaServiceConfig {
                sitekey: recaptcha.sitekey.clone(),
                secret: recaptcha.secret.clone(),
                min_score: recaptcha.min_score,
            }),
            None => CaptchaServiceConfig::Disabled,
        }
        .into();
        let recaptcha_api_service_config = RecaptchaApiServiceConfig::new(
            config
                .recaptcha
                .as_ref()
                .and_then(|recaptcha| recaptcha.siteverify_endpoint_override.clone()),
        )
        .into();

        let session_service_config = SessionServiceConfig {
            login_fails_before_captcha: config.session.login_fails_before_captcha,
        };

        let oauth2_service_config = OAuth2ServiceConfig {
            providers: config
                .oauth2
                .iter()
                .flat_map(|oauth2| oauth2.providers.iter())
                .map(|(id, provider)| {
                    (
                        id.clone(),
                        OAuth2Provider {
                            name: provider.name.clone(),
                            client_id: provider.client_id.clone(),
                            client_secret: Some(provider.client_secret.clone()),
                            auth_url: provider.auth_url.clone(),
                            token_url: provider.token_url.clone(),
                            userinfo_url: provider.userinfo_url.clone(),
                            userinfo_id_key: provider.userinfo_id_key.clone(),
                            userinfo_name_key: provider.userinfo_name_key.clone(),
                            scopes: provider.scopes.clone(),
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
                .into(),
        };

        Ok(Self {
            _state: Default::default(),
            auth_service_config,
            jwt_service_config,
            user_update_name_command_service_config,
            health_service_config,
            user_request_subscribe_newsletter_email_command_service_config,
            contact_service_config,
            user_request_verification_email_command_service_config,
            user_request_password_reset_email_command_service_config,
            totp_service_config,
            captcha_service_config,
            recaptcha_api_service_config,
            session_service_config,
            oauth2_service_config,
        })
    }
}

#[cfg(test)]
mod tests {
    use academy_cache_valkey::ValkeyCache;
    use academy_config::DEFAULT_CONFIG_PATH;
    use academy_di::Provides;
    use academy_email_impl::EmailServiceImpl;
    use academy_persistence_postgres::PostgresDatabase;
    use types::RestServer;

    use super::*;

    #[tokio::test]
    async fn provide_rest_server() {
        let config = academy_config::load(&[DEFAULT_CONFIG_PATH]).unwrap();
        let config_provider = ConfigProvider::new(&config).unwrap();

        let database = PostgresDatabase::dummy().await;
        let cache = ValkeyCache::dummy().await;
        let email = EmailServiceImpl::dummy().await;

        let mut provider = Provider::new(config_provider, database, cache, email);
        let _: RestServer = provider.provide();
    }
}
