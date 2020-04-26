use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Respone for client `super::request::Request::Greeting`
    Wellcome,
}
