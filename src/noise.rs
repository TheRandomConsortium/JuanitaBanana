use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::AppConfig;

pub trait NoiseProvider: Send + Sync {
    fn get_keywords(&self, count: usize) -> Vec<String>;
}

pub struct RssNoiseProvider {
    keywords: Arc<Mutex<Vec<String>>>,
}

impl RssNoiseProvider {
    pub fn new(config: &AppConfig) -> Self {
        let provider = Self {
            keywords: Arc::new(Mutex::new(Vec::new())),
        };

        let urls: Vec<String> = config.rss_sources.iter().map(|s| s.url.clone()).collect();
        let keywords_clone = provider.keywords.clone();

        // Spawn a background thread to fetch RSS feeds
        thread::spawn(move || {
            let mut all_words = Vec::new();
            for url in urls {
                if let Ok(resp) = reqwest::blocking::get(&url) {
                    if let Ok(text) = resp.text() {
                        if let Ok(doc) = roxmltree::Document::parse(&text) {
                            for node in doc.descendants() {
                                if node.has_tag_name("title") {
                                    if let Some(text) = node.text() {
                                        // Simple tokenization: split by space, remove punctuation
                                        let words: Vec<String> = text
                                            .split_whitespace()
                                            .map(|s| {
                                                s.chars()
                                                    .filter(|c| c.is_alphanumeric())
                                                    .collect::<String>()
                                                    .to_lowercase()
                                            })
                                            .filter(|s| s.len() > 3)
                                            .collect();
                                        
                                        // Group into n-grams (sizes 2 to 5) for more human-like queries
                                        for n in 2..=5 {
                                            for window in words.windows(n) {
                                                all_words.push(window.join(" "));
                                            }
                                        }
                                        all_words.extend(words);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !all_words.is_empty() {
                // Deduplicate
                all_words.sort();
                all_words.dedup();
                
                if let Ok(mut lock) = keywords_clone.lock() {
                    *lock = all_words;
                    println!("[NOISE] Loaded {} fresh keywords from RSS.", lock.len());
                }
            }
        });

        provider
    }
}

impl NoiseProvider for RssNoiseProvider {
    fn get_keywords(&self, count: usize) -> Vec<String> {
        let mut rng = thread_rng();
        if let Ok(lock) = self.keywords.lock() {
            if lock.is_empty() {
                return vec!["privacy".to_string(); count];
            }
            let mut selected = Vec::with_capacity(count);
            for _ in 0..count {
                if let Some(word) = lock.choose(&mut rng) {
                    selected.push(word.clone());
                }
            }
            selected
        } else {
            vec!["error".to_string(); count]
        }
    }
}
