use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WikiPage {
    pub(crate) id: u64,
    pub(crate) path: String,
    pub(crate) hash: String,
    title: String,
    description: String,
    pub(crate) content: String,
    content_type: String,
    created_at: String,
    updated_at: String,
    editor_key: String,
    locale_code: String,
    author_id: u64,
    creator_id: u64,
}

impl From<RenamedWikiPage> for WikiPage {
    fn from(renamed: RenamedWikiPage) -> Self {
        WikiPage {
            id: renamed.id,
            path: renamed.destination_path,
            hash: renamed.destination_hash,
            title: renamed.title,
            description: renamed.description,
            content: renamed.content,
            content_type: renamed.content_type,
            created_at: renamed.created_at,
            updated_at: renamed.updated_at,
            editor_key: renamed.editor_key,
            locale_code: renamed.locale_code,
            author_id: renamed.author_id,
            creator_id: renamed.creator_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenamedWikiPage {
    id: u64,
    path: String,
    hash: String,
    title: String,
    description: String,
    content: String,
    content_type: String,
    created_at: String,
    updated_at: String,
    editor_key: String,
    locale_code: String,
    author_id: u64,
    creator_id: u64,
    destination_path: String,
    destination_locale_code: String,
    destination_hash: String,
}
