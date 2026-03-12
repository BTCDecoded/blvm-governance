# blvm-governance

Governance webhook and economic node tracking module for blvm-node.

## Overview

This module provides governance integration for blvm-node, including:
- Webhook notifications to blvm-commons
- Economic node tracking
- Veto system integration
- Governance proposal monitoring

## Installation

```bash
# Install via cargo
cargo install blvm-governance

# Or install via cargo-blvm-module
cargo install cargo-blvm-module
cargo blvm-module install blvm-governance
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
name = "blvm-governance"
version = "0.1.0"
description = "Governance webhook and economic node tracking module"
author = "Bitcoin Commons Team"
entry_point = "blvm-governance"

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

