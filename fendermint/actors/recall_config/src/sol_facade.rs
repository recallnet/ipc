use fvm_shared::address::Address;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_sol_facade::config as sol;
use recall_sol_facade::types::H160;

pub struct ConfigAdminSet {
    admin: Address,
}
impl ConfigAdminSet {
    pub fn new(admin: Address) -> Self {
        Self { admin }
    }
}
impl TryIntoEVMEvent for ConfigAdminSet {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        let admin: H160 = self.admin.try_into()?;
        Ok(sol::Event::ConfigAdminSet(sol::ConfigAdminSet {
            admin: admin.into(),
        }))
    }
}