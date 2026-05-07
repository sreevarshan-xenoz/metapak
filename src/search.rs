//! Enhanced search implementation for package matching.
//!
//! This module provides advanced search capabilities including:
//! - Fuzzy matching with scoring
//! - Query syntax: AND (space/&), OR (|), NOT (!)
//! - Quick filters: repo:, aur:, installed:, outdated:, size>:, size<:
//! - Regex mode support
//! - Search history

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum SearchToken {
    And(String),
    Or(String),
    Not(String),
    Filter(FilterType, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    Repo,
    Aur,
    Installed,
    Outdated,
    SizeGreater,
    SizeLess,
    Maintainer,
    License,
    Group,
    Regex,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub tokens: Vec<SearchToken>,
    pub is_regex: bool,
}

impl SearchQuery {
    pub fn parse(input: &str) -> Self {
        let mut tokens = Vec::new();
        let mut is_regex = false;

        let input = input.trim();

        if input.starts_with("regex:") {
            is_regex = true;
        }

        let parts = split_query(input);

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part.starts_with("repo:") {
                let value = part.trim_start_matches("repo:").trim();
                if !value.is_empty() {
                    tokens.push(SearchToken::Filter(FilterType::Repo, value.to_string()));
                }
            } else if part.starts_with("aur:") {
                let value = part.trim_start_matches("aur:").trim();
                if !value.is_empty() {
                    tokens.push(SearchToken::Filter(FilterType::Aur, value.to_string()));
                } else {
                    tokens.push(SearchToken::Filter(FilterType::Aur, "".to_string()));
                }
            } else if part.starts_with("installed:") {
                let value = part.trim_start_matches("installed:").trim();
                tokens.push(SearchToken::Filter(
                    FilterType::Installed,
                    value.to_string(),
                ));
            } else if part.starts_with("outdated:") {
                let value = part.trim_start_matches("outdated:").trim();
                tokens.push(SearchToken::Filter(FilterType::Outdated, value.to_string()));
            } else if part.starts_with("size>:") {
                let value = part.trim_start_matches("size>:").trim();
                tokens.push(SearchToken::Filter(
                    FilterType::SizeGreater,
                    value.to_string(),
                ));
            } else if part.starts_with("size<:") {
                let value = part.trim_start_matches("size<:").trim();
                tokens.push(SearchToken::Filter(FilterType::SizeLess, value.to_string()));
            } else if part.starts_with("maintainer:") {
                let value = part.trim_start_matches("maintainer:").trim();
                tokens.push(SearchToken::Filter(
                    FilterType::Maintainer,
                    value.to_string(),
                ));
            } else if part.starts_with("license:") {
                let value = part.trim_start_matches("license:").trim();
                tokens.push(SearchToken::Filter(FilterType::License, value.to_string()));
            } else if part.starts_with("group:") {
                let value = part.trim_start_matches("group:").trim();
                tokens.push(SearchToken::Filter(FilterType::Group, value.to_string()));
            } else if part.starts_with("regex:") {
                let value = part.trim_start_matches("regex:").trim();
                if !value.is_empty() {
                    tokens.push(SearchToken::Filter(FilterType::Regex, value.to_string()));
                }
            } else if part.starts_with('-') || part.starts_with('!') {
                let value = part.trim_start_matches('-').trim_start_matches('!');
                if !value.is_empty() {
                    tokens.push(SearchToken::Not(value.to_string()));
                }
            } else if part == "OR" || part == "|" {
                if let Some(SearchToken::And(prev)) = tokens.pop() {
                    tokens.push(SearchToken::Or(prev));
                }
                continue;
            } else {
                tokens.push(SearchToken::And(part.to_string()));
            }
        }

        Self { tokens, is_regex }
    }

    pub fn matches_package(
        &self,
        name: &str,
        description: &str,
        package: &crate::models::Package,
    ) -> bool {
        if self.tokens.is_empty() {
            return true;
        }

        let mut has_positive_match = false;

        for token in &self.tokens {
            match token {
                SearchToken::And(term) => {
                    let term_lower = term.to_lowercase();
                    let name_lower = name.to_lowercase();
                    let desc_lower = description.to_lowercase();

                    if name_lower.contains(&term_lower) || desc_lower.contains(&term_lower) {
                        has_positive_match = true;
                    } else {
                        return false;
                    }
                }
                SearchToken::Or(term) => {
                    let term_lower = term.to_lowercase();
                    let name_lower = name.to_lowercase();
                    let desc_lower = description.to_lowercase();

                    if name_lower.contains(&term_lower) || desc_lower.contains(&term_lower) {
                        return true;
                    }
                }
                SearchToken::Not(term) => {
                    let term_lower = term.to_lowercase();
                    let name_lower = name.to_lowercase();
                    let desc_lower = description.to_lowercase();

                    if name_lower.contains(&term_lower) || desc_lower.contains(&term_lower) {
                        return false;
                    }
                }
                SearchToken::Filter(filter_type, value) => match filter_type {
                    FilterType::Repo => {
                        if value.is_empty() {
                            if !matches!(package.source, crate::models::PackageSource::Pacman) {
                                return false;
                            }
                        } else if !value.eq_ignore_ascii_case("core")
                            && !value.eq_ignore_ascii_case("extra")
                            && !value.eq_ignore_ascii_case("community")
                        {
                            return false;
                        }
                    }
                    FilterType::Aur => {
                        if value.is_empty() {
                            if !matches!(package.source, crate::models::PackageSource::Aur) {
                                return false;
                            }
                        } else {
                            let term_lower = value.to_lowercase();
                            let name_lower = name.to_lowercase();
                            if !name_lower.contains(&term_lower) {
                                return false;
                            }
                        }
                        has_positive_match = true;
                    }
                    FilterType::Installed => {
                        let check_installed = value.eq_ignore_ascii_case("yes")
                            || value.eq_ignore_ascii_case("true")
                            || value.is_empty();
                        if check_installed != package.is_installed {
                            return false;
                        }
                        has_positive_match = true;
                    }
                    FilterType::Outdated => {
                        let check_outdated = value.eq_ignore_ascii_case("yes")
                            || value.eq_ignore_ascii_case("true")
                            || value.is_empty();
                        if check_outdated != package.is_outdated {
                            return false;
                        }
                        has_positive_match = true;
                    }
                    FilterType::SizeGreater => {
                        if let Some(size) = parse_size(value) {
                            if let Some(pkg_size) = package.installed_size {
                                if pkg_size <= size {
                                    return false;
                                }
                                has_positive_match = true;
                            } else {
                                return false;
                            }
                        }
                    }
                    FilterType::SizeLess => {
                        if let Some(size) = parse_size(value) {
                            if let Some(pkg_size) = package.installed_size {
                                if pkg_size >= size {
                                    return false;
                                }
                                has_positive_match = true;
                            } else {
                                return false;
                            }
                        }
                    }
                    FilterType::Maintainer => {
                        let term_lower = value.to_lowercase();
                        if !package
                            .maintainers
                            .iter()
                            .any(|m| m.to_lowercase().contains(&term_lower))
                        {
                            return false;
                        }
                        has_positive_match = true;
                    }
                    FilterType::License => {
                        let term_lower = value.to_lowercase();
                        if !package
                            .licenses
                            .iter()
                            .any(|l| l.to_lowercase().contains(&term_lower))
                        {
                            return false;
                        }
                        has_positive_match = true;
                    }
                    FilterType::Group => {
                        let term_lower = value.to_lowercase();
                        if !package
                            .groups
                            .iter()
                            .any(|g| g.to_lowercase().contains(&term_lower))
                        {
                            return false;
                        }
                        has_positive_match = true;
                    }
                    FilterType::Regex => {
                        if let Ok(re) = Regex::new(value) {
                            if !re.is_match(name) && !re.is_match(description) {
                                return false;
                            }
                            has_positive_match = true;
                        }
                    }
                },
            }
        }

        has_positive_match || self.tokens.iter().all(|t| matches!(t, SearchToken::Not(_)))
    }
}

fn split_query(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut parens_depth = 0;

    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '(' => {
                parens_depth += 1;
                current.push(ch);
            }
            ')' => {
                parens_depth -= 1;
                current.push(ch);
            }
            ' ' | '\t' if !in_quotes && parens_depth == 0 => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn parse_size(value: &str) -> Option<u64> {
    let value = value.trim().to_uppercase();
    let multiplier: u64 = if value.ends_with("TIB")
        || value.ends_with("TI")
        || value.ends_with("TB")
        || value.ends_with("T")
    {
        1024 * 1024 * 1024
    } else if value.ends_with("GIB")
        || value.ends_with("GI")
        || value.ends_with("GB")
        || value.ends_with("G")
    {
        1024 * 1024
    } else if value.ends_with("MIB")
        || value.ends_with("MI")
        || value.ends_with("MB")
        || value.ends_with("M")
    {
        1024
    } else if value.ends_with("KIB")
        || value.ends_with("KI")
        || value.ends_with("KB")
        || value.ends_with("K")
    {
        1
    } else {
        1
    };

    let num_part: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num_part
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64) as u64)
}

pub struct EnhancedSearch {
    fuzzy: FuzzySearch,
    history: Vec<String>,
    max_history: usize,
}

impl EnhancedSearch {
    pub fn new() -> Self {
        Self {
            fuzzy: FuzzySearch::new(),
            history: Vec::new(),
            max_history: 50,
        }
    }

    pub fn match_with_score(&self, text: &str, pattern: &str) -> Option<(i64, Vec<usize>)> {
        let query = SearchQuery::parse(pattern);
        if query.is_regex {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(text) {
                    let indices: Vec<usize> = text
                        .match_indices(
                            &regex::Regex::find(&re, text)
                                .map(|m| m.as_str())
                                .unwrap_or(""),
                        )
                        .map(|(i, _)| i)
                        .collect();
                    return Some((100, indices));
                }
            }
        }
        self.fuzzy.matcher.fuzzy_indices(text, pattern)
    }

    pub fn match_simple(&self, text: &str, pattern: &str) -> bool {
        self.match_with_score(text, pattern).is_some()
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

        let query = SearchQuery::parse(pattern);

        let mut results: Vec<(&'a str, i64, Vec<usize>)> = items
            .iter()
            .filter_map(|(name, desc)| {
                if query.is_regex {
                    if let Ok(re) = Regex::new(pattern) {
                        if re.is_match(name) || re.is_match(desc) {
                            return Some((name.as_str(), 100, vec![]));
                        }
                    }
                    None
                } else {
                    self.match_with_score(name, pattern)
                        .map(|(score, indices)| (name.as_str(), score, indices))
                }
            })
            .collect();

        results.sort_by_key(|(_name, score, _indices)| std::cmp::Reverse(*score));
        results
    }

    pub fn filter_packages<'a>(
        &self,
        packages: &'a [crate::models::Package],
        pattern: &str,
    ) -> Vec<&'a crate::models::Package> {
        let query = SearchQuery::parse(pattern);

        if pattern.is_empty() {
            return packages.iter().collect();
        }

        packages
            .iter()
            .filter(|pkg| query.matches_package(&pkg.name, &pkg.description, pkg))
            .collect()
    }

    pub fn add_to_history(&mut self, query: String) {
        if !query.trim().is_empty() && !self.history.contains(&query) {
            self.history.insert(0, query);
            if self.history.len() > self.max_history {
                self.history.pop();
            }
        }
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn get_suggestions(
        &self,
        prefix: &str,
        packages: &[crate::models::Package],
    ) -> Vec<String> {
        if prefix.len() < 2 {
            return vec![];
        }

        let query = SearchQuery::parse(prefix);

        let mut suggestions = Vec::new();
        let mut seen = HashSet::new();

        for pkg in packages {
            if suggestions.len() >= 10 {
                break;
            }

            if query.matches_package(&pkg.name, &pkg.description, pkg) {
                if seen.insert(pkg.name.clone()) {
                    suggestions.push(pkg.name.clone());
                }
            }
        }

        suggestions
    }
}

impl Default for EnhancedSearch {
    fn default() -> Self {
        Self::new()
    }
}

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
    fn test_query_parse_simple() {
        let query = SearchQuery::parse("firefox");
        assert_eq!(query.tokens.len(), 1);
        assert!(matches!(query.tokens[0], SearchToken::And(_)));
    }

    #[test]
    fn test_query_parse_and() {
        let query = SearchQuery::parse("firefox vim");
        assert_eq!(query.tokens.len(), 2);
    }

    #[test]
    fn test_query_parse_or() {
        let query = SearchQuery::parse("firefox | chrome");
        assert!(query.tokens.iter().any(|t| matches!(t, SearchToken::Or(_))));
    }

    #[test]
    fn test_query_parse_not() {
        let query = SearchQuery::parse("firefox -chrome");
        assert!(query
            .tokens
            .iter()
            .any(|t| matches!(t, SearchToken::Not(_))));
    }

    #[test]
    fn test_query_parse_filters() {
        let query = SearchQuery::parse("repo:core aur: true installed:");
        assert!(query
            .tokens
            .iter()
            .any(|t| matches!(t, SearchToken::Filter(FilterType::Repo, _))));
        assert!(query
            .tokens
            .iter()
            .any(|t| matches!(t, SearchToken::Filter(FilterType::Aur, _))));
        assert!(query
            .tokens
            .iter()
            .any(|t| matches!(t, SearchToken::Filter(FilterType::Installed, _))));
    }

    #[test]
    fn test_query_parse_regex() {
        let query = SearchQuery::parse("regex:^fire");
        assert!(query.is_regex);
    }

    #[test]
    fn test_enhanced_search_history() {
        let mut search = EnhancedSearch::new();
        search.add_to_history("firefox".to_string());
        search.add_to_history("vim".to_string());
        search.add_to_history("firefox".to_string());

        assert_eq!(search.get_history().len(), 2);
        assert_eq!(search.get_history()[0], "vim");
    }

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
    fn test_parse_size() {
        assert_eq!(parse_size("100MB"), Some(102400));
        assert_eq!(parse_size("1GB"), Some(1048576));
        assert_eq!(parse_size("500KB"), Some(500));
    }
}
