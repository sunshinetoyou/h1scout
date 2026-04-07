# h1scout 개발 계획

> HackerOne 버그 바운티 프로그램 자동 선별 프레임워크
> Android/모빌리티 특화 필터 + 종합 점수 랭킹

---

## 배경 및 목적

기존 도구(bbscope 등)는 "어디서 뭘 테스트할 수 있냐"를 알려주는 스코프 수집 도구에 그친다.
h1scout는 한 발 더 나아가 **"지금 내 스킬셋 기준으로 어떤 프로그램을 먼저 공략해야 하냐"** 를
점수로 계산해서 알려주는 선별 도구다.

특히 Android 앱 역공학 + 커넥티드카 취약점 분석 경험을 가진 연구자 관점에서,
모빌리티/Android 스코프를 우선 필터링하는 기능이 핵심 차별점이다.

---

## 마일스톤

### Phase 1 — 데이터 수집 레이어 (1~2주)

**목표**: H1 API에서 프로그램 목록과 스코프를 수집하고 SQLite에 캐싱

#### 1-1. 프로젝트 초기화

```bash
cargo new h1scout
cd h1scout
```

`Cargo.toml` 의존성:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
anyhow = "1"
tabled = "0.15"
toml = "0.8"

[dev-dependencies]
httpmock = "0.7"
tokio-test = "0.4"
```

#### 1-2. API 모델 정의 (src/api/models.rs)

HackerOne JSON API spec 기반 serde 구조체:

```rust
#[derive(Debug, Deserialize)]
pub struct ProgramList {
    pub data: Vec<ProgramData>,
    pub links: Option<Links>,
}

#[derive(Debug, Deserialize)]
pub struct ProgramData {
    pub id: String,
    pub attributes: ProgramAttributes,
}

#[derive(Debug, Deserialize)]
pub struct ProgramAttributes {
    pub handle: String,
    pub name: String,
    pub offers_bounties: bool,
    pub submission_state: String,  // "open" | "closed" | "disabled"
    pub fast_payments: Option<bool>,
    pub open_scope: Option<bool>,
    pub started_accepting_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScopeList {
    pub data: Vec<ScopeData>,
}

#[derive(Debug, Deserialize)]
pub struct ScopeData {
    pub id: String,
    pub attributes: ScopeAttributes,
}

#[derive(Debug, Deserialize)]
pub struct ScopeAttributes {
    pub asset_type: String,
    pub asset_identifier: String,
    pub eligible_for_bounty: bool,
    pub eligible_for_submission: bool,
    pub max_severity: Option<String>,
}
```

#### 1-3. API 클라이언트 (src/api/client.rs)

- `reqwest::Client` 재사용 (connection pool)
- Basic Auth 헤더 자동 주입
- 페이지네이션 자동 처리 (링크 기반)
- 429 Too Many Requests 시 exponential backoff

```rust
pub struct H1Client {
    client: reqwest::Client,
    username: String,
    api_token: String,
}

impl H1Client {
    pub async fn fetch_all_programs(&self) -> anyhow::Result<Vec<ProgramData>>
    pub async fn fetch_scopes(&self, handle: &str) -> anyhow::Result<Vec<ScopeData>>
}
```

**테스트 하네스**:
```rust
#[cfg(test)]
mod tests {
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_programs_pagination() {
        // mock 서버로 2페이지 응답 시뮬레이션
        // 전체 프로그램 수 검증
    }

    #[tokio::test]
    async fn test_rate_limit_backoff() {
        // 429 응답 후 재시도 검증
    }

    #[tokio::test]
    async fn test_fetch_scopes_android() {
        // ANDROID asset_type 포함 응답 파싱 검증
    }
}
```

#### 1-4. SQLite 캐시 (src/db/cache.rs)

스키마:
```sql
CREATE TABLE programs (
    handle TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    offers_bounties INTEGER,
    submission_state TEXT,
    fast_payments INTEGER,
    open_scope INTEGER,
    fetched_at INTEGER  -- Unix timestamp
);

CREATE TABLE scopes (
    id TEXT PRIMARY KEY,
    program_handle TEXT,
    asset_type TEXT,
    asset_identifier TEXT,
    eligible_for_bounty INTEGER,
    max_severity TEXT,
    FOREIGN KEY (program_handle) REFERENCES programs(handle)
);
```

TTL 로직: `fetched_at + 86400 < NOW()` 이면 stale, 재수집

**테스트 하네스**:
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_cache_ttl_expired() {
        // fetched_at을 25시간 전으로 설정 후 is_stale() 검증
    }

    #[tokio::test]
    async fn test_cache_hit() {
        // 동일 handle 재조회 시 DB에서 반환 검증
    }
}
```

---

### Phase 2 — 점수 계산 엔진 (2~3주)

**목표**: 수집된 데이터를 기반으로 프로그램별 종합 점수 계산

#### 2-1. 가중치 설정 (src/scorer/weights.rs)

```rust
#[derive(Debug, Deserialize)]
pub struct Weights {
    pub bounty_scale: f64,      // 기본 0.30
    pub response_speed: f64,    // 기본 0.25
    pub scope_quality: f64,     // 기본 0.25
    pub program_health: f64,    // 기본 0.20
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            bounty_scale: 0.30,
            response_speed: 0.25,
            scope_quality: 0.25,
            program_health: 0.20,
        }
    }
}
```

`~/.h1scout/config.toml`에서 오버라이드:
```toml
[weights]
bounty_scale = 0.40      # 바운티를 더 중요하게
scope_quality = 0.30     # 스코프 퀄리티 상향
response_speed = 0.20
program_health = 0.10
```

#### 2-2. 점수 계산 (src/scorer/engine.rs)

각 항목 0.0~100.0 정규화 후 가중 합산:

```rust
pub struct ProgramScore {
    pub handle: String,
    pub name: String,
    pub total_score: f64,
    pub bounty_score: f64,
    pub response_score: f64,
    pub scope_score: f64,
    pub health_score: f64,
    pub has_android: bool,
    pub has_mobility: bool,
}

pub fn score_program(
    program: &ProgramData,
    scopes: &[ScopeData],
    weights: &Weights,
) -> ProgramScore
```

**바운티 점수**: H1 공개 통계 또는 hacktivity 기반 max/avg 정규화.
초기에는 `offers_bounties` + `fast_payments` 조합으로 근사치 사용.

**응답 속도 점수**: H1 프로그램 페이지의 response time 통계.
초기에는 `fast_payments` 플래그로 대체.

**스코프 점수**:
- `eligible_for_bounty` 스코프 수 × 기본 가중
- ANDROID asset_type 존재 시 +20 보너스
- 모빌리티 키워드 매칭 시 +15 보너스
- WILDCARD 존재 시 +10 보너스

**건강도 점수**:
- `submission_state == "open"`: +40
- `fast_payments == true`: +30
- `open_scope == true`: +20
- 최근 6개월 내 활동: +10

**테스트 하네스**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_score_android_bonus() {
        // ANDROID 스코프 포함 시 scope_score > 기본값 검증
    }

    #[test]
    fn test_score_closed_program() {
        // submission_state == "closed" 시 health_score 낮음 검증
    }

    #[test]
    fn test_weights_sum_to_one() {
        let w = Weights::default();
        let sum = w.bounty_scale + w.response_speed + w.scope_quality + w.program_health;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalization_bounds() {
        // 모든 점수가 0.0~100.0 범위 내 검증
    }
}
```

---

### Phase 3 — 특화 필터 + CLI (1~2주)

**목표**: 모빌리티/Android 필터 완성 및 CLI 인터페이스 마무리

#### 3-1. 모빌리티 필터 (src/filter/mobility.rs)

```rust
const MOBILITY_KEYWORDS: &[&str] = &[
    "automotive", "vehicle", "connected car", "bluelink",
    "telematics", "obd", "can bus", "ecu", "v2x",
    "infotainment", "ivi", "fleet", "carplay", "android auto",
    "tesla", "gm", "ford", "hyundai", "kia", "toyota",
    "bmw", "mercedes", "volkswagen", "volvo",
];

pub fn is_mobility_target(program: &ProgramData, scopes: &[ScopeData]) -> bool {
    // program.name + scope asset_identifier + scope description 전체 검색
}
```

**테스트 하네스**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_mobility_keyword_match_name() {
        // 프로그램 이름에 "automotive" 포함 시 true
    }

    #[test]
    fn test_mobility_keyword_match_scope() {
        // 스코프 identifier에 "telematics" 포함 시 true
    }

    #[test]
    fn test_no_false_positive() {
        // 일반 웹 서비스 프로그램에서 false 반환 검증
    }
}
```

#### 3-2. Android 필터 (src/filter/android.rs)

```rust
pub fn has_android_scope(scopes: &[ScopeData]) -> bool {
    scopes.iter().any(|s| {
        s.attributes.asset_type == "ANDROID"
            && s.attributes.eligible_for_bounty
    })
}

pub fn extract_android_packages(scopes: &[ScopeData]) -> Vec<String> {
    // com.* 패턴 파싱하여 APK 패키지명 추출
}
```

#### 3-3. CLI (src/cli.rs)

```rust
#[derive(Parser)]
#[command(name = "h1scout")]
pub enum Cli {
    /// H1에서 프로그램 데이터 수집 및 캐시 갱신
    Fetch {
        #[arg(long)]
        force: bool,
    },
    /// 점수 계산 후 랭킹 출력
    List {
        #[arg(long, default_value = "20")]
        top: usize,
        #[arg(long)]
        filter: Vec<String>,  // "android", "mobility"
        #[arg(long, default_value = "table")]
        format: String,       // "table" | "json" | "csv"
    },
    /// JSON/CSV로 전체 결과 export
    Export {
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}
```

---

## 출력 예시

```
$ h1scout list --top 5 --filter android

┌─────────────────────┬───────┬─────────┬────────┬───────┬─────────┬──────────┐
│ program             │ score │ bounty  │ resp   │ scope │ android │ mobility │
├─────────────────────┼───────┼─────────┼────────┼───────┼─────────┼──────────┤
│ general-motors      │ 87.3  │ 84.0    │ 91.0   │ 88.0  │  YES    │  YES     │
│ uber                │ 82.1  │ 90.0    │ 75.0   │ 82.0  │  YES    │  NO      │
│ grab                │ 78.4  │ 72.0    │ 88.0   │ 74.0  │  YES    │  YES     │
│ lyft                │ 74.2  │ 68.0    │ 82.0   │ 72.0  │  YES    │  NO      │
│ ford                │ 71.0  │ 65.0    │ 70.0   │ 78.0  │  YES    │  YES     │
└─────────────────────┴───────┴─────────┴────────┴───────┴─────────┴──────────┘
```

---

## 테스트 실행

```bash
# 전체 테스트
cargo test

# 특정 모듈 테스트
cargo test --test scorer_test
cargo test scorer::tests

# mock API 포함 통합 테스트
cargo test --test api_mock
```

---

## 향후 개선 방향

- hacktivity API 크롤링으로 실제 바운티 지급 데이터 수집
- 프로그램별 최근 resolved 리포트 수 기반 활성도 점수 개선
- Discord/Slack webhook 연동 (새 모빌리티 프로그램 알림)
- bugcrowd, intigriti 플랫폼 지원 확장

---

## 참고 자료

- [HackerOne Hacker API](https://api.hackerone.com/hacker-reference/)
- [bbscope](https://github.com/sw33tLie/bbscope) — 스코프 수집 레퍼런스
- [bounty-targets-data](https://github.com/arkadiyt/bounty-targets-data) — 공개 바운티 데이터
