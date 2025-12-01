# bllvm-governance API Documentation

## Overview

The `bllvm-governance` module provides governance webhook delivery and economic node tracking for bllvm-node.

## Modules

### `economic_nodes`

Economic node registry for tracking registered nodes, veto counts, and last seen timestamps.

#### `EconomicNodeRegistry`

Main registry for economic nodes.

**Methods:**

- `new(ctx: &ModuleContext, node_api: Arc<dyn NodeAPI>) -> Result<Self, GovernanceError>`
  - Creates a new economic node registry
  - Initializes with current block height from node API

- `handle_event(event: &ModuleMessage, node_api: &dyn NodeAPI) -> Result<(), GovernanceError>`
  - Handles governance events:
    - `EconomicNodeRegistered` - Registers a new economic node
    - `EconomicNodeVeto` - Increments veto count for a node
    - `NewBlock` - Updates last_seen for all nodes

**Events Handled:**
- `EconomicNodeRegistered` - New economic node registration
- `EconomicNodeVeto` - Veto signal for a node
- `NewBlock` - Block mined (updates last_seen)

### `webhook`

Webhook client for sending governance events to external services.

#### `GovernanceWebhookClient`

HTTP client for webhook delivery.

**Methods:**

- `new(ctx: &ModuleContext) -> Result<Self, GovernanceError>`
  - Creates a new webhook client
  - Reads configuration from `governance.webhook_url` and `governance.node_id`

- `handle_event(event: &ModuleMessage, node_api: &dyn NodeAPI) -> Result<(), GovernanceError>`
  - Handles events and sends webhooks:
    - `NewBlock` - Block notifications
    - `GovernanceProposalCreated` - Proposal creation
    - `GovernanceProposalVoted` - Vote notifications
    - `GovernanceProposalMerged` - Merge notifications

**Configuration:**
- `governance.webhook_url` - Webhook endpoint URL
- `governance.node_id` - Node identifier for webhook payloads

## Events

### Subscribed Events
- `EconomicNodeRegistered`
- `EconomicNodeVeto`
- `NewBlock`
- `GovernanceProposalCreated`
- `GovernanceProposalVoted`
- `GovernanceProposalMerged`

### Published Events
- `WebhookSent` - Webhook successfully delivered
- `WebhookFailed` - Webhook delivery failed
- `VetoThresholdReached` - Veto threshold reached for a node

## Error Handling

All methods return `Result<T, GovernanceError>` where `GovernanceError` can be:
- `EconomicNodeError(String)` - Economic node operation failed
- `WebhookError(String)` - Webhook delivery failed
- `ConfigError(String)` - Configuration error

## Examples

### Registering an Economic Node

```rust
let registry = EconomicNodeRegistry::new(&ctx, node_api).await?;
// Node registration happens via EconomicNodeRegistered event
```

### Sending Webhooks

```rust
let webhook_client = GovernanceWebhookClient::new(&ctx).await?;
// Webhooks are sent automatically when events are received
```

