# Tunic — PROJECT.md

> **Tagline**: Beautiful Addresses, Made for You
> **Version**: V1 (MVP)
> **Status**: In Development

---

## 1. Project Summary

Tunic is a fully client-side web tool that generates vanity Ethereum (EVM) wallet addresses. Users input a custom pattern (prefix or suffix), and Tunic brute-forces keypairs until it finds an address that matches. The private key and matching address are displayed once to the user and never sent anywhere.

There is no server, no backend, no database, and no network requests during the generation process.

---

## 2. Goals

- Generate vanity EVM addresses based on user-defined prefix or suffix pattern
- Keep all cryptographic operations fully client-side inside the browser
- Never expose, transmit, or store the private key outside of the user's browser session
- Ship as a static web app deployable on Vercel or Cloudflare Pages

---

## 3. Non-Goals (V1)

The following are explicitly out of scope for V1:

- Smart contract wallet (ERC-4337) — planned for V2
- WebAuthn / biometric signer — planned for V2
- WalletConnect / DeFi connectivity — planned for V3
- Backend API or server of any kind
- User authentication or accounts
- Storing generated addresses or history
- Mobile app or browser extension

---

## 4. Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| Vanity engine | Rust (compiled to WASM) | Core brute-force logic, keypair generation, pattern matching |
| WASM bridge | wasm-bindgen | Exposes Rust functions to JavaScript |
| Threading | Web Worker | Runs brute-force in background thread, keeps UI responsive |
| Frontend framework | Next.js (TypeScript) | Static export mode, no SSR needed |
| Styling | Tailwind CSS | Utility-first, no component library |
| Hosting | Vercel or Cloudflare Pages | Static files only |

---

## 5. Project Structure

```
tunic/
├── engine/                             # Rust crate, compiled to WASM via wasm-pack
│   ├── src/
│   │   ├── lib.rs                    # Entry point, exposes generate_vanity() to JS
│   │   ├── generator.rs              # Brute-force loop: generate keypair, check pattern
│   │   ├── matcher.rs                # Pattern matching logic (prefix and suffix)
│   │   └── crypto.rs                 # secp256k1 keypair generation, keccak256 hashing
│   ├── tests/
│   │   └── integration_test.rs       # Test correctness of address derivation and matching
│   └── Cargo.toml
│
└── frontend/                              # Next.js frontend
    ├── src/
    │   ├── app/
    │   │   └── page.tsx              # Main page, renders InputForm and ResultCard
    │   ├── components/
    │   │   ├── InputForm.tsx         # Pattern input, prefix/suffix toggle, generate button
    │   │   └── ResultCard.tsx        # Displays generated address and private key
    │   ├── hooks/
    │   │   └── useVanityGenerator.ts # Spawns Web Worker, manages state (idle/loading/done/error)
    │   └── workers/
    │       └── vanity.worker.ts      # Loads WASM, calls generate_vanity(), posts result back
    ├── public/
    │   └── wasm/                     # Output from wasm-pack build, loaded by the worker
    └── package.json
```

---

## 6. Rust Engine (`engine/`)

### 6.1 Dependencies (`Cargo.toml`)

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
k256 = "0.13"
tiny-keccak = { version = "2.0", features = ["keccak"] }
getrandom = { version = "0.2", features = ["js"] }
serde = { version = "1.0", features = ["derive"] }
serde-json = "1.0"
```

### 6.2 Public API (`lib.rs`)

One function is exposed to JavaScript:

```rust
#[wasm_bindgen]
pub fn generate_vanity(pattern: &str, is_suffix: bool) -> JsValue
```

Returns a JSON string: `{ "address": "0x...", "private_key": "0x..." }`

### 6.3 Internal Flow (`generator.rs`)

```
loop:
  1. Generate 32 random bytes as private key
     └── via getrandom with "js" feature (uses browser crypto.getRandomValues)

  2. Derive public key from private key
     └── via k256, ECDSA secp256k1

  3. Hash public key with keccak256
     └── via tiny-keccak
     └── take last 20 bytes as Ethereum address

  4. Check pattern match via matcher.rs
     └── is_suffix=true  → address ends with pattern
     └── is_suffix=false → address starts with pattern (after "0x")

  5. Match found → serialize to JSON, return to JS
     No match → continue loop
```

### 6.4 Pattern Matching (`matcher.rs`)

- Pattern matching is case-insensitive
- Prefix check skips the leading `0x`
- Only hex characters (0-9, a-f) are valid in patterns
- Empty pattern should be rejected before entering the loop

### 6.5 Address Derivation (`crypto.rs`)

Standard EVM address derivation:
1. Generate secp256k1 keypair
2. Take the uncompressed public key (64 bytes, strip the 0x04 prefix)
3. keccak256 hash the 64-byte public key
4. Take the last 20 bytes (40 hex chars) as the address
5. Prepend `0x`

No EIP-55 checksum is applied at this stage. Can be added as a utility later.

---

## 7. Frontend (`web/`)

### 7.1 Dependencies (`package.json`)

```json
{
  "dependencies": {
    "next": "latest",
    "react": "latest",
    "react-dom": "latest"
  },
  "devDependencies": {
    "typescript": "latest",
    "tailwindcss": "latest",
    "@types/react": "latest",
    "@types/node": "latest"
  }
}
```

No viem, no ethers.js, no wallet libraries for V1.

### 7.2 Next.js Configuration

Use static export mode. Add to `next.config.ts`:

```ts
const nextConfig = {
  output: "export",
}
```

This produces a fully static build with no Node.js server required.

### 7.3 `useVanityGenerator.ts`

Manages the full lifecycle of the generation process:

```
States: idle | generating | done | error

Actions:
  - startGeneration(pattern, isSuffix): spawns Web Worker, sends message
  - onWorkerMessage: receives { address, private_key }, updates state to done
  - onWorkerError: updates state to error
  - reset(): clears result, terminates worker, returns to idle
```

The hook never touches cryptographic logic directly. It only manages Worker communication and UI state.

### 7.4 `vanity.worker.ts`

Runs in a separate thread. Responsibilities:

```
1. Receive { pattern, isSuffix } from main thread via onmessage
2. Load and initialize WASM module from /wasm/
3. Call generate_vanity(pattern, isSuffix)
4. Post result { address, private_key } back to main thread via postMessage
```

WASM is loaded once per worker instance. Worker is terminated after posting the result.

### 7.5 `InputForm.tsx`

Fields:
- Text input for pattern (hex characters only, max 8 characters for reasonable UX)
- Toggle for prefix or suffix
- Generate button

Validation before triggering generation:
- Pattern must not be empty
- Pattern must only contain valid hex characters: `[0-9a-fA-F]`
- Show inline error if invalid

### 7.6 `ResultCard.tsx`

Displayed after generation completes:
- Show the full generated address
- Show the private key with a copy button
- Display a non-skippable warning before revealing the private key:
  > "Save your private key in a password manager. Do not screenshot it. Do not share it with anyone. This will not be shown again."
- After user clicks confirm, reveal the private key
- Provide a "Generate Another" button that calls reset()

**Critical**: Never store address or private key in localStorage, sessionStorage, or any persistent browser storage. State lives only in React component memory and is cleared on reset.

---

## 8. Generation Flow (End to End)

```
User enters pattern "3504", selects suffix
            ↓
InputForm validates input (hex only, not empty)
            ↓
useVanityGenerator.startGeneration("3504", true)
            ↓
Spawns vanity.worker.ts in background thread
            ↓
Worker loads WASM from /public/wasm/
            ↓
Worker calls generate_vanity("3504", true)
            ↓
Rust brute-force loop runs (blocking inside worker thread)
            ↓
Match found: address ending in "3504"
            ↓
Worker posts { address: "0x...3504", private_key: "0x..." }
            ↓
useVanityGenerator receives message, sets state to done
            ↓
ResultCard renders with warning → user confirms → private key revealed
            ↓
User copies private key, imports to Rabby or MetaMask
            ↓
User clicks "Generate Another" → state reset, worker terminated
```

---

## 9. Security Model

| Concern | Mitigation |
|---|---|
| Private key exfiltration via network | No network requests during generation. Verified by absence of fetch/XHR in worker and engine |
| Weak randomness | Uses `crypto.getRandomValues` via `getrandom` js feature, not `Math.random()` |
| Malicious dependency | Minimal dependency tree, no unnecessary third-party packages |
| Private key persisted in browser | Never written to localStorage, sessionStorage, or IndexedDB |
| Supply chain attack | Open source, encourage offline usage, minimal deps |
| UI thread blocked during generation | Web Worker isolates brute-force from main thread |

### Offline Usage

Users should be encouraged to:
1. Download the static build
2. Disconnect from the internet
3. Open in browser locally
4. Generate address offline

Instructions for this will be included in the README and on the site.

---

## 10. Build Process

### Rust → WASM

```bash
cd engine
wasm-pack build --target frontend --out-dir ../frontend/public/wasm
```

Output files placed in `web/public/wasm/`:
- `tunic_engine.js`
- `tunic_engine_bg.wasm`
- `package.json`

### Frontend

```bash
cd web
npm install
npm run build
```

Output is a static site in `web/out/`, ready for deployment to Vercel or Cloudflare Pages.

---

## 11. Estimated Generation Speed (Browser, Single Thread)

| Pattern Length | Estimated Time |
|---|---|
| 4 characters | under 1 second |
| 6 characters | a few seconds |
| 8 characters | several minutes |
| 10+ characters | tens of minutes or more |

Suffix patterns are faster than prefix patterns because the prefix `0x` reduces the effective matching space.

---

## 13. Key Constraints and Rules for AI Agents

- Never add server-side logic, API routes, or backend of any kind to V1
- Never use localStorage or sessionStorage for private key or address storage
- Never add network requests inside `vanity.worker.ts` or `engine/`
- Keep Rust dependencies minimal, only what is listed in section 6.1
- Keep frontend dependencies minimal, only what is listed in section 7.1
- All cryptographic logic lives in Rust engine, not in JavaScript
- Web Worker is mandatory, never run brute-force on the main thread
- Pattern validation must happen on the frontend before passing to WASM
- Static export must be maintained, do not introduce SSR or API routes
- Private key must be cleared from state after user resets or closes ResultCard