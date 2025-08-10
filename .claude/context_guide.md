# 🧭 Context Guide - Understanding Where You Are

*Clear framework for determining your current context and focus*

## 🎯 Three Types of Context

### 1. 🏗️ Infrastructure Context
**Where your code runs in the CIM architecture**

- **Client** - Local NATS, development environment
- **Leaf** - Single node hosting services  
- **Cluster** - 3+ leaf nodes for high availability
- **Super-cluster** - 3+ clusters for global scale

*Detected by: `bash .claude/scripts/detect-context.sh`*

### 2. 🎯 Domain Context  
**What business domain you're working in**

- **Document** - Document management, versioning, content intelligence
- **Identity** - Authentication, authorization, user management
- **Network** - Routing, switching, IP management, connectivity
- **Workflow** - Process orchestration, state machines, task management
- **Storage** - Persistence, caching, data retrieval
- **Communication** - Messaging, notifications, NATS integration

*Determined by: Repository name (cim-domain-*) and business requirements*

### 3. 🎬 Activity Context
**What you're currently doing**

- **Planning** - Architecture, design, event storming
- **Coding** - Implementation, TDD, feature development  
- **Testing** - Unit tests, integration tests, verification
- **Debugging** - Fixing errors, troubleshooting, analysis
- **Documentation** - User guides, API docs (only when asked)

*Determined by: User's current request and task type*

## 🚨 Context Priority Rules

### Context Hierarchy
1. **Activity Context** (highest priority) - Determines immediate approach
2. **Domain Context** (medium priority) - Determines business focus  
3. **Infrastructure Context** (lowest priority) - Determines deployment patterns

### Decision Framework
```
What should I focus on RIGHT NOW?

1. ACTIVITY: What am I doing?
   ├── Planning → Use architectural patterns, event storming
   ├── Coding → Follow TDD, event-driven patterns
   ├── Testing → Focus on coverage, verification
   ├── Debugging → Analyze errors, fix compilation
   └── Documentation → Create only when explicitly requested

2. DOMAIN: What business problem?
   ├── Document → Content intelligence, versioning, metadata
   ├── Identity → Auth, permissions, user lifecycle  
   ├── Network → Routing, connectivity, IP management
   ├── Workflow → Process orchestration, state transitions
   └── [Other domains] → Specific business logic

3. INFRASTRUCTURE: Where does it run?
   ├── Client → Local development, testing
   ├── Leaf → Single service deployment
   ├── Cluster → High availability, failover
   └── Super-cluster → Global scale, multi-region
```

## 📍 Context Detection Commands

### Quick Context Check
```bash
# Detect infrastructure context
bash .claude/scripts/detect-context.sh

# Check current domain
basename $(git rev-parse --show-toplevel)

# Determine activity from user request
# (analyzed contextually from conversation)
```

### Full Context Analysis
```bash
# Repository context
echo "Repository: $(basename $(git rev-parse --show-toplevel))"
echo "Current branch: $(git branch --show-current)"
echo "Working directory: $(pwd)"

# Git status for current activity
git status --porcelain

# Available services/modules
ls -la src/ 2>/dev/null || ls -la services/ 2>/dev/null

# Recent activity
git log --oneline -5
```

## 🎯 Context-Specific Guidelines

### Planning Context
- Focus: Architecture, event storming, design patterns
- Tools: Mermaid diagrams, event modeling, DDD patterns
- Output: Design documents, architectural decisions
- Read: [patterns/domain-driven-design.md](./patterns/domain-driven-design.md)

### Coding Context  
- Focus: TDD, event-driven implementation, Rust best practices
- Tools: Cargo, tests, event sourcing patterns
- Output: Working code, passing tests
- Read: [standards/rust-coding-standards.md](./standards/rust-coding-standards.md)

### Testing Context
- Focus: Verification, coverage, integration testing
- Tools: Cargo test, integration tests, property testing
- Output: Test suites, coverage reports
- Read: [standards/test-driven-development.md](./standards/test-driven-development.md)

### Debugging Context
- Focus: Error analysis, compilation fixes, troubleshooting
- Tools: Compiler output, logs, debugging traces
- Output: Fixed code, resolved issues
- Read: [troubleshooting.md](./troubleshooting.md)

## 🔄 Context Switching

When context changes:
1. **Acknowledge the switch** - "Moving from coding to testing context"
2. **Adjust approach** - Different tools, patterns, focus areas
3. **Update progress** - Mark previous context tasks complete
4. **Reference appropriate docs** - Context-specific guidelines

## ⚠️ Common Context Confusion

### What NOT to conflate:
- **Infrastructure ≠ Domain** - Where code runs vs. what business problem
- **Current Activity ≠ Repository Type** - What you're doing vs. what repo contains
- **Domain ≠ Service** - Business area vs. technical component

### Context Boundaries:
- Stay focused on current activity until complete
- Don't jump between domains without completing tasks  
- Keep infrastructure concerns separate from business logic
- Activity context trumps other contexts for immediate decisions

---
*Generated: $(date -I)*  
*Single source of truth for context determination*