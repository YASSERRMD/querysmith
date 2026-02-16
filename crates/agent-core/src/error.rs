use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Agent error: {0}")]
    Agent(String),
    #[error("Tool error: {0}")]
    Tool(String),
    #[error("LLM error: {0}")]
    Llm(String),
}
