use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("RAG error: {0}")]
    Rag(String),
    #[error("Embedding error: {0}")]
    Embedding(String),
    #[error("Vector store error: {0}")]
    VectorStore(String),
}
