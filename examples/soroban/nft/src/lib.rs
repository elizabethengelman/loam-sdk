#![no_std]
use loam_sdk::soroban_contract;
use loam_subcontract_core::{admin::Admin, Core};
use loam_soroban_sdk::{Address, Bytes};

pub mod nft;
pub mod subcontract;

use nft::MyNonFungibleToken;
use subcontract::{Initable, NonFungible};

pub struct Contract;

impl Core for Contract {
    type Impl = Admin;
}

impl NonFungible for Contract {
    type Impl = MyNonFungibleToken;
}

impl Initable for Contract {
    type Impl = MyNonFungibleToken;
}

soroban_contract!();