# Feature 02 — Broadcast & CSV Messaging

## Goal
Allow users to send the same message to many recipients at once, either via a number list in the API or by uploading a CSV file.

## Scope

- **Broadcast API**: send one message body to a list of JIDs/phone numbers.
- **CSV upload**: upload a CSV where one column is the destination number and optional columns feed variable substitution.
- Rate limiting / queueing to avoid triggering WhatsApp spam detection.
- Progress tracking for large broadcasts.

## API / UI

### Broadcast

```http
POST /api/broadcasts
Content-Type: application/json

{
  "to": ["12345", "67890"],
  "message": "Hi {{name}}!",
  "variables": {
    "12345": { "name": "Alice" },
    "67890": { "name": "Bob" }
  },
  "delay_ms": { "min": 1000, "max": 3000 }
}
```

Response:

```json
{
  "broadcast_id": "bcast_uuid",
  "total": 2,
  "status": "queued"
}
```

### CSV upload

```http
POST /api/broadcasts/csv
Content-Type: multipart/form-data

file: contacts.csv
mapping: { "phone": "phone_column", "name": "name_column" }
message: "Hi {{name}}!"
```

CSV example:

```csv
phone,name
628123456789,Alice
628987654321,Bob
```

### Frontend

- New "Broadcast" view with:
  - textarea for message
  - number list input or CSV drag-and-drop
  - variable preview
  - progress bar

## Data model changes

- New types:
  - `BroadcastJob { id, message, total, sent, failed, status, created_at }`
  - `BroadcastRecipient { job_id, jid, variables, status, error }`
- Add to `whatsapp-api-types`.

## Dependencies

- Feature 01 (Core Messaging) for single send and status tracking.
- Feature 05 (Templates & Variables) for variable substitution.
- Feature 09 (Sending Strategies) for random delay.

## Implementation notes

- Persist broadcast jobs in SQLite (`DB_PATH`) so progress survives restarts.
- Run broadcasts via a background tokio task, not inline in the HTTP handler.
- Respect `Random Delay` settings.
- Reuse the engine's `send_text` for each recipient; never call `whatsapp-rust` directly outside the engine crate.

## Acceptance criteria

- [ ] Broadcast API queues a job and returns an ID.
- [ ] CSV upload accepts a mapped file and queues a broadcast job.
- [ ] Progress events are pushed over `/ws`.
- [ ] Failed numbers are recorded with a reason.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
