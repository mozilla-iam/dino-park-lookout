use crate::auth::check::check_claim_set;
use crate::auth::error::ServiceError;
use crate::auth::provider::TokenChecker;
use actix_web::middleware::Middleware;
use actix_web::middleware::Started;
use actix_web::HttpRequest;
use actix_web::Result;
use biscuit::ValidationOptions;

#[derive(Clone)]
pub struct AuthMiddleware<T: TokenChecker + 'static> {
    pub checker: T,
    pub validation_options: ValidationOptions,
}

impl<T: TokenChecker, S> Middleware<S> for AuthMiddleware<T> {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        if req.method() == "OPTIONS" {
            return Ok(Started::Done);
        }

        let auth_header = req
            .headers()
            .get("AUTHORIZATION")
            .map(|value| value.to_str().ok())
            .ok_or(ServiceError::Unauthorized)?;

        if let Some(auth_header) = auth_header {
            if let Some(token) = get_token(auth_header) {
                let claim_set = self.checker.verify_and_decode(&token)?;
                check_claim_set(&claim_set, self.validation_options.clone())?;
                return Ok(Started::Done);
            }
        }
        Err(ServiceError::Unauthorized.into())
    }
}

fn get_token(auth_header: &str) -> Option<&str> {
    match auth_header.get(0..7) {
        Some("Bearer ") => auth_header.get(7..),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_token() {
        let token = "Bearer FOOBAR…";
        assert_eq!(get_token(token), Some("FOOBAR…"));
    }
}
