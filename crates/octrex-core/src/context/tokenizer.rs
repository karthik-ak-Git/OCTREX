use crate::context::errors::ContextError;
use crate::context::types::TokenCountKind;
use crate::models::runtime::{DefaultTokenizer, TokenCount, Tokenizer};

pub struct ContextTokenCounter {
    inner: Box<dyn Tokenizer>,
    safety_factor: f32, // Multiplier for estimated counts (e.g. 1.15 for 15% margin)
}

impl Default for ContextTokenCounter {
    fn default() -> Self {
        Self::new(Box::new(DefaultTokenizer), 1.15)
    }
}

impl ContextTokenCounter {
    pub fn new(tokenizer: Box<dyn Tokenizer>, safety_factor: f32) -> Self {
        Self {
            inner: tokenizer,
            safety_factor: safety_factor.max(1.0),
        }
    }

    pub fn default_counter() -> Self {
        Self::default()
    }

    pub fn count(&self, text: &str) -> Result<(usize, TokenCountKind), ContextError> {
        match self.inner.count_tokens(text) {
            TokenCount::Exact(n) => Ok((n, TokenCountKind::Exact)),
            TokenCount::Estimated(n) => {
                let conservative = (n as f32 * self.safety_factor).ceil() as usize;
                Ok((conservative, TokenCountKind::Estimated))
            }
            TokenCount::Unknown => {
                if text.is_empty() {
                    Ok((0, TokenCountKind::Unknown))
                } else {
                    // Conservative estimation: ~3 chars per token
                    let fallback = (text.chars().count() + 2) / 3;
                    let conservative = (fallback as f32 * self.safety_factor).ceil() as usize;
                    Ok((conservative, TokenCountKind::Unknown))
                }
            }
        }
    }

    pub fn merge_kinds(a: TokenCountKind, b: TokenCountKind) -> TokenCountKind {
        match (a, b) {
            (TokenCountKind::Exact, TokenCountKind::Exact) => TokenCountKind::Exact,
            (TokenCountKind::Unknown, _) | (_, TokenCountKind::Unknown) => TokenCountKind::Unknown,
            _ => TokenCountKind::Estimated,
        }
    }
}
