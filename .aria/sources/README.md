# Sources Directory

Drop research materials here for ARIA to analyze:

- PDFs (papers, articles)
- URLs (saved as .url or .txt files)
- Screenshots
- Any reference documents

## Usage

When running the **Research** workflow:

1. Drop your source files here
2. Run `/aria-start` with research intent
3. ARIA will analyze and generate IDEA.md

## Supported Formats

| Format | Extension | Notes |
|--------|-----------|-------|
| PDF | `.pdf` | Papers, articles |
| Text | `.txt`, `.md` | Plain text, notes |
| URL | `.url`, `.txt` | Web links to fetch |
| Images | `.png`, `.jpg` | Screenshots, diagrams |

## Example

```
.aria/sources/
├── attention-is-all-you-need.pdf
├── notes.md
└── related-links.txt
```

Files here are used by:
- `researcher.md` skill (extraction)
- `slide-generation.md` skill (presentations)
- `brainstorming.md` skill (synthesis)
