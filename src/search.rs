//! Fuzzy search implementation for package matching.
//!
//! This module provides fuzzy matching with scoring and highlighted results.

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct FuzzySearch {
    matcher: SkimMatcherV2,
}

impl FuzzySearch {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn match_with_score(&self, text: &str, pattern: &str) -> Option<(i64, Vec<usize>)> {
        self.matcher.fuzzy_indices(text, pattern)
    }

    pub fn match_simple(&self, text: &str, pattern: &str) -> bool {
        self.matcher.fuzzy_match(text, pattern).is_some()
    }

    pub fn filter_and_sort<'a>(
        &self,
        items: &'a [(String, String)],
        pattern: &str,
    ) -> Vec<(&'a str, i64, Vec<usize>)> {
        if pattern.is_empty() {
            return items
                .iter()
                .map(|(name, _)| (name.as_str(), 0, vec![]))
                .collect();
        }

        let mut results: Vec<(&'a str, i64, Vec<usize>)> = items
            .iter()
            .filter_map(|(name, _)| {
                self.match_with_score(name, pattern)
                    .map(|(score, indices)| (name.as_str(), score, indices))
            })
            .collect();

        results.sort_by_key(|(_name, score, _indices)| std::cmp::Reverse(*score));
        results
    }
}

impl Default for FuzzySearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        let search = FuzzySearch::new();

        assert!(search.match_simple("package-name", "pkg"));
        assert!(search.match_simple("package-name", "pkge"));
        assert!(search.match_simple("firefox", "fire"));
        assert!(search.match_simple("firefox", "fox"));
        assert!(!search.match_simple("firefox", "chrome"));
    }

    #[test]
    fn test_fuzzy_indices() {
        let search = FuzzySearch::new();

        let result = search.match_with_score("package-name", "pkname");
        assert!(result.is_some());

        let (_, indices) = result.unwrap();
        assert!(!indices.is_empty());
    }

    #[test]
    fn test_filter_and_sort() {
        let search = FuzzySearch::new();

        let items = vec![
            ("firefox".to_string(), "".to_string()),
            ("chromium".to_string(), "".to_string()),
            ("firefox-i18n".to_string(), "".to_string()),
            ("pacman".to_string(), "".to_string()),
        ];

        let results = search.filter_and_sort(&items, "fire");
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "firefox");
    }
}
