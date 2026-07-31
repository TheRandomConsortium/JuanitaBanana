use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::util::config::AppConfig;

pub trait NoiseProvider: Send + Sync {
    fn get_keywords(&self, count: usize) -> Vec<String>;
}

/// Tokenizes text into words and generates n-grams (sizes 1 to 5)
pub fn tokenize_to_ngrams(text: &str) -> Vec<String> {
    let mut all_words = Vec::new();
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
    all_words
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
                                        all_words.extend(tokenize_to_ngrams(text));
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
                    crate::log!(
                        Info,
                        NOISE,
                        "Loaded {} fresh keywords from RSS.",
                        lock.len()
                    );
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
            vec!["privacy".to_string(); count]
        }
    }
}

pub struct GossipNoiseProvider;

impl NoiseProvider for GossipNoiseProvider {
    fn get_keywords(&self, count: usize) -> Vec<String> {
        let mut rng = thread_rng();
        let mut selected = Vec::with_capacity(count);

        if let Ok(pool_lock) = crate::privacy::search::gossip::GLOBAL_NOISE_POOL.lock() {
            let pool_terms = pool_lock.get_all_terms();
            if !pool_terms.is_empty() {
                for _ in 0..count {
                    if let Some(entry) = pool_terms.choose(&mut rng) {
                        selected.push(entry.term.clone());
                    }
                }
            }
        }
        selected
    }
}

pub struct WeightedCompositeNoiseProvider<P1: NoiseProvider, P2: NoiseProvider> {
    pub provider1: P1,
    pub provider2: P2,
    pub provider1_weight_percent: u8,
}

impl<P1: NoiseProvider, P2: NoiseProvider> NoiseProvider for WeightedCompositeNoiseProvider<P1, P2> {
    fn get_keywords(&self, count: usize) -> Vec<String> {
        let weight1 = self.provider1_weight_percent.min(100) as usize;
        let count1 = (count * weight1) / 100;
        let count2 = count.saturating_sub(count1);

        let mut res = Vec::with_capacity(count);
        if count1 > 0 {
            res.extend(self.provider1.get_keywords(count1));
        }
        if count2 > 0 {
            res.extend(self.provider2.get_keywords(count2));
        }

        while res.len() < count {
            res.push("privacy".to_string());
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_to_ngrams() {
        let input = "The quick! brown fox... jumps over the lazy dog.";
        let ngrams = tokenize_to_ngrams(input);

        // Single words (filtered len > 3)
        assert!(ngrams.contains(&"quick".to_string()));
        assert!(ngrams.contains(&"brown".to_string()));
        assert!(ngrams.contains(&"jumps".to_string()));
        assert!(ngrams.contains(&"over".to_string()));
        assert!(ngrams.contains(&"lazy".to_string()));

        // Excluded short words (len <= 3)
        assert!(!ngrams.contains(&"the".to_string()));
        assert!(!ngrams.contains(&"fox".to_string()));
        assert!(!ngrams.contains(&"dog".to_string()));

        // Check some n-grams
        assert!(ngrams.contains(&"quick brown".to_string()));
        assert!(ngrams.contains(&"quick brown jumps".to_string()));
        assert!(ngrams.contains(&"quick brown jumps over".to_string()));
        assert!(ngrams.contains(&"quick brown jumps over lazy".to_string()));
    }

    #[test]
    fn test_gossip_and_composite_noise_providers() {
        if let Ok(mut pool) = crate::privacy::search::gossip::GLOBAL_NOISE_POOL.lock() {
            pool.add_term("privacy tool".to_string(), "node-1".to_string(), 30);
        }

        let gossip_provider = GossipNoiseProvider;
        let terms = gossip_provider.get_keywords(3);
        assert_eq!(terms.len(), 3);
        assert_eq!(terms[0], "privacy tool");

        struct DummyProvider;
        impl NoiseProvider for DummyProvider {
            fn get_keywords(&self, count: usize) -> Vec<String> {
                vec!["rss_term".to_string(); count]
            }
        }

        let composite = WeightedCompositeNoiseProvider {
            provider1: DummyProvider,
            provider2: GossipNoiseProvider,
            provider1_weight_percent: 50,
        };
        let comp_terms = composite.get_keywords(4);
        assert_eq!(comp_terms.len(), 4);
        assert!(comp_terms.contains(&"rss_term".to_string()));
        assert!(comp_terms.contains(&"privacy tool".to_string()));
    }
}
