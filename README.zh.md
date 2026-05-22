<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Posto" width="120" height="120" />
</p>

<h1 align="center">Posto</h1>

<p align="center">
  <strong>以 Agent 为中心的邮件管理工具。</strong><br>
  所有账号汇入一个收件箱。每封邮件都被读过、分类过、可以直接行动。
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

一个跨平台的桌面邮件客户端。Gmail、Outlook、iCloud、QQ、163,任意 IMAP —— 所有账号汇入同一个收件箱。AI Agent 在每封邮件抵达时就把它读过、分类、总结、抽出待办、起草回复,等你确认。

## 为什么

- **所有邮箱一个收件箱。** Gmail · Outlook · iCloud · QQ · 163 · 加任意 IMAP。支持 OAuth 的走 OAuth。
- **Agent 做分诊工作。** 自动分类、总结、抽待办、起草回复。长期学习你的联系人和项目。
- **四层独立语言模型。** UI · 阅读 · 回复 · 提示词 语言彼此独立 —— 中文界面读邮件,英文回外国客户。
- **密钥只在本地。** OAuth refresh token 和 API key 都进系统钥匙串,从不明文存储。

## 安装

| 平台 | 安装包 |
|---|---|
| macOS (Apple Silicon) | `Posto_X.Y.Z_aarch64.dmg` |
| Windows | `Posto_X.Y.Z_x64-setup.exe`(NSIS)· `_x64_en-US.msi`(WiX) |
| Linux | `Posto_X.Y.Z_amd64.AppImage` · `_amd64.deb` · `Posto-X.Y.Z-1.x86_64.rpm` |

→ 在 [Releases](https://github.com/CatVinci-Studio/Posto/releases/latest) 下载最新版。当前未签名,首次启动 macOS 需右键「打开」,Windows SmartScreen 选「更多信息 → 仍要运行」。

## 快速开始

1. 打开 Posto → **添加账号**,登录 Gmail / Outlook / iCloud(OAuth)或粘贴 IMAP 账密。
2. 进入**设置 → LLM Provider**,粘贴 OpenAI API key,点**测试连接**。
3. 进收件箱。新邮件同步进来时,Agent 自动跑 Triage / Summary / Action / Reflection。

## Agent 做什么

每封新邮件:

- **Triage** —— 按重要性和意图打标(订阅 / 私人 / 任务 / 收据……)
- **Summary** —— 用你的阅读语言生成「一行 + 三个要点」总结
- **Action** —— 抽待办、日期、关键附件;一键建任务
- **Reply drafts** —— 用你的回复语言起草;不会未经确认就发
- **Reflection** —— 周期性把模式蒸馏进长期记忆(「妈妈紧急事更喜欢用微信」「QQ 广告 = 直接归档」)

三档信任(手动 / 建议 / 自动)。发送和删除永远要确认。所有 Agent 动作可撤销。

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

需要 [Bun](https://bun.sh)、Rust(1.77+)、平台编译工具链(macOS:Xcode CLT;Linux:build-essential + WebKitGTK;Windows:MSVC)。Gmail / Outlook 登录需要构建时提供 OAuth client ID:

```bash
GOOGLE_OAUTH_CLIENT_ID=... MICROSOFT_OAUTH_CLIENT_ID=... bun run tauri build
```

两个 provider 的 redirect URI 都是:`posto://oauth/callback`。代码结构见 [CLAUDE.md](./CLAUDE.md)。

## License

[MIT](./LICENSE) © CatVinci Studio
