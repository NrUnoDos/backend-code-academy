use chrono::{DateTime, Utc};
use nutype::nutype;
use url::Url;

use crate::{
    macros::{id, nutype_string},
    user::UserId,
};

id!(OAuth2LinkId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2Provider {
    pub name: OAuth2ProviderName,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: Url,
    pub token_url: Url,
    pub userinfo_url: Url,
    pub userinfo_id_key: String,
    pub userinfo_name_key: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2ProviderSummary {
    pub id: OAuth2ProviderId,
    pub name: OAuth2ProviderName,
    pub auth_url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2Link {
    pub id: OAuth2LinkId,
    pub user_id: UserId,
    pub provider_id: OAuth2ProviderId,
    pub created_at: DateTime<Utc>,
    pub remote_user: OAuth2UserInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2UserInfo {
    pub id: OAuth2RemoteUserId,
    pub name: OAuth2RemoteUserName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2Login {
    pub provider_id: OAuth2ProviderId,
    pub code: OAuth2AuthorizationCode,
    pub redirect_uri: Url,
}

nutype_string!(OAuth2ProviderId);
nutype_string!(OAuth2ProviderName);

nutype_string!(OAuth2AuthorizationCode(validate(len_char_max = 256)));

nutype_string!(OAuth2RemoteUserId(validate(len_char_max = 256)));
nutype_string!(OAuth2RemoteUserName(validate(len_char_max = 256)));
