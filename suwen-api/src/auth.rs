#![allow(unused)]

use std::sync::LazyLock;

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, IntoActiveModel, JoinType, TransactionTrait,
    TryIntoModel,
};
use sea_orm::{ActiveValue::Set, DatabaseConnection};
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use serde::{Deserialize, Serialize};
use suwen_config::CONFIG;

#[derive(Serialize, Deserialize)]
pub(crate) struct Claims {
    pub id: i32,
    pub exp: usize,
}

impl Claims {
    pub fn of(me: suwen_entity::user::Model, ttl: usize) -> Self {
        Self {
            id: me.id,
            exp: chrono::Utc::now().timestamp() as usize + ttl,
        }
    }

    pub fn encode(&self) -> String {
        static ENCODING_KEY: LazyLock<EncodingKey> =
            LazyLock::new(|| EncodingKey::from_secret(CONFIG.jwt_secret.as_bytes()));
        let header = jsonwebtoken::Header::default();
        jsonwebtoken::encode(&header, &self, &ENCODING_KEY).expect("Failed to encode JWT")
    }

    pub fn decode(token: &str) -> Result<Self> {
        static DECODING_KEY: LazyLock<DecodingKey> =
            LazyLock::new(|| DecodingKey::from_secret(CONFIG.jwt_secret.as_bytes()));
        let data = jsonwebtoken::decode::<Self>(
            token,
            &DECODING_KEY,
            &jsonwebtoken::Validation::default(),
        )?;
        Ok(data.claims)
    }
}

#[derive(Clone)]
pub(super) enum Identity {
    Admin {
        me: suwen_entity::user::Model,
        identity: Option<suwen_entity::identity::Model>,
    },
    Authenticated {
        me: suwen_entity::user::Model,
        identity: Option<suwen_entity::identity::Model>,
    },
    Anonymous {
        uuid: uuid::Uuid,
        identity: Option<suwen_entity::identity::Model>,
    },
    None,
}

impl Identity {
    /// 尝试获取当前的 identity，如果没有 identity 则返回 None
    pub fn identity(&self) -> Option<&suwen_entity::identity::Model> {
        match self {
            Identity::None => None,
            Identity::Anonymous { identity, .. } => identity.as_ref(),
            Identity::Authenticated { identity, .. } => identity.as_ref(),
            Identity::Admin { identity, .. } => identity.as_ref(),
        }
    }

    /// 如果不存在 identity 则新建，确保 identity 存在，在用户进行交互操作时调用
    pub async fn ensure_identity(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<&suwen_entity::identity::Model> {
        match self {
            Identity::None => Err(anyhow::anyhow!("No identity available")),
            Identity::Anonymous { uuid, identity } => match identity {
                Some(identity) => Ok(identity),
                None => {
                    let new_identity = suwen_entity::identity::ActiveModel {
                        uuid: Set(Some(*uuid)),
                        ..Default::default()
                    }
                    .save(db)
                    .await?;
                    *identity = Some(new_identity.try_into_model()?);
                    Ok(identity.as_ref().unwrap())
                }
            },
            Identity::Authenticated { me, identity } | Identity::Admin { me, identity } => {
                match identity {
                    Some(identity) => Ok(identity),
                    None => {
                        let new_identity = suwen_entity::identity::ActiveModel {
                            user_id: Set(Some(me.id)),
                            ..Default::default()
                        }
                        .save(db)
                        .await?;
                        *identity = Some(new_identity.try_into_model()?);
                        Ok(identity.as_ref().unwrap())
                    }
                }
            }
        }
    }
}
