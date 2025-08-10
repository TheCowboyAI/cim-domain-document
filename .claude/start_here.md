# 🎯 START HERE - CIM Development Assistant

**Current Context: Domain Implementation (document)**  
**Repository**: cim-domain-document v0.3.0  
**Role**: Assemble CIM modules for document business logic

## 🚨 CRITICAL RULES - READ FIRST

### Filename Convention
**NEVER CREATE UPPERCASE FILENAMES**
- ALL filenames MUST be lowercase with underscores (snake_case)
- Examples: `readme.md`, `user_story.md`, `event_store.rs`

### Date Handling
**NEVER generate dates from memory**
- Use `$(date -I)` for current date
- Use `$(git log -1 --format=%cd --date=short)` for git dates
- Never hardcode or guess dates

### Git Requirements
- **MUST** `git add` new files for compilation
- Capture git hashes for completed work
- Commit before updating progress tracking

## 📍 Context Detection

Run this to understand your current context:
```bash
bash .claude/scripts/detect-context.sh
```

### Context Types
1. **Infrastructure** - NATS architecture (client/leaf/cluster/super-cluster)
2. **Domain** - Business logic (document, identity, network, workflow)
3. **Activity** - What you're doing (planning, coding, testing, debugging)

## 🎯 Core Development Approach

### Assembly-First Development
- **ASSEMBLE existing cim-* modules** - don't build from scratch
- Create thin domain-specific extensions
- Each CIM targets ONE specific business domain
- Reuse infrastructure: identity, security, storage, workflow

### Single Responsibility Principle
**EVERYTHING has ONE and ONLY ONE responsibility**
- Elements do one thing
- Use dependency injection over direct creation

## 🗂️ Quick Navigation

### By Activity
- **Planning**: [workflows/implementation-flow.md](./workflows/implementation-flow.md)
- **Coding**: [standards/rust-coding-standards.md](./standards/rust-coding-standards.md)
- **Testing**: [standards/test-driven-development.md](./standards/test-driven-development.md)
- **Debugging**: [troubleshooting.md](./troubleshooting.md)

### By Domain Focus
- **Document Domain**: [patterns/domain-driven-design.md](./patterns/domain-driven-design.md)
- **Event Sourcing**: [patterns/event-sourcing-detailed.md](./patterns/event-sourcing-detailed.md)
- **NATS Integration**: [nats-refactoring-plan.md](./nats-refactoring-plan.md)

### By Infrastructure
- **Client Context**: [contexts/client.md](./contexts/client.md)
- **Leaf Node**: [contexts/leaf.md](./contexts/leaf.md)
- **Cluster**: [contexts/cluster.md](./contexts/cluster.md)
- **Super-cluster**: [contexts/super-cluster.md](./contexts/super-cluster.md)

## ⚡ Quick Start Decision Tree

```
What are you doing?
├── 🔍 Understanding the codebase
│   └── Read: INDEX.md → patterns/domain-driven-design.md
├── 📋 Planning new features
│   └── Read: workflows/implementation-flow.md → patterns/event-sourcing-detailed.md
├── 💻 Writing code
│   └── Read: standards/rust-coding-standards.md → standards/nixos-development.md
├── 🧪 Testing
│   └── Read: standards/test-driven-development.md → standards/quality-assurance.md
└── 🐛 Debugging issues
    └── Read: troubleshooting.md → common-tasks.md
```

## 📋 Progress Tracking

All work must update progress tracking:
- Use TodoWrite tool for task management
- Update memory/state.md for major changes
- Follow Definition of Done: DESIGNED → PLANNED → IMPLEMENTED → VERIFIED → TESTED → COMPLETE → DONE

## 🚨 Conflict Resolution

If there is ANY discrepancy between instructions:
1. **STOP immediately**
2. **Ask for guidance**
3. **Do not proceed with assumptions**

## 📚 Complete Documentation

For comprehensive documentation, see:
- **[INDEX.md](./INDEX.md)** - Complete navigation guide
- **[CLAUDE.md](./CLAUDE.md)** - Detailed CIM development instructions
- **[MANDATORY_CHECKLIST.md](./MANDATORY_CHECKLIST.md)** - Required validation steps

---
*Generated: $(date -I)*  
*Repository: cim-domain-document*  
*Context: Domain Implementation*