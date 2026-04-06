# h1scout

HackerOne 버그 바운티 프로그램 자동 선별 CLI.
Android/모빌리티 특화 필터 + 종합 점수 랭킹.

## 빠른 시작

### 1. 컨테이너 빌드 & 실행

```bash
git clone https://github.com/YOUR_USERNAME/h1scout.git
cd h1scout
docker compose up --build -d
```

### 2. VSCode SSH 연결

`~/.ssh/config`에 추가:
```
Host h1scout-dev
    HostName localhost
    Port 2222
    User root
```

VSCode → Remote Explorer → h1scout-dev 연결
비밀번호: `root`

### 3. 컨테이너 안에서 Claude Code 실행

VSCode 터미널에서:
```bash
cd /app
claude --dangerously-skip-permissions
```

Claude Code가 CLAUDE.md를 읽고 Phase 1~3을 자율 구현합니다.
Claude Code 구독 플랜으로 실행되므로 별도 API 과금 없음.

## 로컬 실행 (Rust 환경)

```bash
cargo build --release
cargo test
```

## 사용법 (구현 완료 후)

```bash
h1scout fetch
h1scout list --filter android --filter mobility --top 20
h1scout export --format json --output results.json
```

## CI

GitHub push 시 자동 빌드/테스트/clippy 실행.
