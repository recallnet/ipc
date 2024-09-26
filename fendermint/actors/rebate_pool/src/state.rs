use fvm_ipld_encoding::tuple::*;
use crate::Alpha;

/// Rebate Pool actor state.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    alpha: Alpha
}

impl State {
    pub fn new(alpha: Alpha) -> Self {
        Self {
            alpha
        }
    }
}
