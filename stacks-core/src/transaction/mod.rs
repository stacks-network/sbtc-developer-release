/*!
Utilities and types for working with Stacks transactions.
*/

use secp256k1::PublicKey;

use crate::{address::StacksAddress, contract_name::ContractName, crypto::hash160::Hash160Hash};

/// Stacks transaction version
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionVersion {
    Mainnet = 0,
    Testnet = 128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinglesigHashMode {
    P2PKH,
    P2WPKH,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultisigHashMode {
    P2SH,
    P2WSH,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionPublicKeyEncoding {
    Compressed,
    Uncompressed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature([u8; 65]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleSignatureSpendingCondition {
    pub hash_mode: SinglesigHashMode,
    pub signer: Hash160Hash,
    pub nonce: u64,
    pub tx_fee: u64,
    pub key_encoding: TransactionPublicKeyEncoding,
    pub signature: MessageSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionAuthField {
    PublicKey(PublicKey),
    Signature(TransactionPublicKeyEncoding, MessageSignature),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultisigSpendingCondition {
    pub hash_mode: MultisigHashMode,
    pub signer: Hash160Hash,
    pub nonce: u64,
    pub tx_fee: u64,
    pub fields: Vec<TransactionAuthField>,
    pub signatures_required: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionSpendingCondition {
    Singlesig(SingleSignatureSpendingCondition),
    Multisig(MultisigSpendingCondition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionAuth {
    Standard(TransactionSpendingCondition),
    Sponsored(TransactionSpendingCondition, TransactionSpendingCondition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionAnchorMode {
    OnChainOnly,
    OffChainOnly,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionPostConditionMode {
    Allow,
    Deny,
}

pub enum PostConditionPrincipal {
    Origin,
    Standard(StacksAddress),
    Contract(StacksAddress, ContractName),
}

pub enum FungibleConditionCode {
    SentEq,
    SentGt,
    SentGe,
    SentLt,
    SentLe,
}

pub struct AssetInfo {
    pub contract_address: StacksAddress,
    pub contract_name: ContractName,
    pub asset_name: ClarityName,
}

pub enum NonfungibleConditionCode {
    Sent,
    NotSent,
}

pub enum TransactionPostCondition {
    STX(PostConditionPrincipal, FungibleConditionCode, u64),
    Fungible(
        PostConditionPrincipal,
        AssetInfo,
        FungibleConditionCode,
        u64,
    ),
    Nonfungible(
        PostConditionPrincipal,
        AssetInfo,
        Value,
        NonfungibleConditionCode,
    ),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub version: TransactionVersion,
    pub chain_id: u32,
    pub auth: TransactionAuth,
    pub anchor_mode: TransactionAnchorMode,
    pub post_condition_mode: TransactionPostConditionMode,
    pub post_conditions: Vec<TransactionPostCondition>,
    pub payload: TransactionPayload,
}
