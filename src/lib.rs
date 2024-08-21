mod logger;
mod page;
mod query;
mod transformers;

use crate::page::RenamedWikiPage;
use crate::query::{PageSearchResponse, SerializableSearchResult};
use logger::WasmLogger;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::documents::DocumentDeletionQuery;
use page::WikiPage;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct WikiSearchEngine {
    client: Client,
    index_name: String,
}

impl WikiSearchEngine {
    fn log_and_return_js_error<E: std::fmt::Display>(message: &str, error: E) -> JsValue {
        log::error!("{}: {}", message, error);
        JsValue::from_str(&format!("{}: {}", message, error))
    }

    async fn execute_query(&self, q: &str) -> Result<SerializableSearchResult, JsValue> {
        log::info!("Executing query: {}", q);

        self.client
            .index(&self.index_name)
            .search()
            .with_attributes_to_search_on(&[
                "title",
                "description",
                "content",
                "tags",
                "author_name",
            ])
            .with_query(q)
            .execute()
            .await
            .map_err(|e| Self::log_and_return_js_error("Error executing query", e))
            .map(SerializableSearchResult::from)
    }

    async fn handle_document_operation<F, Fut>(
        &self,
        page: &JsValue,
        operation: F,
    ) -> Result<(), JsValue>
    where
        F: FnOnce(WikiPage) -> Fut,
        Fut: std::future::Future<Output = Result<(), JsValue>>,
    {
        let page = from_value::<RenamedWikiPage>(page.clone())
            .map(WikiPage::from) // Directly map to WikiPage if successful
            .or_else(|_e| {
                // If deserialization to RenamedWikiPage fails, try deserializing to WikiPage
                from_value::<WikiPage>(page.clone()).map_err(|inner_e| {
                    // If both deserialization fail, log the error and return a JS error
                    Self::log_and_return_js_error("Error deserializing page", inner_e)
                })
            })?;

        if page.is_published && !page.is_private {
            operation(page).await
        } else {
            log::warn!(
                "Skipping operation on unpublished or private page: {}",
                page.id
            );
            Ok(())
        }
    }

    async fn add_document(&self, page: WikiPage) -> Result<(), JsValue> {
        log::info!("Adding document: {:?}", serde_json::to_string_pretty(&page));
        self.client
            .index(&self.index_name)
            .add_documents(&[page], Some("id"))
            .await
            .map(|_| ())
            .map_err(|e| Self::log_and_return_js_error("Error adding document", e))
    }
}

#[wasm_bindgen]
impl WikiSearchEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(
        millisearch_host: String,
        millisearch_api_key: String,
        index_name: String,
        _timeout: u64,
    ) -> Result<WikiSearchEngine, JsValue> {
        WasmLogger::init("millisearch::WikiSearchEngine");

        let host = millisearch_host.clone();
        let client = Client::new(host.clone(), Some(millisearch_api_key))
            .map_err(|e| WikiSearchEngine::log_and_return_js_error("Failed to create client", e))?;

        log::info!("WikiSearchEngine created with index: {}", index_name);

        Ok(WikiSearchEngine { client, index_name })
    }

    #[wasm_bindgen]
    pub async fn healthcheck(&self) -> Result<(), JsValue> {
        log::info!("Health checking search engine");
        self.client
            .health()
            .await
            .map(|_| ())
            .map_err(|e| WikiSearchEngine::log_and_return_js_error("Error health checking", e))
    }

    #[wasm_bindgen]
    pub async fn activated(&self) -> Result<(), JsValue> {
        log::info!("Activating search engine");

        self.client
            .create_index(self.index_name.as_str(), Some("id"))
            .await
            .map_err(|e| {
                WikiSearchEngine::log_and_return_js_error("Error creating index", e);
            })
            .expect("Error creating index")
            .wait_for_completion(&self.client, None, None)
            .await
            .map_err(|e| {
                WikiSearchEngine::log_and_return_js_error("Error creating index", e);
            })
            .expect("Error creating index");
        let index = self.client.index(self.index_name.as_str());
        index
            .set_filterable_attributes(&["path", "hash", "id"])
            .await
            .map_err(|e| {
                WikiSearchEngine::log_and_return_js_error("Error setting filterable attributes", e);
            })
            .expect("Error setting filterable attributes");
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn suggest(&self, q: &str) -> Result<JsValue, JsValue> {
        log::info!("Suggesting query: {}", q);

        let results = self.execute_query(q).await?;

        log::info!("Suggestion results found: {}", results.hits.len());

        to_value(&PageSearchResponse::from(results))
            .map_err(|e| WikiSearchEngine::log_and_return_js_error("Error serializing results", e))
    }

    #[wasm_bindgen]
    pub async fn query(&self, q: &str) -> Result<JsValue, JsValue> {
        log::info!("Querying: {}", q);

        let results = self.execute_query(q).await?;

        log::info!("Query results found: {}", results.hits.len());

        to_value(&PageSearchResponse::from(results)).map_err(|e| {
            WikiSearchEngine::log_and_return_js_error("Error serializing query results", e)
        })
    }

    #[wasm_bindgen]
    pub async fn created(&self, page: &JsValue) -> Result<(), JsValue> {
        self.handle_document_operation(page, |page| async move { self.add_document(page).await })
            .await
    }

    #[wasm_bindgen]
    pub async fn updated(&self, page: &JsValue) -> Result<(), JsValue> {
        self.handle_document_operation(page, |page| async move { self.add_document(page).await })
            .await
    }

    #[wasm_bindgen]
    pub async fn deleted(&self, page: &JsValue) -> Result<(), JsValue> {
        self.handle_document_operation(page, |page| async move {
            log::warn!("Deleting page: {}", page.id);
            let index = self.client.index(&self.index_name);
            let filter = format!("path = {}", page.path);
            let mut binding = DocumentDeletionQuery::new(&index);
            let query = binding.with_filter(&filter);
            index
                .delete_documents_with(query)
                .await
                .map(|_| ())
                .map_err(|e| Self::log_and_return_js_error("Error deleting document", e))
        })
        .await
    }
}
