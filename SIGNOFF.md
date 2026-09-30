# Signing Out — Day 30/30 ✅

**Emmanuel Phiri · September 30, 2026 · 30 Days, 30 Services Challenge**

Thirty days ago I set out to build and ship one service every day. Today the
count reads **30/30**. This file signs out the challenge from inside its final
build: AaaS v2, the hardened Rust identity provider that now guards the other
twenty-nine.

## The journey

| Days | Stretch | Repos |
|---|---|---|
| 1–4 | Foundations: auth, notify, payments, files | `AaaS`, `Halla`, `Dobadoba`, `FuM` |
| 5–13 | Platform services: gateway, mail, flags, geo, FX, search, reviews, idempotency, audit | `Kachale`, `Kuemail`, `Ffs`, `Paja`, `CfxcS`, `SaaS`, `Maganizo`, `Idk`, `Mbiri` |
| 14–22 | Workers: PDFs, queues, images, addresses, QR, slugs, barcodes, sheets, speech | `Pgi`, `Gobo`, `Sharp`, `Padoor`, `Kode`, `Kalibho`, `B-Pls`, `Konza`, `Tts-Stt` |
| 23–28 | Polyglot stretch: Python, Rust, Go | `Uthenga`, `PgsC`, `Chikalata`, `Sr-gA`, `Mtundu`, `Khomo` |
| 29 | The arena itself | `30days-30projects` |
| 30 | Capstone rebuild (this repo) | `AaaS-v2` — **you are here** |

Full verified map with languages and purposes: `AaaS-v2.md` §14. Every repo
name above was verified live against `github.com/Normious` on Sep 30 — one
repo created per day, Sep 1–29, no gaps.

## By the numbers

- **30** services shipped in **30** days, **4** languages (TypeScript/Node,
  Python, Go, Rust), ~50,000+ lines
- **AaaS v2** (Day 30): 2,870 lines across 35 files, `cargo test` 4/4, full
  live E2E (register → 2FA → lockout → theft detection), re-verified inside
  Docker, 31 MB distroless image
- **Docs**: full spec, 6 guide pages, 11-stop CodeTour, interactive
  architecture diagram (showcase 9/9, browser-checked at 4 viewports)

## Why end on a rewrite

Rule #10 allows building on an earlier day only as a genuinely new,
more complete implementation. Day 1's AaaS proved the idea in TypeScript;
Day 30's AaaS v2 earns the rewrite in Rust: Argon2id over PBKDF2, Ed25519
over HS256, rotation with theft detection, TOTP, lockout, audit. Same API
contract — every sibling can point at it without changes.

## Open items (post-challenge)

1. Create `Normious/AaaS-v2` on GitHub and push this build (code, Dockerfile,
   compose, docs, diagram, SIGNOFF).
2. Point siblings at it: replace shared-secret checks with the Ed25519
   public key; keep `GET /auth/verify` as fallback.
3. Rotate the dev JWT keys used during local testing before any real users.

## Thank you

To everyone who starred, forked, filed issues, or just lurked: this portfolio
exists because shipping daily forces decisions, and decisions compound. The
services are small on purpose. The discipline was the point.

**30/30. Build. Ship. Repeat.** 🚀

— Emmanuel
