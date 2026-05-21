// Long-term memory system.
//
// 5 memory types: contact / preference / project / rule / fact.
// Default scope: global (cross-account); per-account scope opt-in via settings.
// Embeddings: default cloud (OpenAI text-embedding-3-small), abstracted so
// local fastembed-rs can replace it.

pub mod commands;
pub mod retriever;
pub mod store;
pub mod writer;

pub use retriever::RetrievedMemory;
pub use writer::MemoryCandidate;
