# Feature 08 — Devices & Session Management

## Goal
Support multiple WhatsApp devices per account with routing, status visibility and session recovery.

## Scope

- Register multiple devices (each with its own QR scan / SQLite store).
- Per-device status: `connecting`, `connected`, `disconnected`, `logged_out`.
- Random/round-robin device routing for sends/broadcasts.
- Restart and reconnect logic.

## API / UI

### Devices

```http
POST /api/devices
Content-Type: application/json

{
  "name": "Office Phone",
  "description": "Primary sender"
}
```

Response includes `device_id` and QR code event stream.

```http
GET    /api/devices
GET    /api/devices/{id}
DELETE /api/devices/{id}
POST   /api/devices/{id}/reconnect
```

### Send with device routing

```http
POST /api/chats/{jid}/messages
Content-Type: application/json

{
  "body": "Hi",
  "device_id": "optional_specific_device"
}
```

Or broadcast-wide routing:

```json
{
  "to": [...],
  "message": "Hi",
  "device_strategy": "random"
}
```

### Frontend

- "Devices" view listing all devices and connection status.
- QR scan modal for adding a new device.
- Indicator showing which device sent each message.

## Data model changes

- New type `Device`:
  - `id`, `name`, `description`, `status`, `store_path`, `created_at`
- Engine must support multiple `WhatsappEngine` instances keyed by device ID.

## Dependencies

- Refactor `whatsapp-api-engine` to manage multiple `Bot` instances.
- Feature 01 (Core Messaging) for send routing.

## Implementation notes

- Each device needs a unique SQLite store path (e.g., `whatsapp_{device_id}.db`).
- The `WaEngine` trait remains the abstraction; `WhatsappEngine` becomes a device-scoped instance.
- Random/round-robin routing must skip offline devices.
- The current auto-restart every 2s becomes per-device health loop.

## Acceptance criteria

- [ ] Can register and connect multiple devices.
- [ ] Each device has independent connection status.
- [ ] Sending picks an online device when no device is specified.
- [ ] Can force reconnect a specific device.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
