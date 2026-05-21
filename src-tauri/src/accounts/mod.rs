// Account management module.
//
// Submodules will be added by parallel work streams:
//   - provider_catalog: static table mapping domains to provider configs
//   - imap_password:    IMAP + password / app-password / 授权码
//   - oauth:            OAuth 2.0 for Gmail, Outlook
//
// Supported first-class providers: Gmail, Outlook, iCloud, QQ, 163.
// Anything else falls through to generic IMAP/SMTP manual entry.
