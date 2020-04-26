use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// First message send by client after connecting
    Greeting,
}
