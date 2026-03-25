// DEBT: Quoting the lint:
//     non-local `impl` definition, `impl` blocks should be written at the same
//     level as their item
#![allow(non_local_definitions)]

use reqwest::Error;

#[derive(Debug, Fail)]
pub enum UpdateError {
    #[fail(display = "error updating orgchart: {}", _0)]
    OrgchartUpdate(Error),
    #[fail(display = "error updating search: {}", _0)]
    SearchUpdate(Error),
    #[fail(display = "error updating groups: {}", _0)]
    GroupsUpdate(Error),
    #[fail(display = "error deleting from orgchart: {}", _0)]
    OrgchartDelete(Error),
    #[fail(display = "error deleting from search: {}", _0)]
    SearchDelete(Error),
    #[fail(display = "error deleting from pictures: {}", _0)]
    PicturesDelete(Error),
    #[fail(display = "error deleting from groups: {}", _0)]
    GroupsDelete(Error),
    #[fail(display = "error updating")]
    Other,
}
