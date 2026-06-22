# Before We Build: The Swarm-OS Pre-Flight Checklist

> Everything that needs to happen before Phase 0 implementation begins — and the day-by-day roadmap for Phase 0 itself. Written for the founder, not the engineer.

---

The Swarm-OS repository now contains a complete plan: system architecture, technology stack, UI specifications, governance model, credit economy, and a research report that tested every open-source dependency we intend to use. That plan went through two rounds of brutal critique. The critique found 28 issues. The research validated some assumptions and overturned others.

Six of those issues and three security fixes must be resolved in the planning documents before anyone writes implementation code. This guide explains each one in plain language, then lays out a day-by-day roadmap for Phase 0 — your first working product.

Think of it this way: these are the foundation inspections before breaking ground on a building. Skip them, and the building looks fine until the third floor. Then cracks appear.

---

# Part 1: Before You Write a Single Line of Code

---

## Chapter 1: The Sharding Problem — How Computers Share a Brain

### What This Is

Swarm-OS promises that multiple computers can work together to run one AI model that is too large for any single machine. A 70-billion-parameter model needs about 40 GB of GPU memory. Most gaming PCs have 8-24 GB. The solution is to split the model into slices — called "shards" — and give each computer a few layers.

Computer A processes the first layers, passes the intermediate result to Computer B, which processes the next layers, and so on. Like an assembly line in a garment factory: one station cuts the fabric, the next sews the seams, the next adds buttons. Each station only needs the tools for its job, and the garment moves down the line.

### Why It Matters

No existing open-source library does this the way we need:

- **exo** (our design reference) is licensed under GPL-3.0. We legally cannot copy its code into our Apache-2.0 project. We can study how it works, but we must write our own version from scratch.
- **distributed-llama** was previously recommended as a replacement. Our research report found it is fundamentally wrong for our use case. It uses a different approach called "matrix-parallel" — imagine splitting each page of a book into strips and giving one strip to each reader, instead of giving each reader complete chapters. This requires the readers to constantly compare notes at every single page (an "all-reduce" synchronization step), adding nearly one full second of delay per computation step over regular internet. It also requires exactly 2, 4, 8, or 16 computers — not 3, not 5, not 7. And it uses a custom file format that is incompatible with the standard GGUF format everyone else uses.
- **Petals** (an academic project from BigScience) validated the assembly-line approach at production scale and added a clever trick: when a worker drops out mid-job, the system can reroute work to a replacement worker without starting over from scratch. We want to study and adapt this for Phase 2.

### The Analogy

Imagine translating a 400-page book. You cannot just give 100 pages to each of four translators independently — each chapter builds on context from earlier chapters. Instead, Translator A reads and translates chapters 1-10, then passes a summary of the story so far to Translator B, who does chapters 11-20, and so on.

That "summary" is the activation tensor — the intermediate result that moves between computers. Designing how to efficiently pass that summary is the core engineering challenge.

### The Numbers

Our research measured the actual size of these activation tensors for a 70-billion-parameter model:

| Scenario | Tensor Size Per Hop | Time at 50 Mbps (BD broadband) |
|----------|-------------------|-------------------------------|
| Generating tokens one at a time (decode) | ~16 KiB | 2.6 ms — negligible |
| Processing the initial question (prefill) | ~32 MiB | 5.12 seconds — far too slow |

Three hops on a wide-area network = over 15 seconds just for the prefill phase. That is completely unusable. But on a fast local network (1 Gbps, like a university lab), the same three hops take only 768 milliseconds — perfectly acceptable.

### What "Done" Looks Like

A written design document describing the Rust pipeline-parallel architecture. A working prototype that transfers one activation tensor between two computers and gets the correct result out the other end. This is a Phase 2 deliverable — Phase 0 is single-device only, which is the correct starting point.

**Status: Design decision made. Implementation deferred to Phase 2. No blocker for Phase 0.**

---

## Chapter 2: Don't Put the House Key Under the Welcome Mat

### What This Is

When a user sends an AI request through Swarm-OS, the system creates a "job" and writes it to etcd — the shared coordination database that all nodes can read. The original design stored the user's API key inside that job record. Every node operator with access to etcd could read every user's permanent API key. No hacking required. Just reading what is openly visible.

### Why It Matters

If a malicious node operator reads a user's API key, they can impersonate that user, drain their credits, or use the service for free. This would destroy trust the moment anyone looks at the code. It is like writing your bank PIN on the whiteboard in a shared office.

### The Analogy

Think of a restaurant kitchen with an order board. Currently, the order slip says: "Table 7, chicken biryani, and here is the customer's credit card number." The fix: the order slip says "Table 7, chicken biryani, authorization token #4829 (valid for 60 seconds)." The kitchen only needs to know what to cook and that the order has been authorized. It never needs the customer's permanent credential.

### What "Done" Looks Like

The job record in etcd contains only: which model to use, a one-time authorization token (valid for 60 seconds, tied to that specific job), the maximum number of tokens to generate, and whether to stream the response. The API key field is completely absent. The architecture document has been updated to reflect this as a hard rule.

**Status: Design decision made. Reflected in architecture.md and CLAUDE.md.**

---

## Chapter 3: The Ledger's Ticking Time Bomb

### What This Is

The credit ledger is the financial heart of Swarm-OS. It records every credit earned by contributors and every credit spent by consumers. Each entry contains a cryptographic fingerprint of the previous entry, forming an unbreakable chain — like a chain of linked receipts. If anyone tampers with an old entry, the fingerprints stop matching, and the tampering is immediately detectable.

The original design stored this ledger in etcd, the same database used for real-time coordination. The problem: etcd has a feature called "auto-compaction" that permanently deletes old records to save disk space. When etcd deletes an old record, the chain of fingerprints breaks. Silently. No error. No warning. The ledger can no longer prove it has not been tampered with, and nobody knows.

### Why It Matters

The tamper-evident ledger is Swarm-OS's trust mechanism. Contributors trust they will be paid accurately because every entry is cryptographically signed and chained. If the chain silently breaks, the entire trust model collapses. A contributor could be underpaid, or a consumer overcharged, and there would be no way to detect or prove it.

### The Analogy

Imagine keeping your company's financial records in a filing cabinet that automatically shreds documents older than 30 days. Your accountant says "I can prove no one has tampered with the books." But the proof depends on comparing today's records to last month's records — which were shredded.

The fix: move the records to a proper safe (SQLite with write-ahead logging) that never deletes anything. Keep only a single note in the old filing cabinet that says "the latest record number is XYZ" so the coordination system can still reference it quickly.

### What "Done" Looks Like

The architecture document specifies SQLite in WAL (Write-Ahead Logging) mode as the ledger's storage, running on the orchestrator. The ledger table is append-only — the database itself prevents updates and deletes. etcd stores only a single pointer per node: the hash of the most recent entry. The full chain is verifiable from start to finish without depending on etcd's history.

**Status: Design decision made. Reflected in architecture.md and CLAUDE.md.**

---

## Chapter 4: Password-Grade Protection for API Keys

### What This Is

When a user creates an API key, we store a fingerprint of that key so we can verify it later when they use it. The original design used SHA-256, a well-known hashing algorithm. The problem: SHA-256 is designed to be fast. "Fast" is good for verifying files, but terrible for protecting secrets.

An attacker with a modern GPU can compute billions of SHA-256 hashes per second. If they get access to your database (even an old backup), they can try every possible key until they find a match. This is called a "rainbow table" attack, and against SHA-256, it works frighteningly well.

### Why It Matters

API keys are the gateway to user accounts and credits. If an attacker cracks the keys, they own every account. The difference between SHA-256 and proper password hashing is the difference between hours and years of attack time.

### The Analogy

SHA-256 is like a lock that a skilled lockpicker can open in one second. Argon2id is like a lock that forces the lockpicker to wait 3 seconds per attempt AND use 64 megabytes of expensive equipment for each try. Even if the lockpicker has thousands of copies of your lock, each one still takes 3 seconds and enormous resources to test. At billions of keys, this difference makes cracking mathematically impractical.

### What "Done" Looks Like

The security specification states: all API keys are hashed using Argon2id with these parameters:
- Time cost: 3 iterations
- Memory cost: 64 MB per hash
- Parallelism: 4 threads
- A unique random 32-byte salt per key

These are the OWASP 2025 minimum recommended values. The Rust crate `argon2` (from RustCrypto) is the implementation. The research report contains the exact code pattern.

**Status: Design decision made. Reflected in architecture.md, governance.md, and CLAUDE.md.**

---

## Chapter 5: Keeping Conversations Private

### What This Is

The original design stored user prompts — the actual questions and conversations people send to the AI — inside the job record in etcd. Every node operator could read every user's private conversations. Even if node operators are trusted today, the system should make it structurally impossible to snoop, not just trust people not to look.

### Why It Matters

Privacy is a non-negotiable expectation. If users discover that their prompts are readable by any node operator, they will never use the service. This is especially important under Bangladesh's Cyber Security Act 2023, which creates broad content liability. If prompts are stored in a shared database, the operator could be held legally responsible for their content.

### The Analogy

Currently, the system works like a shared intercom where everyone in the building can hear what you say to the delivery person. The fix: a private phone line between you and the delivery person. The building's intercom only says "delivery for apartment 7B." It never broadcasts what you ordered.

### What "Done" Looks Like

The job record in etcd contains only routing metadata: model ID, job token, maximum tokens, and stream flag. No `messages` field. No prompt content whatsoever. Prompts are transmitted directly over the encrypted WireGuard tunnel from the API gateway to the assigned node(s) only. The architecture document states this as a security invariant — a rule that can never be broken.

**Status: Design decision made. Reflected in architecture.md and CLAUDE.md.**

---

## Chapter 6: The Missing First Impression — Model Downloads

### What This Is

The onboarding flow detects your hardware and tells you which AI models you can run. But it never explains how to actually get those models onto your computer. A 7-billion-parameter model is about 4 GB. A 70-billion-parameter model is over 40 GB. Users need to download these files, and the experience needs a progress bar, a verification step (to make sure the file was not corrupted), and clear information about how much disk space is needed.

### Why It Matters

This is the single biggest gap in the user experience. Contributors cannot contribute to the swarm without a model loaded on their device. If they have to figure out on their own where to download a 4 GB AI model file and where to put it, most will give up within minutes.

### The Analogy

This is like selling a printer that does not come with a USB cable and does not tell you which cable to buy. The experience should be seamless: the app detects your hardware, shows you which models fit (with file sizes and memory requirements), lets you pick one, downloads it with a progress bar, verifies it was downloaded correctly, and then registers your node.

### What "Done" Looks Like

A designed flow that goes:

1. **Hardware detection** — your GPU, VRAM, RAM, CPU are identified
2. **Model browser** — shows eligible models with file size, memory requirement, and a "compatible with your device" indicator
3. **Download** — progress bar, estimated time, pause/resume capability
4. **Verification** — BLAKE3 hash check (takes under 2 seconds for a 4 GB file, versus 12 seconds with the older SHA-256 method)
5. **Registration** — your node cannot join the network without at least one verified model downloaded

**Status: Design decision made. UI wireframe needed in ui_ux.md. Implementation in Phase 0 Week 2.**

---

## Chapter 7: The Security Triple-Lock

This chapter consolidates the three security fixes that must be locked into the design before any code is written.

### Lock 1: API Keys Never Touch Shared Storage

As explained in Chapter 2, the user's permanent API key must never appear in etcd or any shared database. Only one-time job tokens are written to the coordination layer. This prevents credential exfiltration without any special security infrastructure — the credentials simply are not there to steal.

**Design effort:** 2 hours. **Implementation:** Phase 0-1.

### Lock 2: Argon2id for API Key Hashing

As explained in Chapter 4, all API keys are hashed with Argon2id (time=3, memory=64 MB, parallelism=4, per-key salt) instead of SHA-256. This makes stolen database backups useless to attackers.

**Design effort:** 1 hour. **Implementation:** Phase 0.

### Lock 3: Secrets Management

The third fix addresses how we store sensitive configuration values: database passwords, API keys for external services like SSLCommerz, headscale pre-authentication keys, and similar secrets.

Without a plan, these values will end up in `.env` files scattered across servers — a common and dangerous pattern. A `.env` file is just a plain text file. If it accidentally gets committed to the code repository (which happens more often than anyone admits), every secret in it is permanently exposed.

The design must specify which secrets management approach to use before coding begins. Options include HashiCorp Vault (self-hosted), cloud-native key managers, or at minimum a structured approach with environment-variable injection that keeps secrets out of the filesystem entirely.

**Design effort:** 1 day. **Implementation:** Phase 0-1.

---

## Chapter 8: What the Research Already Told Us

Before anyone writes implementation code, the plan calls for cloning every open-source dependency, running it locally, and extracting its actual behavior — API shapes, error codes, performance numbers, failure modes. This homework has been completed. The results are in `research.md` (over 60 pages of detailed findings).

Here is what we learned:

### The Verdicts

| Component | Verdict | What It Means |
|-----------|---------|---------------|
| **llama.cpp** | ADOPT | The AI inference engine. Battle-tested, runs on every platform. This is our core. |
| **llama-cpp-2** (Rust binding) | ADOPT | The translator between our Rust code and the AI engine. The only maintained option — the older `llama-rs` library has been archived and abandoned. |
| **distributed-llama** | AVOID | Wrong architecture for our use case. Like trying to split a car engine across three garages connected by dirt roads. |
| **Petals** | STUDY & ADAPT | Has a clever trick for recovering when a worker drops out. Worth adapting in Phase 2. |
| **Tauri v2** | ADOPT | The desktop app framework. 7-9 MB binary instead of Electron's 150 MB. |
| **LiteLLM** | ADAPT | The API gateway translator. Works, but its internal hooks break across versions. Pin to version 1.82.0 or higher. |
| **etcd** | ADOPT | The coordination database. Reliable lease heartbeats, good for real-time state tracking. |
| **headscale** | ADOPT | The mesh networking control plane. Handles the tricky NAT situations common with Bangladesh ISPs. |
| **iroh** | ADOPT (Phase 4) | For distributing AI model files between nodes. Faster than BitTorrent for our use case. |
| **SSLCommerz** | ADOPT | Bangladesh payment gateway. Supports bKash, Nagad, Visa, MasterCard. |

### The Numbers That Matter

| What | Number | Why It Matters |
|------|--------|---------------|
| 7B model GPU memory (warm) | 4.42 GiB | Need 5.20 GiB free to be safe |
| 70B model GPU memory (warm) | 43.12 GiB | Need 48 GiB free — requires multi-node sharding |
| Apple Metal context limit | 8,192 tokens | Crashes above this. We enforce the limit in software |
| RTX 4090 generation speed | 88.5 tokens/sec | High-end GPU benchmark |
| M3 Max generation speed | 48.2 tokens/sec | Apple Silicon benchmark |
| CPU-only generation speed | 8.1 tokens/sec | Usable but slow — for contributors with no GPU |
| Tauri app binary size | 7-9 MiB | With LTO optimization. Electron would be 150+ MiB |
| Tauri IPC latency | 1.42 ms (Mac), 2.10 ms (Win) | Fast enough for real-time token streaming |
| BLAKE3 file verification | 1.45 seconds for 4 GB | SHA-256 takes 11.75 seconds for the same file |
| WireGuard tunnel setup (CGNAT) | 1,200 ms direct, 3,500 ms relay | Under Bangladesh residential internet conditions |
| etcd lease expiry detection | ~150 ms after TTL | Fast enough for 10-second heartbeat monitoring |
| LiteLLM minimum version | 1.82.0+ | Older versions have breaking changes in callback hooks |

### What This Means

The research homework is done. Every Phase 0 dependency has been tested, its quirks documented, and its integration pattern recorded with exact code samples. There are no unknown risks remaining for Phase 0. The team can proceed to implementation with confidence.

---

# Part 2: Phase 0 — Your First Working Product

---

## Chapter 9: Four Weeks to a Working Prototype

Phase 0 delivers one thing: a desktop application that runs AI models locally on a single device, accessible via the standard OpenAI API format. No networking between devices. No shared compute pool. No credit economy. Just the core inference pipeline, polished and working.

Here is the day-by-day plan.

---

### Week 1: Laying the Foundation

**Goal:** A desktop app that can run a local AI model and stream responses.

**Day 1 — Development Environment**
Set up the toolchain on your development machine:
- Install Rust (version 1.78 or higher) and the Tauri CLI
- Install pnpm (Node.js package manager) for the React frontend
- Install cmake and clang (needed to compile llama.cpp's C++ code)
- Create the `rust-toolchain.toml` file pinning the Rust version (prevents Windows MSVC build issues)

*What you can see:* Running `cargo --version` and `pnpm --version` both succeed. The development environment is ready.

**Day 2 — App Scaffold**
Create the Tauri v2 application shell:
- Run `cargo create-tauri-app` with the React + TypeScript template
- Set up the project structure: `src-tauri/` for Rust backend, `src/` for React frontend
- Configure Cargo.toml with release optimizations (LTO, strip symbols, optimize for size)
- Verify the empty app launches on your machine — a blank window with a system tray icon

*What you can see:* A desktop window opens. The app icon appears in your system tray. Nothing useful yet, but the foundation is solid.

**Day 3 — First AI Response (Standalone)**
Before integrating with the app, verify that the AI engine works on its own:
- Download a small test model: Llama 3.2 1B (~1 GB, runs on almost any hardware)
- Run `llama-server -m model.gguf --port 8080` directly from the command line
- Send a test request using curl or a simple script
- Observe the response format — the SSE streaming events that the research report documented

*What you can see:* You type a question in the terminal, and the AI answers token by token. The engine works.

**Day 4 — Rust Integration**
Connect the AI engine to the Tauri app:
- Add the `llama-cpp-2` crate to the Rust dependencies
- Write a Tauri command that loads a GGUF model file and runs a single inference
- Handle errors gracefully — if the model file is missing, the GPU has insufficient memory, or the model format is wrong
- Test: call the command from the React frontend and display the response

*What you can see:* You type a question in the app window, click send, and the complete AI response appears. Not streaming yet — just the whole response at once.

**Day 5 — Real-Time Streaming**
Make tokens appear one at a time, like ChatGPT:
- Implement the Tauri IPC Channel: the Rust backend sends each token to the React frontend as it is generated
- The React side receives tokens via `Channel.onmessage` and appends them to the display
- Test with different prompt lengths to verify stability

*What you can see:* You type a question and watch the answer appear word by word in real time. The core experience works.

**What you have by Friday:** A desktop app that runs a local AI model and streams responses in real time. It is ugly, has no settings, and only works with one hardcoded model. But the most difficult integration (Rust ↔ llama.cpp ↔ React) is proven.

---

### Week 2: Making It Useful

**Goal:** The app knows your hardware, lets you download appropriate models, and handles edge cases.

**Day 6 — Hardware Detection**
Build the resource profiler:
- Use the `sysinfo` crate to detect CPU model, core count, and available RAM
- Use the `nvml-wrapper` crate to detect NVIDIA GPUs — model name, total VRAM, free VRAM
- For Apple Silicon, detect the Metal backend capabilities and the 75% memory limit
- Store the results in a structured format the UI can display

*What you can see:* A "System Info" panel in the app shows your exact hardware: "NVIDIA RTX 3060, 12 GB VRAM, 32 GB RAM, 8 CPU cores."

**Day 7 — Hardware Capabilities Display**
Turn raw numbers into actionable information:
- Calculate which models can run on this hardware using the VRAM headroom table from research
- Show a capability score: the same `(vram × 4) + (ram × 0.5) + (cpu × 0.25) + backend_bonus` formula from the architecture
- Display the backend type: CUDA, Metal, Vulkan, or CPU-only
- Show a clear message for low-end devices (4 GB RAM, no GPU): "Your device is better suited for using the API as a consumer"

*What you can see:* The app tells you: "Your hardware can run models up to 8B parameters. Recommended: Llama 3.1 8B (4.1 GB download, needs 5.2 GB VRAM)."

**Day 8 — Model Download UI**
Build the model browser and downloader:
- Display a list of available models with file size, memory requirement, and a "compatible" indicator
- Implement download with a progress bar, speed indicator, and estimated time remaining
- Allow pause and resume — Bangladesh internet connections can be unreliable
- Store downloaded models in `~/.swarm-os/models/`

*What you can see:* A model browser screen showing 4-5 models. Green checkmarks next to compatible ones, grey locks next to ones that need more hardware. A download progress bar.

**Day 9 — Model Verification and Selection**
Complete the model management experience:
- After download, verify the file with BLAKE3 hashing (~1.5 seconds for a 4 GB file)
- Show a "Verified" badge next to downloaded models
- Build a model selector dropdown — choose which model to load
- Implement load/unload: changing models releases the previous model's memory before loading the new one

*What you can see:* You download a model, see "Verified" appear almost instantly, select it from a dropdown, and start chatting with a different AI model.

**Day 10 — Error Handling**
Make the app robust against real-world problems:
- VRAM exhaustion: if the model needs more memory than available, show a friendly message suggesting a smaller model or quantization
- Metal 8K context limit: on Apple Silicon, enforce the 8,192-token context limit to prevent crashes
- Download corruption: if BLAKE3 verification fails, offer to re-download
- Disk space check: warn before downloading if insufficient space

*What you can see:* You try to load a model that is too large for your GPU. Instead of crashing, the app says: "This model needs 6 GB of GPU memory, but you have 4 GB free. Try Llama 3.2 3B instead."

**What you have by Friday:** The app knows your hardware, lets you browse and download AI models appropriate for your device, verifies downloads, and handles errors gracefully. A non-technical user could set this up.

---

### Week 3: The API Gateway

**Goal:** Any application that uses the OpenAI SDK can talk to your local AI model with zero code changes.

**Day 11 — LiteLLM Proxy Setup**
Set up the API translation layer:
- Install LiteLLM (Python, pinned to version 1.82.0 or higher)
- Configure it as a proxy server on port 4000
- Point it at the local llama.cpp server running inside the Tauri app
- Verify the proxy starts and responds to health checks

*What you can see:* A second process running alongside the Tauri app. Visiting `http://localhost:4000/health` returns "OK."

**Day 12 — Custom Swarm Provider**
Wire up the routing between LiteLLM and the local inference engine:
- Implement the `SwarmRouterCallback` using LiteLLM's `CustomLogger` pattern
- Configure model name mapping: when a user requests "llama-3.1-8b", route to the local llama.cpp instance
- Handle the case where the model is not loaded — return a clear error instead of crashing

*What you can see:* Sending a request to `http://localhost:4000/v1/models` returns the list of models available on your device.

**Day 13 — OpenAI SDK Compatibility**
The moment of truth — prove that standard tools work:
- Install the OpenAI Python SDK (`pip install openai`)
- Point it at your local proxy: `base_url="http://localhost:4000/v1"`
- Run `client.chat.completions.create()` with a test prompt
- Verify the response format matches what the OpenAI SDK expects

*What you can see:* A Python script that looks identical to one that calls ChatGPT — except it runs entirely on your machine. No internet required. No API fees.

**Day 14 — Usage Tracking**
Prepare for the credit economy (Phase 1) by tracking usage now:
- Wire up LiteLLM's success callback to count tokens per request
- Log: input tokens, output tokens, model used, response time
- Store locally for now — this data feeds into the credit ledger in Phase 1

*What you can see:* After each request, the app logs "34 input tokens, 128 output tokens, 1.4 seconds, llama-3.1-8b."

**Day 15 — End-to-End Streaming**
Complete the API streaming experience:
- Verify SSE (Server-Sent Events) streaming works through the full chain: client → LiteLLM → llama.cpp → back
- Test with a long response (500+ tokens) to verify stability
- Measure latency: time from request to first token, and sustained tokens per second
- Compare numbers against the research benchmarks to ensure nothing is unexpectedly slow

*What you can see:* A streaming API call that returns tokens one at a time, just like the ChatGPT API. First token appears in under 2 seconds on a decent GPU.

**What you have by Friday:** Any application built for the OpenAI API works with your local AI model. Developers can switch from ChatGPT to Swarm-OS by changing one line of code — the base URL.

---

### Week 4: Polish and Ship

**Goal:** A shippable desktop application ready for testers.

**Day 16 — System Tray**
Build the persistent desktop presence:
- System tray icon with a context menu: Show/Hide window, Start/Stop inference engine, Quit
- Status indicator: green (running), yellow (loading model), grey (stopped)
- Click the tray icon to toggle the main window visibility
- The app continues running in the background when the window is closed

*What you can see:* Close the app window. The tray icon stays green. Send an API request from another application. It still works — the AI engine runs in the background.

**Day 17 — Live Stats and Resource Controls**
Give users visibility and control:
- Display live statistics: tokens generated today, current inference speed, model loaded
- Resource throttle slider: "Use 25% / 50% / 75% / 100% of GPU" — this prepares for Phase 1 when users will share their compute
- Show GPU temperature and memory usage in real time

*What you can see:* A dashboard showing "1,247 tokens generated today | 45.2 tok/sec | Llama 3.1 8B loaded | GPU: 62C, 4.1/12.0 GB VRAM."

**Day 18 — Build Optimization**
Prepare release-quality binaries:
- Enable Link-Time Optimization (LTO) and symbol stripping in Cargo.toml
- Target binary size: 7-9 MiB (versus 25-30 MiB without optimization)
- Create `rust-toolchain.toml` pinning Rust version for Windows MSVC compatibility
- Build for all three platforms: macOS (Apple Silicon), Windows (x64), Linux (x64 .deb package)

*What you can see:* Three installer files ready for distribution. The macOS app is 8.1 MiB. The Windows installer is 9.4 MiB. The Linux .deb is 7.1 MiB.

**Day 19 — Cross-Platform Testing**
Verify the app works everywhere:
- Test on macOS: install, download a model, run inference, verify Metal backend context limit
- Test on Windows: install, verify MSVC build succeeds, test CUDA backend
- Test on Linux: install the .deb, test both CUDA and CPU-only modes
- Document any platform-specific quirks

*What you can see:* The app installs and runs correctly on all three operating systems. Screenshots of each platform for the README.

**Day 20 — End-to-End Acceptance Test**
The final validation — pretend you are a new user:
- Start with a clean machine (or a fresh VM)
- Install the app from the binary
- Go through the full flow: launch → hardware detection → browse models → download → verify → start → send API request → get streaming response
- Update the README with a "Quick Start" section showing the experience
- Record a 60-second screen capture of the full flow for marketing

*What you can see:* A complete, working product. A new user can go from zero to their first AI response in under 5 minutes (plus model download time).

**What you have by Friday:** Phase 0 is complete.

---

## Chapter 10: What You'll Have When Phase 0 Is Done

### The Experience

A user downloads a small desktop application — about 8 MB. They install it. The app detects their hardware: "You have an NVIDIA RTX 3060 with 12 GB VRAM, 32 GB RAM, 8 CPU cores." It shows them compatible models: "You can run Llama 3.1 8B (4.1 GB download, needs 5.2 GB VRAM)." They click download, see a progress bar, and the model is verified in under 2 seconds.

They click "Start." The app's tray icon turns green.

Now, from any Python script, JavaScript app, or command-line tool, they can call the API:

```
POST http://localhost:4000/v1/chat/completions
{"model": "llama-3.1-8b", "messages": [{"role": "user", "content": "Hello from Bangladesh!"}]}
```

And get back a streaming response, token by token, from an AI model running entirely on their own device.

### What It Is

- A polished desktop app (~8 MiB) that runs on Mac, Windows, and Linux
- Local AI inference with real models (7B-13B on most gaming PCs)
- An OpenAI-compatible API on localhost — any developer tool that works with ChatGPT works with this
- Hardware detection that knows exactly what your machine can handle
- A model downloader with progress bar and integrity verification
- System tray integration for background operation

### What It Is Not (Yet)

- No networking between devices — that comes in Phase 1
- No shared compute pool — that comes in Phase 1
- No credit economy — that comes in Phase 1
- No WireGuard mesh — that comes in Phase 1
- No etcd coordination — that comes in Phase 1
- No admin dashboard — that comes in Phase 3
- No bKash payments — that comes in Phase 4

### Success Criteria

| Metric | Target |
|--------|--------|
| App installs on macOS, Windows, Linux | All three platforms work |
| Hardware detection accuracy | Correctly identifies GPU, VRAM, RAM, CPU |
| Model download and verification | At least Llama 3.1 8B downloads and BLAKE3-verifies |
| Inference throughput | Within 10% of research benchmarks per platform |
| OpenAI SDK compatibility | `openai.ChatCompletion.create()` works unmodified |
| Streaming | Tokens appear one at a time, not as a single block |
| Binary size | Under 10 MiB with LTO + strip |
| System tray | Start/stop/status from tray; background operation works |

### Why This Matters

Phase 0 is the foundation everything else builds on. If the single-device experience is slow, buggy, or hard to set up, no amount of distributed systems engineering will save the product. Phase 0 must be polished enough that a developer installs it, runs one API call, and thinks: "This works. This is fast. I want more of this."

### What Comes Next

| Phase | Timeline | What Gets Added |
|-------|----------|----------------|
| **Phase 1** | Weeks 5-10 | Two devices share work. WireGuard mesh, etcd coordination, basic credit earning. |
| **Phase 2** | Weeks 11-18 | 5-20 devices in a pool. Model sharding on LAN, handling devices going offline. |
| **Phase 3** | Weeks 19-24 | Monitoring and management. Grafana dashboards, admin portal, ledger auditing. |
| **Phase 4** | Weeks 25-32 | Bangladesh launch. bKash payments, Bangla UI, public API at api.swarm-os.dev. |

---

# Appendices

---

## Appendix A: Glossary

| Term | Plain English |
|------|--------------|
| **GGUF** | The file format for AI models. Like .mp3 is for music, .gguf is for AI models that run on llama.cpp. |
| **VRAM** | Video RAM — the memory on your graphics card. AI models live here during inference. More VRAM = bigger models you can run. |
| **Quantization (Q4_K_M)** | Compressing an AI model to use less memory. Like converting a WAV file to MP3 — smaller, slightly lower quality, but usually indistinguishable. Q4_K_M is the most popular compression level. |
| **Tokens** | The units AI models think in. Roughly 1 token = 0.75 words in English. "Hello, how are you?" is about 6 tokens. |
| **Inference** | Running an AI model to generate a response. "Doing inference" = "asking the AI a question and getting an answer." |
| **TTL (Time To Live)** | An expiration timer. A TTL of 10 seconds means the record automatically disappears after 10 seconds unless refreshed. Used for heartbeat monitoring — if a node stops refreshing, it is assumed dead. |
| **etcd** | A distributed database designed for coordination. Swarm-OS uses it as a shared bulletin board where nodes post their status and pick up job assignments. |
| **SQLite WAL** | A lightweight database that uses Write-Ahead Logging — a technique that ensures data is never lost, even during power failures. Used for the credit ledger. |
| **SHA-256** | A one-way fingerprinting function. Give it any data, and it produces a unique 256-bit fingerprint. Used to verify file integrity and chain ledger entries. |
| **Argon2id** | A password hashing function designed to be deliberately slow and memory-hungry, making brute-force attacks impractical. The current industry standard (OWASP 2025). |
| **Ed25519** | A digital signature algorithm. Each node has a private key (secret) and a public key (shared). The node signs data with its private key; anyone can verify the signature with the public key. Like a wax seal on a letter. |
| **WireGuard** | A modern VPN protocol that creates encrypted tunnels between devices. All Swarm-OS inter-node traffic travels through WireGuard tunnels. |
| **DERP Relay** | A fallback relay server for when two devices cannot connect directly (common with Bangladesh ISPs that use CGNAT). Traffic goes: Device A → DERP server → Device B, adding latency but ensuring connectivity. |
| **CGNAT** | Carrier-Grade NAT — a technique ISPs use to share one public IP address among many customers. Common in Bangladesh. Makes direct device-to-device connections difficult without a relay. |
| **Pipeline Parallelism** | Splitting an AI model across multiple devices by layers, like an assembly line. Device A handles layers 1-10, Device B handles 11-20, etc. |
| **Activation Tensor** | The intermediate result passed between devices during pipeline-parallel inference. Like the partially-translated manuscript passed from one translator to the next. |
| **SSE (Server-Sent Events)** | A web standard for streaming data from server to client. Each AI token is sent as a separate SSE event, allowing real-time display. |
| **LTO (Link-Time Optimization)** | A compiler technique that optimizes the entire program at once instead of file by file, producing smaller and faster binaries. |
| **IPC (Inter-Process Communication)** | How different parts of an application talk to each other. In Tauri, IPC is how the Rust backend sends data to the React frontend. |
| **BLAKE3** | A very fast hashing algorithm. Used to verify downloaded model files are not corrupted. About 8x faster than SHA-256. |
| **Clean-Room Implementation** | Writing code based on understanding an algorithm's logic, without looking at or copying another project's source code. Required when the source project uses an incompatible license (like GPL-3.0). |

## Appendix B: Source Documents

| Document | What It Contains |
|----------|-----------------|
| [guide.md](./guide.md) | This document — the non-technical step-by-step guide you are reading |
| [research.md](./research.md) | Component research: benchmarks, API shapes, code samples, verdicts for all dependencies |
| [project.md](./project.md) | Product identity, features F1-F9, phase roadmap, market context, success metrics |
| [architecture.md](./architecture.md) | System diagram, Blackboard pattern, scheduler, model sharding, API flow, security, failure modes |
| [tech_stack.md](./tech_stack.md) | Open-source dependency list with licenses, dependency map, model support matrix, build toolchain |
| [ui_ux.md](./ui_ux.md) | Screen-by-screen UI specification for web portal, Tauri tray agent, Grafana dashboards |
| [governance.md](./governance.md) | Role hierarchy, admin portal, configuration schemas, ledger audit protocol, abuse prevention |
| [critique.md](./critique.md) | 28 issues severity-ranked with fixes, consolidated pre-Phase 0 action plan |
| [verify-prompt.md](./verify-prompt.md) | Structured verification prompt for community validation of architecture decisions |

## Appendix C: The Numbers That Matter

### Performance

| Hardware | Generation Speed |
|----------|-----------------|
| NVIDIA RTX 4090 | 88.5 tokens/sec |
| Apple M3 Max | 48.2 tokens/sec |
| Intel i9-14900K (CPU only) | 8.1 tokens/sec |

### Memory Requirements (Q4_K_M Quantization)

| Model Size | VRAM Needed (Warm) | Safe Headroom |
|-----------|-------------------|---------------|
| 7B (Llama 3.1) | 4.42 GiB | 5.20 GiB |
| 13B (Llama 2) | 8.11 GiB | 9.50 GiB |
| 70B (Llama 3.1) | 43.12 GiB | 48.00 GiB |

### Model Load Times (NVMe SSD)

| Model | Cold Cache | Warm Cache |
|-------|-----------|------------|
| 7B Q4_K_M | 3.40 seconds | 0.85 seconds |
| 70B Q4_K_M | 38.50 seconds | 9.40 seconds |

### Safety Limits

| Limit | Value | Consequence of Exceeding |
|-------|-------|-------------------------|
| Apple Metal context window | 8,192 tokens | Crash (segmentation fault) |
| LiteLLM minimum version | 1.82.0 | Callback hooks may break silently |
| etcd value size limit | 1.5 MiB | Request rejected |

### Network (Bangladesh Conditions)

| Metric | Value |
|--------|-------|
| WireGuard tunnel setup (direct) | 1,200 ms |
| WireGuard tunnel setup (DERP relay) | 3,500 ms |
| 70B activation tensor (prefill) per hop | 32 MiB |
| Prefill hop time at 50 Mbps | 5.12 seconds |
| Prefill hop time at 1 Gbps LAN | 256 ms |

### Application

| Metric | Value |
|--------|-------|
| Tauri binary size (optimized) | 7.1-9.4 MiB |
| Tauri IPC roundtrip (macOS) | 1.42 ms |
| Tauri IPC roundtrip (Windows) | 2.10 ms |
| BLAKE3 verification (4 GB file) | 1.45 seconds |
| SHA-256 verification (4 GB file) | 11.75 seconds |
