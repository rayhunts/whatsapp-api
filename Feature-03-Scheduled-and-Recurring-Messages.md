# Feature 03 — Scheduled & Recurring Messages

## Goal
Let users schedule messages to be sent once at a future time or repeat on a schedule.

## Scope

- **Scheduled**: one-time future send.
- **Recurring**: cron-like repetition (e.g., daily, weekly, monthly).
- List, edit and cancel scheduled jobs.
- Timezone-aware scheduling.

## API / UI

### Schedule a message

```http
POST /api/schedules
Content-Type: application/json

{
  "to": "12345@s.whatsapp.net",
  "message": "Reminder",
  "type": "text",
  "send_at": "2026-08-20T09:00:00+07:00",
  "recurrence": null
}
```

### Recurring

```json
{
  "recurrence": {
    "frequency": "daily",
    "interval": 1,
    "until": "2026-12-31T23:59:59Z"
  }
}
```

### Management

```http
GET    /api/schedules
GET    /api/schedules/{id}
DELETE /api/schedules/{id}
PATCH  /api/schedules/{id}
```

### Frontend

- "Schedules" view with calendar/list, create form, edit, delete.

## Data model changes

- New type `ScheduledMessage`:
  - `id`, `to`, `message`, `type`, `send_at`
  - `recurrence: Option<Recurrence>`
  - `status: active | paused | completed | cancelled`
  - `created_at`

## Dependencies

- Feature 01 (Core Messaging) for actual sending.
- A scheduler/task runner in `whatsapp-api-server` (tokio interval or `tokio-cron-scheduler`).

## Implementation notes

- Store schedules in SQLite.
- Use a single background tokio task that wakes every minute and enqueues due messages.
- For recurring jobs, compute the next occurrence after a successful send.
- Idempotency: skip if already sent for a given occurrence.

## Acceptance criteria

- [ ] Can create a one-time scheduled message.
- [ ] Can create a recurring message.
- [ ] Due messages are sent automatically.
- [ ] Can list, edit, pause and cancel schedules.
- [ ] Recurrence correctly computes the next occurrence.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
