# Retposto

Agent-based email manager — turns your inbox into a task list, powered by AI.

Tauri 2 · Rust · React · TypeScript · cross-platform desktop (macOS, Windows, Linux).

## Highlights

- **One inbox for everything** — Gmail · Outlook · iCloud · QQ · 163, plus generic IMAP for the rest
- **Agent does the heavy lifting** — auto-classify, summarize, extract todos, draft replies
- **Long-term memory** — learns your contacts, preferences, ongoing projects, automation rules
- **Four-layer language model** — UI language, display language, reply language, prompt language all independent
- **Offline-first** — full content cached locally; agents run and queue actions offline
- **Trust-first automation** — three trust levels; send/delete always require confirmation; every agent action is undoable
- **Local-only secrets** — OAuth refresh tokens and API keys live in the system keychain, never in plaintext

## Quick start

Prerequisites: [Bun](https://bun.sh), [Rust](https://rustup.rs) (1.77+), and platform build tools (Xcode CLT on macOS; build-essential + WebKitGTK on Linux; MSVC on Windows).

```bash
bun install
bun run tauri dev
```

For a release bundle:

```bash
bun run tauri build
```

The bundles land in `src-tauri/target/release/bundle/` (`.app` / `.dmg` / `.msi` / `.deb` / `.AppImage`).

## Configuration

### LLM provider

Configure an OpenAI API key in Settings → LLM Provider. (ChatGPT OAuth sign-in is experimental and gated behind a future release.)

### OAuth client IDs

For Gmail / Outlook sign-in to work, register OAuth Desktop apps and provide the client IDs via env vars at build time:

```bash
GOOGLE_OAUTH_CLIENT_ID=... MICROSOFT_OAUTH_CLIENT_ID=... bun run tauri build
```

- Google Cloud Console → APIs & Services → OAuth client → Desktop application; add `retposto://oauth/callback` as redirect.
- Microsoft Entra (Azure AD) → App registrations → Public client; add `retposto://oauth/callback` as redirect.

## Mobile (iOS / Android)

The project is configured for Tauri 2 mobile builds: `tauri.conf.json` declares
the iOS / Android bundle keys, a `capabilities/mobile.json` capabilities file
is in place, the UI is responsive (sidebar drawer + iOS-style swipe-to-archive
in `InboxList`), and the deep-link plugin is configured for both desktop URL
schemes and mobile `https://` association.

To initialize mobile platforms locally (one-time, requires Xcode 14+ / Android
Studio + Android SDK 24+):

```bash
bun run tauri ios init
bun run tauri android init
bun run tauri ios dev       # or:  bun run tauri android dev
```

Known limitations on Android:

- `keyring` 3.x has no Android backend. OAuth refresh tokens and IMAP
  passwords need to be migrated to a platform-specific secure store before
  shipping Android builds — `tauri-plugin-stronghold` is the recommended
  replacement.

## Architecture

```
React UI  ──IPC──▶  Rust Core
                     ├─ Accounts (OAuth + IMAP)
                     ├─ Sync engine (polling; IDLE planned)
                     ├─ Storage (SQLite + FTS5)
                     ├─ LLM provider (OpenAI; abstracted)
                     ├─ Agents (Triage / Summary / Action / Reflection)
                     ├─ Memory (5 types, vector retrieval)
                     └─ Translation (cached per-message)
```

## License

MIT — see [LICENSE](./LICENSE).
