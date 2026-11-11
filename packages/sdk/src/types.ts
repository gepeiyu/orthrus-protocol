export interface AgentPolicyConfig {
  maxSingleSpendNear: number;
  hourlySpendLimitNear: number;
  dailySpendLimitNear: number;
  allowedTargets: string[];
  allowedMethods?: Record<string, string[]>;
}

export interface SpendingVelocityStatus {
  totalSpentNear: number;
  spentCurrentHourNear: number;
  spentCurrentDayNear: number;
  remainingHourlyQuotaNear: number;
  remainingDailyQuotaNear: number;
  totalTransactionsExecuted: number;
  totalViolationsPrevented: number;
}

export interface SemanticCheckResult {
  passed: boolean;
  riskScore: number; // 0 to 100
  threatType?: string;
  reason?: string;
  flaggedPatterns?: string[];
}

export interface ExecutionIntent {
  rawPromptOrContext?: string;
  targetContract: string;
  methodName: string;
  args?: Record<string, any>;
  depositNear: number;
  gasUnits?: string;
}

export interface ExecutionResult {
  success: boolean;
  head1Check: SemanticCheckResult;
  head2Check: {
    allowed: boolean;
    reason: string;
  };
  transactionHash?: string;
  error?: string;
}
