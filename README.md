<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Posto" width="120" height="120" />
</p>

<h1 align="center">Posto</h1>

<p align="center">
  <strong>Agent-based email manager.</strong><br>
  Turns your inbox into a task list. Multi-account, multilingual, agent-first.
</p>

<p align="center">
  <a href="https://github.com/CatVinci-Studio/Posto/releases/latest"><strong>Download</strong></a> ·
  <a href="./README.zh.md">中文</a>
</p>

<p align="center">
  <a href="https://github.com/CatVinci-Studio/Posto/releases/latest"><img alt="version" src="https://img.shields.io/github/v/release/CatVinci-Studio/Posto"></a>
  <img alt="platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey">
  <a href="./LICENSE"><img alt="license" src="https://img.shields.io/badge/license-MIT-yellow"></a>
</p>

---

## What it is

A cross-platform desktop email client (Tauri 2 + Rust + React) where every account — Gmail, Outlook, iCloud, QQ, 163, or any IMAP — lands in one inbox. An LLM-driven agent reads each message, classifies it, summarizes it, extracts todos, drafts replies, and remembers what matters about your contacts and projects.

## Why

- **One inbox for everything.** Gmail · Outlook · iCloud · QQ · 163 · plus generic IMAP for the rest. OAuth where supported, app passwords elsewhere.
- **Agent does the heavy lifting.** Auto-classify, summarize, extract todos, draft replies. Long-term memory learns your contacts, ongoing projects, automation rules.
- **Four-layer language model.** UI / display / reply / prompt languages are all independent — read 中文 chrome with English replies for a foreign correspondent, or vice versa.
- **Offline-first.** Full content cached locally; agents run and queue actions offline; resync when you're back online.
- **Trust-first automation.** Three trust levels (manual / suggest / auto); send and delete always require confirmation; every agent action is undoable.
- **Local-only secrets.** OAuth refresh tokens and API keys live in the system keychain, never in plaintext.

## Install

| Platform | Installer |
|---|---|
| macOS (Apple Silicon) | `Posto_X.Y.Z_aarch64.dmg` |
| macOS (Intel) | `Posto_X.Y.Z_x64.dmg` |
| Windows | `Posto_X.Y.Z_x64-setup.exe` (NSIS) · `_x64_en-US.msi` (WiX) |
| Linux | `Posto_X.Y.Z_amd64.AppImage` · `_amd64.deb` · `Posto-X.Y.Z-1.x86_64.rpm` |

→ Get the latest at [Releases](https://github.com/CatVinci-Studio/Posto/releases/latest). Builds are unsigned for now — first launch may need a right-click → Open on macOS, or "More info → Run anyway" on Windows SmartScreen.

## Quick start

1. Launch Posto → **Add account**, sign in to Gmail / Outlook / iCloud (OAuth) or paste IMAP credentials.
2. Open **Settings → LLM Provider**, paste an OpenAI API key, click **Test connection**.
3. Open the inbox. The agent runs Triage / Summary / Action / Reflection on new messages as they sync.

## What the agent can do

- **Triage** — labels each message by importance and intent (newsletter / personal / task / receipt …).
- **Summary** — one-line + 3-bullet summary per thread; multilingual.
- **Action** — extracts todos, dates, attachments-of-interest; offers to create tasks.
- **Reply drafts** — produces a draft in your preferred reply language; never sends without confirmation.
- **Reflection** — periodically distills patterns into long-term memory ("Mom prefers WeChat for urgent matters"; "QQ ads = always archive").

The agent pipeline lives entirely in Rust (`src-tauri/src/agents/`). The React frontend only displays results — no LLM keys touch the renderer.

## Architecture

```
React UI  ──IPC──▶  Rust Core
                     ├─ Accounts (provider catalog, OAuth + IMAP)
                     ├─ Sync engine (polling; IDLE planned)
                     ├─ Storage (sqlx + SQLite + FTS5)
                     ├─ Messages (inbox query + mark/flag/archive/delete)
                     ├─ LLM (OpenAI; abstracted via LlmProvider trait)
                     ├─ Agents (Triage / Summary / Action / Reflection)
                     ├─ Memory (5 types, semantic retrieval)
                     └─ Translation (cached per-message)
```

## OAuth client IDs

Gmail / Outlook sign-in requires Desktop OAuth client IDs supplied at build time:

```bash
GOOGLE_OAUTH_CLIENT_ID=... MICROSOFT_OAUTH_CLIENT_ID=... bun run tauri build
```

Redirect URI for both providers: `posto://oauth/callback`.

- Google Cloud Console → APIs & Services → OAuth client → Desktop application.
- Microsoft Entra (Azure AD) → App registrations → Public client.

## Build from source

```bash
git clone https://github.com/CatVinci-Studio/Posto.git
cd Posto
bun install

bun run tauri dev     # full Tauri app (Rust shell + Vite renderer)
bun run tauri build   # bundle .dmg / .msi / .AppImage / .deb / .rpm
bun test              # vitest unit tests
bun run typecheck     # tsc --noEmit
bun run lint          # eslint over src/
```

Requires [Bun](https://bun.sh), [Rust](https://rustup.rs) (1.77+), and platform build tools (Xcode CLT on macOS; build-essential + WebKitGTK on Linux; MSVC on Windows). Codebase layout: [CLAUDE.md](./CLAUDE.md).

## Mobile (iOS / Android)

`tauri.conf.json` declares iOS / Android bundle keys; `capabilities/mobile.json` is in place; the UI is responsive (sidebar drawer + iOS-style swipe-to-archive in `InboxList`). To initialize mobile platforms (one-time):

```bash
bun run tauri ios init
bun run tauri android init
bun run tauri ios dev       # or:  bun run tauri android dev
```

Android caveat: `keyring` 3.x has no Android backend; OAuth refresh tokens need to migrate to `tauri-plugin-stronghold` before shipping Android builds.

## License

[MIT](./LICENSE) © CatVinci Studio
