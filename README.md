<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Posto" width="120" height="120" />
</p>

<h1 align="center">Posto</h1>

<p align="center">
  <strong>Agent-based email manager.</strong><br>
  Every account in one inbox. Every message read, classified, and ready to act on.
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

A cross-platform desktop email client where every account — Gmail, Outlook, iCloud, QQ, 163, or any IMAP — lands in one inbox. An AI agent reads each message as it arrives, classifies it, summarizes it, extracts the todos, and drafts replies for you to approve.

## Why

- **One inbox for everything.** Gmail · Outlook · iCloud · QQ · 163 · plus generic IMAP. OAuth where supported.
- **Agent does the triage.** Auto-classify, summarize, extract todos, draft replies. Learns your contacts and projects over time.
- **Four-layer language model.** UI · display · reply · prompt languages all independent — read 中文 chrome and reply in English to foreign correspondents.
- **Local-only secrets.** OAuth refresh tokens and API keys live in the system keychain, never in plaintext.

## Install

| Platform | Installer |
|---|---|
| macOS (Apple Silicon) | `Posto_X.Y.Z_aarch64.dmg` |
| Windows | `Posto_X.Y.Z_x64-setup.exe` (NSIS) · `_x64_en-US.msi` (WiX) |
| Linux | `Posto_X.Y.Z_amd64.AppImage` · `_amd64.deb` · `Posto-X.Y.Z-1.x86_64.rpm` |

→ Get the latest at [Releases](https://github.com/CatVinci-Studio/Posto/releases/latest). Builds are unsigned for now — first launch may need a right-click → Open on macOS, or "More info → Run anyway" on Windows SmartScreen.

## Quick start

1. Launch Posto → **Add account**, sign in to Gmail / Outlook / iCloud (OAuth) or paste IMAP credentials for the rest.
2. Open **Settings → LLM Provider**, paste an OpenAI API key, click **Test connection**.
3. Open the inbox. The agent runs Triage / Summary / Action / Reflection on new messages as they sync.

## What the agent does

For every incoming message:

- **Triage** — labels by importance and intent (newsletter / personal / task / receipt …)
- **Summary** — one-line + 3-bullet summary, in your display language
- **Action** — extracts todos, dates, attachments-of-interest; offers to create tasks
- **Reply drafts** — in your reply language; never sends without confirmation
- **Reflection** — periodically distills patterns into long-term memory ("Mom prefers WeChat for urgent matters", "QQ ads = always archive")

Three trust levels (manual / suggest / auto). Send and delete always require confirmation. Every agent action is undoable.

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

Requires [Bun](https://bun.sh), [Rust](https://rustup.rs) (1.77+), and platform build tools (Xcode CLT on macOS; build-essential + WebKitGTK on Linux; MSVC on Windows). Gmail / Outlook sign-in needs Desktop OAuth client IDs supplied at build time:

```bash
GOOGLE_OAUTH_CLIENT_ID=... MICROSOFT_OAUTH_CLIENT_ID=... bun run tauri build
```

Redirect URI for both: `posto://oauth/callback`. Codebase layout: [CLAUDE.md](./CLAUDE.md).

## License

[MIT](./LICENSE) © CatVinci Studio
