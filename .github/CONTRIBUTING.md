# Contributing to Posto

Thanks for your interest in contributing! Here's everything you need to know.

## Branch strategy

| Branch | Purpose |
|--------|---------|
| `main` | Stable releases only. Never commit directly. |
| `dev` | Active development. All PRs should target this branch. |

## Development setup

```bash
git clone https://github.com/CatVinci-Studio/Posto.git
cd Posto
bun install
bun run tauri dev
```

Requires [Bun](https://bun.sh), [Rust](https://rustup.rs) (1.77+), and platform build tools (Xcode CLT on macOS; build-essential + WebKitGTK on Linux; MSVC on Windows).

## Before submitting a PR

```bash
bun run typecheck   # Must pass with 0 errors
bun run lint        # Must pass with 0 errors
bun test            # All tests must pass
```

## Commit style

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Outlook OAuth provider
fix: prevent crash when IMAP TLS handshake fails
docs: document four-layer language model
refactor: extract message FTS query into Repository
chore: bump tauri to 2.12
```

## Code conventions

- **No hardcoded colors** in TSX files — always use the design tokens (`bg-background`, `text-foreground`, …) or `hsl(var(--token))`.
- **All IPC goes through `src/lib/tauri.ts`** — never call `invoke()` from a view; wrap it first.
- **Sanitize all inbound HTML** through `sanitizeHtml()` before rendering.
- **Comments explain the WHY**, not the what. Rename the function before adding a what-comment.
- **Keep Tauri command handlers thin** — business logic belongs in the subdomain modules (`accounts`, `sync`, `llm`, `agents`), not in `*::commands`.

## Reporting bugs

Use the [Bug Report](.github/ISSUE_TEMPLATE/bug_report.yml) template. Include version, platform, reproduction steps, and logs.

## Requesting features

Use the [Feature Request](.github/ISSUE_TEMPLATE/feature_request.yml) template. Describe the problem first, then your proposed solution.
