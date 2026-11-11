import { AgentPolicyConfig, ExecutionIntent, SpendingVelocityStatus } from './types.js';
export declare class LocalPolicyEngine {
    private policy;
    private isPaused;
    private totalSpentNear;
    private currentHourWindowStart;
    private spentCurrentHourNear;
    private currentDayWindowStart;
    private spentCurrentDayNear;
    private totalTxCount;
    private totalViolationsPrevented;
    constructor(policy: AgentPolicyConfig);
    setPaused(paused: boolean): void;
    getStatus(): SpendingVelocityStatus;
    evaluatePolicy(intent: ExecutionIntent): {
        allowed: boolean;
        reason: string;
    };
    recordSuccess(depositNear: number): void;
    private refreshWindows;
}
//# sourceMappingURL=policy-engine.d.ts.map