use oauth2::PkceCodeChallenge;

/// Holds all three PKCE/state values generated for one OAuth initiation.
pub struct Pkce {
    /// The raw verifier sent to the token endpoint.
    pub verifier: String,
    /// The SHA-256 / base64url challenge sent to the auth endpoint.
    pub challenge: String,
    /// Random, opaque CSRF token sent in both the auth URL and the callback.
    pub state: String,
}

/// Generate a fresh PKCE bundle.  Uses the `oauth2` crate for the
/// verifier + challenge pair (S256), and `uuid` for the state token.
pub fn generate() -> Pkce {
    // oauth2 v5: new_random_sha256() returns (PkceCodeChallenge, PkceCodeVerifier)
    let (challenge_obj, verifier_obj) = PkceCodeChallenge::new_random_sha256();

    Pkce {
        verifier: verifier_obj.secret().clone(),
        challenge: challenge_obj.as_str().to_owned(),
        state: uuid::Uuid::new_v4().simple().to_string(),
    }
}
