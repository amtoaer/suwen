use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Content {
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub published_at: DateTime<Local>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Metadata {
    pub content: MetadataContent,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct MetadataContent {
    pub attachments: Vec<Attachment>,
    pub attributes: Vec<Attribute>,
    pub content: String,
    pub tags: Vec<String>,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Attachment {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Attribute {
    pub trait_type: String,
    pub value: serde_json::Value,
}
