# Old Repository Style Analysis

Analysis of pre-Cursor AI repository style patterns: games, netwatch, ref, pkgrank, project-euler

## Common Style Patterns

### README Style

**Extremely Minimal:**
- `games`: Just a list of questions about games, no installation, no examples
- `ref`: One sentence: "A collection of reference works such as favorite words, and phrases, and shell tricks."
- `project-euler`: One line: "Explanations for straight-forward problems are in code comments."

**Moderately Detailed (but still concise):**
- `netwatch`: What it does → Examples → Concepts → Wishlist → Disclaimer
- `pkgrank`: Brief description → Example usage → Technical details

### Key Characteristics

1. **No Marketing Language**
   - Direct, factual descriptions
   - No "powerful", "amazing", "best" type language
   - Focus on what it does, not why it's special

2. **Examples First**
   - `netwatch`: Shows example output immediately
   - `pkgrank`: Example usage right after description
   - Concrete, runnable examples

3. **Technical Honesty**
   - `netwatch`: "I'm naive, curious, and selfishly motivated by personal learning. Please forgive what may seem like reinventing the wheel"
   - `pkgrank`: "Current implementation is naive by calling out to `go list` subprocesses"
   - Acknowledges limitations upfront

4. **Dual Licensing**
   - MIT OR UNLICENSE (netwatch, pkgrank)
   - Simple, permissive

5. **No Excessive Documentation**
   - No "Contributing" sections
   - No "See Also" sections
   - No badges (except pkgrank has one Go Report Card badge)
   - Just what's needed

6. **Commit Messages**
   - Short, descriptive: "egyption-ratscrew: update broken cell output"
   - "readme: fix readme to work on example"
   - "dep: use better go.mod path, and update deps"
   - Direct, no elaborate explanations

7. **File Structure**
   - Minimal: Just what's needed
   - `games`: Just README.md
   - `ref`: README + data files (yaml, txt)
   - `project-euler`: README + problem files

### Comparison with Current Repos

**Old Style:**
- Minimal READMEs
- No badges (or very few)
- No "Contributing" sections
- No "See Also" sections
- Technical honesty about limitations
- Examples shown early
- Dual-licensed (MIT/UNLICENSE)

**Current Style (rank-eval, axum-mcp):**
- More detailed READMEs
- Multiple badges (CI, crates.io, docs, license)
- "Contributing" sections
- "See Also" sections with links
- More polished presentation
- Still technically honest but less self-deprecating

## Recommendations

For maintaining consistency with old style:
1. Keep READMEs minimal and direct
2. Show examples early
3. Acknowledge limitations honestly
4. Avoid excessive sections (Contributing, See Also, etc.) unless truly needed
5. Use dual-licensing (MIT/UNLICENSE) when appropriate
6. Keep commit messages short and descriptive

