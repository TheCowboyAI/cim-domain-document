# 🚨 CORE RULES - ABSOLUTE REQUIREMENTS

*Single source of truth for all critical development rules*

## 🔴 CRITICAL - NEVER VIOLATE

### Filename Convention
**NEVER CREATE UPPERCASE FILENAMES**
- ALL filenames MUST be lowercase with underscores (snake_case)
- Examples: `readme.md`, `user_story.md`, `event_store.rs`
- This applies to ALL file types without exception

### Date Handling
**NEVER generate dates from memory - ALWAYS use system commands**
- Current date: `$(date -I)` or `$(date +%Y-%m-%d)`
- Git commit dates: `$(git log -1 --format=%cd --date=short)`
- Existing dates: Read from files being processed
- Never hardcode or guess dates

### Git Requirements
- **MUST** `git add` new files for compilation
- Capture git hashes for completed work
- Commit before updating progress tracking
- **NEVER** update git config
- **NEVER** push unless explicitly requested

## 🟡 ESSENTIAL - ALWAYS FOLLOW

### Single Responsibility Principle
**EVERYTHING has ONE and ONLY ONE responsibility**
- Elements do one thing perfectly
- Strive to be the one irreducible way to accomplish the task
- Use dependency injection over direct creation
- Fix unused/incorrect APIs, don't delete them

### Assembly-First Development
- **ASSEMBLE existing cim-* modules** - don't build from scratch
- Create thin domain-specific extensions (e.g., cim-domain-document)
- Each CIM targets ONE specific business domain
- Reuse infrastructure: identity, security, storage, workflow

### Documentation Requirements
- ALWAYS document and justify actions
- Follow documentation in `/doc` organized by context
- Update progress tracking for all enhancements/features
- Use TodoWrite tool for task management

## 🔵 IMPORTANT - MAINTAIN CONSISTENCY

### Quality Standards
- Confirm operation before moving to next phase
- All features MUST work and pass tests
- Use Test-Driven Development (TDD)
- Build in modules, one at a time
- Keep scope as small as possible

### Progress Tracking
Update progress for:
- Enhancements or extensions
- Completed features
- State transitions

Definition of Done:
1. DESIGNED
2. PLANNED
3. IMPLEMENTED
4. VERIFIED
5. TESTED
6. COMPLETE
7. DONE

### Development Environment
- You are ALWAYS in a NixOS devshell
- Adjust shell commands for NixOS environment
- Use available MCP tools alongside built-in tools
- Run lint and typecheck commands when available

## 🚨 CONFLICT RESOLUTION

If there is ANY discrepancy between rules or instructions:
1. **STOP immediately**
2. **Ask for guidance**
3. **Do not proceed with assumptions**

## 📋 VALIDATION CHECKLIST

Before completing any task:
- [ ] Filenames are lowercase with underscores
- [ ] Dates generated using system commands
- [ ] New files added to git
- [ ] Documentation updated
- [ ] Progress tracking updated
- [ ] Tests passing
- [ ] Single responsibility maintained

---
*This is the authoritative source for all critical rules*  
*Generated: $(date -I)*  
*Any conflicts with other files should defer to this document*