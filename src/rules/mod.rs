mod remove;

use serde::{Deserialize, Serialize};

pub use remove::*;



#[derive(Debug,Serialize,Deserialize)]
pub enum RuleType{
    Remove
}

