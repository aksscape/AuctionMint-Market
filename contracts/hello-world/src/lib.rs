#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short, Address};

// Structure to store traveler identity information
#[contracttype]
#[derive(Clone)]
pub struct TravelerIdentity {
    pub traveler_id: u64,
    pub name: String,
    pub passport_hash: String,  // Hash of passport for privacy
    pub nationality: String,
    pub wallet_address: Address,
    pub verification_level: u32,  // 1-Basic, 2-Standard, 3-Enhanced, 4-Biometric
    pub is_verified: bool,
    pub registration_time: u64,
    pub last_updated: u64,
}

// Structure to store verification records
#[contracttype]
#[derive(Clone)]
pub struct VerificationRecord {
    pub verification_id: u64,
    pub traveler_id: u64,
    pub verifier_address: Address,  // Hotel/Airport wallet address
    pub location: String,
    pub verification_type: String,  // "check-in", "boarding", "security"
    pub timestamp: u64,
    pub notes: String,
}

// Structure to track platform statistics
#[contracttype]
#[derive(Clone)]
pub struct PlatformStats {
    pub total_travelers: u64,
    pub total_verifications: u64,
    pub verified_travelers: u64,
    pub active_verifiers: u64,
}

// Enum for mapping traveler ID to TravelerIdentity struct
#[contracttype]
pub enum TravelerBook {
    Traveler(u64)
}

// Enum for mapping verification ID to VerificationRecord struct
#[contracttype]
pub enum VerificationBook {
    Verification(u64)
}

// Symbol for tracking total traveler count
const TRAVELER_COUNT: Symbol = symbol_short!("T_COUNT");
// Symbol for tracking total verification count
const VERIFY_COUNT: Symbol = symbol_short!("V_COUNT");
// Symbol for tracking platform statistics
const PLATFORM_STATS: Symbol = symbol_short!("P_STATS");

#[contract]
pub struct BlockTravelContract;

#[contractimpl]
impl BlockTravelContract {
    
    /// Register a new traveler identity on the blockchain
    /// Returns the unique traveler ID
    pub fn register_traveler(
        env: Env,
        name: String,
        passport_hash: String,
        nationality: String,
        wallet_address: Address,
        verification_level: u32
    ) -> u64 {
        // Validate verification level
        if verification_level < 1 || verification_level > 4 {
            log!(&env, "Invalid verification level! Must be 1-4");
            panic!("Invalid verification level!");
        }
        
        // Get current traveler count
        let mut traveler_count: u64 = env.storage().instance().get(&TRAVELER_COUNT).unwrap_or(0);
        traveler_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new traveler identity
        let new_traveler = TravelerIdentity {
            traveler_id: traveler_count,
            name: name,
            passport_hash: passport_hash,
            nationality: nationality,
            wallet_address: wallet_address,
            verification_level: verification_level,
            is_verified: false,  // Requires verification by authorized entity
            registration_time: timestamp,
            last_updated: timestamp,
        };
        
        // Update platform statistics
        let mut stats = Self::get_platform_stats(env.clone());
        stats.total_travelers += 1;
        
        // Store the traveler identity and updated statistics
        env.storage().instance().set(&TravelerBook::Traveler(traveler_count), &new_traveler);
        env.storage().instance().set(&TRAVELER_COUNT, &traveler_count);
        env.storage().instance().set(&PLATFORM_STATS, &stats);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(100000, 100000);
        
        log!(&env, "Traveler registered with ID: {}", traveler_count);
        traveler_count
    }
    
    /// Verify a traveler's identity (called by hotels/airports)
    /// Returns verification record ID
    pub fn verify_identity(
        env: Env,
        traveler_id: u64,
        verifier_address: Address,
        location: String,
        verification_type: String,
        notes: String
    ) -> u64 {
        // Retrieve the traveler identity
        let mut traveler = Self::get_traveler_identity(env.clone(), traveler_id);
        
        // Check if traveler exists
        if traveler.traveler_id == 0 {
            log!(&env, "Traveler does not exist!");
            panic!("Traveler does not exist!");
        }
        
        // Get current verification count
        let mut verify_count: u64 = env.storage().instance().get(&VERIFY_COUNT).unwrap_or(0);
        verify_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create verification record
        let verification = VerificationRecord {
            verification_id: verify_count,
            traveler_id: traveler_id,
            verifier_address: verifier_address,
            location: location,
            verification_type: verification_type,
            timestamp: timestamp,
            notes: notes,
        };
        
        // Update traveler's verification status if first verification
        if !traveler.is_verified {
            traveler.is_verified = true;
            traveler.last_updated = timestamp;
            
            // Update statistics
            let mut stats = Self::get_platform_stats(env.clone());
            stats.verified_travelers += 1;
            env.storage().instance().set(&PLATFORM_STATS, &stats);
        } else {
            traveler.last_updated = timestamp;
        }
        
        // Update verification count in statistics
        let mut stats = Self::get_platform_stats(env.clone());
        stats.total_verifications += 1;
        
        // Store updated data
        env.storage().instance().set(&VerificationBook::Verification(verify_count), &verification);
        env.storage().instance().set(&VERIFY_COUNT, &verify_count);
        env.storage().instance().set(&TravelerBook::Traveler(traveler_id), &traveler);
        env.storage().instance().set(&PLATFORM_STATS, &stats);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(100000, 100000);
        
        log!(&env, "Verification recorded with ID: {}", verify_count);
        verify_count
    }
    
    /// Get traveler identity details by ID
    pub fn get_traveler_identity(env: Env, traveler_id: u64) -> TravelerIdentity {
        let key = TravelerBook::Traveler(traveler_id);
        
        env.storage().instance().get(&key).unwrap_or(TravelerIdentity {
            traveler_id: 0,
            name: String::from_str(&env, "Not_Found"),
            passport_hash: String::from_str(&env, "Not_Found"),
            nationality: String::from_str(&env, "Not_Found"),
            wallet_address: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            verification_level: 0,
            is_verified: false,
            registration_time: 0,
            last_updated: 0,
        })
    }
    
    /// Get verification record details by ID
    pub fn get_verification_record(env: Env, verification_id: u64) -> VerificationRecord {
        let key = VerificationBook::Verification(verification_id);
        
        env.storage().instance().get(&key).unwrap_or(VerificationRecord {
            verification_id: 0,
            traveler_id: 0,
            verifier_address: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            location: String::from_str(&env, "Not_Found"),
            verification_type: String::from_str(&env, "Not_Found"),
            timestamp: 0,
            notes: String::from_str(&env, "Not_Found"),
        })
    }
    
    /// Get overall platform statistics
    pub fn get_platform_stats(env: Env) -> PlatformStats {
        env.storage().instance().get(&PLATFORM_STATS).unwrap_or(PlatformStats {
            total_travelers: 0,
            total_verifications: 0,
            verified_travelers: 0,
            active_verifiers: 0,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, Address};

    #[test]
    fn test_register_and_verify_traveler() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BlockTravelContract);
        let client = BlockTravelContractClient::new(&env, &contract_id);
        
        let traveler_wallet = Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));
        let hotel_wallet = Address::from_string(&String::from_str(&env, "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBWHF"));
        
        // Register a traveler
        let traveler_id = client.register_traveler(
            &String::from_str(&env, "John Doe"),
            &String::from_str(&env, "hash_of_passport_data"),
            &String::from_str(&env, "USA"),
            &traveler_wallet,
            &3
        );
        
        assert_eq!(traveler_id, 1);
        
        // Verify the traveler at hotel
        let verification_id = client.verify_identity(
            &traveler_id,
            &hotel_wallet,
            &String::from_str(&env, "Marriott Hotel NYC"),
            &String::from_str(&env, "check-in"),
            &String::from_str(&env, "Standard check-in procedure")
        );
        
        assert_eq!(verification_id, 1);
        
        // Check statistics
        let stats = client.get_platform_stats();
        assert_eq!(stats.total_travelers, 1);
        assert_eq!(stats.verified_travelers, 1);
        assert_eq!(stats.total_verifications, 1);
    }
}