<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Posto" width="120" height="120" />
</p>

<h1 align="center">Posto</h1>

<p align="center">
  <strong>以 Agent 为中心的邮件管理工具。</strong><br>
  把收件箱变成任务列表。多账号、多语言、Agent 优先。
</p>

<p align="center">
  <a href="https://github.com/CatVinci-Studio/Posto/releases/latest"><strong>下载</strong></a> ·
  <a href="./README.md">English</a>
</p>

<p align="center">
  <a href="https://github.com/CatVinci-Studio/Posto/releases/latest"><img alt="version" src="https://img.shields.io/github/v/release/CatVinci-Studio/Posto"></a>
  <img alt="platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey">
  <a href="./LICENSE"><img alt="license" src="https://img.shields.io/badge/license-MIT-yellow"></a>
</p>

---

## 这是什么

一个跨平台的桌面邮件客户端(Tauri 2 + Rust + React)。Gmail、Outlook、iCloud、QQ、163、任意 IMAP —— 所有账号汇入同一个收件箱。LLM 驱动的 Agent 自动读每一封邮件:分类、总结、抽取待办、起草回复,并把"和谁聊过什么、有哪些进行中的项目"长期记下来。

## 为什么

- **所有邮箱一个收件箱。** Gmail · Outlook · iCloud · QQ · 163,加任意 IMAP。支持 OAuth 的走 OAuth,不支持的走应用密码。
- **杂活交给 Agent。** 自动分类、总结、抽待办、起草回复。长期记忆学习你的联系人、项目、自动化规则。
- **四层独立语言模型。** UI / 阅读 / 回复 / 提示词 语言彼此独立 —— 中文界面 + 英文回信外国客户,或反过来,都行。
- **离线优先。** 邮件正文全部本地缓存;断网时 Agent 照跑,操作进队列;恢复联网后同步。
- **可信先于自动。** 三档信任(手动 / 建议 / 自动);发送和删除永远要确认;所有 Agent 动作可撤销。
- **密钥只在本地。** OAuth refresh token 和 API key 都进系统钥匙串,从不明文存储。

## 安装

| 平台 | 安装包 |
|---|---|
| macOS (Apple Silicon) | `Posto_X.Y.Z_aarch64.dmg` |
| macOS (Intel) | `Posto_X.Y.Z_x64.dmg` |
| Windows | `Posto_X.Y.Z_x64-setup.exe`(NSIS)· `_x64_en-US.msi`(WiX) |
| Linux | `Posto_X.Y.Z_amd64.AppImage` · `_amd64.deb` · `Posto-X.Y.Z-1.x86_64.rpm` |

→ 在 [Releases](https://github.com/CatVinci-Studio/Posto/releases/latest) 下载最新版。当前未签名,首次启动 macOS 需右键「打开」,Windows SmartScreen 选「更多信息 → 仍要运行」。

## 快速开始

1. 打开 Posto → **添加账号**,登录 Gmail / Outlook / iCloud(OAuth)或粘贴 IMAP 账密。
2. 进入**设置 → LLM Provider**,粘贴 OpenAI API key,点**测试连接**。
3. 进收件箱。新邮件同步进来时,Agent 会自动跑 Triage / Summary / Action / Reflection。

## Agent 能做什么

- **Triage** —— 按重要性和意图给每封邮件打标(订阅 / 私人 / 任务 / 收据……)。
- **Summary** —— 每个会话一行 + 三个要点的多语言总结。
- **Action** —— 抽取待办、日期、关键附件;支持一键建任务。
- **Reply drafts** —— 按你的回复语言起草;不会未经确认就发送。
- **Reflection** —— 周期性把模式蒸馏进长期记忆("妈妈紧急事更喜欢用微信"、"QQ 广告 = 直接归档")。

Agent pipeline 完全在 Rust(`src-tauri/src/agents/`)里跑。前端只负责展示 —— LLM 密钥从不进 renderer 进程。

## 架构

```
React UI  ──IPC──▶  Rust Core
                     ├─ Accounts(供应商目录、OAuth + IMAP)
                     ├─ Sync engine(轮询;IDLE 计划中)
                     ├─ Storage(sqlx + SQLite + FTS5)
                     ├─ Messages(收件箱查询 + mark/flag/archive/delete)
                     ├─ LLM(OpenAI;通过 LlmProvider trait 抽象)
                     ├─ Agents(Triage / Summary / Action / Reflection)
                     ├─ Memory(5 种类型、语义检索)
                     └─ Translation(每封邮件缓存)
```

## OAuth Client ID

Gmail / Outlook 登录需要构建时提供 Desktop OAuth client ID:

```bash
GOOGLE_OAUTH_CLIENT_ID=... MICROSOFT_OAUTH_CLIENT_ID=... bun run tauri build
```

两个 provider 的 redirect URI 都是:`posto://oauth/callback`。

- Google Cloud Console → APIs & Services → OAuth client → Desktop application。
- Microsoft Entra(Azure AD)→ App registrations → Public client。

## 从源码构建

```bash
git clone https://github.com/CatVinci-Studio/Posto.git
cd Posto
bun install

bun run tauri dev     # 桌面端
bun run tauri build   # 安装包输出到 src-tauri/target/release/bundle/
bun test              # vitest 单测
bun run typecheck     # tsc --noEmit
bun run lint          # eslint 全 src/
```

需要 [Bun](https://bun.sh)、Rust(1.77+)、平台编译工具链(macOS:Xcode CLT;Linux:build-essential + WebKitGTK;Windows:MSVC)。代码结构见 [CLAUDE.md](./CLAUDE.md)。

## 移动端(iOS / Android)

`tauri.conf.json` 已声明 iOS / Android bundle 配置;`capabilities/mobile.json` 已就位;UI 已响应式(侧栏抽屉 + iOS 风滑动归档)。首次初始化移动平台:

```bash
bun run tauri ios init
bun run tauri android init
bun run tauri ios dev       # 或 bun run tauri android dev
```

Android 注意事项:`keyring` 3.x 没有 Android 后端,在发 Android 之前需要把 OAuth refresh token 迁到 `tauri-plugin-stronghold`。

## License

[MIT](./LICENSE) © CatVinci Studio
