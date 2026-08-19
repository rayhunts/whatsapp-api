# Feature 06 — Autoreply & Chatbot

## Goal
Provide flexible automatic replies to incoming messages, from simple keyword matching to dynamic webhook responses and multi-step forms.

## Scope

- **Default reply**: fallback message for any unmatched incoming text.
- **Keyword reply**: exact or regex match triggers a templated reply.
- **Name personalization**: autoreply can include the sender's name/contact.
- **Webhook reply**: forward the incoming message to a URL and reply with the URL's response.
- **Submission / chained questions**: ask a series of questions and collect answers.
- **Quick reply**: predefined reply buttons/templates in the UI.

## API / UI

### Rules

```http
POST /api/autoreplies
Content-Type: application/json

{
  "type": "keyword",
  "pattern": "price",
  "match_mode": "contains",
  "reply": {
    "type": "text",
    "body": "Our price list is..."
  },
  "priority": 10,
  "enabled": true
}
```

Types: `default`, `keyword`, `webhook`, `submission`.

### Webhook rule

```json
{
  "type": "webhook",
  "url": "https://example.com/whatsapp-webhook",
  "method": "POST",
  "headers": { "Authorization": "Bearer token" }
}
```

Expected response:

```json
{
  "reply": {
    "type": "text",
    "body": "Thanks {{name}}!"
  }
}
```

### Submission / chained questions

```json
{
  "type": "submission",
  "steps": [
    { "ask": "What is your name?", "store_as": "name" },
    { "ask": "What is your email?", "store_as": "email" }
  ],
  "final_reply": "Thank you {{name}}, we will email you at {{email}}."
}
```

### Frontend

- "Autoreplies" view to manage rules.
- Test simulator (send a test message, see matched reply).
- Webhook log for debugging.

## Data model changes

- New type `AutoreplyRule`:
  - `id`, `type`, `pattern`, `match_mode`, `reply`, `priority`, `enabled`
- Submission state per chat stored in SQLite.

## Dependencies

- Feature 01 (Core Messaging) for outgoing replies.
- Feature 05 (Templates & Variables) for templated replies.
- HTTP client for webhooks (use `ureq` or `reqwest`; keep outside `whatsapp-api-types`).

## Implementation notes

- Evaluate rules in priority order; first match wins unless `default`.
- For webhook rules, enforce a timeout (e.g., 5s) and fail gracefully.
- Submission state must be per chat JID and expire after a configurable timeout.
- Quick reply can be implemented as a UI shortcut that calls the send API.

## Acceptance criteria

- [ ] Default reply sends for unmatched messages.
- [ ] Keyword reply matches exact, contains and regex modes.
- [ ] Webhook reply forwards the message and sends back the response.
- [ ] Submission collects multi-step answers and sends a final message.
- [ ] Autoreply rules can be enabled/disabled and reordered.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
