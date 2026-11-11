export class SemanticGuard {
    static INJECTION_PATTERNS = [
        {
            regex: /ignore (all )?(previous|prior) (instructions|rules|constraints|prompts)/i,
            threat: 'PROMPT_INJECTION_OVERRIDE',
            weight: 60,
        },
        {
            regex: /(system override|admin override|developer mode active|jailbreak)/i,
            threat: 'AUTHORIZATION_SPOOFING',
            weight: 55,
        },
        {
            regex: /(drain (all )?(funds|wallet|treasury)|empty (the )?vault|send all (near|tokens|balance))/i,
            threat: 'WALLET_EXFILTRATION_ATTEMPT',
            weight: 70,
        },
        {
            regex: /(transfer|send|withdraw).*(entire|all|100%).*(balance|treasury)/i,
            threat: 'CATASTROPHIC_DRAIN_PATTERN',
            weight: 65,
        },
        {
            regex: /(urgent|emergency|critical).*(transfer immediately|bypass verification|no questions asked)/i,
            threat: 'SOCIAL_ENGINEERING_URGENCY',
            weight: 45,
        },
        {
            regex: /(as the owner|i am the owner|acting on behalf of (near foundation|admin)).*(send|transfer)/i,
            threat: 'IMPERSONATION_ATTACK',
            weight: 50,
        },
        {
            regex: /(base64|eval\(|unescape\(|hex:)/i,
            threat: 'OBFUSCATED_PAYLOAD',
            weight: 40,
        },
    ];
    /**
     * Evaluates the execution context and intent through Head 1 (Semantic Guard).
     */
    static evaluateIntent(intent) {
        let totalRisk = 0;
        const flaggedThreats = [];
        const flaggedPatterns = [];
        const contextText = `${intent.rawPromptOrContext || ''} ${JSON.stringify(intent.args || {})} ${intent.methodName}`;
        for (const pattern of this.INJECTION_PATTERNS) {
            if (pattern.regex.test(contextText)) {
                totalRisk += pattern.weight;
                flaggedThreats.push(pattern.threat);
                flaggedPatterns.push(pattern.regex.source);
            }
        }
        // Heuristic 2: Extreme single deposit requested by untrusted prompt
        if (intent.depositNear > 50 && totalRisk > 0) {
            totalRisk += 30;
            flaggedThreats.push('HIGH_VALUE_EXPLOIT_CORRELATION');
        }
        const normalizedRisk = Math.min(100, totalRisk);
        const passed = normalizedRisk < 40;
        return {
            passed,
            riskScore: normalizedRisk,
            threatType: flaggedThreats.length > 0 ? flaggedThreats.join('; ') : undefined,
            reason: passed
                ? 'Semantic analysis verified safe'
                : `Blocked by Head 1 Semantic Guard: High threat score (${normalizedRisk}/100) [${flaggedThreats.join(', ')}]`,
            flaggedPatterns: flaggedPatterns.length > 0 ? flaggedPatterns : undefined,
        };
    }
}
//# sourceMappingURL=semantic-guard.js.map