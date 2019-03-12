use actix::prelude::*;
use actix_web::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bulk {}

impl Message for Bulk {
    type Result = Result<String, Error>;
}
