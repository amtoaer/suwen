use serde::Serialize;
use suwen_entity::{identity, user};

use crate::auth::Identity;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityInfo {
    id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    display_name: String,
    is_anonymous: bool,
    is_admin: bool,
}

impl From<Identity> for IdentityInfo {
    fn from(value: Identity) -> Self {
        let is_admin = matches!(value, Identity::Admin { .. });
        match value {
            Identity::Authenticated { me, identity } | Identity::Admin { me, identity } => Self {
                id: identity.map(|i| i.id).unwrap_or(0),
                avatar_url: Some(me.avatar_url),
                display_name: me.display_name,
                is_anonymous: false,
                is_admin,
            },
            Identity::Anonymous { uuid, identity } => Self {
                id: identity.map(|i| i.id).unwrap_or(0),
                avatar_url: None,
                display_name: format!("匿名用户-{}", &uuid.as_simple().to_string()[26..]),
                is_anonymous: true,
                is_admin,
            },
            Identity::None => Self {
                id: 0,
                avatar_url: None,
                display_name: "匿名用户".to_owned(),
                is_anonymous: true,
                is_admin,
            },
        }
    }
}

impl From<(identity::Model, Option<user::Model>)> for IdentityInfo {
    fn from((identity, user): (identity::Model, Option<user::Model>)) -> Self {
        if let Some(user) = user {
            Self {
                id: identity.id,
                avatar_url: Some(user.avatar_url),
                display_name: user.display_name,
                is_anonymous: false,
                is_admin: user.id == 1,
            }
        } else if let Some(uuid) = identity.uuid {
            Self {
                id: identity.id,
                avatar_url: None,
                display_name: format!("匿名用户-{}", &uuid.as_simple().to_string()[26..]),
                is_anonymous: true,
                is_admin: false,
            }
        } else {
            Self {
                id: identity.id,
                avatar_url: None,
                display_name: "匿名用户".to_owned(),
                is_anonymous: true,
                is_admin: false,
            }
        }
    }
}
