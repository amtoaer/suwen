use serde::Serialize;

use crate::auth::Identity;

#[derive(Serialize)]
pub(crate) struct IdentityInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    is_anonymous: bool,
    is_admin: bool,
}

impl From<Identity> for IdentityInfo {
    fn from(value: Identity) -> Self {
        let is_admin = matches!(value, Identity::Admin { .. });
        match value {
            Identity::Authenticated { me } | Identity::Admin { me } => Self {
                avatar_url: Some(me.avatar_url),
                display_name: Some(me.display_name),
                is_anonymous: false,
                is_admin,
            },
            Identity::Anonymous { id } => Self {
                avatar_url: None,
                display_name: Some(id.simple().to_string()),
                is_anonymous: true,
                is_admin,
            },
            Identity::None => Self {
                avatar_url: None,
                display_name: None,
                is_anonymous: true,
                is_admin,
            },
        }
    }
}
