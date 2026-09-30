# 30 Days, 30 Services — The Complete Showcase 🚀

**By [Emmanuel Phiri](https://github.com/Normious) · September 2026**

One service shipped every single day for a month: identity, payments,
notifications, storage, search, PDFs, jobs, images, geo, currency, gateways —
across TypeScript, Python, Go, and Rust. Everything below is open source.
Browse it, run it, break it, improve it.

⭐ **If anything here helps or inspires you, star the repo** — stars are what
keep a streak like this visible. Thank you.

## The 30

| Day | Project | What it is |
|---|---|---|
| 01 | [AaaS](https://github.com/Normious/AaaS) | Auth-as-a-Service: JWT auth microservice, the original gatekeeper (Hono + D1) |
| 02 | [Halla](https://github.com/Normious/Halla) | WhatsApp notifier API with true multi-session support (Express + Baileys) |
| 03 | [Dobadoba](https://github.com/Normious/Dobadoba) | Multi-tenant PayChangu payment gateway (Hono + D1) |
| 04 | [FuM](https://github.com/Normious/FuM) | File Upload Manager: R2 storage with auto thumbnails and on-the-fly resizing |
| 05 | [Kachale](https://github.com/Normious/Kachale) | Rate limiter and API gateway for the whole ecosystem |
| 06 | [Kuemail](https://github.com/Normious/kuemail) | Transactional email service (Fastify + Resend) |
| 07 | [Ffs](https://github.com/Normious/Ffs) | Feature flag service — *Mphamvu*, power/authority (Hono + D1) |
| 08 | [Paja](https://github.com/Normious/Paja) | Geocoding and places: Nominatim, OSRM routes, Photon |
| 09 | [CfxcS](https://github.com/Normious/CfxcS) | FX conversion: single source of truth for currency rates (Hono + D1 + KV) |
| 10 | [SaaS](https://github.com/Normious/SaaS) | Search-as-a-Service: full-text search (Fastify + Meilisearch) |
| 11 | [Maganizo](https://github.com/Normious/Maganizo) | Review and rating engine (Hono + D1) |
| 12 | [Idk](https://github.com/Normious/Idk) | Idempotency-key service: exactly-once execution for microservices |
| 13 | [Mbiri](https://github.com/Normious/Mbiri) | Audit-log service: the history keeper |
| 14 | [Pgi](https://github.com/Normious/Pgi) | PDF generation from JSON + HTML templates (Puppeteer) |
| 15 | [Gobo](https://github.com/Normious/Gobo) | Queue and job dispatcher for async work (BullMQ + Redis) |
| 16 | [Sharp](https://github.com/Normious/Sharp) | Image optimizer service |
| 17 | [Padoor](https://github.com/Normious/Padoor) | Address autocomplete and reverse-geocoding |
| 18 | [Kode](https://github.com/Normious/Kode) | QR generation and scanning engine |
| 19 | [Kalibho](https://github.com/Normious/Kalibho) | Slug generation with collision checking |
| 20 | [B-Pls](https://github.com/Normious/B-Pls) | Barcode-to-product lookup |
| 21 | [Konza](https://github.com/Normious/Konza) | Spreadsheet converter: CSV/Excel in, typed JSON out |
| 22 | [Tts-Stt](https://github.com/Normious/Tts-Stt) | Speech synthesis and recognition (Edge-TTS + Whisper) |
| 23 | [Uthenga](https://github.com/Normious/Uthenga) | Link unfurler: give it a URL, get a card (FastAPI) |
| 24 | [PgsC](https://github.com/Normious/PgsC) | Password Service: crypto-grade secrets, PINs, API keys (Rust + Axum) |
| 25 | [Chikalata](https://github.com/Normious/Chikalata) | PDF text extraction to structured JSON (FastAPI) |
| 26 | [Sr-gA](https://github.com/Normious/Sr-gA) | Sitemap and robots.txt generator (Go, 10 MB scratch image) |
| 27 | [Mtundu](https://github.com/Normious/Mtundu) | Image palette extraction: dominant colors as a service (Rust + Axum) |
| 28 | [Khomo](https://github.com/Normious/Khomo) | Mini API gateway: one endpoint, one CORS policy, one audit trail (Go) |
| 29 | [30days-30projects](https://github.com/Normious/30days-30projects) | The challenge platform itself: join, submit, get a portfolio page (Next.js + Supabase) |
| 30 | [AaaS-v2](https://github.com/Normious/AaaS-v2) | Hardened identity provider: Argon2id, Ed25519 JWTs, rotation + theft detection, TOTP (Rust — **you are here**) |

Names and purposes verified against every repo's README on Sep 30, 2026.

## Pick your adventure

- **New to the ecosystem?** Start with [AaaS-v2](https://github.com/Normious/AaaS-v2) (this repo): it guards everything else, and its [CodeTour](.tours/new-joiner-aaas-v2.tour) walks you through real code in 11 stops.
- **Frontend dev?** `30days-30projects`, `SaaS`, `Paja` are the friendliest doors in.
- **Rust-curious?** `PgsC` (small), `Mtundu` (medium), `AaaS-v2` (full system) — a graded path.
- **Go-curious?** `Sr-gA` then `Khomo`. Both ship as ~10 MB scratch images.

## Contributions welcome 🤝

This is a real, lived-in codebase collection — which means there are real
rough edges with your name on them:

1. **Pick a repo** from the table above (or the [full profile](https://github.com/Normious?tab=repositories)).
2. **Open an issue first** for anything beyond a typo fix — a sentence on what you want to change and why.
3. **Send the PR** against `main`: small, focused, with a before/after note. Tests green (`cargo test`, `pnpm test`, whatever that repo runs).
4. Ideas with no code yet? Open an issue tagged as a proposal — Day 31 and beyond is unwritten.

Bug reports are contributions too: a failing `curl` command pasted into an
issue is worth more than a vague "it doesn't work". Be kind in review;
every repo here was built in a day, and maintained with care anyway.

## Star the streak ⭐

Thirty days of shipping is easy to start and hard to finish. If this
collection taught you something, saved you an hour, or just made you smile —
**star the repos that earned it**, share the table with someone learning to
build, and come back for whatever ships next.

**Build. Ship. Repeat.** 🚀
