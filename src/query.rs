use crate::page::WikiPage;
use glob::Pattern;
use meilisearch_sdk::search::SearchResults;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_page(id: u64, content: &str) -> WikiPage {
        let v = json!({
            "id": id,
            "path": format!("/p/{id}"),
            "hash": format!("h{id}"),
            "title": format!("Title {id}"),
            "description": "desc",
            "content": content,
            "contentType": "markdown",
            "createdAt": "2020-01-01T00:00:00Z",
            "updatedAt": "2020-01-01T00:00:00Z",
            "editorKey": "code",
            "localeCode": "en",
            "authorId": 1,
            "creatorId": 1
        });
        serde_json::from_value(v).expect("valid WikiPage json")
    }

    fn make_result(
        query: &str,
        pages: Vec<WikiPage>,
        total_hits: Option<usize>,
    ) -> SerializableSearchResult {
        SerializableSearchResult {
            hits: pages,
            offset: None,
            limit: None,
            estimated_total_hits: None,
            page: None,
            hits_per_page: None,
            total_hits,
            total_pages: None,
            processing_time_ms: 1,
            query: query.to_string(),
            index_uid: None,
        }
    }

    #[test]
    fn suggestions_basic_match() {
        let p1 = make_page(1, "First line\nSome query match here\nAnother line");
        let p2 = make_page(2, "Completely unrelated\nNothing to see");
        let res = make_result("match", vec![p1, p2], None);

        let page_response = PageSearchResponse::from(res);

        // order not guaranteed; check membership
        assert!(
            page_response
                .suggestions
                .iter()
                .any(|s| s == "Some query match here")
        );
        assert_eq!(page_response.results.len(), 2);
        assert_eq!(page_response.total_hits, 2);
    }

    #[test]
    fn suggestions_excludes_exact_line_match() {
        // exact lower-case match line should be excluded
        let p = make_page(1, "hello world\nThis has hello world inside\nHELLO WORLD");
        let res = make_result("hello world", vec![p], None);
        let out = PageSearchResponse::from(res);

        // cleaned content removes punctuation only, so exact line "hello world" exists and must be excluded
        assert!(!out.suggestions.iter().any(|s| s == "hello world"));
        // but the line that contains the query should be included
        assert!(
            out.suggestions
                .iter()
                .any(|s| s == "This has hello world inside")
        );
    }

    #[test]
    fn suggestions_are_deduplicated() {
        let p1 = make_page(1, "foo bar baz\nneedle in a haystack");
        let p2 = make_page(2, "prefix needle in a haystack suffix\nother");
        let res = make_result("needle", vec![p1, p2], None);
        let out = PageSearchResponse::from(res);

        // Two different lines match but only one identical appears once when identical
        // In this case, lines differ, ensure at least one expected is present and dedup holds when identical
        assert!(
            out.suggestions
                .iter()
                .any(|s| s.contains("needle") && s.contains("haystack"))
        );

        // Now check true de-dup using identical lines across pages
        let p3 = make_page(3, "repeat me please\nother");
        let p4 = make_page(4, "repeat me please\nzzz");
        let res2 = make_result("repeat", vec![p3, p4], None);
        let out2 = PageSearchResponse::from(res2);
        assert_eq!(out2.suggestions.len(), 1);
        assert_eq!(out2.suggestions[0], "repeat me please");
    }

    #[test]
    fn total_hits_prefers_reported_total() {
        let p = make_page(1, "some content");
        let res = make_result("content", vec![p], Some(42));
        let out = PageSearchResponse::from(res);
        assert_eq!(out.total_hits, 42);
    }
}
