#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    Token,
    AmountPerUser,
    Beneficiary(Address), // maps to a boolean (true if eligible)
}

#[contract]
pub struct ReliefContract;

#[contractimpl]
impl ReliefContract {
    /// Initializes the relief fund. Called once by the NGO.
    pub fn init(env: Env, admin: Address, token: Address, amount_per_user: i128) {
        admin.require_auth();
        assert!(!env.storage().instance().has(&DataKey::Admin), "Already initialized");

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::AmountPerUser, &amount_per_user);
    }

    /// Whitelists a victim's address. Called by the NGO Admin.
    pub fn add_beneficiary(env: Env, user: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Not initialized");
        admin.require_auth();
        
        env.storage().persistent().set(&DataKey::Beneficiary(user), &true);
    }

    /// Victim claims their relief funds.
    pub fn claim(env: Env, user: Address) {
        user.require_auth();

        // Verify eligibility
        let is_beneficiary: bool = env.storage().persistent().get(&DataKey::Beneficiary(user.clone())).unwrap_or(false);
        assert!(is_beneficiary, "Not a registered beneficiary");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::AmountPerUser).unwrap();

        // Disburse funds
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &user, &amount);

        // Remove from persistent storage to prevent double claims
        env.storage().persistent().remove(&DataKey::Beneficiary(user));
    }
}