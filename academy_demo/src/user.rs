use std::sync::LazyLock;

use academy_models::user::{User, UserComposite, UserDetails, UserInvoiceInfo, UserLocale, UserPassword, UserProfile};
use academy_persistence_contracts::user::UserRepository;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use uuid::uuid;

pub static ALL_USERS: LazyLock<Vec<&UserComposite>> =
    LazyLock::new(|| vec![&ADMIN, &ADMIN2, &FOO, &BAR]);

pub static ADMIN: LazyLock<UserComposite> = LazyLock::new(|| UserComposite {
    user: User {
        id: uuid!("e3f8a50a-a5a3-444a-9026-77336f716d03").into(),
        name: "admin".try_into().unwrap(),
        email: Some("admin@example.com".parse().unwrap()),
        email_verified: true,
        created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        last_login: Some(Utc.with_ymd_and_hms(2024, 4, 7, 10, 23, 0).unwrap()),
        last_name_change: None,
        enabled: true,
        admin: true,
        newsletter: false,
        preferred_language: UserLocale::De,
        timezone: Tz::Europe__Berlin,
    },
    profile: UserProfile {
        display_name: "Administrator".try_into().unwrap(),
        bio: Default::default(),
        tags: Default::default(),
    },
    details: UserDetails {
        mfa_enabled: false,
        password_login: true,
        oauth2_login: false,
    },
    invoice_info: UserInvoiceInfo::default(),
});

pub static ADMIN_PASSWORD: LazyLock<UserPassword> =
    LazyLock::new(|| "secure admin password".try_into().unwrap());

pub static ADMIN2: LazyLock<UserComposite> = LazyLock::new(|| UserComposite {
    user: User {
        id: uuid!("4670f4e0-f227-4b8d-a95f-22806acb27ee").into(),
        name: "admin2".try_into().unwrap(),
        email: Some("admin2@example.com".parse().unwrap()),
        email_verified: true,
        created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        last_login: Some(Utc.with_ymd_and_hms(2024, 4, 7, 10, 23, 0).unwrap()),
        last_name_change: None,
        enabled: true,
        admin: true,
        newsletter: true,
        preferred_language: UserLocale::De,
        timezone: Tz::Europe__Berlin,
    },
    profile: UserProfile {
        display_name: "Administrator2".try_into().unwrap(),
        bio: Default::default(),
        tags: Default::default(),
    },
    details: UserDetails {
        mfa_enabled: true,
        password_login: true,
        oauth2_login: false,
    },
    invoice_info: UserInvoiceInfo::default(),
});

pub static ADMIN2_PASSWORD: LazyLock<UserPassword> =
    LazyLock::new(|| "secure admin2 password".try_into().unwrap());

pub static FOO: LazyLock<UserComposite> = LazyLock::new(|| UserComposite {
    user: User {
        id: uuid!("a8d95e0f-71ae-4c49-995e-695b7c93848c").into(),
        name: "foo".try_into().unwrap(),
        email: Some("foo@example.com".parse().unwrap()),
        email_verified: true,
        created_at: Utc.with_ymd_and_hms(2024, 3, 14, 13, 37, 42).unwrap(),
        last_login: Some(Utc.with_ymd_and_hms(2024, 3, 15, 13, 37, 0).unwrap()),
        last_name_change: Some(Utc.with_ymd_and_hms(2024, 3, 14, 13, 50, 0).unwrap()),
        enabled: true,
        admin: false,
        newsletter: true,
        preferred_language: UserLocale::De,
        timezone: Tz::Europe__Berlin,
    },
    profile: UserProfile {
        display_name: "Foo 42".try_into().unwrap(),
        bio: "blubb".try_into().unwrap(),
        tags: ["foo", "bar", "baz"]
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .try_into()
            .unwrap(),
    },
    details: UserDetails {
        mfa_enabled: false,
        password_login: true,
        oauth2_login: true,
    },
    invoice_info: UserInvoiceInfo {
        business: Some(true),
        first_name: Some("x".try_into().unwrap()),
        last_name: Some("y".try_into().unwrap()),
        street: Some("asdf".try_into().unwrap()),
        zip_code: Some("1234".try_into().unwrap()),
        city: Some("xyz".try_into().unwrap()),
        country: Some("asdf".try_into().unwrap()),
        vat_id: Some("1234".try_into().unwrap()),
    },
});

pub static FOO_PASSWORD: LazyLock<UserPassword> =
    LazyLock::new(|| "foo password".try_into().unwrap());

pub static BAR: LazyLock<UserComposite> = LazyLock::new(|| UserComposite {
    user: User {
        id: uuid!("94d0e3ca-bf16-486b-a172-b87f4bcbd039").into(),
        name: "bar".try_into().unwrap(),
        email: None,
        email_verified: false,
        created_at: Utc.with_ymd_and_hms(2024, 6, 28, 3, 14, 15).unwrap(),
        last_login: None,
        last_name_change: None,
        enabled: false,
        admin: false,
        newsletter: false,
        preferred_language: UserLocale::En,
        timezone: Tz::UTC,
    },
    profile: UserProfile {
        display_name: "Bar".try_into().unwrap(),
        bio: "a very interesting text".try_into().unwrap(),
        tags: ["42", "7"]
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .try_into()
            .unwrap(),
    },
    details: UserDetails {
        mfa_enabled: false,
        password_login: true,
        oauth2_login: false,
    },
    invoice_info: UserInvoiceInfo::default(),
});

pub static BAR_PASSWORD: LazyLock<UserPassword> =
    LazyLock::new(|| "password for bar".try_into().unwrap());

fn hash_password(password: &str) -> anyhow::Result<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(Into::into)
}

pub async fn create<Txn: Send + Sync + 'static>(
    txn: &mut Txn,
    repo: impl UserRepository<Txn>,
) -> anyhow::Result<()> {
    for &user in &*ALL_USERS {
        repo.create(txn, &user.user, &user.profile, &user.invoice_info)
            .await?;
    }

    repo.save_password_hash(txn, ADMIN.user.id, hash_password(&ADMIN_PASSWORD)?)
        .await?;
    repo.save_password_hash(txn, ADMIN2.user.id, hash_password(&ADMIN2_PASSWORD)?)
        .await?;
    repo.save_password_hash(txn, FOO.user.id, hash_password(&FOO_PASSWORD)?)
        .await?;
    repo.save_password_hash(txn, BAR.user.id, hash_password(&BAR_PASSWORD)?)
        .await?;

    Ok(())
}
