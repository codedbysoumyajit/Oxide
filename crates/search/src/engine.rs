use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use ignore::WalkBuilder;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub is_regex: bool,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub max_results: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResultMatch {
    pub path: PathBuf,
    pub line_number: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub line_text: String,
}

pub struct SearchEngine;

impl SearchEngine {
    pub fn search(roots: &[PathBuf], query: &SearchQuery) -> Vec<SearchResultMatch> {
        if query.query.is_empty() || roots.is_empty() {
            return Vec::new();
        }

        // Collect searchable file paths respecting .gitignore
        let mut file_paths = Vec::new();
        for root in roots {
            let walker = WalkBuilder::new(root)
                .hidden(true)
                .git_ignore(true)
                .parents(true)
                .build();

            for result in walker.flatten() {
                if result.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    file_paths.push(result.into_path());
                }
            }
        }

        let regex_pattern = if query.is_regex {
            query.query.clone()
        } else {
            let escaped = regex::escape(&query.query);
            if query.whole_word {
                format!(r"\b{}\b", escaped)
            } else {
                escaped
            }
        };

        let regex_builder = regex::RegexBuilder::new(&regex_pattern)
            .case_insensitive(!query.case_sensitive)
            .build();

        let Ok(re) = regex_builder else {
            return Vec::new();
        };

        let total_matches = Arc::new(AtomicUsize::new(0));
        let max_results = query.max_results;

        let results: Vec<SearchResultMatch> = file_paths
            .par_iter()
            .flat_map(|path| {
                if total_matches.load(Ordering::Relaxed) >= max_results {
                    return Vec::new();
                }

                let mut file_matches = Vec::new();
                if let Ok(file) = File::open(path) {
                    let reader = BufReader::new(file);
                    for (line_idx, line_res) in reader.lines().enumerate() {
                        if total_matches.load(Ordering::Relaxed) >= max_results {
                            break;
                        }

                        if let Ok(line) = line_res {
                            for mat in re.find_iter(&line) {
                                total_matches.fetch_add(1, Ordering::Relaxed);
                                file_matches.push(SearchResultMatch {
                                    path: path.clone(),
                                    line_number: line_idx + 1,
                                    col_start: mat.start(),
                                    col_end: mat.end(),
                                    line_text: line.clone(),
                                });

                                if total_matches.load(Ordering::Relaxed) >= max_results {
                                    break;
                                }
                            }
                        }
                    }
                }
                file_matches
            })
            .collect();

        results
    }
}
