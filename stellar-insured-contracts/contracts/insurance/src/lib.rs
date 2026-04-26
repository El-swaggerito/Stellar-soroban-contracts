#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_borrows_for_generic_args
)]

//! Property insurance contract module wiring, types, and delegated implementations.

use ink::storage::Mapping;

mod rbac;
pub use rbac::{Role, RoleManager};

/// Decentralized Property Insurance Platform
#[ink::contract]
mod propchain_insurance {
    use super::*;
    use ink::prelude::{string::String, vec::Vec};
    use crate::{Role, RoleManager};

    pub use crate::types::{
        ActuarialModel, BatchClaimResult, BatchClaimSummary, ClaimStatus, CoverageType,
        EvidenceItem, EvidenceMetadata, EvidenceVerification, InsuranceClaim, InsuranceError,
        InsurancePolicy, InsuranceToken, PolicyStatus, PolicyType, PoolLiquidityProvider,
        PremiumCalculation, ReinsuranceAgreement, RiskAssessment, RiskLevel, RiskPool,
        UnderwritingCriteria, REWARD_PRECISION,
    };

    // =========================================================================
    // EVENTS  (extracted to events.rs, included here so ink! macros see them)
    // =========================================================================
    include!("events.rs");

    // =========================================================================
    // STORAGE
    // =========================================================================

    #[ink(storage)]
    pub struct PropertyInsurance {
        admin: AccountId,

        // Role-based access control
        role_manager: RoleManager,

        // Policies
        policies: Mapping<u64, InsurancePolicy>,
        policy_count: u64,
        policyholder_policies: Mapping<AccountId, Vec<u64>>,
        property_policies: Mapping<u64, Vec<u64>>,

        // Claims
        claims: Mapping<u64, InsuranceClaim>,
        claim_count: u64,
        policy_claims: Mapping<u64, Vec<u64>>,

        // Risk Pools
        pools: Mapping<u64, RiskPool>,
        pool_count: u64,

        // Risk Assessments
        risk_assessments: Mapping<u64, RiskAssessment>,

        // Reinsurance
        reinsurance_agreements: Mapping<u64, ReinsuranceAgreement>,
        reinsurance_count: u64,

        // Insurance Tokens (secondary market)
        insurance_tokens: Mapping<u64, InsuranceToken>,
        token_count: u64,
        token_listings: Vec<u64>,

        // Actuarial Models
        actuarial_models: Mapping<u64, ActuarialModel>,
        model_count: u64,

        // Underwriting
        underwriting_criteria: Mapping<u64, UnderwritingCriteria>,

        // Liquidity providers
        liquidity_providers: Mapping<(u64, AccountId), PoolLiquidityProvider>,
        pool_providers: Mapping<u64, Vec<AccountId>>,

        // Oracle addresses
        authorized_oracles: Mapping<AccountId, bool>,

        // Assessors
        authorized_assessors: Mapping<AccountId, bool>,

        // Claim cooldown: property_id -> last_claim_timestamp
        claim_cooldowns: Mapping<u64, u64>,
        // Rate limiting: caller -> last_submit_claim_timestamp
        caller_last_claim: Mapping<AccountId, u64>,

        // Evidence tracking
        evidence_count: u64,
        evidence_items: Mapping<u64, EvidenceItem>,
        claim_evidence: Mapping<u64, Vec<u64>>,
        evidence_verifications: Mapping<u64, Vec<EvidenceVerification>>,

        // Oracle contract for parametric claims
        oracle_contract: Option<AccountId>,

        // Platform settings
        platform_fee_rate: u32,
        claim_cooldown_period: u64,
        min_pool_capital: u128,
        dispute_window_seconds: u64,
        arbiter: Option<AccountId>,

        // Security: track used evidence nonces to prevent replay attacks
        used_evidence_nonces: Mapping<(u64, String), bool>, // (property_id, nonce) -> bool
        
        // Per-caller monotonic nonce counter for replay protection (#349)
        // Callers must supply their current nonce; it is incremented on each accepted submit_claim.
        caller_nonces: Mapping<AccountId, u64>,
        
        // Emergency pause mechanism
        is_paused: bool,
        // Time-lock for admin operations
        pending_pause_after: Option<u64>,
        pending_admin: Option<AccountId>,
        pending_admin_after: Option<u64>,
        admin_timelock_delay: u64,

        // Fee tracking
        total_platform_fees_collected: u128,

        // Minimum premium to prevent rounding exploits
        min_premium_amount: u128,
    }

    // =========================================================================
    // IMPLEMENTATION  (extracted to insurance_impl.rs)
    // =========================================================================

    #[ink(event)]
    pub struct PolicyCreated {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        policyholder: AccountId,
        #[ink(topic)]
        property_id: u64,
        coverage_type: CoverageType,
        coverage_amount: u128,
        premium_amount: u128,
        start_time: u64,
        end_time: u64,
    }

    #[ink(event)]
    pub struct PolicyIssued {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        holder: AccountId,
        coverage_amount: u128,
        premium_amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PolicyCancelled {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        policyholder: AccountId,
        cancelled_at: u64,
        reason: Option<String>,
    }

    #[ink(event)]
    pub struct PolicyRenewed {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        holder: AccountId,
        renewal_premium: u128,
        new_end_time: u64,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PolicyExpired {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        holder: AccountId,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct ClaimSubmitted {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        claimant: AccountId,
        claim_amount: u128,
        submitted_at: u64,
    }

    #[ink(event)]
    pub struct ClaimApproved {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        policy_id: u64,
        payout_amount: u128,
        approved_by: AccountId,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct ClaimRejected {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        policy_id: u64,
        reason: String,
        rejected_by: AccountId,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PayoutExecuted {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        recipient: AccountId,
        amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PoolCapitalized {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct LiquidityDeposited {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct LiquidityWithdrawn {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        principal: u128,
        rewards_paid: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct RewardsReinvested {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        new_stake: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct RewardsVestingStarted {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        vesting_start: u64,
        vesting_cliff: u64,
        vesting_duration: u64,
    }

    #[ink(event)]
    pub struct VestedRewardsClaimed {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct ReinsuranceActivated {
        #[ink(topic)]
        claim_id: u64,
        agreement_id: u64,
        recovery_amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct InsuranceTokenMinted {
        #[ink(topic)]
        token_id: u64,
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        owner: AccountId,
        face_value: u128,
    }

    #[ink(event)]
    pub struct InsuranceTokenTransferred {
        #[ink(topic)]
        token_id: u64,
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        price: u128,
    }

    #[ink(event)]
    pub struct RiskAssessmentUpdated {
        #[ink(topic)]
        property_id: u64,
        overall_score: u32,
        risk_level: RiskLevel,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct EvidenceSubmitted {
        #[ink(topic)]
        evidence_id: u64,
        #[ink(topic)]
        raised_by: AccountId,
        dispute_deadline: u64,
        previous_status: ClaimStatus,
        timestamp: u64,
        claim_id: u64,
        evidence_type: String,
        ipfs_hash: String,
        submitter: AccountId,
        submitted_at: u64,
    }

    #[ink(event)]
    pub struct EvidenceVerified {
        #[ink(topic)]
        evidence_id: u64,
        verified_by: AccountId,
        is_valid: bool,
        verified_at: u64,
    }
    
    #[ink(event)]
    pub struct ContractPaused {
        #[ink(topic)]
        paused_by: AccountId,
        timestamp: u64,
    }
    
    #[ink(event)]
    pub struct ContractUnpaused {
        #[ink(topic)]
        unpaused_by: AccountId,
        timestamp: u64,
    }

    /// Emitted when a pause is proposed; executes after time-lock delay (#301)
    #[ink(event)]
    pub struct PauseProposed {
        #[ink(topic)]
        proposed_by: AccountId,
        earliest_execution: u64,
    }

    /// Emitted when a new admin is proposed; executes after time-lock delay (#301)
    #[ink(event)]
    pub struct AdminProposed {
        #[ink(topic)]
        proposed_by: AccountId,
        #[ink(topic)]
        new_admin: AccountId,
        earliest_execution: u64,
    }

    /// Emitted when a pending admin change is executed (#301)
    #[ink(event)]
    pub struct AdminChanged {
        #[ink(topic)]
        old_admin: AccountId,
        #[ink(topic)]
        new_admin: AccountId,
        timestamp: u64,
    }

    /// Emitted when a claim submission is accepted and the caller nonce is consumed (#349)
    #[ink(event)]
    pub struct ReplayProtected {
        #[ink(topic)]
        caller: AccountId,
        nonce: u64,
        claim_id: u64,
    }

    /// Emitted when a role is granted to an account (#346)
    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        account: AccountId,
        role: Role,
        granted_by: AccountId,
    }

    /// Emitted when a role is revoked from an account (#346)
    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        account: AccountId,
        role: Role,
        revoked_by: AccountId,
    }

    // =========================================================================
    // IMPLEMENTATION
    // =========================================================================

    // Core contract behavior is extracted to keep the root module focused on types and wiring.
    include!("insurance_impl.rs");

    impl Default for PropertyInsurance {
        fn default() -> Self {
            Self::new(AccountId::from([0x0; 32]))
        }
    }
}

pub use crate::propchain_insurance::{InsuranceError, PropertyInsurance};

#[cfg(test)]
mod insurance_tests {
    include!("insurance_tests.rs");
}
