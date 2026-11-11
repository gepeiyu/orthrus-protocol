import { AgentPolicyConfig, ExecutionIntent, ExecutionResult, SpendingVelocityStatus } from './types.js';
export interface OrthrusClientOptions {
    agentId: string;
    policy: AgentPolicyConfig;
    contractId?: string;
    enableOnChainSimulation?: boolean;
}
export declare class OrthrusClient {
    readonly agentId: string;
    readonly contractId?: string;
    private policyEngine;
    constructor(options: OrthrusClientOptions);
    getStatus(): SpendingVelocityStatus;
    triggerKillSwitch(paused: boolean): void;
    /**
     * Dual-Headed Execution Pipeline:
     * 1. Head 1: Semantic Guard analyzes prompt injection and rogue intent
     * 2. Head 2: Policy Engine enforces velocity limits, whitelists and kill-switch
     */
    execute(intent: ExecutionIntent): Promise<ExecutionResult>;
}
//# sourceMappingURL=orthrus-client.d.ts.map