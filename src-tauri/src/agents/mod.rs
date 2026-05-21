// Agent orchestration.
//
// Pipeline: Triage -> Summary+Action (parallel) -> Reflection.
// All system prompts in English; output language is a variable.
//
// Trust levels (default = medium):
//   - aggressive: more auto-execution
//   - medium:     auto-execute reversible actions with undo
//   - conservative: ask before non-trivial actions
//
// Destructive actions (send / delete) ALWAYS require user confirmation
// regardless of trust level.

pub mod action;
pub mod commands;
pub mod pipeline;
pub mod prompts;
pub mod reflection;
pub mod summary;
pub mod triage;

pub use action::ActionOutput;
pub use pipeline::{run_pipeline, PipelineOutput};
pub use reflection::ReflectionOutput;
pub use summary::SummaryOutput;
pub use triage::TriageOutput;
