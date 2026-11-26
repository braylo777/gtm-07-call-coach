# gtm-07-call-coach

**GTM Engineering — Stage 7 of 12: AI Sales Enablement & Coaching**

Turn sales calls into coaching and battlecards: vector search over transcripts, per-call metrics with coaching flags and rep trends, and competitor battlecards built from real objections and playbook answers.

**Live demo:** https://gtm-07-call-coach.braylo777.workers.dev — the Rust core runs in your browser as WebAssembly. No server, no tracking.

## Why this matters

Every sales team records its calls, and almost nobody listens to them. The value is in three questions. Where did a buyer say X? Is this rep getting better? What do we say when a competitor comes up? An LLM can draft the answers, but the facts must come from the transcript, with timestamps, so a manager can check them in ten seconds. This repo is the retrieval and metrics layer an LLM summary would stand on.

## What it does

- TF-IDF vector index over utterance windows, cosine ranking, overlap suppression, call / speaker / timestamp on each hit
- Per-call coaching: rep talk %, rep questions, longest monologue in seconds, topic hits with timestamps, next steps found
- Coaching flags against targets (talk ratio, questions, monologue, no next step) and a 0–100 call score
- Rep trend across calls: talk % and questions first vs last call, improving yes / no
- Battlecards: competitor aliases, mentions, win rate when mentioned, objections in the same or next utterance, playbook response and proof, verbatim quotes

## What it does not do (yet)

- No Gong / Chorus API connection: transcripts are synthetic JSON; competitor names are fictional
- Vectors are TF-IDF, not neural embeddings; the index is rebuilt per request
- No LLM summarization call: the outputs here are the cited facts a summary prompt would use

## Operations

Every operation takes JSON in and gives JSON out. The CLI, the tests and the demo page all use the same entry point.

| Operation | What it does |
|---|---|
| `search_calls` | Search three synthetic calls for a phrase. Each hit has the call, timestamp and two utterances of context. |
| `coach_calls` | Measure each call against coaching targets and show the rep's trend from first call to last. |
| `battlecards` | Build one battlecard per competitor from the calls, with the objections buyers raised and the playbook answer. |

## Run it

You need Rust (stable). No Node, no npm.

```bash
git clone https://github.com/braylo777/gtm-07-call-coach
cd gtm-07-call-coach
cargo test
cargo run -- ops
```

Try the first demo example from the command line:

```bash
python3 -c 'import json;print(json.dumps(json.load(open("web/examples.json"))["ops"][0]["input"]))' | cargo run -q -- search_calls
```

## Build and deploy the demo

```bash
./build.sh    # tests, then compiles the core to web/app.wasm
./deploy.sh   # serves web/ as a Cloudflare Workers static-assets site (needs a Cloudflare login)
```

`web/` is a plain static site (`index.html`, `app.js`, `app.wasm`). It has no build step, so any static host works.

## Layout

```
src/lib.rs        operation list and router
src/api.rs        JSON entry point + the 3 WASM exports (alloc, dealloc, run)
src/main.rs       CLI
src/*.rs          one module per capability, each with unit tests
tests/            integration tests (every demo example must pass)
web/              demo page; examples.json drives both the page and this README
```

## Design choices

- **Rust, few dependencies.** Only `serde_json`, pinned to an exact version with a committed `Cargo.lock`.
- **Deterministic.** Time, randomness and keys are inputs, so every result can be replayed in a test or an audit.
- **Fail closed.** Unknown input or missing fields give an error, never a silent default.

## Part of a series

This is one of 12 repos that work through a public Go-To-Market (GTM) Engineering roadmap: [12 stages by Suraj Sharma (@suraj_sharma14)](https://x.com/suraj_sharma14/status/2125552583452270692). The roadmap is the inspiration; the code here is original. All 12: [github.com/braylo777?q=gtm-](https://github.com/braylo777?tab=repositories&q=gtm-).

## Data and privacy

Every example uses synthetic data. Nothing calls a paid API, scrapes a site, identifies a real visitor or sends an email.

## License

MIT — see [LICENSE](LICENSE). Built by [Braylo SI](https://braylo.si).
