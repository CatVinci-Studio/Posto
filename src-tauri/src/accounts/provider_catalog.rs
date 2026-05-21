use crate::accounts::provider::{
    AuthMethod, Encryption, ImapConfig, ProviderConfig, ProviderKind, SmtpConfig,
};

/// Detect a provider from an email address by inspecting its domain.
/// Returns `None` for unknown domains (caller should fall through to GenericImap).
pub fn detect_by_domain(email: &str) -> Option<ProviderKind> {
    let domain = email.split('@').nth(1)?.to_lowercase();

    // Gmail
    if domain == "gmail.com" || domain == "googlemail.com" {
        return Some(ProviderKind::Gmail);
    }

    // Outlook / Microsoft (exact + any outlook.* subdomain)
    if matches!(
        domain.as_str(),
        "outlook.com" | "hotmail.com" | "live.com" | "msn.com"
    ) || domain.starts_with("outlook.")
    {
        return Some(ProviderKind::Outlook);
    }

    // iCloud
    if matches!(domain.as_str(), "icloud.com" | "me.com" | "mac.com") {
        return Some(ProviderKind::ICloud);
    }

    // QQ
    if matches!(domain.as_str(), "qq.com" | "foxmail.com" | "vip.qq.com") {
        return Some(ProviderKind::Qq);
    }

    // 163 / NetEase
    if matches!(domain.as_str(), "163.com" | "126.com" | "yeah.net") {
        return Some(ProviderKind::Mail163);
    }

    None
}

/// Return the static configuration for a given provider kind.
pub fn config(kind: ProviderKind) -> ProviderConfig {
    match kind {
        ProviderKind::Gmail => ProviderConfig {
            kind,
            display_name: "Gmail",
            auth_method: AuthMethod::Oauth2 {
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                scopes: vec![
                    "https://mail.google.com/".to_string(),
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            },
            imap: Some(ImapConfig {
                host: "imap.gmail.com".to_string(),
                port: 993,
                encryption: Encryption::Tls,
            }),
            smtp: Some(SmtpConfig {
                host: "smtp.gmail.com".to_string(),
                port: 465,
                encryption: Encryption::Tls,
            }),
            setup_url: None,
            setup_guide_key: None,
        },

        ProviderKind::Outlook => ProviderConfig {
            kind,
            display_name: "Outlook",
            auth_method: AuthMethod::Oauth2 {
                auth_url:
                    "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
                token_url:
                    "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                scopes: vec![
                    "offline_access".to_string(),
                    "https://outlook.office.com/IMAP.AccessAsUser.All".to_string(),
                    "https://outlook.office.com/SMTP.Send".to_string(),
                ],
            },
            imap: Some(ImapConfig {
                host: "outlook.office365.com".to_string(),
                port: 993,
                encryption: Encryption::Tls,
            }),
            smtp: Some(SmtpConfig {
                host: "smtp.office365.com".to_string(),
                port: 587,
                encryption: Encryption::StartTls,
            }),
            setup_url: None,
            setup_guide_key: None,
        },

        ProviderKind::ICloud => ProviderConfig {
            kind,
            display_name: "iCloud Mail",
            auth_method: AuthMethod::AppPassword,
            imap: Some(ImapConfig {
                host: "imap.mail.me.com".to_string(),
                port: 993,
                encryption: Encryption::Tls,
            }),
            smtp: Some(SmtpConfig {
                host: "smtp.mail.me.com".to_string(),
                port: 587,
                encryption: Encryption::StartTls,
            }),
            setup_url: Some("https://account.apple.com/account/manage"),
            setup_guide_key: Some("onboarding.guide.icloud"),
        },

        ProviderKind::Qq => ProviderConfig {
            kind,
            display_name: "QQ Mail",
            auth_method: AuthMethod::AuthCode,
            imap: Some(ImapConfig {
                host: "imap.qq.com".to_string(),
                port: 993,
                encryption: Encryption::Tls,
            }),
            smtp: Some(SmtpConfig {
                host: "smtp.qq.com".to_string(),
                port: 465,
                encryption: Encryption::Tls,
            }),
            setup_url: Some("https://wx.mail.qq.com/account"),
            setup_guide_key: Some("onboarding.guide.qq"),
        },

        ProviderKind::Mail163 => ProviderConfig {
            kind,
            display_name: "163 Mail",
            auth_method: AuthMethod::AuthCode,
            imap: Some(ImapConfig {
                host: "imap.163.com".to_string(),
                port: 993,
                encryption: Encryption::Tls,
            }),
            smtp: Some(SmtpConfig {
                host: "smtp.163.com".to_string(),
                port: 465,
                encryption: Encryption::Tls,
            }),
            setup_url: Some("https://mail.163.com/"),
            setup_guide_key: Some("onboarding.guide.163"),
        },

        ProviderKind::GenericImap => ProviderConfig {
            kind,
            display_name: "Generic IMAP",
            auth_method: AuthMethod::Password,
            imap: None,  // user provides all fields manually
            smtp: None,
            setup_url: None,
            setup_guide_key: None,
        },
    }
}

/// All provider kinds in the order they should appear in the account-picker UI.
pub fn all_for_picker() -> Vec<ProviderKind> {
    vec![
        ProviderKind::Gmail,
        ProviderKind::Outlook,
        ProviderKind::ICloud,
        ProviderKind::Qq,
        ProviderKind::Mail163,
        ProviderKind::GenericImap,
    ]
}
