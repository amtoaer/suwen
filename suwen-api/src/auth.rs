#![allow(unused)]

use std::sync::LazyLock;

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use suwen_config::CONFIG;

#[derive(Serialize, Deserialize)]
pub(crate) struct Claims {
    pub id: i32,
    pub passwd_version: usize,
    pub exp: usize,
}

impl Claims {
    pub fn of(me: suwen_entity::user::Model, ttl: usize) -> Self {
        Self {
            id: me.id,
            passwd_version: 1,
            exp: chrono::Utc::now().timestamp() as usize + ttl,
        }
    }

    pub fn encode(&self) -> String {
        static ENCODING_KEY: LazyLock<EncodingKey> =
            LazyLock::new(|| EncodingKey::from_secret(CONFIG.jwt_secret.as_bytes()));
        let header = jsonwebtoken::Header::default();
        jsonwebtoken::encode(&header, &self, &*ENCODING_KEY).expect("Failed to encode JWT")
    }

    pub fn decode(token: &str) -> Result<Self> {
        static DECODING_KEY: LazyLock<DecodingKey> =
            LazyLock::new(|| DecodingKey::from_secret(CONFIG.jwt_secret.as_bytes()));
        let data = jsonwebtoken::decode::<Self>(
            token,
            &*DECODING_KEY,
            &jsonwebtoken::Validation::default(),
        )?;
        Ok(data.claims)
    }
}

#[derive(Clone)]
pub(super) enum Identity {
    Admin { me: suwen_entity::user::Model },
    Authenticated { me: suwen_entity::user::Model },
    Anonymous { id: uuid::Uuid },
    None,
}
