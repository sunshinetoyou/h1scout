# h1scout — CLAUDE.md

You are a Rust engineer. Work autonomously. Do not ask for confirmation.
Follow the TDD cycle strictly for every step.

---

## TDD Cycle — MANDATORY for every step

For each step below, execute this exact sequence:

```
1. WRITE TEST   — write the test code first (must compile but fail)
2. RUN FAIL     — run `cargo test <test_name>` and confirm it fails (red)
3. WRITE CODE   — implement the minimum code to make the test pass
4. RUN PASS     — run `cargo test <test_name>` and confirm it passes (green)
5. REFACTOR     — clean up if needed, run full `cargo test` to confirm no regression
6. COMMIT       — `git add -A && git commit -m "step X-X: <description>"`
```

Never write implementation code before its test exists.
Never move to the next step until current step's tests are green.
Never skip the commit.

---

## Environment

- Rust edition 2021
- No real H1 API calls — use httpmock + fixtures only
- SQLite via sqlx
- Commit after every green step

---

## Phase 1 — Data Layer

### Step 1-1: Fixtures

Create these files (no test needed, just create):

**tests/fixtures/programs_page1.json**
```json
{
  "data": [
    {"id":"1","type":"program","attributes":{"handle":"general-motors","name":"General Motors","offers_bounties":true,"submission_state":"open","fast_payments":true,"open_scope":false}},
    {"id":"2","type":"program","attributes":{"handle":"uber","name":"Uber","offers_bounties":true,"submission_state":"open","fast_payments":true,"open_scope":true}},
    {"id":"3","type":"program","attributes":{"handle":"grab","name":"Grab","offers_bounties":true,"submission_state":"open","fast_payments":false,"open_scope":false}}
  ],
  "links": {"next":"https://api.hackerone.com/v1/hackers/programs?page[number]=2"}
}
```

**tests/fixtures/programs_page2.json**
```json
{
  "data": [
    {"id":"4","type":"program","attributes":{"handle":"lyft","name":"Lyft","offers_bounties":true,"submission_state":"open","fast_payments":true,"open_scope":false}},
    {"id":"5","type":"program","attributes":{"handle":"ford","name":"Ford Motor Company","offers_bounties":true,"submission_state":"open","fast_payments":false,"open_scope":true}}
  ],
  "links": {}
}
```

**tests/fixtures/scopes_android.json**
```json
{
  "data": [
    {"id":"s1","type":"structured-scope","attributes":{"asset_type":"ANDROID","asset_identifier":"com.gm.myvehicle","eligible_for_bounty":true,"eligible_for_submission":true,"max_severity":"critical"}},
    {"id":"s2","type":"structured-scope","attributes":{"asset_type":"URL","asset_identifier":"*.gm.com","eligible_for_bounty":true,"eligible_for_submission":true,"max_severity":"high"}},
    {"id":"s3","type":"structured-scope","attributes":{"asset_type":"ANDROID","asset_identifier":"com.uber.driver","eligible_for_bounty":true,"eligible_for_submission":true,"max_severity":"critical"}}
  ],
  "links": {}
}
```

Commit: `git add -A && git commit -m "step 1-1: add test fixtures"`

---

### Step 1-2: API Models

TDD cycle:

**TEST FIRST** — in `src/api/models.rs`, write:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program_list() {
        let json = include_str!("../../tests/fixtures/programs_page1.json");
        let result: ProgramList = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].attributes.handle, "general-motors");
        assert!(result.data[0].attributes.offers_bounties);
        assert!(result.links.as_ref().unwrap().next.is_some());
    }

    #[test]
    fn test_parse_scope_list() {
        let json = include_str!("../../tests/fixtures/scopes_android.json");
        let result: ScopeList = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].attributes.asset_type, "ANDROID");
    }
}
```

Run `cargo test` → confirm FAIL (structs don't exist yet).

**IMPLEMENT** structs: `ProgramList`, `ProgramData`, `ProgramAttributes`, `ScopeList`, `ScopeData`, `ScopeAttributes`, `Links`.

Run `cargo test` → confirm PASS.

Commit: `git add -A && git commit -m "step 1-2: api models with tests green"`

---

### Step 1-3: API Client

TDD cycle:

**TEST FIRST** — in `tests/api_mock.rs`:
```rust
#[tokio::test]
async fn test_pagination_fetches_all_programs() {
    // mock page1 → 3 programs + next link
    // mock page2 → 2 programs + no next link
    // assert total = 5
}

#[tokio::test]
async fn test_rate_limit_retries() {
    // mock first call → 429
    // mock second call → 200 with 1 program
    // assert success after retry
}

#[tokio::test]
async fn test_auth_error_returns_err() {
    // mock → 401
    // assert result.is_err()
}
```

Run `cargo test` → FAIL.

**IMPLEMENT** `src/api/client.rs`:
- `H1Client { client: reqwest::Client, username: String, api_token: String }`
- `H1Client::new(username, api_token) -> Self`
- `async fn fetch_all_programs(&self) -> anyhow::Result<Vec<ProgramData>>`
- `async fn fetch_scopes(&self, handle: &str) -> anyhow::Result<Vec<ScopeData>>`
- Pagination: loop until `links.next` is None
- Rate limit: on 429, sleep 3s, retry max 3 times

Run `cargo test` → PASS.

Commit: `git add -A && git commit -m "step 1-3: api client with pagination and retry"`

---

### Step 1-4: SQLite Cache

TDD cycle:

**TEST FIRST** — in `src/db/cache.rs` tests:
```rust
#[tokio::test]
async fn test_ttl_expired() {
    // insert with fetched_at = now - 90000
    // assert is_stale() == true
}

#[tokio::test]
async fn test_ttl_fresh() {
    // insert with fetched_at = now - 3600
    // assert is_stale() == false
}

#[tokio::test]
async fn test_upsert_idempotent() {
    // upsert same program twice
    // assert count == 1
}
```

Run `cargo test` → FAIL.

**IMPLEMENT** `src/db/cache.rs`:
- Schema (programs + scopes tables as in plan.md)
- `Cache::new(db_path) -> anyhow::Result<Self>` — creates file + tables if not exist
- `async fn upsert_programs(&self, programs: &[ProgramData])`
- `async fn upsert_scopes(&self, handle: &str, scopes: &[ScopeData])`
- `async fn is_stale(&self, ttl_secs: u64) -> bool`
- `async fn get_all_programs(&self) -> Vec<ProgramData>`
- `async fn get_scopes_for(&self, handle: &str) -> Vec<ScopeData>`

Run `cargo test` → PASS.

Phase 1 done check: `cargo test` all green.

Commit: `git add -A && git commit -m "step 1-4: sqlite cache — phase 1 complete"`

---

## Phase 2 — Scorer

### Step 2-1: Weights

**TEST FIRST**:
```rust
#[test]
fn test_weights_sum_to_one() {
    let w = Weights::default();
    let sum = w.bounty_scale + w.response_speed + w.scope_quality + w.program_health;
    assert!((sum - 1.0).abs() < 1e-9);
}

#[test]
fn test_weights_from_missing_config_uses_default() {
    let w = Weights::from_config("/nonexistent/path.toml");
    assert!((w.bounty_scale - 0.30).abs() < 1e-9);
}
```

Run → FAIL. Implement. Run → PASS.

Commit: `git add -A && git commit -m "step 2-1: weights struct"`

---

### Step 2-2: Score Engine

**TEST FIRST**:
```rust
#[test]
fn test_android_scope_gives_bonus() {
    // program with ANDROID eligible scope
    // assert score.scope_score >= 80.0
    // assert score.has_android == true
}

#[test]
fn test_closed_program_low_health() {
    // submission_state = "closed"
    // assert score.health_score <= 10.0
}

#[test]
fn test_all_scores_in_bounds() {
    // any program
    // assert bounty/response/scope/health all in 0.0..=100.0
    // assert total in 0.0..=100.0
}

#[test]
fn test_sort_order() {
    // 3 programs with different scores
    // sort by total desc
    // assert first.total >= second.total >= third.total
}
```

Run → FAIL.

**IMPLEMENT** `src/scorer/engine.rs`:
- `score_program(program, scopes, weights) -> ProgramScore`
- bounty_score: offers_bounties→60, fast_payments→+40 (max 100)
- response_score: fast_payments→80, else→40
- scope_score: eligible count×5 (max 60) + ANDROID+20 + WILDCARD+10 + mobility_keyword+15 (clamp 0..100)
- health_score: open+40, fast_payments+30, open_scope+20, offers_bounties+10
- total = Σ(score×weight), clamp 0..100

Run → PASS.

Commit: `git add -A && git commit -m "step 2-2: score engine — phase 2 complete"`

---

## Phase 3 — Filters + CLI

### Step 3-1: Mobility Filter

**TEST FIRST**:
```rust
#[test]
fn test_match_by_program_name() {
    // program name = "General Motors"
    // assert is_mobility_target() == true
}

#[test]
fn test_match_by_scope_identifier() {
    // scope identifier = "telematics.example.com"
    // assert is_mobility_target() == true
}

#[test]
fn test_no_false_positive() {
    // program = "Airbnb", scopes = ["*.airbnb.com"]
    // assert is_mobility_target() == false
}
```

Run → FAIL. Implement. Run → PASS.

Commit: `git add -A && git commit -m "step 3-1: mobility filter"`

---

### Step 3-2: Android Filter

**TEST FIRST**:
```rust
#[test]
fn test_android_detected() { /* ANDROID asset_type → true */ }

#[test]
fn test_no_android() { /* URL only → false */ }

#[test]
fn test_package_extraction() {
    // ANDROID scopes with com.gm.myvehicle, com.uber.driver
    // assert packages == ["com.gm.myvehicle", "com.uber.driver"]
}
```

Run → FAIL. Implement. Run → PASS.

Commit: `git add -A && git commit -m "step 3-2: android filter"`

---

### Step 3-3: Output

Implement `src/output/table.rs` and `src/output/json.rs`.

table.rs — tabled crate, columns: program | score | bounty | resp | scope | android | mobility
json.rs — serde_json::to_string_pretty

No TDD required for output formatting. Manual verification sufficient.

Commit: `git add -A && git commit -m "step 3-3: output formatters"`

---

### Step 3-4: CLI + main.rs

Wire everything together:

```
h1scout fetch [--force] [--dry-run]
h1scout list [--top N] [--filter android|mobility]... [--format table|json|csv]
h1scout export [--format json|csv] [--output path]
```

Config: `~/.h1scout/config.toml`
DB: `~/.h1scout/h1scout.db`
Env: `H1_USERNAME`, `H1_API_TOKEN`

After wiring, run:
```bash
cargo build --release
cargo test
cargo clippy -- -D warnings
```

All must pass.

Commit: `git add -A && git commit -m "step 3-4: cli wired — phase 3 complete"`

---

## Definition of Done

```bash
cargo test          # zero failures
cargo build --release   # zero warnings
cargo clippy -- -D warnings  # zero warnings
```

All phase checklists green. Every step has a commit.
