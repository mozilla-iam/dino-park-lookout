use biscuit::ClaimPresenceOptions;
use biscuit::ClaimsSet;
use biscuit::Presence;
use biscuit::StringOrUri;
use biscuit::Validation;
use biscuit::ValidationOptions;
use chrono::Duration;
use failure::Error;
use serde_json::Value;

pub fn check_claim_set(
    claim_set: &ClaimsSet<Value>,
    validation_options: ValidationOptions,
) -> Result<(), Error> {
    claim_set
        .registered
        .validate(validation_options)
        .map_err(Into::into)
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthValidationSettings {
    pub audience: String,
    pub issuer: Option<String>,
    pub issued_at: Option<i64>,
}

impl AuthValidationSettings {
    pub fn to_validation_options(&self) -> ValidationOptions {
        let claim_presence_options = ClaimPresenceOptions {
            audience: Presence::Required,
            expiry: Presence::Required,
            issuer: self
                .issuer
                .as_ref()
                .map(|_| Presence::Required)
                .unwrap_or_default(),
            issued_at: self
                .issued_at
                .map(|_| Presence::Required)
                .unwrap_or_default(),
            ..Default::default()
        };
        ValidationOptions {
            claim_presence_options,
            audience: Validation::Validate(StringOrUri::String(self.audience.clone())),
            issuer: self
                .issuer
                .as_ref()
                .map(|s| Validation::Validate(StringOrUri::String(s.clone())))
                .unwrap_or_default(),
            issued_at: self
                .issued_at
                .map(|s| Validation::Validate(Duration::seconds(s)))
                .unwrap_or_default(),
            ..Default::default()
        }
    }
}
