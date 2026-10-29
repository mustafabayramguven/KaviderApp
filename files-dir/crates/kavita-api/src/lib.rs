//! `kavita-api` — typed async client for the Kavita HTTP API.
//!
//! Source of truth for endpoints, field names and the auth flow is the server's own
//! OpenAPI document (`docs/kavita-openapi.json`), not this file or anyone's memory.
//! Nothing is implemented yet: M1 fills this crate (auth, token refresh de-duplication,
//! tolerant DTOs, mock-server fixtures and an `examples/cli.rs`).

/// Base URL of a Kavita server, normalised from whatever the user typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseUrl(String);

impl BaseUrl {
    /// Wraps an already-normalised base URL. Normalisation itself (M1) is what turns
    /// `kavita.example.com`, `192.168.1.20:5000` or `https://example.com/kavita/` into
    /// this value; it is deliberately not guessed here.
    pub fn from_normalised(url: impl Into<String>) -> Self {
        Self(url.into())
    }

    /// The canonical base URL, without a trailing slash.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::BaseUrl;

    #[test]
    fn a_normalised_base_url_round_trips() {
        let base = BaseUrl::from_normalised("https://kavita.example.com");
        assert_eq!(base.as_str(), "https://kavita.example.com");
        assert_eq!(base, BaseUrl::from_normalised("https://kavita.example.com"));
    }
}
