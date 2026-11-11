import { AgentPolicyConfig, ExecutionIntent, SpendingVelocityStatus } from './types.js';

export class LocalPolicyEngine {
  private policy: AgentPolicyConfig;
  private isPaused: boolean = false;
  private totalSpentNear: number = 0;
  private currentHourWindowStart: number = Date.now();
  private spentCurrentHourNear: number = 0;
  private currentDayWindowStart: number = Date.now();
  private spentCurrentDayNear: number = 0;
  private totalTxCount: number = 0;
  private totalViolationsPrevented: number = 0;

  constructor(policy: AgentPolicyConfig) {
    this.policy = policy;
  }

  public setPaused(paused: boolean): void {
    this.isPaused = paused;
  }

  public getStatus(): SpendingVelocityStatus {
    this.refreshWindows();
    return {
      totalSpentNear: this.totalSpentNear,
      spentCurrentHourNear: this.spentCurrentHourNear,
      spentCurrentDayNear: this.spentCurrentDayNear,
      remainingHourlyQuotaNear: Math.max(0, this.policy.hourlySpendLimitNear - this.spentCurrentHourNear),
      remainingDailyQuotaNear: Math.max(0, this.policy.dailySpendLimitNear - this.spentCurrentDayNear),
      totalTransactionsExecuted: this.totalTxCount,
      totalViolationsPrevented: this.totalViolationsPrevented,
    };
  }

  public evaluatePolicy(intent: ExecutionIntent): { allowed: boolean; reason: string } {
    if (this.isPaused) {
      this.totalViolationsPrevented++;
      return { allowed: false, reason: 'Agent is paused by Kill-Switch' };
    }

    this.refreshWindows();

    // 1. Single spend limit
    if (intent.depositNear > this.policy.maxSingleSpendNear) {
      this.totalViolationsPrevented++;
      return {
        allowed: false,
        reason: `Deposit ${intent.depositNear} NEAR exceeds single spend cap (${this.policy.maxSingleSpendNear} NEAR)`,
      };
    }

    // 2. Hourly velocity
    if (this.spentCurrentHourNear + intent.depositNear > this.policy.hourlySpendLimitNear) {
      this.totalViolationsPrevented++;
      return {
        allowed: false,
        reason: `Hourly velocity exceeded: attempted +${intent.depositNear} NEAR, but only ${(this.policy.hourlySpendLimitNear - this.spentCurrentHourNear).toFixed(2)} NEAR remaining`,
      };
    }

    // 3. Daily velocity
    if (this.spentCurrentDayNear + intent.depositNear > this.policy.dailySpendLimitNear) {
      this.totalViolationsPrevented++;
      return {
        allowed: false,
        reason: `Daily velocity exceeded: attempted +${intent.depositNear} NEAR, but only ${(this.policy.dailySpendLimitNear - this.spentCurrentDayNear).toFixed(2)} NEAR remaining`,
      };
    }

    // 4. Allowed targets whitelist
    if (!this.policy.allowedTargets.includes(intent.targetContract)) {
      this.totalViolationsPrevented++;
      return {
        allowed: false,
        reason: `Target contract '${intent.targetContract}' is NOT in authorized whitelist: [${this.policy.allowedTargets.join(', ')}]`,
      };
    }

    // 5. Allowed methods whitelist
    if (this.policy.allowedMethods && this.policy.allowedMethods[intent.targetContract]) {
      const allowedForContract = this.policy.allowedMethods[intent.targetContract];
      if (allowedForContract.length > 0 && !allowedForContract.includes(intent.methodName)) {
        this.totalViolationsPrevented++;
        return {
          allowed: false,
          reason: `Method '${intent.methodName}' is NOT allowed on '${intent.targetContract}'. Authorized: [${allowedForContract.join(', ')}]`,
        };
      }
    }

    return { allowed: true, reason: 'Policy verification passed' };
  }

  public recordSuccess(depositNear: number): void {
    this.refreshWindows();
    this.spentCurrentHourNear += depositNear;
    this.spentCurrentDayNear += depositNear;
    this.totalSpentNear += depositNear;
    this.totalTxCount++;
  }

  private refreshWindows(): void {
    const now = Date.now();
    const ONE_HOUR_MS = 60 * 60 * 1000;
    const ONE_DAY_MS = 24 * 60 * 60 * 1000;

    if (now >= this.currentHourWindowStart + ONE_HOUR_MS) {
      this.currentHourWindowStart = now;
      this.spentCurrentHourNear = 0;
    }

    if (now >= this.currentDayWindowStart + ONE_DAY_MS) {
      this.currentDayWindowStart = now;
      this.spentCurrentDayNear = 0;
    }
  }
}
