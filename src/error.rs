use reqwest::Error;

#[derive(Debug, Fail)]
pub enum UpdateError {
    #[fail(display = "error updating orgchart: {}", _0)]
    OrgchartUpdate(Error),
    #[fail(display = "error updating search: {}", _0)]
    SearchUpdate(Error),
    #[fail(display = "error updating groups: {}", _0)]
    GroupsUpdate(Error),
    #[fail(display = "error updating")]
    Other,
}
