// LLM provider abstraction.
//
// Submodules to be added by parallel work streams:
//   - provider: trait LlmProvider { complete, stream, embeddings, tools, translate }
//   - openai:   OpenAI implementation (API key + ChatGPT OAuth)
//   - registry: provider switching + token usage tracking
