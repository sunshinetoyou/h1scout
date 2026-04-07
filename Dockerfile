FROM rust:1.77-slim-bookworm

# 시스템 패키지 + SSH 서버
RUN apt-get update && apt-get install -y \
    curl \
    git \
    libsqlite3-dev \
    pkg-config \
    ca-certificates \
    openssh-server \
    && rm -rf /var/lib/apt/lists/*

# SSH 설정 — root 로그인 허용, 패스워드 인증 허용
RUN mkdir /var/run/sshd \
    && echo "root:root" | chpasswd \
    && sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config \
    && sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config

# Node.js (Claude Code CLI 의존성)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Claude Code CLI 설치
RUN npm install -g @anthropic-ai/claude-code

# git 전역 설정 (Claude Code가 git 상태를 읽음)
RUN git config --global user.email "claude@h1scout" \
    && git config --global user.name "Claude Code" \
    && git config --global init.defaultBranch main

# cargo 캐시 레이어 최적화 — 의존성 먼저 빌드
WORKDIR /app
COPY Cargo.toml ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
    && cargo build 2>/dev/null || true \
    && rm -rf src

# 프로젝트 전체 복사 (.dockerignore로 .git, target 제외됨)
COPY . .

# git 초기화 — COPY 이후에 실행
RUN git init \
    && git add -A \
    && git commit -m "initial scaffold"

# cargo 캐시를 볼륨으로 분리하기 위한 env
ENV CARGO_HOME=/cargo-cache
ENV PATH=$CARGO_HOME/bin:$PATH

WORKDIR /app

# SSH 접속 시 Rust PATH + .env 자동 로드
RUN echo 'export PATH="/usr/local/cargo/bin:$PATH"' >> /root/.bashrc \
    && echo 'rustup default stable >/dev/null 2>&1' >> /root/.bashrc \
    && echo '[ -f /app/.env ] && export $(grep -v "^#" /app/.env | xargs)' >> /root/.bashrc

EXPOSE 22

CMD ["/usr/sbin/sshd", "-D"]
