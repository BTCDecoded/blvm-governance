# blvm-governance

Governance webhook and proposal module for blvm-node.

## Overview

This module is not loaded by a fresh node. Add it on a node you run. It posts events to a URL. On one machine it can also listen and forward block posts to `blvm-commons`. It does not start `blvm-commons`, and it does not accept GitHub webhooks.

## Installation

```bash
# Install via cargo
cargo install blvm-governance

# Or install via cargo-blvm-module
cargo install cargo-blvm-module
cargo blvm-module install blvm-governance
```

## Configuration

A fresh node does not download this module. On a node you set up, add the pin beside the modules you already enable:

```toml
[modules.blvm-governance]
version = "0.1.*"
webhook_url = "http://YOUR_PROXY:9417/webhooks/block"
node_id = "this-node"
```

On the one machine that sits in front of `blvm-commons`, listen and name that app. Leave `webhook_url` empty there. Block posts from this process go straight to commons.

```toml
[modules.blvm-governance]
version = "0.1.*"
listen = "0.0.0.0:9417"
commons_url = "http://127.0.0.1:3000"
```

`blvm-commons` stays its own service. GitHub keeps calling it at `/webhooks/github`.

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

