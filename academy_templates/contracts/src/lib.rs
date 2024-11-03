use std::fmt::Debug;

use academy_assets::templates;
use serde::Serialize;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait TemplateService: Send + Sync + 'static {
    /// Render the given template.
    fn render<T: Template + 'static>(&self, template: &T) -> anyhow::Result<String>;
}

#[cfg(feature = "mock")]
impl MockTemplateService {
    pub fn with_render<T: Template + Send + PartialEq + std::fmt::Debug + 'static>(
        mut self,
        template: T,
        result: String,
    ) -> Self {
        self.expect_render()
            .once()
            .with(mockall::predicate::eq(template))
            .return_once(|_| Ok(result));
        self
    }
}

pub trait Template: Serialize + Debug {
    const NAME: &'static str;
    const TEMPLATE: &'static str;
}

macro_rules! templates {
    ($( $ident:ident ( $template:expr ), )* ) => {
        $(
            impl Template for $ident {
                const NAME: &'static str = stringify!($ident);
                const TEMPLATE: &'static str = $template;
            }
        )*

        pub const TEMPLATES: &[(&str, &str)] = &[
            $( ($ident::NAME, $ident::TEMPLATE) ),*
        ];
    };
}

templates! {
    ResetPasswordTemplate(templates::RESET_PASSWORD_HTML),
    VerifyEmailTemplate(templates::VERIFY_EMAIL_HTML),
    SubscribeNewsletterTemplate(templates::SUBSCRIBE_NEWSLETTER_HTML),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResetPasswordTemplate {
    pub code: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyEmailTemplate {
    pub code: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubscribeNewsletterTemplate {
    pub code: String,
    pub url: String,
}
