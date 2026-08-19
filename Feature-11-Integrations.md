# Feature 11 — Integrations

## Goal
Make the API easy to integrate with external tools and provide ready-made examples for popular form/CMS platforms.

## Scope

- OpenAPI/Swagger documentation.
- Outgoing webhooks for message events.
- API token authentication.
- Example integrations: Google Forms, Contact Form 7, WooCommerce, Elementor, Caldera Forms, Formidable Forms.

## API / UI

### API tokens

```http
POST /api/tokens
Content-Type: application/json

{
  "name": "WooCommerce store"
}
```

Response:

```json
{
  "token": "wfa_xxxxxxxx",
  "name": "WooCommerce store",
  "created_at": "..."
}
```

```http
GET    /api/tokens
DELETE /api/tokens/{id}
```

All API requests require:

```http
Authorization: Bearer wfa_xxxxxxxx
```

### Webhook subscriptions

```http
POST /api/webhooks
Content-Type: application/json

{
  "url": "https://example.com/whatsapp-webhook",
  "events": ["message_received", "message_status_update", "device_status_change"]
}
```

### OpenAPI

- Serve `/api/openapi.json` and `/docs` (Swagger UI or ReDoc).

### Frontend

- "API & Integrations" view with token management and webhook log.
- Copy-paste code snippets for PHP/WordPress.

## Data model changes

- New types:
  - `ApiToken { id, name, token_hash, created_at }`
  - `Webhook { id, url, events, secret, enabled, created_at }`
  - `WebhookDelivery { id, webhook_id, event, payload, response_status, created_at }`
- Store in SQLite.

## Dependencies

- Feature 06 (Autoreply) webhook reply overlaps; reuse HTTP client.
- Feature 10 (Message History) for event payloads.

## Implementation notes

- Hash tokens with SHA-256 before storage; only show the token once on creation.
- Webhook deliveries should be retried with exponential backoff (max 3 attempts).
- Sign webhook payloads with a per-webhook secret (`X-Webhook-Signature`).
- Integration examples can live in a new `examples/` folder or `docs/`.

## Acceptance criteria

- [ ] API requests require a valid bearer token.
- [ ] OpenAPI spec is served and up to date.
- [ ] Webhooks are delivered for subscribed events.
- [ ] Failed webhook deliveries are retried.
- [ ] Example WordPress/WooCommerce snippets are documented.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
