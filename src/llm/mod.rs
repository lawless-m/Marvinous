//! LLM interaction module
//!
//! "I'd make a suggestion, but you wouldn't listen. No one ever does."

pub mod ollama;
pub mod prompt;

pub use ollama::OllamaClient;
pub use prompt::build_prompt;
