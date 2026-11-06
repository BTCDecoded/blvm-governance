# blvm-governance

Governance webhook and proposal module for blvm-node.

## Overview

This module provides governance integration for blvm-node, including:
- Webhook notifications to blvm-commons
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
version = "0.1"
description = "Governance webhook and proposal module"
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
- `NewBlock` - Block notification

### Published Events
- `WebhookSent` - Webhook notification sent
- `WebhookFailed` - Webhook delivery failed

## License

MIT License - see LICENSE file for details.

