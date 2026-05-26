#![cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};
    use soroban_sdk::token::Client as TokenClient;
    use soroban_sdk::token::StellarAssetClient;

    fn setup_env<'a>() -> (Env, ReliefContractClient<'a>, TokenClient<'a>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let victim = Address::generate(&env);

        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract(token_admin.clone());
        let token = TokenClient::new(&env, &token_contract);
        let token_admin_client = StellarAssetClient::new(&env, &token_contract);
        
        let contract_id = env.register_contract(None, ReliefContract);
        let contract_client = ReliefContractClient::new(&env, &contract_id);

        // Fund the contract natively
        token_admin_client.mint(&contract_client.address, &1000);

        (env, contract_client, token, admin, victim)
    }

    #[test]
    fn test_1_happy_path() {
        let (env, contract, token, admin, victim) = setup_env();
        
        contract.init(&admin, &token.address, &50);
        contract.add_beneficiary(&victim);
        
        assert_eq!(token.balance(&victim), 0);
        contract.claim(&victim);
        assert_eq!(token.balance(&victim), 50);
    }

    #[test]
    #[should_panic(expected = "Not a registered beneficiary")]
    fn test_2_edge_case_unauthorized_claim() {
        let (env, contract, token, admin, _victim) = setup_env();
        let random_user = Address::generate(&env);

        contract.init(&admin, &token.address, &50);
        // Victim is NOT added by admin
        contract.claim(&random_user); // Should fail
    }

    #[test]
    fn test_3_state_verification() {
        let (env, contract, token, admin, victim) = setup_env();
        contract.init(&admin, &token.address, &50);
        contract.add_beneficiary(&victim);
        
        contract.claim(&victim);
        
        // Ensure contract balance decreased by exact amount
        let contract_address = contract.address.clone();
        assert_eq!(token.balance(&contract_address), 950); 
    }

    #[test]
    #[should_panic(expected = "Not a registered beneficiary")]
    fn test_4_edge_case_double_claim() {
        let (env, contract, token, admin, victim) = setup_env();
        contract.init(&admin, &token.address, &50);
        contract.add_beneficiary(&victim);
        
        contract.claim(&victim);
        contract.claim(&victim); // Second claim should fail because key was removed
    }

    #[test]
    #[should_panic(expected = "Already initialized")]
    fn test_5_edge_case_double_init() {
        let (env, contract, token, admin, _victim) = setup_env();
        contract.init(&admin, &token.address, &50);
        contract.init(&admin, &token.address, &100); // Should fail
    }
}