use academy_utils::patch::Patch;
use chrono::{DateTime, Utc};
use nutype::nutype;

use crate::{macros::id, user::UserId, Sha256Hash};

id!(SessionId);

#[derive(Debug, Clone, PartialEq, Eq, Patch)]
pub struct Session {
    #[no_patch]
    pub id: SessionId,
    #[no_patch]
    pub user_id: UserId,
    pub device_name: Option<DeviceName>,
    #[no_patch]
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[nutype(
    validate(len_char_max = DeviceName::MAX_LEN),
    derive(Debug, Clone, PartialEq, Eq, Deref, TryFrom, Serialize, Deserialize)
)]
pub struct DeviceName(String);

impl DeviceName {
    const MAX_LEN: usize = 256;

    pub fn from_string_truncated(mut s: String) -> Self {
        s.truncate(Self::MAX_LEN);
        Self::try_new(s).unwrap()
    }
}

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Deref, From, Serialize, Deserialize,))]
pub struct SessionRefreshTokenHash(Sha256Hash);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_name_from_string_truncated() {
        // Arrange
        let input = std::iter::once('A')
            .chain(std::iter::repeat('B'))
            .take(DeviceName::MAX_LEN + 20)
            .collect();
        let expected = std::iter::once('A')
            .chain(std::iter::repeat('B'))
            .take(DeviceName::MAX_LEN)
            .collect::<String>();

        // Act
        let result = DeviceName::from_string_truncated(input);

        // Assert
        assert_eq!(result.into_inner(), expected);
    }
}
