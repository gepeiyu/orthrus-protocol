use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near, require, AccountId, Gas, NearToken, PanicOnDefault, Promise,
};
use std::collections::{HashMap, HashSet};

const ONE_HOUR_NS: u64 = 60 * 60 * 1_000_000_000;
const ONE_DAY_NS: u64 = 24 * 60 * 60 * 1_000_000_000;
const NO_DEPOSIT: NearToken = NearToken::from_yoctonear(0);

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AgentPolicy {
    pub max_single_spend: NearToken,
    pub hourly_spend_limit: NearToken,
    pub daily_spend_limit: NearToken,
    pub allowed_targets: HashSet<AccountId>,
    pub allowed_methods: HashMap<AccountId, HashSet<String>>,
    pub is_paused: bool,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AgentPolicyInput {
    pub max_single_spend: NearToken,
    pub hourly_spend_limit: NearToken,
    pub daily_spend_limit: NearToken,
    pub allowed_targets: Vec<AccountId>,
    pub allowed_methods: HashMap<AccountId, Vec<String>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AgentSpendingStats {
    pub total_spent: NearToken,
    pub current_hour_window_start: u64,
    pub spent_current_hour: NearToken,
    pub current_day_window_start: u64,
    pub spent_current_day: NearToken,
    pub total_transactions_executed: u64,
    pub total_violations_prevented: u64,
    pub last_active_timestamp: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SecurityAlert {
    pub id: u64,
    pub agent_id: AccountId,
    pub timestamp: u64,
    pub alert_type: String,
    pub attempted_target: Option<AccountId>,
    pub attempted_method: Option<String>,
    pub attempted_amount: NearToken,
    pub details: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct PolicyCheckResult {
    pub allowed: bool,
    pub reason: String,
    pub remaining_hourly_quota: NearToken,
    pub remaining_daily_quota: NearToken,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct OrthrusContract {
    pub owner_id: AccountId,
    pub guardian_id: Option<AccountId>,
    pub global_circuit_breaker: bool,
    pub agents: LookupMap<AccountId, AgentPolicy>,
    pub agent_stats: LookupMap<AccountId, AgentSpendingStats>,
    pub security_alerts: Vector<SecurityAlert>,
    pub total_alerts_count: u64,
}

#[near]
impl OrthrusContract {
    #[init]
    pub fn new(owner_id: Option<AccountId>, guardian_id: Option<AccountId>) -> Self {
        require!(!env::state_exists(), "Contract is already initialized");
        let owner = owner_id.unwrap_or_else(env::signer_account_id);
        Self {
            owner_id: owner,
            guardian_id,
            global_circuit_breaker: false,
            agents: LookupMap::new(b"a"),
            agent_stats: LookupMap::new(b"s"),
            security_alerts: Vector::new(b"l"),
            total_alerts_count: 0,
        }
    }

    // --- OWNER / GUARDIAN MANAGEMENT ---

    pub fn set_guardian(&mut self, guardian_id: Option<AccountId>) {
        self.assert_owner();
        self.guardian_id = guardian_id;
    }

    pub fn register_agent(&mut self, agent_id: AccountId, policy: AgentPolicyInput) {
        self.assert_owner();
        let now = env::block_timestamp();

        let mut allowed_methods = HashMap::new();
        for (target, methods) in policy.allowed_methods {
            allowed_methods.insert(target, methods.into_iter().collect::<HashSet<_>>());
        }

        let new_policy = AgentPolicy {
            max_single_spend: policy.max_single_spend,
            hourly_spend_limit: policy.hourly_spend_limit,
            daily_spend_limit: policy.daily_spend_limit,
            allowed_targets: policy.allowed_targets.into_iter().collect(),
            allowed_methods,
            is_paused: false,
            created_at: now,
        };

        let initial_stats = AgentSpendingStats {
            total_spent: NO_DEPOSIT,
            current_hour_window_start: now,
            spent_current_hour: NO_DEPOSIT,
            current_day_window_start: now,
            spent_current_day: NO_DEPOSIT,
            total_transactions_executed: 0,
            total_violations_prevented: 0,
            last_active_timestamp: now,
        };

        self.agents.insert(&agent_id, &new_policy);
        self.agent_stats.insert(&agent_id, &initial_stats);

        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"orthrus\",\"version\":\"1.0.0\",\"event\":\"agent_registered\",\"data\":{{\"agent_id\":\"{}\"}}}}",
            agent_id
        ));
    }

    pub fn update_agent_policy(&mut self, agent_id: AccountId, policy: AgentPolicyInput) {
        self.assert_owner();
        let existing = self
            .agents
            .get(&agent_id)
            .expect("Agent is not registered");

        let mut allowed_methods = HashMap::new();
        for (target, methods) in policy.allowed_methods {
            allowed_methods.insert(target, methods.into_iter().collect::<HashSet<_>>());
        }

        let updated_policy = AgentPolicy {
            max_single_spend: policy.max_single_spend,
            hourly_spend_limit: policy.hourly_spend_limit,
            daily_spend_limit: policy.daily_spend_limit,
            allowed_targets: policy.allowed_targets.into_iter().collect(),
            allowed_methods,
            is_paused: existing.is_paused,
            created_at: existing.created_at,
        };

        self.agents.insert(&agent_id, &updated_policy);
    }

    // --- EMERGENCY CONTROLS (KILL-SWITCH) ---

    pub fn trigger_agent_kill_switch(&mut self, agent_id: AccountId, reason: String) {
        self.assert_owner_or_guardian();
        let mut policy = self
            .agents
            .get(&agent_id)
            .expect("Agent is not registered");
        policy.is_paused = true;
        self.agents.insert(&agent_id, &policy);

        self.record_security_alert(
            &agent_id,
            "AGENT_KILL_SWITCH_TRIGGERED",
            None,
            None,
            NO_DEPOSIT,
            format!("Agent frozen by supervisor. Reason: {}", reason),
        );
    }

    pub fn resume_agent(&mut self, agent_id: AccountId) {
        self.assert_owner();
        let mut policy = self
            .agents
            .get(&agent_id)
            .expect("Agent is not registered");
        policy.is_paused = false;
        self.agents.insert(&agent_id, &policy);
    }

    pub fn trigger_global_circuit_breaker(&mut self, reason: String) {
        self.assert_owner_or_guardian();
        self.global_circuit_breaker = true;

        let owner = self.owner_id.clone();
        self.record_security_alert(
            &owner,
            "GLOBAL_CIRCUIT_BREAKER_TRIGGERED",
            None,
            None,
            NO_DEPOSIT,
            format!("Global circuit breaker activated. Reason: {}", reason),
        );
    }

    pub fn resume_global(&mut self) {
        self.assert_owner();
        self.global_circuit_breaker = false;
    }

    // --- EXECUTION ENGINE (HEAD 2: ON-CHAIN POLICY ENFORCEMENT) ---

    pub fn execute_as_agent(
        &mut self,
        target_contract: AccountId,
        method_name: String,
        args: Vec<u8>,
        deposit: NearToken,
        gas: Gas,
    ) -> Promise {
        let agent_id = env::predecessor_account_id();
        let now = env::block_timestamp();

        require!(
            !self.global_circuit_breaker,
            "Orthrus: Global circuit breaker is active"
        );

        let policy = self
            .agents
            .get(&agent_id)
            .expect("Orthrus: Caller is not a registered AI agent");

        require!(!policy.is_paused, "Orthrus: Agent is currently paused");

        let mut stats = self
            .agent_stats
            .get(&agent_id)
            .expect("Orthrus: Stats missing for agent");

        // 1. Single spend limit check
        if deposit > policy.max_single_spend {
            self.fail_policy_check(
                &agent_id,
                &mut stats,
                "SINGLE_SPEND_LIMIT_EXCEEDED",
                Some(&target_contract),
                Some(&method_name),
                deposit,
                format!(
                    "Attempted deposit {} yoctoNEAR exceeds single limit {} yoctoNEAR",
                    deposit.as_yoctonear(),
                    policy.max_single_spend.as_yoctonear()
                ),
            );
        }

        // 2. Refresh rolling velocity windows
        if now >= stats.current_hour_window_start.saturating_add(ONE_HOUR_NS) {
            stats.current_hour_window_start = now;
            stats.spent_current_hour = NO_DEPOSIT;
        }

        if now >= stats.current_day_window_start.saturating_add(ONE_DAY_NS) {
            stats.current_day_window_start = now;
            stats.spent_current_day = NO_DEPOSIT;
        }

        // 3. Hourly limit check
        let new_hourly = NearToken::from_yoctonear(
            stats
                .spent_current_hour
                .as_yoctonear()
                .saturating_add(deposit.as_yoctonear()),
        );
        if new_hourly > policy.hourly_spend_limit {
            self.fail_policy_check(
                &agent_id,
                &mut stats,
                "HOURLY_VELOCITY_EXCEEDED",
                Some(&target_contract),
                Some(&method_name),
                deposit,
                format!(
                    "Hourly spend {} yoctoNEAR would exceed cap {} yoctoNEAR",
                    new_hourly.as_yoctonear(),
                    policy.hourly_spend_limit.as_yoctonear()
                ),
            );
        }

        // 4. Daily limit check
        let new_daily = NearToken::from_yoctonear(
            stats
                .spent_current_day
                .as_yoctonear()
                .saturating_add(deposit.as_yoctonear()),
        );
        if new_daily > policy.daily_spend_limit {
            self.fail_policy_check(
                &agent_id,
                &mut stats,
                "DAILY_VELOCITY_EXCEEDED",
                Some(&target_contract),
                Some(&method_name),
                deposit,
                format!(
                    "Daily spend {} yoctoNEAR would exceed cap {} yoctoNEAR",
                    new_daily.as_yoctonear(),
                    policy.daily_spend_limit.as_yoctonear()
                ),
            );
        }

        // 5. Allowed target whitelist check
        if !policy.allowed_targets.contains(&target_contract) {
            self.fail_policy_check(
                &agent_id,
                &mut stats,
                "UNAUTHORIZED_TARGET_CONTRACT",
                Some(&target_contract),
                Some(&method_name),
                deposit,
                format!(
                    "Target contract {} is not in authorized whitelist",
                    target_contract
                ),
            );
        }

        // 6. Allowed method whitelist check (if defined for this contract)
        if let Some(methods) = policy.allowed_methods.get(&target_contract) {
            if !methods.is_empty() && !methods.contains(&method_name) {
                self.fail_policy_check(
                    &agent_id,
                    &mut stats,
                    "UNAUTHORIZED_CONTRACT_METHOD",
                    Some(&target_contract),
                    Some(&method_name),
                    deposit,
                    format!(
                        "Method {} on contract {} is not authorized",
                        method_name, target_contract
                    ),
                );
            }
        }

        // All checks passed! Update stats
        stats.spent_current_hour = new_hourly;
        stats.spent_current_day = new_daily;
        stats.total_spent = NearToken::from_yoctonear(
            stats
                .total_spent
                .as_yoctonear()
                .saturating_add(deposit.as_yoctonear()),
        );
        stats.total_transactions_executed += 1;
        stats.last_active_timestamp = now;
        self.agent_stats.insert(&agent_id, &stats);

        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"orthrus\",\"version\":\"1.0.0\",\"event\":\"policy_passed\",\"data\":{{\"agent_id\":\"{}\",\"target\":\"{}\",\"method\":\"{}\",\"deposit\":\"{}\"}}}}",
            agent_id,
            target_contract,
            method_name,
            deposit.as_yoctonear()
        ));

        // Dispatch call to target contract
        Promise::new(target_contract).function_call(method_name, args, deposit, gas)
    }

    // --- VIEW / AUDIT METHODS ---

    pub fn simulate_policy_check(
        &self,
        agent_id: AccountId,
        target_contract: AccountId,
        method_name: String,
        deposit: NearToken,
    ) -> PolicyCheckResult {
        if self.global_circuit_breaker {
            return PolicyCheckResult {
                allowed: false,
                reason: "Global circuit breaker is active".to_string(),
                remaining_hourly_quota: NO_DEPOSIT,
                remaining_daily_quota: NO_DEPOSIT,
            };
        }

        let policy = match self.agents.get(&agent_id) {
            Some(p) => p,
            None => {
                return PolicyCheckResult {
                    allowed: false,
                    reason: "Agent is not registered".to_string(),
                    remaining_hourly_quota: NO_DEPOSIT,
                    remaining_daily_quota: NO_DEPOSIT,
                }
            }
        };

        if policy.is_paused {
            return PolicyCheckResult {
                allowed: false,
                reason: "Agent is paused".to_string(),
                remaining_hourly_quota: NO_DEPOSIT,
                remaining_daily_quota: NO_DEPOSIT,
            };
        }

        if deposit > policy.max_single_spend {
            return PolicyCheckResult {
                allowed: false,
                reason: format!(
                    "Deposit exceeds single spend limit: {} > {}",
                    deposit.as_yoctonear(),
                    policy.max_single_spend.as_yoctonear()
                ),
                remaining_hourly_quota: NO_DEPOSIT,
                remaining_daily_quota: NO_DEPOSIT,
            };
        }

        if !policy.allowed_targets.contains(&target_contract) {
            return PolicyCheckResult {
                allowed: false,
                reason: format!("Contract {} is not whitelisted", target_contract),
                remaining_hourly_quota: NO_DEPOSIT,
                remaining_daily_quota: NO_DEPOSIT,
            };
        }

        if let Some(methods) = policy.allowed_methods.get(&target_contract) {
            if !methods.is_empty() && !methods.contains(&method_name) {
                return PolicyCheckResult {
                    allowed: false,
                    reason: format!(
                        "Method {} is not whitelisted on {}",
                        method_name, target_contract
                    ),
                    remaining_hourly_quota: NO_DEPOSIT,
                    remaining_daily_quota: NO_DEPOSIT,
                };
            }
        }

        let stats = self.agent_stats.get(&agent_id).unwrap();
        let remaining_hourly = NearToken::from_yoctonear(
            policy
                .hourly_spend_limit
                .as_yoctonear()
                .saturating_sub(stats.spent_current_hour.as_yoctonear()),
        );
        let remaining_daily = NearToken::from_yoctonear(
            policy
                .daily_spend_limit
                .as_yoctonear()
                .saturating_sub(stats.spent_current_day.as_yoctonear()),
        );

        if deposit > remaining_hourly {
            return PolicyCheckResult {
                allowed: false,
                reason: "Deposit exceeds remaining hourly limit".to_string(),
                remaining_hourly_quota: remaining_hourly,
                remaining_daily_quota: remaining_daily,
            };
        }

        if deposit > remaining_daily {
            return PolicyCheckResult {
                allowed: false,
                reason: "Deposit exceeds remaining daily limit".to_string(),
                remaining_hourly_quota: remaining_hourly,
                remaining_daily_quota: remaining_daily,
            };
        }

        PolicyCheckResult {
            allowed: true,
            reason: "Policy check passed".to_string(),
            remaining_hourly_quota: remaining_hourly,
            remaining_daily_quota: remaining_daily,
        }
    }

    pub fn get_agent_policy(&self, agent_id: AccountId) -> Option<AgentPolicy> {
        self.agents.get(&agent_id)
    }

    pub fn get_agent_stats(&self, agent_id: AccountId) -> Option<AgentSpendingStats> {
        self.agent_stats.get(&agent_id)
    }

    pub fn get_security_alerts(&self, from_index: u64, limit: u64) -> Vec<SecurityAlert> {
        let total = self.security_alerts.len();
        let start = from_index.min(total);
        let end = (start + limit).min(total);

        (start..end)
            .filter_map(|i| self.security_alerts.get(i))
            .collect()
    }

    pub fn get_total_alerts_count(&self) -> u64 {
        self.total_alerts_count
    }

    pub fn is_globally_paused(&self) -> bool {
        self.global_circuit_breaker
    }

    // --- INTERNAL HELPERS ---

    fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Orthrus: Caller is not the contract owner"
        );
    }

    fn assert_owner_or_guardian(&self) {
        let caller = env::predecessor_account_id();
        let is_owner = caller == self.owner_id;
        let is_guardian = self.guardian_id.as_ref() == Some(&caller);
        require!(
            is_owner || is_guardian,
            "Orthrus: Caller is neither owner nor guardian"
        );
    }

    fn record_security_alert(
        &mut self,
        agent_id: &AccountId,
        alert_type: &str,
        target: Option<&AccountId>,
        method: Option<&str>,
        amount: NearToken,
        details: String,
    ) {
        let id = self.total_alerts_count;
        self.total_alerts_count += 1;

        let alert = SecurityAlert {
            id,
            agent_id: agent_id.clone(),
            timestamp: env::block_timestamp(),
            alert_type: alert_type.to_string(),
            attempted_target: target.cloned(),
            attempted_method: method.map(|s| s.to_string()),
            attempted_amount: amount,
            details: details.clone(),
        };

        self.security_alerts.push(&alert);

        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"orthrus\",\"version\":\"1.0.0\",\"event\":\"security_alert\",\"data\":{{\"id\":{},\"agent_id\":\"{}\",\"type\":\"{}\",\"details\":\"{}\"}}}}",
            id, agent_id, alert_type, details
        ));
    }

    fn fail_policy_check(
        &mut self,
        agent_id: &AccountId,
        stats: &mut AgentSpendingStats,
        alert_type: &str,
        target: Option<&AccountId>,
        method: Option<&str>,
        amount: NearToken,
        details: String,
    ) -> ! {
        stats.total_violations_prevented += 1;
        self.agent_stats.insert(agent_id, stats);

        self.record_security_alert(
            agent_id,
            alert_type,
            target,
            method,
            amount,
            details.clone(),
        );

        env::panic_str(&format!("Orthrus Policy Violation: {}", details));
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder.block_timestamp(1_000_000_000);
        builder
    }

    #[test]
    fn test_initialization() {
        let context = get_context(accounts(0));
        testing_env!(context.build());

        let contract = OrthrusContract::new(Some(accounts(0)), Some(accounts(1)));
        assert_eq!(contract.owner_id, accounts(0));
        assert_eq!(contract.guardian_id, Some(accounts(1)));
        assert!(!contract.is_globally_paused());
        assert_eq!(contract.get_total_alerts_count(), 0);
    }

    #[test]
    fn test_register_and_simulate_policy() {
        let context = get_context(accounts(0));
        testing_env!(context.build());

        let mut contract = OrthrusContract::new(Some(accounts(0)), None);

        let target_contract: AccountId = "ref-finance.near".parse().unwrap();
        let agent: AccountId = accounts(2);

        let mut allowed_methods = HashMap::new();
        allowed_methods.insert(
            target_contract.clone(),
            vec!["swap".to_string(), "deposit".to_string()],
        );

        let policy_input = AgentPolicyInput {
            max_single_spend: NearToken::from_near(2),
            hourly_spend_limit: NearToken::from_near(5),
            daily_spend_limit: NearToken::from_near(10),
            allowed_targets: vec![target_contract.clone()],
            allowed_methods,
        };

        contract.register_agent(agent.clone(), policy_input);

        // Simulation: valid call
        let result = contract.simulate_policy_check(
            agent.clone(),
            target_contract.clone(),
            "swap".to_string(),
            NearToken::from_near(1),
        );
        assert!(result.allowed);

        // Simulation: unauthorized contract
        let rogue_target: AccountId = "attacker-drainer.near".parse().unwrap();
        let result = contract.simulate_policy_check(
            agent.clone(),
            rogue_target,
            "transfer".to_string(),
            NearToken::from_near(1),
        );
        assert!(!result.allowed);
        assert!(result.reason.contains("not whitelisted"));

        // Simulation: unauthorized method
        let result = contract.simulate_policy_check(
            agent.clone(),
            target_contract.clone(),
            "drain_pool".to_string(),
            NearToken::from_near(1),
        );
        assert!(!result.allowed);
        assert!(result.reason.contains("Method drain_pool is not whitelisted"));

        // Simulation: single spend limit exceed
        let result = contract.simulate_policy_check(
            agent.clone(),
            target_contract.clone(),
            "swap".to_string(),
            NearToken::from_near(3),
        );
        assert!(!result.allowed);
        assert!(result.reason.contains("Deposit exceeds single spend limit"));
    }

    #[test]
    fn test_kill_switch() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());

        let mut contract = OrthrusContract::new(Some(accounts(0)), Some(accounts(1)));
        let agent: AccountId = accounts(2);

        contract.register_agent(
            agent.clone(),
            AgentPolicyInput {
                max_single_spend: NearToken::from_near(5),
                hourly_spend_limit: NearToken::from_near(10),
                daily_spend_limit: NearToken::from_near(20),
                allowed_targets: vec!["dex.near".parse().unwrap()],
                allowed_methods: HashMap::new(),
            },
        );

        // Guardian triggers kill switch
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.trigger_agent_kill_switch(agent.clone(), "Anomalous behavior observed".to_string());

        let policy = contract.get_agent_policy(agent.clone()).unwrap();
        assert!(policy.is_paused);
        assert_eq!(contract.get_total_alerts_count(), 1);

        // Check simulation reflects paused state
        let result = contract.simulate_policy_check(
            agent.clone(),
            "dex.near".parse().unwrap(),
            "swap".to_string(),
            NearToken::from_near(1),
        );
        assert!(!result.allowed);
        assert_eq!(result.reason, "Agent is paused");

        // Owner resumes agent
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.resume_agent(agent.clone());
        let policy_resumed = contract.get_agent_policy(agent.clone()).unwrap();
        assert!(!policy_resumed.is_paused);
    }
}
