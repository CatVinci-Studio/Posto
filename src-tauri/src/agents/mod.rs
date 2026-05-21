// Agent orchestration.
//
// Pipeline: Triage -> Summary -> Action -> Rule -> Draft.
// All system prompts in English; output language is a variable.
//
// Trust levels (default = medium):
//   - aggressive: more auto-execution
//   - medium:     auto-execute reversible actions with undo
//   - conservative: ask before non-trivial actions
//
// Destructive actions (send / delete) ALWAYS require user confirmation
// regardless of trust level.
