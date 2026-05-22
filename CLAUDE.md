# Posto — Claude Code Guide

## Project Overview
Agent-based email manager. Built with Tauri 2 (Rust shell) + React 19. Multi-provider IMAP/OAuth in Rust, an LLM-driven agent pipeline (Triage / Summary / Action / Reflection) that turns the inbox into a task list, plus a four-layer language model (UI / display / reply / prompt all independent). The agent runs **in Rust** and exposes its results through Tauri commands — the React frontend is mainly UI + state.

## Tech Stack
- **Runtime**: Tauri 2 (Rust shell) + Vite 8 (renderer bundle)
- **Frontend**: React 19, Tailwind CSS 4 (CSS-first via `@import 'tailwindcss'` + `@config`), `cn()` helper over clsx + tailwind-merge, HSL CSS-variable design tokens (Apple Mail palette)
- **State**: Zustand 5 for **UI** state — `inboxStore`, `composeStore`, `settingsStore`. **Server data** (messages / accounts / agent runs) lives in TanStack Query over the Tauri IPC layer.
- **Data fetching**: TanStack Query v5 over `invoke()` in `src/lib/tauri.ts`
- **Router**: react-router-dom v6 (BrowserRouter)
- **Editor**: native textarea (no CodeMirror) — message composition is plain text + HTML sanitization
- **i18n**: i18next + react-i18next (four-layer language model — UI / display / reply / prompt all independent; see `src/i18n/`)
- **Sanitization**: isomorphic-dompurify for inbound HTML rendering (`src/lib/sanitizeHtml.ts`)
- **Tests**: Vitest 4 (Node env; targets `src/**/*.test.ts`)
- **Package manager**: Bun
- **Style enforcement**: `.editorconfig` (indent / EOL) + TypeScript strict mode + ESLint 10 flat config (`eslint.config.js` — JS/TS recommended + `react-hooks/{rules-of-hooks,exhaustive-deps}` + `react-refresh/only-export-components`)

## Repository Layout
```
src/                            # React frontend (Vite + Tauri webview)
  main.tsx                      # App entry, mounts QueryClient + BrowserRouter
  App.tsx                       # Route table + global chrome
  views/                        # Top-level routed views
    Inbox/                      #   message list + reading pane
    Compose/                    #   message draft editor
    Settings/                   #   account / LLM provider / language tabs
    Onboarding/                 #   first-run flow
  components/
    ui/                         # Apple Mail-inspired primitives (PascalCase):
                                #   Button / Card / Dialog / Input / Select /
                                #   SegmentedControl / Switch / Textarea /
                                #   Swipeable / Tooltip
    email/                      # Message-specific composites (EmailCard, …)
    activity/                   # Agent-activity feed components
    Sidebar.tsx                 # Account / folder nav
  stores/                       # Zustand UI stores
  hooks/                        # useMediaQuery (responsive sidebar drawer)
  lib/
    tauri.ts                    #   Typed wrapper around invoke() for every command
    sanitizeHtml.ts             #   DOMPurify config tuned for inbound email
    utils.ts                    #   cn() + formatRelativeTime()
  i18n/                         # i18next config + locales/
  messages/                     # Inbox view-models, derived from messages::* IPC
  types/                        # Activity / Compose / Email / Settings TS types
  index.css                     # @import 'tailwindcss' + @config + HSL tokens

src-tauri/                      # Rust shell — runs the whole backend
  src/
    main.rs / lib.rs            #   App entry, command registration
    commands.rs                 #   Built-ins (greet, app_version, open_external)
    error.rs                    #   App-wide error type
    accounts/                   #   Provider catalog, IMAP probe, OAuth flows
    sync/                       #   IMAP polling, attachment fetch, send via lettre
    storage/                    #   sqlx + SQLite + FTS5 schema and migrations
    messages/                   #   Inbox query layer, mark_read / flag / archive
    llm/                        #   LLM trait + OpenAI provider + key storage
    agents/                     #   Triage / Summary / Action / Reflection pipeline
    memory/                     #   5-type long-term memory + semantic search
    translation/                #   Cached per-message translation
  capabilities/                 #   Tauri permissions
  tauri.conf.json               #   Window/build/bundle config
```

**Sibling-project alignment.** [Verko](https://github.com/CatVinci-Studio/Verko) (also Tauri 2 + React 19 + Tailwind 4 + Bun) uses a different split: its agent runs in TS in the renderer, so it has `src/shared` (runtime-neutral TS used by both renderer and tests) and `src/renderer/src` (UI). Posto's agent lives in Rust, so the frontend is a flat `src/` — no shared layer needed.

## Architecture
```
React UI  ──IPC──▶  Rust Core
                     ├─ Accounts (provider catalog, OAuth + IMAP login)
                     ├─ Sync engine (polling; IDLE planned)
                     ├─ Storage (sqlx + SQLite + FTS5)
                     ├─ Messages (inbox query + mark/flag/archive/delete)
                     ├─ LLM (OpenAI; abstracted via `LlmProvider` trait)
                     ├─ Agents (Triage / Summary / Action / Reflection pipeline)
                     ├─ Memory (5 types, semantic retrieval)
                     └─ Translation (cached per-message)
```

## IPC Pattern
All cross-layer calls go through Tauri commands enumerated in `src-tauri/src/lib.rs::run`. Each subdomain (`accounts`, `sync`, `llm`, `agents`, `messages`, `memory`, `translation`) owns a `commands` module. The frontend wraps `@tauri-apps/api`'s `invoke()` in `src/lib/tauri.ts` with typed signatures — never call `invoke` directly from view code.

`open_external` (in `commands.rs`) is the single channel for opening URLs in the system browser via `tauri-plugin-opener`.

## Storage / Secrets
- **Messages, accounts, memories, agent runs** → SQLite via `sqlx` (full-text search on body via FTS5).
- **OAuth refresh tokens, IMAP passwords, LLM API keys** → OS keychain via `keyring` (apple-native / windows-native / secret-service). Never in plaintext, never logged.
- **User prefs** (UI language, display language, reply language, theme) → Zustand → `localStorage`.

## Trust model
Per project README: every agent action is undoable; send / delete always require explicit confirmation. Three trust levels for automation (manual / suggest / auto). LLM provider keys live only in the OS keychain.

## Theme System
HSL CSS variables in `src/index.css`. Apple Mail-inspired palette:
- **Light**: white background `#FFFFFF`, near-black text, iOS system blue `#007AFF` accent.
- **Dark**: iOS dark grouped grays (`#1c1c1e` / `#2c2c2e`), lighter system blue.

Theme switches by adding/removing the `dark` class on `<html>`. Tokens are consumed via Tailwind utilities (`bg-background`, `text-foreground`, `border-border`, etc.) — never hard-code hex.

## i18n — four-layer language model
Posto's language preferences are deliberately decoupled into four independent layers:
- **UI language** — chrome (menus / buttons / tooltips); switched via `react-i18next`.
- **Display language** — message body rendering language; controls `lang=` and any in-place translation overlay.
- **Reply language** — composed draft language; influences the LLM draft-reply prompt.
- **Prompt language** — language the LLM is **instructed in** (system + tool prompts).

The four don't have to match — e.g. UI in 中文, but reply in English for a foreign correspondent. See `src/i18n/` for config and `src/stores/settingsStore.ts` for persistence.

## Commands
```bash
bun run dev          # vite — renderer dev server (Tauri picks this up via beforeDevCommand)
bun run tauri dev    # full Tauri app (Rust shell + Vite renderer)
bun run build        # tsc -b && vite build — production renderer
bun run tauri build  # bundle .dmg / .msi / .AppImage / .deb / .rpm
bun test             # vitest run
bun run typecheck    # tsc --noEmit
bun run lint         # eslint over src/
bun run lint:fix     # eslint with --fix
```

## Mobile (iOS / Android)
The project is configured for Tauri 2 mobile builds: `tauri.conf.json` declares iOS / Android bundle keys, `capabilities/mobile.json` is in place, the UI is responsive (sidebar drawer + iOS-style swipe-to-archive in `InboxList`), and the deep-link plugin is configured for both desktop URL schemes and mobile `https://` association.

To initialize mobile platforms locally (one-time):
```bash
bun run tauri ios init
bun run tauri android init
bun run tauri ios dev       # or:  bun run tauri android dev
```

Known limitation on Android: `keyring` 3.x has no Android backend. OAuth refresh tokens and IMAP passwords need to migrate to a platform-specific secure store (`tauri-plugin-stronghold` recommended) before shipping Android builds.

## OAuth client IDs
Gmail / Outlook sign-in needs Desktop OAuth client IDs supplied at build time:
```bash
GOOGLE_OAUTH_CLIENT_ID=... MICROSOFT_OAUTH_CLIENT_ID=... bun run tauri build
```
Redirect URI for both is `posto://oauth/callback`.

## Testing
- Specs live under `src/**/__tests__/`.
- `vitest.config.ts` picks up `src/**/*.test.ts` in node env.
- Add a test next to the unit it covers — e.g. `src/lib/__tests__/utils.test.ts` for `src/lib/utils.ts`.

## Code Conventions
- **No hardcoded colors** in TSX. Use the design tokens (`bg-background`, `text-foreground`, `border-border`, `bg-primary`, …) or `hsl(var(--token))` for one-offs.
- **All IPC goes through `src/lib/tauri.ts`** — never call `invoke()` from a view; wrap it first.
- **Sanitize all inbound HTML** through `sanitizeHtml()` before rendering; never use `dangerouslySetInnerHTML` directly.
- **UI primitives**: PascalCase file (`Button.tsx`), `React.forwardRef` for ref-forwarding wrappers, variants/sizes as TS literal unions (see `Button.tsx` as the canonical example). Sibling project Verko uses kebab-case + cva + Radix for the same role — Posto's UI is hand-rolled to match the Apple Mail aesthetic and stays that way unless a primitive needs Radix's accessibility (focus trap, portal, etc.).
- **State**: keep Zustand stores narrow (one slice per concern); push server data into TanStack Query, not Zustand.
- **Comments explain WHY, not WHAT.** If a line of code needs a comment to be understood, rewrite the code first.
- **Tests** describe behavior in `it` strings (`it('returns the latest paint after merging concurrent edits')`), not implementation (`it('calls Object.assign')`).
- **Two-space indent, double quotes (TSX), no trailing whitespace** — locked by `.editorconfig` and existing code.
