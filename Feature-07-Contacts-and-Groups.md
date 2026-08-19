# Feature 07 — Contacts & Groups

## Goal
Manage a contact book and contact groups for targeted messaging and group support.

## Scope

- CRUD contacts (name, phone, email, notes).
- Contact groups / labels.
- Use groups as broadcast targets.
- Send messages to WhatsApp groups (`@g.us`).
- Sync contacts from phone/address book (optional future).

## API / UI

### Contacts

```http
POST /api/contacts
Content-Type: application/json

{
  "name": "Alice",
  "phone": "628123456789",
  "email": "alice@example.com",
  "notes": "VIP customer"
}
```

```http
GET    /api/contacts
GET    /api/contacts/{id}
PATCH  /api/contacts/{id}
DELETE /api/contacts/{id}
```

### Groups

```http
POST /api/contact-groups
Content-Type: application/json

{
  "name": "Customers",
  "contact_ids": ["uuid1", "uuid2"]
}
```

```http
POST /api/broadcasts
Content-Type: application/json

{
  "group_ids": ["group_uuid"],
  "message": "Hello valued customers!"
}
```

### Frontend

- "Contacts" view with table, search, import/export.
- "Groups" view to create groups and add contacts.
- Group picker in broadcast form.

## Data model changes

- New types:
  - `Contact { id, name, phone, email, notes, created_at }`
  - `ContactGroup { id, name, contact_ids, created_at }`
- Store in SQLite.

## Dependencies

- Feature 02 (Broadcast & CSV) to use groups as targets.

## Implementation notes

- Normalize phone numbers to international format (strip leading `0`, add country code if configured).
- Validate that a phone number can be turned into a WA JID.
- Groups are local labels; WhatsApp group chats are handled by the engine via `@g.us` JIDs.

## Acceptance criteria

- [x] Can create, list, update and delete contacts.
- [x] Can create contact groups and add/remove members.
- [x] Can broadcast to one or more contact groups.
- [x] WhatsApp group JIDs (`@g.us`) are accepted as send targets.
- [x] `cargo build --workspace` and `cargo clippy --workspace` pass.
