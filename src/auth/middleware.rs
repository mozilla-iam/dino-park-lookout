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

        let token = req
            .headers()
            .get("AUTHORIZATION")
            .map(|value| value.to_str().ok())
            .ok_or(ServiceError::Unauthorized)?;

        match token {
            Some(t) => {
                let claim_set = self.checker.verify_and_decode(&t)?;
                check_claim_set(&claim_set, self.validation_options.clone())?;
                Ok(Started::Done)
            }
            None => Err(ServiceError::Unauthorized.into()),
        }
    }
}
