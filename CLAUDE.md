# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

xyz project - [Add description here]

## Development Commands

### Core Commands
```bash
# Add common commands here
npm start
npm test
npm run build
```

## Architecture

[Add project-specific architecture notes here]

## Key Dependencies

[List important dependencies and external services]

## Development Workflow

### Adding New Features
1. Check project-specific guidelines below
2. Follow existing patterns and conventions
3. Run tests before committing
4. Update documentation as needed

### Testing Strategy
[Add testing approach and commands]

### Code Style
- Follow existing code patterns
- Use consistent naming conventions
- Keep functions focused and small
- Comment complex business logic

## Platform Requirements

[Add platform-specific requirements]

## Performance Considerations

[Add performance notes if relevant]

## Worklog Management

**CRITICAL:** Maintain `docs/worklog.md` as a running AI development diary.

**Rules:**
- Always append new entries to the TOP of the file
- Never delete or update older entries - only add new ones
- Use UTC timestamps for all entries: `date -u +"%Y-%m-%d %H:%M:%S UTC"`
- Log all significant activities: progress, decisions, issues, context, business logic

**Entry Format:**
```
## YYYY-MM-DD HH:MM:SS UTC

**Project**: xyz
**Activity**: Brief description
**What**: What was accomplished
**Details**: Technical details, decisions made, issues encountered
**Next**: What needs follow-up (if applicable)

---
```

**When to Log:**
- Starting/completing development sessions
- Making architectural decisions
- Encountering blocking issues
- Implementing new features or major changes
- Debugging complex problems
- Performance optimizations
- Schema changes or migrations
- Plus anything else worth noting, saving, or mentioning
- Progress updates, thoughts, context, business logic
- Issues, solutions, workarounds, or interesting findings
- Use your judgment - the above are important but not limiting

This allows future Claude instances to understand what has been done, what was learned, and what needs attention, providing continuity across development sessions.

## Project-Specific Guidelines

[Add any project-specific notes, conventions, or important context here]