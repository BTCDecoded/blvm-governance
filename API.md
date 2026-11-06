# blvm-governance API Documentation

## Overview

The `blvm-governance` module delivers governance webhooks and stores proposals published by the node.

## Modules

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

### `proposals`

Stores proposals received from the node.

## Events

### Subscribed Events
- `GovernanceProposalCreated`
- `GovernanceProposalVoted`
- `GovernanceProposalMerged`
- `NewBlock`

### Published Events
- `WebhookSent` - Webhook successfully delivered
- `WebhookFailed` - Webhook delivery failed

## Error Handling

All methods return `Result<T, GovernanceError>` where `GovernanceError` can be:
- `WebhookError(String)` - Webhook delivery failed
- `ConfigError(String)` - Configuration error

## Examples

### Sending Webhooks

```rust
let webhook_client = GovernanceWebhookClient::new(&ctx).await?;
// Webhooks are sent automatically when events are received
```
