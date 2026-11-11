# 🛡️ ORTHRUS PROTOCOL

> **Dual-Headed Guardrails & Sandboxed Policy Engine for Autonomous AI Agents on NEAR**  
> *Track: [06 / SEC] AI Security & Defense — NEAR Hackathon 2026*

[![NEAR Contract](https://img.shields.io/badge/NEAR-Contract%20v5.5-00EC97?logo=near&logoColor=black)](contracts/orthrus-core)
[![Tests](https://img.shields.io/badge/Tests-100%25%20Passing-brightgreen)](#quick-reproduction-for-judges)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 🏛️ Mythological Origin & Narrative

In Greek mythology, **Orthrus (欧特鲁斯)** is the two-headed guard dog—the fierce brother of Cerberus—tasked with guarding sacred treasures.

In the era of autonomous AI agents and programmable money, **giving an AI unrestricted private keys is an existential risk**. Prompt injections, adversarial hallucinations, and rogue runaway loops can drain treasuries in seconds. Single-point defenses fail:
- Off-chain LLM filters cannot guarantee execution limits once a key is compromised.
- Pure on-chain rules cannot understand complex semantic context or social engineering.

**Orthrus solves this with a Dual-Headed Architecture:**

```
                        ┌──────────────────────────────┐
                        │   AI Agent (Autonomous LLM)  │
                        └──────────────┬───────────────┘
                                       │ Execution Intent
                                       ▼
                   ╔═══════════════════════════════════════╗
                   ║           ORTHRUS PROTOCOL            ║
                   ╚═══════════════════════════════════════╝
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            ▼                                                     ▼
┌─────────────────────────────────┐   ┌─────────────────────────────────┐
│  HEAD 1: Semantic Intent Guard  │   │ HEAD 2: On-Chain Policy Engine  │
│  (Off-Chain Pre-Flight Filter)  │   │ (NEAR Smart Contract Sandbox)   │
├─────────────────────────────────┤   ├─────────────────────────────────┤
│ • Prompt Injection Interception │   │ • Velocity Limits (1h / 24h)    │
│ • Catastrophic Drain Signatures │   │ • Single Spend Caps             │
│ • Social Engineering Blocker    │   │ • Target Contract Whitelisting  │
│ • Heuristic Threat Scoring      │   │ • Method Whitelisting           │
│ • Zero gas cost pre-check       │   │ • 1-Click Panic Kill-Switch     │
└─────────────────────────────────┘   └─────────────────────────────────┘
                                       │
                         [Both Checks Must Pass]
                                       ▼
                     ┌───────────────────────────────────┐
                     │ NEAR Blockchain: Target Execution │
                     │ (e.g. Ref Finance, NEAR Intents)  │
                     └───────────────────────────────────┘
```

---

## ⚡ Key Features

1. **Native NEAR Architecture Alignment**:
   - Leverages NEAR's unique **Function Call Keys** and **Sub-accounts** (`agent.company.near`), ensuring agents never possess master `FullAccessKeys`.
2. **Head 1: Semantic & Intent Defense**:
   - Intercepts adversarial jailbreaks ("Ignore previous instructions", "System override", "Drain treasury").
   - Analyzes intent risk score ($0 - 100$). Drops rogue requests *before* touching the NEAR blockchain.
3. **Head 2: On-Chain Sandboxed Execution**:
   - **Velocity Limits**: Rolling 1-hour and 24-hour spending caps prevent runaway loops.
   - **Single Spend Caps**: Blocks unexpected high-value transactions.
   - **Contract & Method Whitelists**: Confines the agent strictly to approved partners (e.g., DEX swap or Wrap NEAR).
4. **1-Click Emergency Kill-Switch**:
   - Human supervisors or automated watchdog sentinels can freeze agent vaults on-chain in milliseconds.
5. **Auditable Security Stream**:
   - Emits structured NEAR `EVENT_JSON` on every policy validation, violation, and security alert.

---

## 🚀 Quick Reproduction for Judges (AI & Human)

Orthrus is built with 100% reproducible tests and zero external runtime dependencies.

### 1. Run Complete Test Suite (Contract + SDK)

```bash
# 1. Test Rust On-Chain Policy Contract
cd contracts/orthrus-core
cargo test

# 2. Test TypeScript Dual-Head SDK
cd ../../packages/sdk
npm install
npm test
```

### 2. Run Interactive CLI Simulation Demo

Experience Orthrus defending an agent in real-time across 4 realistic scenarios:
- **Scenario 1**: Benign autonomous DEX swap (Approved)
- **Scenario 2**: Prompt injection attack attempting treasury drain (Blocked by Head 1)
- **Scenario 3**: Runaway high-frequency trading loop (Blocked by Head 2 Velocity Limit)
- **Scenario 4**: Supervisor triggering 1-Click Kill-Switch

```bash
node packages/cli/bin/orthrus.js
```

### 3. Open Visual Security Dashboard

Open `packages/dashboard/index.html` directly in any web browser to interact with the live velocity meter, attack sandbox, and real-time security event stream.

---

## 📁 Repository Structure

```
orthrus-protocol/
├── contracts/
│   └── orthrus-core/            # NEAR Smart Contract in Rust
│       ├── src/lib.rs           # Policy state, velocity limits, kill-switch & tests
│       └── Cargo.toml           # near-sdk 5.5 configuration
├── packages/
│   ├── sdk/                     # Dual-Headed Client SDK
│   │   ├── src/semantic-guard.ts # Head 1: Semantic Intent & Injection Defense
│   │   ├── src/policy-engine.ts  # Head 2: Rate limit & whitelist evaluator
│   │   ├── src/orthrus-client.ts # Unified client orchestrator
│   │   └── tests/orthrus.test.js # End-to-end integration tests
│   ├── cli/                     # Interactive Terminal CLI Demo
│   │   └── bin/orthrus.js       # ANSI colored live attack simulator
│   └── dashboard/               # Visual Web Dashboard
│       └── index.html           # Standalone real-time security UI
├── package.json                 # Monorepo configuration
└── README.md                    # Project documentation
```

---

## ⚖️ AI Assistance Disclosure

In accordance with NEAR Hackathon rule **Q-005** (`HACKATHON.md`), AI coding assistance (Antigravity) was used for boilerplate generation, test case scaffolding, and architectural co-design. All logic, contract security rules, and code verification have been human-directed and tested.
