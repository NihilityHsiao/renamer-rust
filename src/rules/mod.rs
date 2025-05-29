mod remove;

use serde::{Deserialize, Serialize};



#[derive(Debug,Serialize,Deserialize)]
pub enum RuleType{
    Remove
}

