# 🌊 Git Workflow para Aevum & Bond

## 🎯 Estratégia Git IA-Enhanced

O workflow Git do Aevum & Bond foi otimizado para **desenvolvimento colaborativo com IA**, integrando GitHub Copilot, Claude e desenvolvimento humano em um fluxo eficiente e seguro.

## 🌳 Branch Strategy

### **📋 Branch Naming Convention**
```
Branch Types:
├── 🏠 main                    # Production-ready code
├── 🚀 develop                 # Integration branch (future)
├── ✨ feature/[name]          # New features  
├── 🐛 bugfix/[name]           # Bug fixes
├── 🔧 refactor/[name]         # Code refactoring
├── 📚 docs/[name]             # Documentation updates
├── 🤖 ai/[tool]-[enhancement] # IA-specific improvements
└── 🔒 security/[fix]          # Security patches
```

### **🎯 Current Branch Structure**
```bash
# ✅ Estado atual (Foundation Complete):
* main                    # Production: Sprint 1+2 consolidadas
  
# 🔜 Próximas branches (Sprint 3+ Feature-Driven):
feature/sprint-3-p2p-networking    # P2P implementation  
feature/sprint-4-consensus         # Consensus algorithm
feature/sprint-5-smart-contracts   # Smart contract VM
feature/sprint-6-wallet            # Wallet integration
# ... até Sprint 14
```

### **📊 Estratégia de Consolidação vs Feature Branches**

#### **Marco 1 (Sprints 1-2): Foundation Consolidation ✅**
**Rationale**: Foundation sprints com alta interdependência arquitetural
```bash
✅ Abordagem Utilizada:
main ← feat(foundation): Sprint 1 & 2 complete
├── Sprint 1: Blockchain core (PoW, UTXO, hashing)
└── Sprint 2: ML-DSA-65 post-quantum cryptography

🎯 Benefícios:
- Arquitetura consolidada e testada
- Interdependências resolvidas
- Base sólida para features futuras
- Histórico git limpo para foundation
```

#### **Marco 2+ (Sprint 3+): Feature-Driven Development 🔜**
**Rationale**: Features independentes com review individual
```bash
🔜 Abordagem Futura:
main ← feature/sprint-3-p2p-networking
main ← feature/sprint-4-consensus  
main ← feature/sprint-5-smart-contracts
# etc...

🎯 Benefícios:
- Review focused em feature específica
- Rollback granular se necessário  
- Desenvolvimento paralelo possível
- IA contribution tracking por feature
```

## 🔄 Workflow Detalhado

### **1. 🎯 Feature Development Flow**

#### **Starting New Feature:**
```bash
# 1. Ensure main is up to date
git checkout main
git pull origin main

# 2. Create feature branch  
git checkout -b feature/pqc-integration

# 3. Set up tracking
git push -u origin feature/pqc-integration
```

#### **IA-Assisted Development Cycle:**
```bash
# Daily development with IA tools:

# Morning: Sync and plan
git pull origin feature/pqc-integration
# 🤖 Claude consultation: Review architecture
# 📋 Plan day's implementation

# Development: IA-assisted coding  
# 🤖 Copilot: Generate boilerplate
# 👨‍💻 Human: Implement core logic
# 🔍 Claude: Code review and optimization

# Frequent commits (recommended: every 1-2 hours)
git add .
git commit -m "feat(pqc): implement ML-DSA key generation

🤖 IA Contribution:
- Copilot: Generated struct boilerplate (70%)  
- Claude: Architectural guidance on memory safety
- Human: Core cryptographic logic implementation"

# End of day: Push progress
git push origin feature/pqc-integration
```

### **2. 📝 Commit Message Convention**

#### **IA-Enhanced Commit Format:**
```
<type>(<scope>): <description>

🤖 IA Contribution:
- Copilot: [specific contributions]
- Claude: [architectural insights]  
- Human: [core logic/validation]

[Optional body providing more details]
[Optional footer with breaking changes/issues]
```

#### **Commit Types:**
```bash
feat:     # New feature
fix:      # Bug fix
docs:     # Documentation update  
style:    # Code formatting (no logic change)
refactor: # Code restructuring (no behavior change)
test:     # Adding/updating tests
chore:    # Build process or auxiliary tool changes
ai:       # IA-specific improvements/optimizations
security: # Security-related changes
perf:     # Performance improvements
```

#### **Example Commits:**
```bash
# ✅ Good commit examples:
git commit -m "feat(crypto): implement ML-DSA-65 signature generation

🤖 IA Contribution:  
- Copilot: Generated PQCSignature struct and basic methods (60%)
- Claude: Reviewed memory safety patterns for private keys
- Human: Implemented core ML-DSA algorithm integration (40%)

Implements Sprint 2 Task 2.3 - signature generation.
Includes secure memory zeroization on Drop trait."

git commit -m "test(blockchain): add comprehensive block validation tests

🤖 IA Contribution:
- Copilot: Generated test scaffolding and edge cases (80%)
- Human: Added blockchain-specific validation logic (20%)

Coverage increased from 85% to 95% for block validation."

git commit -m "ai(copilot): optimize Rust code generation settings

Updated .vscode/settings.json to improve Copilot suggestions:
- Enhanced rust-analyzer integration
- Added blockchain-specific context hints
- Improved completion for cryptographic types"
```

### **3. 🔍 Code Review Process**

#### **Multi-Stage Review:**
```
Review Pipeline:
├── 🤖 Automated Checks (GitHub Actions)
│   ├── cargo fmt --check
│   ├── cargo clippy --workspace  
│   ├── cargo test --workspace
│   └── Security audit (cargo audit)
│
├── 🔍 IA Pre-Review  
│   ├── Claude: Architecture analysis
│   ├── Copilot: Code quality suggestions
│   └── Automated documentation check
│
└── 👨‍💻 Human Review
    ├── Business logic validation
    ├── Security considerations  
    ├── Performance implications
    └── Integration concerns
```

#### **Pull Request Template Usage:**
```markdown
# PR created automatically using template:

## 🚀 Feature: ML-DSA Signature Implementation

### 📋 Description  
Implements post-quantum cryptographic signatures using ML-DSA-65
for the Bond blockchain, completing Sprint 2 Task 2.3.

### 🤖 IA Contribution
- **Copilot**: Generated 65% of boilerplate code including struct definitions,
  error handling patterns, and test scaffolding
- **Claude**: Provided architectural guidance on memory safety for private keys,
  reviewed cryptographic implementation patterns
- **Human**: Implemented core ML-DSA integration, security validation, and
  blockchain-specific transaction signing logic

### ✅ Checklist  
- [x] Tests passing (`cargo test --workspace`)
- [x] Clippy warnings resolved (`cargo clippy --workspace`)
- [x] Documentation updated for new crypto module
- [x] IA review completed (Claude architectural analysis)
- [x] Security scan passed (cargo audit clean)
- [x] Performance benchmarks meet Sprint 2 targets

### 📊 Performance Impact
- Memory usage: +12% (due to ML-DSA key storage)
- Signature generation: ~2.1ms average
- Signature verification: ~1.8ms average  
- Binary size: +450KB (ML-DSA library)
```

### **4. 🚀 Merge Strategies**

#### **Branch Integration:**
```bash
# Feature completion workflow:

# 1. Final sync and rebase  
git checkout feature/pqc-integration
git fetch origin
git rebase origin/main

# 2. Run full test suite
cargo test --workspace
cargo clippy --workspace -- -D warnings  
cargo fmt --all --check

# 3. Create pull request
gh pr create --title "feat(pqc): Complete ML-DSA implementation" \
             --body-file .github/pull_request_template.md

# 4. After approval: Squash and merge
# (Done via GitHub UI with proper commit message)
```

#### **Merge Types by Branch:**
```
Merge Strategies:
├── 🏠 main ← feature/*     # Squash merge (clean history)
├── 🏠 main ← bugfix/*      # Squash merge  
├── 🏠 main ← docs/*        # Squash merge
├── 🤖 feature/* ← ai/*     # Regular merge (preserve IA evolution)
└── 🔒 main ← security/*    # Fast-forward (immediate fixes)
```

## 📊 Git Automation

### **🤖 GitHub Actions Integration**

#### **CI/CD Pipeline (.github/workflows/ci.yml):**
```yaml
name: 🤖 IA-Enhanced CI/CD

on:
  push:
    branches: [ main, develop, 'feature/*' ]
  pull_request:
    branches: [ main ]

jobs:
  # Automated quality checks
  quality-gate:
    name: 🔧 Quality Gate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: 🦀 Setup Rust  
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
          
      - name: 💾 Cache Dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: 🎨 Format Check
        run: cargo fmt --all -- --check
        
      - name: 🔍 Clippy Analysis  
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings
        
      - name: 🧪 Test Suite
        run: cargo test --workspace --verbose
        
      - name: 🔒 Security Audit
        run: |
          cargo install cargo-audit
          cargo audit

  # IA-assisted code analysis  
  ai-review:
    name: 🤖 IA Code Analysis
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
          
      - name: 📊 Generate Diff Analysis
        run: |
          echo "## 🤖 Automated IA Analysis" >> $GITHUB_STEP_SUMMARY
          echo "### 📋 Changed Files:" >> $GITHUB_STEP_SUMMARY
          git diff --name-only origin/main..HEAD >> $GITHUB_STEP_SUMMARY
          echo "### 📊 Code Statistics:" >> $GITHUB_STEP_SUMMARY
          git diff --stat origin/main..HEAD >> $GITHUB_STEP_SUMMARY
```

### **🔧 Git Hooks**

#### **Pre-commit Hook (.git/hooks/pre-commit):**
```bash
#!/bin/sh
# Pre-commit hook for IA-enhanced development

echo "🔍 Pre-commit checks..."

# Format check
if ! cargo fmt --all -- --check > /dev/null 2>&1; then
    echo "❌ Code formatting issues found. Running cargo fmt..."
    cargo fmt --all
    echo "✅ Code formatted. Please review and commit again."
    exit 1
fi

# Quick lint check
if ! cargo clippy --workspace --all-targets -- -D warnings > /dev/null 2>&1; then
    echo "❌ Clippy warnings found. Please fix before committing."
    cargo clippy --workspace --all-targets
    exit 1
fi

# Quick test run (unit tests only)
if ! cargo test --lib --workspace > /dev/null 2>&1; then
    echo "❌ Unit tests failing. Please fix before committing."
    exit 1
fi

echo "✅ Pre-commit checks passed!"
exit 0
```

#### **Commit-msg Hook (.git/hooks/commit-msg):**
```bash
#!/bin/sh
# Validate commit message format

commit_regex='^(feat|fix|docs|style|refactor|test|chore|ai|security|perf)(\(.+\))?: .{1,50}'

if ! grep -qE "$commit_regex" "$1"; then
    echo "❌ Invalid commit message format!"
    echo "Expected: type(scope): description"
    echo "Example: feat(pqc): implement ML-DSA key generation"
    echo "Types: feat, fix, docs, style, refactor, test, chore, ai, security, perf"
    exit 1
fi

echo "✅ Commit message format valid!"
exit 0
```

## 📈 Git Metrics & Analytics

### **📊 Development Velocity Tracking**

#### **Sprint Velocity Measurement:**
```bash
#!/bin/bash
# git-velocity.sh - Sprint velocity analysis

SPRINT_START="2025-09-01"  # Sprint start date
SPRINT_END="2025-09-14"    # Sprint end date

echo "📊 SPRINT VELOCITY ANALYSIS"
echo "=========================="

# Commit frequency
echo "📈 Commits per day:"
git log --since="$SPRINT_START" --until="$SPRINT_END" --format="%ad" --date=short | sort | uniq -c

# Lines of code changes  
echo "📏 Code changes:"
git log --since="$SPRINT_START" --until="$SPRINT_END" --stat --pretty=format:"" | grep -E "files? changed" | tail -1

# Contributors (including IA contribution tracking)
echo "👥 Contributors:"
git log --since="$SPRINT_START" --until="$SPRINT_END" --format="%an" | sort | uniq -c | sort -rn

# Feature branches merged
echo "🌟 Features completed:"  
git log --since="$SPRINT_START" --until="$SPRINT_END" --grep="^feat" --oneline | wc -l

# IA contribution analysis (from commit messages)
echo "🤖 IA Contribution Analysis:"
git log --since="$SPRINT_START" --until="$SPRINT_END" --grep="🤖 IA Contribution" | grep -c "Copilot"
```

### **🎯 Code Quality Metrics**

#### **Quality Dashboard:**
```bash
#!/bin/bash  
# git-quality-dashboard.sh

echo "📋 CODE QUALITY DASHBOARD"
echo "========================"

# Test coverage (if using coverage tool)
echo "🧪 Test Coverage:"
cargo tarpaulin --out Stdout | grep "Coverage Results" || echo "Run 'cargo tarpaulin' for coverage"

# Code complexity (using tokei)
echo "📊 Code Metrics:"
if command -v tokei > /dev/null; then
    tokei --sort=code
else
    find src -name "*.rs" | xargs wc -l | tail -1
fi

# Recent quality trends
echo "📈 Quality Trends (last 7 days):"
echo "Commits: $(git log --since="7 days ago" --oneline | wc -l)"
echo "Files changed: $(git log --since="7 days ago" --name-only --pretty=format:"" | sort | uniq | wc -l)"  
echo "Test files: $(git log --since="7 days ago" --name-only --pretty=format:"" | grep -c "_test.rs\|tests/")"

# Clippy warnings trend
echo "🔍 Code Quality Checks:"
if cargo clippy --workspace 2>&1 | grep -q "0 warnings"; then
    echo "✅ No Clippy warnings"  
else
    echo "⚠️ Clippy warnings found - run 'cargo clippy' to see details"
fi
```

## 🔒 Security & Compliance

### **🛡️ Security Workflow**

#### **Security-First Git Practices:**
```
Security Checkpoints:
├── 📝 Commit Level:
│   ├── No hardcoded secrets (pre-commit hook)
│   ├── Crypto key material exclusion
│   └── Sensitive data scanning
│
├── 🔍 PR Level:  
│   ├── Automated security scanning
│   ├── Dependency vulnerability check
│   └── Code pattern analysis
│
└── 🚀 Merge Level:
    ├── Security review required for crypto changes
    ├── Performance impact assessment
    └── Breaking change documentation
```

#### **Secret Scanning (.gitignore additions):**
```bash
# Security-sensitive files
*.pem
*.key  
*.p12
*.pfx
.env
.secrets/

# Development artifacts that might contain sensitive data
target/release/
*.profdata
flamegraph.svg

# IA tool local configs (may contain API keys)
.copilot/
.claude/
.ai-config/
```

### **📋 Compliance Tracking**

#### **Audit Trail Maintenance:**
```bash
#!/bin/bash
# audit-trail.sh - Generate compliance audit trail

echo "📋 AUDIT TRAIL REPORT - $(date)"
echo "================================"

# Cryptographic changes tracking
echo "🔐 Cryptographic Changes:"
git log --grep="crypto\|pqc\|signature\|key" --oneline --since="30 days ago"

# Security-related commits  
echo "🔒 Security Commits:"
git log --grep="security\|vulnerability\|cve" --oneline --since="30 days ago"

# Code review compliance
echo "👥 Code Review Status:"
git log --merges --since="30 days ago" --pretty=format:"%h %s (reviewed)" | head -10

# Test coverage for security features
echo "🧪 Security Test Coverage:"
find . -name "*test*" -name "*.rs" | xargs grep -l "crypto\|signature\|security" | wc -l
```

## 🎯 Best Practices Summary

### **✅ Do's:**

1. **🤖 Leverage IA Effectively:**
   ```bash
   # Good: Clear, descriptive commits with IA attribution
   git commit -m "feat(pqc): implement key derivation
   
   🤖 IA Contribution:
   - Copilot: Generated BIP32 derivation scaffolding
   - Claude: Reviewed cryptographic correctness  
   - Human: Implemented Bond-specific key paths"
   ```

2. **🔍 Maintain Clean History:**
   ```bash
   # Rebase feature branches before merging
   git rebase -i origin/main
   
   # Squash related commits
   git commit --fixup HEAD~1
   git rebase -i --autosquash origin/main
   ```

3. **🧪 Test Before Pushing:**
   ```bash
   # Always run full test suite
   cargo test --workspace && cargo clippy --workspace && git push
   ```

### **❌ Don'ts:**

1. **🚫 Never Commit:**
   - Private keys or cryptographic material
   - API keys or access tokens
   - Large binary files (>10MB)
   - Generated artifacts (target/, node_modules/)

2. **🚫 Avoid:**
   - Force pushing to main branch
   - Commits without proper IA attribution
   - Merging without code review  
   - Breaking changes without documentation

### **🎯 Workflow Optimization:**

```bash
# Create aliases for common IA-enhanced workflows:
git config alias.ai-commit '!f() { git add . && git commit -m "$1

🤖 IA Contribution:
- Copilot: [describe Copilot contribution]  
- Claude: [describe Claude insights]
- Human: [describe human work]"; }; f'

git config alias.quick-check '!cargo fmt --all && cargo clippy --workspace && cargo test --workspace'

git config alias.feature-start '!f() { git checkout main && git pull && git checkout -b feature/$1; }; f'
```

## 🏆 Conclusão

### **📊 Workflow Achievements (Sprint 1):**
- ✅ 100% commit message compliance
- ✅ Zero security issues in git history  
- ✅ Clean, linear history with proper attribution
- ✅ Full IA contribution tracking
- ✅ Automated quality gates passing

### **🚀 Scaling for 14-Sprint Roadmap:**
- ✅ Branch strategy scales to multiple teams
- ✅ IA attribution supports learning and optimization
- ✅ Security practices ready for production
- ✅ Quality automation prevents regressions

**Este workflow Git estabelece uma base sólida para desenvolvimento colaborativo IA+Human de classe mundial! 🌟**

---

*Workflow atualizado: Sprint 1 completo - 15 de setembro de 2025*