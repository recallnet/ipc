use anyhow::Error;
use fvm_shared::address::Address;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_sol_facade::gas as sol_gas;
use recall_sol_facade::types::H160;

pub(crate) struct GasSponsorSet {
    sponsor: Address,
}
impl GasSponsorSet {
    pub fn mew(sponsor: Address) -> Self {
        Self { sponsor }
    }
}
impl TryIntoEVMEvent for GasSponsorSet {
    type Target = sol_gas::Event;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let sponsor: H160 = self.sponsor.try_into()?;
        Ok(sol_gas::Event::GasSponsorSet(sol_gas::GasSponsorSet {
            sponsor: sponsor.into(),
        }))
    }
}

pub(crate) struct GasSponsorUnset {}
impl GasSponsorUnset {
    pub fn new() -> Self {
        Self {}
    }
}
impl TryIntoEVMEvent for GasSponsorUnset {
    type Target = sol_gas::Event;
    fn try_into_evm_event(self) -> Result<sol_gas::Event, Error> {
        Ok(sol_gas::Event::GasSponsorUnset(sol_gas::GasSponsorUnset {}))
    }
}