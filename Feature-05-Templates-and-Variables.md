# Feature 05 — Templates & Variables

## Goal
Allow users to save reusable message templates and personalize messages with variables like `{{name}}` or `{{order_id}}`.

## Scope

- CRUD message templates.
- Variable substitution in templates, broadcasts and scheduled messages.
- Default/fallback values for missing variables.
- Button messages (deprecated by WhatsApp; keep as optional/low priority).

## API / UI

### Templates

```http
POST /api/templates
Content-Type: application/json

{
  "name": "Order Confirmed",
  "body": "Hi {{name}}, your order {{order_id}} is confirmed!",
  "variables": ["name", "order_id"]
}
```

```http
GET    /api/templates
GET    /api/templates/{id}
PATCH  /api/templates/{id}
DELETE /api/templates/{id}
```

### Send with template

```http
POST /api/chats/{jid}/messages
Content-Type: application/json

{
  "template_id": "tpl_uuid",
  "variables": {
    "name": "Alice",
    "order_id": "ORD-123"
  }
}
```

### Variable syntax

- `{{variable}}` — required, fails if missing.
- `{{variable|default}}` — optional fallback.

### Frontend

- "Templates" view to create/manage templates.
- Template picker in composer/broadcast forms.
- Live preview with sample variables.

## Data model changes

- New type `Template`:
  - `id`, `name`, `body`, `variables: Vec<String>`, `created_at`
- Utility for variable parsing/replacement in `whatsapp-api-types` or server.

## Dependencies

- Feature 01 (Core Messaging) for sending.
- Feature 02 (Broadcast & CSV) for CSV variable mapping.

## Implementation notes

- Keep template storage in SQLite.
- Variable parsing must be deterministic and safe (no code execution).
- Button messages: implement only if `whatsapp-rust` still supports them; otherwise mark deprecated in docs.

## Acceptance criteria

- [ ] Can create, list, update and delete templates.
- [ ] Variables are substituted correctly in single and broadcast sends.
- [ ] Missing required variables return a clear error.
- [ ] Template picker works in the web UI.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
