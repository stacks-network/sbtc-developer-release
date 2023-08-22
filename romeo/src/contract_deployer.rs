use blockstack_lib::vm::types::QualifiedContractIdentifier;
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::event::Event;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ContractDeployer {
    deployed: bool,
    contract: Option<QualifiedContractIdentifier>,
}

impl Actor for ContractDeployer {
    const NAME: &'static str = "contract_deployer";

    fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>> {
        match event {
            Event::Tick => {
                if self.deployed {
                    return Ok(vec![]);
                }

                // todo!("Deploy the contract");
                self.deployed = true;

                // let contract = todo!();
                // self.contract = Some(contract);

                // Ok(vec![Event::AssetContractDeployed(ContractData(contract))])
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }
}
