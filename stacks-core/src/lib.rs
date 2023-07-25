use crate::contract_name::ContractName;

pub mod contract_name;

pub type Hash160 = [u8; 20];

#[derive(Debug, Clone)]
pub struct StacksAddress {
    version: u8,
    hash: Hash160,
}

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
