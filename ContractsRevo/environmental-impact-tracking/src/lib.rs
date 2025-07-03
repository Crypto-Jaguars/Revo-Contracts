#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct EnvironmentalContract;

#[contractimpl]
impl EnvironmentalContract {}

mod carbon;
mod datatypes;
mod error;
mod interfaces;
mod reporting;
mod retirement;
mod verification;

#[cfg(test)]
mod test;
