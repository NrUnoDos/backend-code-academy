use academy_core_oauth2_contracts::OAuth2Service;
use academy_demo::oauth2::{TEST_OAUTH2_PROVIDER, TEST_OAUTH2_PROVIDER_ID};
use academy_extern_contracts::oauth2::MockOAuth2ApiService;
use academy_models::oauth2::OAuth2ProviderSummary;
use url::Url;

use super::Sut;
use crate::OAuth2ServiceImpl;

#[test]
fn ok() {
    // Arrange
    let auth_url = Url::parse("http://test/auth?client_id=test-id").unwrap();

    let oauth2_api = MockOAuth2ApiService::new()
        .with_generate_auth_url(TEST_OAUTH2_PROVIDER.clone(), auth_url.clone());

    let sut = OAuth2ServiceImpl {
        oauth2_api,
        ..Sut::default()
    };

    // Act
    let result = sut.list_providers();

    // Assert
    assert_eq!(
        result,
        [OAuth2ProviderSummary {
            id: TEST_OAUTH2_PROVIDER_ID.clone(),
            name: TEST_OAUTH2_PROVIDER.name.clone(),
            auth_url
        }]
    )
}
