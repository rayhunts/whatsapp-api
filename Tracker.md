# Feature Tracker — WhatsApp API (Fonnte-inspired)

This tracker replaces the old `PLAN.md` MVP-only scope with a feature-by-feature roadmap inspired by [Fonnte](https://fonnte.com/#fitur). Each feature lives in its own markdown file so work can be planned, assigned and closed independently.

## How to use this tracker

1. Pick a feature file below.
2. Update its status and progress as you work.
3. When a feature is complete, tick it here and link the relevant PR/commit.

---

## Phase 0 — Foundation (current codebase)

These already exist from `PLAN.md` and are the baseline for everything else.

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 0.1 | Workspace + Clean Architecture | ✅ Done | `PLAN.md` | Six crates, `domain/`/`application/`/`infrastructure/` |
| 0.2 | `whatsapp-rust` engine wrapper | ✅ Done | `whatsapp-api-engine/` | `WaEngine` trait, event bus |
| 0.3 | REST + WebSocket server | ✅ Done | `whatsapp-api-server/` | Axum, health, chats, messages |
| 0.4 | Dioxus WASM web client | ✅ Done | `whatsapp-api-web/` | QR login, chat view, live messages |
| 0.5 | Send single text message | ✅ Done | `Feature-01-Core-Messaging.md` | `POST /api/chats/{jid}/messages` |

---

## Phase 1 — Core Messaging

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 1.1 | Single text message (polish) | 🚧 In Progress | `Feature-01-Core-Messaging.md` | Validate JID, delivery status, errors |
| 1.2 | Broadcast to many numbers | ⏳ Planned | `Feature-02-Broadcast-and-CSV.md` | API + UI upload |
| 1.3 | CSV broadcast | ⏳ Planned | `Feature-02-Broadcast-and-CSV.md` | Upload CSV, map columns |
| 1.4 | Media / attachment messages | ⏳ Planned | `Feature-04-Media-and-Location.md` | Image, document, audio, video |
| 1.5 | Location messages | ⏳ Planned | `Feature-04-Media-and-Location.md` | Lat/long + place name |

---

## Phase 2 — Scheduling & Automation

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 2.1 | Scheduled messages | ⏳ Planned | `Feature-03-Scheduled-and-Recurring-Messages.md` | One-time future send |
| 2.2 | Recurring messages | ⏳ Planned | `Feature-03-Scheduled-and-Recurring-Messages.md` | Cron-like repeats |
| 2.3 | Follow-up messages | ⏳ Planned | `Feature-09-Sending-Strategies.md` | Delayed after an event |
| 2.4 | Random delay | ⏳ Planned | `Feature-09-Sending-Strategies.md` | Anti-spam throttling |

---

## Phase 3 — Templates, Variables & Personalization

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 3.1 | Message templates | ⏳ Planned | `Feature-05-Templates-and-Variables.md` | Save/reuse templates |
| 3.2 | Variable substitution | ⏳ Planned | `Feature-05-Templates-and-Variables.md` | `{{name}}`, `{{order_id}}`, etc. |
| 3.3 | Button messages (deprecated on WA) | ⏳ Planned | `Feature-05-Templates-and-Variables.md` | Keep optional/low priority |

---

## Phase 4 — Autoreply & Chatbot

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 4.1 | Default autoreply | ⏳ Planned | `Feature-06-Autoreply.md` | Fallback for unmatched messages |
| 4.2 | Keyword-based autoreply | ⏳ Planned | `Feature-06-Autoreply.md` | Exact/regex match |
| 4.3 | Webhook autoreply | ⏳ Planned | `Feature-06-Autoreply.md` | Forward incoming msg to URL, reply with response |
| 4.4 | Submission / chained questions | ⏳ Planned | `Feature-06-Autoreply.md` | Multi-step data collection |
| 4.5 | Quick reply templates | ⏳ Planned | `Feature-06-Autoreply.md` | One-tap replies in UI |

---

## Phase 5 — Contacts & Groups

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 5.1 | Save contacts | ⏳ Planned | `Feature-07-Contacts-and-Groups.md` | CRUD contact book |
| 5.2 | Contact groups / labels | ⏳ Planned | `Feature-07-Contacts-and-Groups.md` | For targeted broadcast |
| 5.3 | WhatsApp group support | ⏳ Planned | `Feature-07-Contacts-and-Groups.md` | Send to `@g.us` JIDs |

---

## Phase 6 — Device Management

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 6.1 | Multi-device per account | ⏳ Planned | `Feature-08-Devices-and-Session-Management.md` | One account, many WA devices |
| 6.2 | Random device routing | ⏳ Planned | `Feature-08-Devices-and-Session-Management.md` | Round-robin / random sender |
| 6.3 | Session status & reconnect | ⏳ Planned | `Feature-08-Devices-and-Session-Management.md` | Online/offline per device |

---

## Phase 7 — History, Status & Billing

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 7.1 | Message history / log | ⏳ Planned | `Feature-10-Message-History-and-Status.md` | All sent/received messages |
| 7.2 | Real-time delivery status | ⏳ Planned | `Feature-10-Message-History-and-Status.md` | Pending, sent, delivered, read, failed |
| 7.3 | Invoice / usage log | ⏳ Planned | `Feature-10-Message-History-and-Status.md` | Low priority; mostly dashboard UI |

---

## Phase 8 — Integrations

| # | Feature | Status | File | Notes |
|---|---------|--------|------|-------|
| 8.1 | Simple REST API docs | ⏳ Planned | `Feature-11-Integrations.md` | OpenAPI/Swagger |
| 8.2 | Webhook outgoing | ⏳ Planned | `Feature-11-Integrations.md` | Event-driven HTTP callbacks |
| 8.3 | Third-party form integrations | ⏳ Planned | `Feature-11-Integrations.md` | Google Forms, CF7, WooCommerce, etc. |

---

## Legend

| Icon | Meaning |
|------|---------|
| ✅ | Done |
| 🚧 | In Progress |
| ⏳ | Planned |
| 🛑 | Blocked |
| ❌ | Won't do / deprecated |
