# Feature 10 — Message History & Delivery Status

## Goal
Give users a complete, searchable log of all sent and received messages with real-time delivery status.

## Scope

- Message history/log page.
- Real-time status updates: `pending`, `sent`, `delivered`, `read`, `failed`.
- Filter by date, JID, direction, status, device.
- Invoice / usage log (basic counters for now).

## API / UI

### History

```http
GET /api/messages?jid=...&status=...&from=...&to=...&page=1&limit=50
```

Response:

```json
{
  "data": [
    {
      "id": "msg_uuid",
      "jid": "12345@s.whatsapp.net",
      "direction": "outgoing",
      "content": { "type": "text", "body": "Hi" },
      "status": "read",
      "device_id": "dev_uuid",
      "created_at": "...",
      "updated_at": "..."
    }
  ],
  "total": 100
}
```

### Status WebSocket event

```json
{
  "type": "message_status_update",
  "message_id": "msg_uuid",
  "status": "delivered",
  "timestamp": "..."
}
```

### Frontend

- "History" view with filters, pagination, search.
- Message status icons in chat bubbles (✓, ✓✓, blue ✓✓, ⚠).

## Data model changes

- Persist all messages in SQLite (replace or supplement the in-memory registry).
- Indexes on `jid`, `status`, `created_at`, `device_id`.

## Dependencies

- Feature 01 (Core Messaging) status foundation.
- `whatsapp-rust` receipts/events for delivery/read updates.

## Implementation notes

- Keep the in-memory registry as a hot cache but persist to SQLite for history.
- Cap history retention via env var or settings (default unlimited, with optional pruning).
- Invoice/usage can be a daily aggregation table: `device_id`, `date`, `sent_count`, `received_count`.

## Acceptance criteria

- [ ] All messages are persisted and queryable.
- [ ] Delivery/read receipts update message status in real time.
- [ ] History view supports filters and pagination.
- [ ] Basic usage counters are available.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
