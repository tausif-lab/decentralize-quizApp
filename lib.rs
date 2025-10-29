#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to store quiz participant data
#[contracttype]
#[derive(Clone)]
pub struct Participant {
    pub address: Address,
    pub score: u64,
    pub quiz_id: u64,
    pub timestamp: u64,
    pub rewarded: bool,
}

// Structure to store overall quiz statistics
#[contracttype]
#[derive(Clone)]
pub struct QuizStats {
    pub total_participants: u64,
    pub total_rewards_distributed: u64,
    pub highest_score: u64,
}

// Mapping for participant records
#[contracttype]
pub enum ParticipantBook {
    Record(Address, u64) // (participant_address, quiz_id)
}

// Symbol for quiz statistics
const QUIZ_STATS: Symbol = symbol_short!("STATS");

#[contract]
pub struct QuizContract;

#[contractimpl]
impl QuizContract {
    
    // Function to submit quiz score
    pub fn submit_score(env: Env, participant: Address, quiz_id: u64, score: u64) {
        // Verify the participant is authorized
        participant.require_auth();
        
        let timestamp = env.ledger().timestamp();
        
        // Create participant record
        let record = Participant {
            address: participant.clone(),
            score: score.clone(),
            quiz_id: quiz_id.clone(),
            timestamp,
            rewarded: false,
        };
        
        // Store participant record
        env.storage().instance().set(
            &ParticipantBook::Record(participant.clone(), quiz_id.clone()),
            &record
        );
        
        // Update quiz statistics
        let mut stats = Self::view_quiz_stats(env.clone());
        stats.total_participants += 1;
        
        if score > stats.highest_score {
            stats.highest_score = score;
        }
        
        env.storage().instance().set(&QUIZ_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Score submitted - Participant: {:?}, Quiz ID: {}, Score: {}", participant, quiz_id, score);
    }
    
    // Function to distribute rewards (admin function)
    pub fn distribute_reward(env: Env, participant: Address, quiz_id: u64, reward_amount: u64) {
        // Get participant record
        let mut record = Self::view_participant_score(env.clone(), participant.clone(), quiz_id.clone());
        
        // Check if participant exists and hasn't been rewarded yet
        if record.score > 0 && record.rewarded == false {
            record.rewarded = true;
            
            // Update participant record
            env.storage().instance().set(
                &ParticipantBook::Record(participant.clone(), quiz_id.clone()),
                &record
            );
            
            // Update statistics
            let mut stats = Self::view_quiz_stats(env.clone());
            stats.total_rewards_distributed += reward_amount;
            
            env.storage().instance().set(&QUIZ_STATS, &stats);
            env.storage().instance().extend_ttl(5000, 5000);
            
            log!(&env, "Reward distributed - Participant: {:?}, Amount: {}", participant, reward_amount);
        } else {
            log!(&env, "Cannot distribute reward - Invalid participant or already rewarded");
            panic!("Cannot distribute reward!");
        }
    }
    
    // Function to view participant score
    pub fn view_participant_score(env: Env, participant: Address, quiz_id: u64) -> Participant {
        let key = ParticipantBook::Record(participant.clone(), quiz_id.clone());
        
        env.storage().instance().get(&key).unwrap_or(Participant {
            address: participant,
            score: 0,
            quiz_id,
            timestamp: 0,
            rewarded: false,
        })
    }
    
    // Function to view overall quiz statistics
    pub fn view_quiz_stats(env: Env) -> QuizStats {
        env.storage().instance().get(&QUIZ_STATS).unwrap_or(QuizStats {
            total_participants: 0,
            total_rewards_distributed: 0,
            highest_score: 0,
        })
    }
}