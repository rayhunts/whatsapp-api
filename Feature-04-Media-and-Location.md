# Feature 04 — Media & Location Messages

## Goal
Support sending media (image, document, audio, video) and location pins through the API and web UI.

## Scope

- Send image, document, audio, video with optional caption.
- Upload media via multipart form data or URL.
- Send location messages (latitude, longitude, name, address).
- Download incoming media via webhook/file API.

## API / UI

### Send media

```http
POST /api/chats/{jid}/messages
Content-Type: multipart/form-data

type: image
file: <binary>
caption: "Look at this"
```

Or via URL:

```json
{
  "type": "image",
  "url": "https://example.com/photo.jpg",
  "caption": "Look at this"
}
```

### Send location

```http
POST /api/chats/{jid}/messages
Content-Type: application/json

{
  "type": "location",
  "latitude": -6.2088,
  "longitude": 106.8456,
  "name": "Jakarta",
  "address": "Indonesia"
}
```

### Frontend

- Composer attachment button (image, doc, audio, video).
- Location picker or manual lat/long input.
- Preview thumbnails for uploaded media.

## Data model changes

- Extend `Message` enum/struct:
  - `MessageContent::Text { body }`
  - `MessageContent::Media { mime_type, url, caption, filename }`
  - `MessageContent::Location { latitude, longitude, name, address }`

## Dependencies

- `whatsapp-rust` media send APIs.
- File storage: local disk or optional object storage (start with local `media/` directory).

## Implementation notes

- Validate file size and MIME type.
- Save uploaded files under `media/{year}/{month}/{uuid}` and serve via a static route or signed URL.
- The engine crate wraps `whatsapp-rust` media sending; server only passes bytes or URL.
- For incoming media, see Feature 06 (Autoreply) `Download File` and Feature 11 (Integrations) webhook.

## Acceptance criteria

- [ ] Can send image, document, audio and video messages.
- [ ] Can send a location message.
- [ ] Media uploads are validated and stored safely.
- [ ] Incoming media metadata is forwarded over `/ws`.
- [ ] `cargo build --workspace` and `cargo clippy --workspace` pass.
