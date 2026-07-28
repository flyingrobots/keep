//! This module owns the admitted fuzz campaign profiles.

use super::FuzzCampaignError;

#[derive(Clone, Copy)]
pub(super) enum CampaignProfile {
    Scheduled,
    Smoke,
}

impl CampaignProfile {
    pub(super) fn parse(value: String) -> Result<Self, FuzzCampaignError> {
        match value.as_str() {
            "scheduled" => Ok(Self::Scheduled),
            "smoke" => Ok(Self::Smoke),
            _ => Err(FuzzCampaignError::UnexpectedArgument(value)),
        }
    }
}
