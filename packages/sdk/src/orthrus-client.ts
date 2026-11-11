import { SemanticGuard } from './semantic-guard.js';
import { LocalPolicyEngine } from './policy-engine.js';
import {
  AgentPolicyConfig,
  ExecutionIntent,
  ExecutionResult,
  SpendingVelocityStatus,
} from './types.js';

export interface OrthrusClientOptions {
  agentId: string;
  policy: AgentPolicyConfig;
  contractId?: string;
  enableOnChainSimulation?: boolean;
}

export class OrthrusClient {
  public readonly agentId: string;
  public readonly contractId?: string;
  private policyEngine: LocalPolicyEngine;

  constructor(options: OrthrusClientOptions) {
    this.agentId = options.agentId;
    this.contractId = options.contractId;
    this.policyEngine = new LocalPolicyEngine(options.policy);
  }

  public getStatus(): SpendingVelocityStatus {
    return this.policyEngine.getStatus();
  }

  public triggerKillSwitch(paused: boolean): void {
    this.policyEngine.setPaused(paused);
  }

  /**
   * Dual-Headed Execution Pipeline:
   * 1. Head 1: Semantic Guard analyzes prompt injection and rogue intent
   * 2. Head 2: Policy Engine enforces velocity limits, whitelists and kill-switch
   */
  public async execute(intent: ExecutionIntent): Promise<ExecutionResult> {
    // --- HEAD 1: Semantic & Intent Defense ---
    const head1Check = SemanticGuard.evaluateIntent(intent);
    if (!head1Check.passed) {
      return {
        success: false,
        head1Check,
        head2Check: {
          allowed: false,
          reason: 'Head 2 skipped: Blocked by Head 1 Semantic Guard',
        },
        error: head1Check.reason,
      };
    }

    // --- HEAD 2: On-Chain Policy & Limits Enforcement ---
    const head2Check = this.policyEngine.evaluatePolicy(intent);
    if (!head2Check.allowed) {
      return {
        success: false,
        head1Check,
        head2Check,
        error: `Blocked by Head 2 Policy Guard: ${head2Check.reason}`,
      };
    }

    // Execution Simulation (or On-Chain RPC Dispatch)
    this.policyEngine.recordSuccess(intent.depositNear);

    const mockTxHash = `orthrus_tx_${Date.now().toString(36)}_${Math.random().toString(36).substring(2, 8)}`;

    return {
      success: true,
      head1Check,
      head2Check,
      transactionHash: mockTxHash,
    };
  }
}
