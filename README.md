# bllvm-governance

Governance webhook and economic node tracking module for bllvm-node.

## Overview

This module provides governance integration for bllvm-node, including:
- Webhook notifications to bllvm-commons
- Economic node tracking
- Veto system integration
- Governance proposal monitoring

## Installation

```bash
# Install via cargo
cargo install bllvm-governance

# Or install via cargo-bllvm-module
cargo install cargo-bllvm-module
cargo bllvm-module install bllvm-governance
```

## Configuration

Create a `config.toml` in the module directory:

```toml
[governance]
webhook_url = "https://governance.example.com/webhook"
node_id = "your_node_id"
enabled = true
```

## Module Manifest

The module includes a `module.toml` manifest:

```toml
name = "bllvm-governance"
version = "0.1.0"
description = "Governance webhook and economic node tracking module"
author = "Bitcoin Commons Team"
entry_point = "bllvm-governance"

capabilities = [
    "read_blockchain",
    "subscribe_events",
]
```

## Events

### Subscribed Events
- `GovernanceProposalCreated` - New governance proposal
- `GovernanceProposalVoted` - Vote cast on proposal
- `GovernanceProposalMerged` - Proposal merged
- `EconomicNodeRegistered` - Economic node registered
- `EconomicNodeVeto` - Economic node veto signal
- `ChainTipUpdated` - For tracking block height

### Published Events
- `WebhookSent` - Webhook notification sent
- `WebhookFailed` - Webhook delivery failed
- `VetoThresholdReached` - Veto threshold reached
- `GovernanceForkDetected` - Governance fork detected

## License

MIT License - see LICENSE file for details.

