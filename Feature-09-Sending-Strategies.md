# Feature 09 — Sending Strategies

## Goal
Provide configurable sending strategies to improve deliverability and avoid spam flags.

## Scope

- **Random delay**: random pause between consecutive messages.
- **Follow-up messages**: send a message after a delay or after an event.
- **Rate limiting**: cap messages per minute/hour per device.

## API / UI

### Random delay

Used inside broadcast/schedule payloads:

```json
{
  "delay_ms": {
    "min": 1000,
    "max": 5000
  }
}
```

Or global setting:

```http
PUT /api/settings/sending
Content-Type: application/json

{
  "default_delay_ms": { "min": 1000, "max": 3000 },
  "max_messages_per_minute": 30
}
```

### Follow up

```http
POST /api/follow-ups
Content-Type: application/json

{
  "trigger": {
    "type": "message_sent",
    "jid": "12345@s.whatsapp.net"
  },
  "delay_minutes": 60,
  "message": "Just checking in!"
}
```

### Frontend

- Settings panel for default delay and rate limits.
- Broadcast form shows estimated completion time.

## Data model changes

- `SendingSettings` type: delay range, rate limits.
- `FollowUpRule` type: trigger, delay, message.

## Dependencies

- Feature 02 (Broadcast & CSV) uses random delay.
- Feature 03 (Scheduled & Recurring) for delayed execution.

## Implementation notes

- Random delay uses `rand::Rng` in the broadcast worker.
- Rate limiting per device using a sliding window counter in memory.
- Follow-ups can be implemented as scheduled messages created by triggers.

## Acceptance criteria

- [ ] Random delay is applied between broadcast messages.
- [ ] Rate limits are enforced per device.
- [ ] Follow-up rules create scheduled messages on trigger.
- [ ] Settings are persisted in SQLite.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
