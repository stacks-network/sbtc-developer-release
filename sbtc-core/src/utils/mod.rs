use stacks_rs::StacksAddress;

use crate::utils::contract_name::ContractName;

pub mod contract_name;

#[derive(Debug, Clone)]
pub struct StandardPrincipalData(u8, StacksAddress);

impl StandardPrincipalData {
    pub fn new(version: u8, address: StacksAddress) -> Self {
        Self(version, address)
    }
}

pub enum PrincipalData {
    Standard(StandardPrincipalData),
    Contract(StandardPrincipalData, ContractName),
}
