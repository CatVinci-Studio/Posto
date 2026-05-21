# Retposto

Agent-based 邮件管理器。Tauri 2 + Rust + React + TypeScript。

## 开发

```bash
npm install
npm run tauri dev
```

## 支持的邮箱

第一方支持（流畅登录）：

- **Gmail** — OAuth 2.0
- **Outlook / Microsoft 365** — OAuth 2.0
- **iCloud Mail** — App-Specific Password
- **QQ 邮箱** — IMAP 授权码
- **163 / 126 网易** — IMAP 授权码

其他邮箱通过通用 IMAP/SMTP 入口手动配置。

## 状态

P0 脚手架。详见对话上下文里的开发行程。
