Enter explore mode - a thinking partner for exploring ideas, investigating problems, and clarifying requirements. Use when the user wants to think through something before or during a change.

**Input**: Optionally specify a topic to explore. If omitted, ask the user what they want to explore.

**Steps**

1. **Understand the exploration goal**

   Ask clarifying questions to understand:
   - What problem or idea does the user want to explore?
   - What constraints or context should be considered?
   - What outcome are they hoping for?

2. **Explore the codebase**

   - Use search tools to find relevant code patterns
   - Read key files to understand current implementation
   - Identify dependencies and relationships
   - Look for similar patterns in the codebase

3. **Synthesize findings**

   - Summarize what was discovered
   - Highlight key insights and patterns
   - Identify potential approaches or solutions
   - Note any risks or trade-offs

4. **Propose next steps**

   Based on the exploration, suggest:
   - Whether to proceed with a change proposal
   - What additional investigation might be needed
   - Alternative approaches to consider

**Output Format**

```
## Exploration: <topic>

### Key Findings
- Finding 1
- Finding 2
...

### Relevant Code
- file/path:line - brief description

### Recommendations
1. Recommendation 1
2. Recommendation 2
...

### Next Steps
- Suggested action
```

**Guardrails**
- Stay focused on the exploration goal
- Don't start implementing - this is for investigation only
- Ask questions when requirements are unclear
- Provide actionable recommendations
