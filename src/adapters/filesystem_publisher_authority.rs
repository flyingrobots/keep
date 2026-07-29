//! This module owns one publisher's ephemeral stage authority.
//!
//! One allocation supplies stable, non-forgeable process-local identity across
//! publisher moves. The token never enters a durable format or content identity.

use std::sync::Arc;

#[derive(Clone)]
pub(super) struct FilesystemPublisherAuthority {
    token: Arc<AuthorityToken>,
}

struct AuthorityToken;

impl FilesystemPublisherAuthority {
    pub(super) fn new() -> Self {
        Self {
            token: Arc::new(AuthorityToken),
        }
    }

    pub(super) fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.token, &other.token)
    }
}
