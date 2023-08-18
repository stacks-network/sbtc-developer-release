/*!
Types for the payload of a Stacks transaction
*/

use crate::{
    address::StacksAddress,
    transaction::token_transfer_memo::TokenTransferMemo,
    utils::PrincipalData,
    validated_strings::{
        clarity_name::ClarityName, contract_name::ContractName, stacks_string::StacksString,
    },
};

pub struct TransactionContractCall {
    pub address: StacksAddress,
    pub contract_name: ContractName,
    pub function_name: ClarityName,
    pub function_args: Vec<Value>,
}

pub struct TransactionSmartContract {
    pub name: ContractName,
    pub code_body: StacksString,
}

pub enum ClarityVersion {
    Clarity1,
    Clarity2,
}

pub enum TransactionPayload {
    TokenTransfer(PrincipalData, u64, TokenTransferMemo),
    ContractCall(TransactionContractCall),
    SmartContract(TransactionSmartContract, Option<ClarityVersion>),
    PoisonMicroblock(StacksMicroblockHeader, StacksMicroblockHeader),
    Coinbase(CoinbasePayload, Option<PrincipalData>),
}
