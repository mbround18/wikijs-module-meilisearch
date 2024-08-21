use crate::page::WikiPage;
use glob::Pattern;
use meilisearch_sdk::search::SearchResults;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FacetStats {
    // Define fields for FacetStats based on your requirements
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableSearchResult {
    pub hits: Vec<WikiPage>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub estimated_total_hits: Option<usize>,
    pub page: Option<usize>,
    pub hits_per_page: Option<usize>,
    pub total_hits: Option<usize>,
    pub total_pages: Option<usize>,
    pub processing_time_ms: usize,
    pub query: String,
    pub index_uid: Option<String>,
}

impl From<SearchResults<WikiPage>> for SerializableSearchResult {
    fn from(search_results: SearchResults<WikiPage>) -> Self {
        SerializableSearchResult {
            hits: search_results
                .hits
                .into_iter()
                .map(|search_result| search_result.result)
                .collect(),
            offset: search_results.offset,
            limit: search_results.limit,
            estimated_total_hits: search_results.estimated_total_hits,
            page: search_results.page,
            hits_per_page: search_results.hits_per_page,
            total_hits: search_results.total_hits,
            total_pages: search_results.total_pages,
            processing_time_ms: search_results.processing_time_ms,
            query: search_results.query,
            index_uid: search_results.index_uid,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageSearchResponse {
    pub suggestions: Vec<String>,
    pub results: Vec<WikiPage>,
    pub total_hits: usize,
}

impl From<SerializableSearchResult> for PageSearchResponse {
    fn from(serializable: SerializableSearchResult) -> Self {
        let results = serializable.hits;
        let total_hits = serializable.total_hits.unwrap_or(results.len());

        let query = serializable.query.to_lowercase();
        let search_pattern =
            Pattern::new(&format!("*{}*", query)).expect("Failed to create search pattern");

        let non_alphanumeric_regex =
            Regex::new(r"[^a-zA-Z0-9\s]").expect("Failed to compile regex");

        let unique_suggestions: HashSet<String> = results
            .iter()
            .filter_map(|result| {
                let cleaned_content = non_alphanumeric_regex.replace_all(&result.content, "");

                cleaned_content.lines().find_map(|line| {
                    let line_lower = line.to_lowercase();
                    let is_match = search_pattern.matches(&line_lower);
                    let is_exact_match = line.trim() == query.trim();

                    if is_match && !is_exact_match {
                        Some(line.to_string())
                    } else {
                        None
                    }
                })
            })
            .collect();

        PageSearchResponse {
            suggestions: unique_suggestions.into_iter().collect(),
            results,
            total_hits,
        }
    }
}
