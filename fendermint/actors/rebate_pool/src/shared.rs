use fvm_ipld_encoding::tuple::*;
use num_derive::FromPrimitive;
use fvm_shared::METHOD_CONSTRUCTOR;

pub const REBATE_POOL_ACTOR_NAME: &str = "rebate_pool";

/// Alpha is a float of range [0, 1].
/// We better not use floating arithmetic though, so it gets represented as a (numerator, denominator) part.
pub type Alpha = (u32, u32);

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
}

/// Params for actor construction.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// Alpha parameter for Cobb-Douglas based share determination.
    pub alpha: Alpha,
}
