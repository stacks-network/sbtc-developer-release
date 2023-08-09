use crate::{
    address::{AddressVersion, StacksAddress},
    contract_name::ContractName,
};

#[derive(Debug, Clone)]
/// Standard principal data type
pub struct StandardPrincipalData(pub AddressVersion, pub StacksAddress);

impl StandardPrincipalData {
    /// Create a new standard principal data type from the provided address version and stacks address
    pub fn new(version: AddressVersion, address: StacksAddress) -> Self {
        Self(version, address)
    }
}

#[derive(Debug, Clone)]
/// Principal Data type
pub enum PrincipalData {
    /// Standard principal data type
    Standard(StandardPrincipalData),
    /// Contract principal data type
    Contract(StandardPrincipalData, ContractName),
}
