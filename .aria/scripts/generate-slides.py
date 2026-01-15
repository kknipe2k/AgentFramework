#!/usr/bin/env python3
"""
ARIA Slide Generation Script

Two paths:
1. NotebookLM (requires notebooklm-py + Google auth)
2. Local pptx (requires python-pptx)

Usage:
    python generate-slides.py --focus .aria/outputs/FOCUS.md \
                              --idea .aria/docs/IDEA.md \
                              --sources paper.pdf \
                              --method nblm|pptx
"""

import argparse
import asyncio
import os
import re
from datetime import datetime
from pathlib import Path

# Constants
ARIA_DIR = Path('.aria')
OUTPUTS_DIR = ARIA_DIR / 'outputs'

FOCUS_PROMPT = """Analyze all sources and provide a structured synthesis:

a. The Core Ideas: Identify the top 3 foundational elements 
   (concepts, entities, or arguments) that are absolutely 
   required to understand this corpus. Define them briefly.

b. The Synthesis Matrix: Coalesce the findings by listing 
   the top 5-10 unifying ideas or themes that connect the 
   "Core Ideas" together. Explain how these ideas turn the 
   separate elements into a cohesive whole."""

SLIDES_PROMPT = """Intent: Explain in detail the key ideas from these docs.
USE THE Focus doc as the guide to highlight each important 
aspect we need to bring forth and explain.

Must DO:
- Provide detailed learning deck to explain the workflow
- Be verbose
- Use unconventional spatial and verbal slide design 
  techniques rooted in cognitive science for maximum learning
- Make this a long deck (20+ slides if necessary)
- Use charts, graphs, flow diagrams, and other visuals 
  liberally to get the message across
- Break concepts down for ease of intake for new learners 
  or people unfamiliar with the content
- Use diagrams liberally
- Capture main steps in all workflows
- Ensure the high level process is clear
- Provide a clear concise view at the end of complete flow"""


def parse_focus_doc(focus_path: Path) -> dict:
    """Parse FOCUS.md to extract Core Ideas and Synthesis Matrix."""
    content = focus_path.read_text()
    
    result = {
        'core_ideas': [],
        'synthesis_matrix': [],
        'raw': content
    }
    
    # Extract Core Ideas section
    core_match = re.search(
        r'(?:Core Ideas?|Foundational Elements?).*?(?=Synthesis|Matrix|$)',
        content, 
        re.IGNORECASE | re.DOTALL
    )
    if core_match:
        ideas = re.findall(r'^\s*[-*\d.]+\s*(.+)$', core_match.group(), re.MULTILINE)
        result['core_ideas'] = [i.strip() for i in ideas[:5]]
    
    # Extract Synthesis Matrix section
    matrix_match = re.search(
        r'(?:Synthesis Matrix|Unifying (?:Ideas|Themes)).*',
        content,
        re.IGNORECASE | re.DOTALL
    )
    if matrix_match:
        themes = re.findall(r'^\s*[-*\d.]+\s*(.+)$', matrix_match.group(), re.MULTILINE)
        result['synthesis_matrix'] = [t.strip() for t in themes[:10]]
    
    return result


def extract_title_from_idea(idea_path: Path) -> str:
    """Extract title from IDEA.md."""
    content = idea_path.read_text()
    
    # Try to find # Title
    title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
    if title_match:
        return title_match.group(1).strip()
    
    # Fallback to first line
    first_line = content.strip().split('\n')[0]
    return first_line[:50].strip('# ')


# ============================================================
# NotebookLM Path
# ============================================================

async def generate_nblm(focus_path: Path, idea_path: Path, source_paths: list) -> Path:
    """Generate slides via NotebookLM."""
    try:
        from notebooklm import NotebookLMClient
    except ImportError:
        print("ERROR: notebooklm-py not installed")
        print("Install with: pip install \"notebooklm-py[browser]\"")
        print("Then run: playwright install chromium")
        print("Then run: notebooklm login")
        print("Falling back to pptx...")
        return await generate_pptx(focus_path, idea_path, source_paths)

    title = extract_title_from_idea(idea_path)
    timestamp = datetime.now().strftime('%Y%m%d-%H%M')

    try:
        async with await NotebookLMClient.from_storage() as client:
            # Create notebook
            print(f"Creating NotebookLM notebook: {title}")
            notebook = await client.notebooks.create(f"Slides: {title}")

            # Add sources - use add_url for text content or add_file for files
            print("Adding FOCUS.md content...")
            # Write temp file for FOCUS content
            focus_temp = OUTPUTS_DIR / f"_temp_focus_{timestamp}.md"
            focus_temp.write_text(focus_path.read_text())
            await client.sources.add_file(notebook.id, str(focus_temp))

            print("Adding IDEA.md content...")
            idea_temp = OUTPUTS_DIR / f"_temp_idea_{timestamp}.md"
            idea_temp.write_text(idea_path.read_text())
            await client.sources.add_file(notebook.id, str(idea_temp))

            for source in source_paths:
                source_path = Path(source)
                if source_path.exists():
                    print(f"Adding {source_path.name}...")
                    await client.sources.add_file(notebook.id, str(source_path))

            # Generate slides
            print("Generating slides (this may take a minute)...")
            await client.chat.ask(notebook.id, SLIDES_PROMPT)
            status = await client.artifacts.generate_slides(notebook.id)
            await client.artifacts.wait_for_completion(notebook.id, status.task_id)

            # Download
            output_path = OUTPUTS_DIR / f"slides-{timestamp}.pdf"
            await client.artifacts.download_slides(notebook.id, str(OUTPUTS_DIR))

            # Cleanup temp files
            focus_temp.unlink(missing_ok=True)
            idea_temp.unlink(missing_ok=True)

            print(f"Slides saved to: {output_path}")
            return output_path

    except Exception as e:
        print(f"NotebookLM error: {e}")
        print("Run 'notebooklm login' to authenticate, then try again.")
        print("Falling back to pptx...")
        return await generate_pptx(focus_path, idea_path, source_paths)


# ============================================================
# pptx Fallback Path
# ============================================================

async def generate_pptx(focus_path: Path, idea_path: Path, source_paths: list) -> Path:
    """Generate slides via python-pptx."""
    try:
        from pptx import Presentation
        from pptx.util import Inches, Pt
        from pptx.enum.text import PP_ALIGN
        from pptx.dml.color import RgbColor
    except ImportError:
        print("ERROR: python-pptx not installed")
        print("Install with: pip install python-pptx")
        raise SystemExit(1)
    
    title = extract_title_from_idea(idea_path)
    focus_data = parse_focus_doc(focus_path)
    idea_content = idea_path.read_text()
    timestamp = datetime.now().strftime('%Y%m%d-%H%M')
    
    # Create presentation
    prs = Presentation()
    prs.slide_width = Inches(13.333)  # 16:9
    prs.slide_height = Inches(7.5)
    
    # Helper functions
    def add_title_slide(title_text, subtitle_text=""):
        slide_layout = prs.slide_layouts[6]  # Blank
        slide = prs.slides.add_slide(slide_layout)
        
        # Title
        title_box = slide.shapes.add_textbox(Inches(0.5), Inches(2.5), Inches(12), Inches(1.5))
        tf = title_box.text_frame
        p = tf.paragraphs[0]
        p.text = title_text
        p.font.size = Pt(44)
        p.font.bold = True
        p.alignment = PP_ALIGN.CENTER
        
        # Subtitle
        if subtitle_text:
            sub_box = slide.shapes.add_textbox(Inches(0.5), Inches(4), Inches(12), Inches(1))
            tf = sub_box.text_frame
            p = tf.paragraphs[0]
            p.text = subtitle_text
            p.font.size = Pt(24)
            p.alignment = PP_ALIGN.CENTER
        
        return slide
    
    def add_content_slide(title_text, bullets, subtitle=""):
        slide_layout = prs.slide_layouts[6]  # Blank
        slide = prs.slides.add_slide(slide_layout)
        
        # Title
        title_box = slide.shapes.add_textbox(Inches(0.5), Inches(0.3), Inches(12), Inches(0.8))
        tf = title_box.text_frame
        p = tf.paragraphs[0]
        p.text = title_text
        p.font.size = Pt(32)
        p.font.bold = True
        
        # Subtitle
        y_offset = 1.1
        if subtitle:
            sub_box = slide.shapes.add_textbox(Inches(0.5), Inches(y_offset), Inches(12), Inches(0.5))
            tf = sub_box.text_frame
            p = tf.paragraphs[0]
            p.text = subtitle
            p.font.size = Pt(18)
            p.font.italic = True
            y_offset = 1.6
        
        # Bullets
        content_box = slide.shapes.add_textbox(Inches(0.5), Inches(y_offset), Inches(12), Inches(5.5))
        tf = content_box.text_frame
        tf.word_wrap = True
        
        for i, bullet in enumerate(bullets):
            if i == 0:
                p = tf.paragraphs[0]
            else:
                p = tf.add_paragraph()
            p.text = f"• {bullet}"
            p.font.size = Pt(20)
            p.space_after = Pt(12)
        
        return slide
    
    def add_diagram_placeholder(title_text, description):
        slide_layout = prs.slide_layouts[6]
        slide = prs.slides.add_slide(slide_layout)
        
        # Title
        title_box = slide.shapes.add_textbox(Inches(0.5), Inches(0.3), Inches(12), Inches(0.8))
        tf = title_box.text_frame
        p = tf.paragraphs[0]
        p.text = title_text
        p.font.size = Pt(32)
        p.font.bold = True
        
        # Placeholder box
        shape = slide.shapes.add_shape(
            1,  # Rectangle
            Inches(1), Inches(1.5), Inches(11), Inches(5)
        )
        shape.fill.solid()
        shape.fill.fore_color.rgb = RgbColor(240, 240, 240)
        
        # Description
        desc_box = slide.shapes.add_textbox(Inches(1.5), Inches(3.5), Inches(10), Inches(1))
        tf = desc_box.text_frame
        p = tf.paragraphs[0]
        p.text = f"[DIAGRAM: {description}]"
        p.font.size = Pt(18)
        p.alignment = PP_ALIGN.CENTER
        
        return slide
    
    # Build presentation
    print("Building pptx presentation...")
    
    # 1. Title slide
    add_title_slide(title, "Research Synthesis Deck")
    
    # 2. Overview slide
    add_content_slide(
        "Overview",
        ["This deck synthesizes key research findings",
         f"Based on {len(focus_data['core_ideas'])} core ideas",
         f"Connected by {len(focus_data['synthesis_matrix'])} unifying themes"],
        "What you'll learn"
    )
    
    # 3. Core Ideas overview
    add_content_slide(
        "Core Ideas",
        focus_data['core_ideas'] or ["Core ideas extracted from Focus document"],
        "Foundational elements required to understand this topic"
    )
    
    # 4. Individual Core Idea slides
    for i, idea in enumerate(focus_data['core_ideas'][:3], 1):
        add_content_slide(
            f"Core Idea {i}",
            [idea, "Key implications:", "• [Detail from source documents]"],
            "Deep dive"
        )
        add_diagram_placeholder(
            f"Core Idea {i}: Visual",
            f"Diagram illustrating: {idea[:50]}..."
        )
    
    # 5. Synthesis Matrix
    add_content_slide(
        "Synthesis Matrix",
        focus_data['synthesis_matrix'][:5] or ["Themes connecting core ideas"],
        "Unifying ideas that create a cohesive whole"
    )
    
    # 6. Individual theme slides
    for i, theme in enumerate(focus_data['synthesis_matrix'][:5], 1):
        add_content_slide(
            f"Theme {i}",
            [theme, "How it connects:", "• [Connection details]"],
            "Synthesis"
        )
    
    # 7. Workflow overview
    add_diagram_placeholder(
        "Complete Workflow",
        "High-level process flow from inputs to outputs"
    )
    
    # 8. Summary slide
    summary_points = [
        f"Core Ideas: {', '.join(focus_data['core_ideas'][:3]) or 'See Focus doc'}",
        f"Key Themes: {len(focus_data['synthesis_matrix'])} unifying concepts",
        "Next Steps: Review source materials for depth"
    ]
    add_content_slide("Summary", summary_points, "Key takeaways")
    
    # 9. Final slide
    add_title_slide("Questions?", f"Generated {datetime.now().strftime('%Y-%m-%d')}")
    
    # Save
    output_path = OUTPUTS_DIR / f"slides-{timestamp}.pptx"
    OUTPUTS_DIR.mkdir(parents=True, exist_ok=True)
    prs.save(str(output_path))
    
    print(f"Slides saved to: {output_path}")
    print(f"Total slides: {len(prs.slides)}")
    print("\nNOTE: pptx contains placeholder diagrams.")
    print("For richer visuals, use NotebookLM path.")
    
    return output_path


# ============================================================
# Main
# ============================================================

async def main():
    parser = argparse.ArgumentParser(description='Generate slides from research artifacts')
    parser.add_argument('--focus', type=Path, default=OUTPUTS_DIR / 'FOCUS.md',
                        help='Path to FOCUS.md')
    parser.add_argument('--idea', type=Path, default=ARIA_DIR / 'docs' / 'IDEA.md',
                        help='Path to IDEA.md')
    parser.add_argument('--sources', nargs='*', default=[],
                        help='Additional source files (PDFs, MDs)')
    parser.add_argument('--method', choices=['nblm', 'pptx'], default='pptx',
                        help='Generation method: nblm (NotebookLM) or pptx')
    
    args = parser.parse_args()
    
    # Validate inputs
    if not args.focus.exists():
        print(f"ERROR: Focus doc not found: {args.focus}")
        print("Run Focus generation first, or create FOCUS.md manually.")
        raise SystemExit(1)
    
    if not args.idea.exists():
        print(f"ERROR: IDEA.md not found: {args.idea}")
        raise SystemExit(1)
    
    # Generate
    if args.method == 'nblm':
        output = await generate_nblm(args.focus, args.idea, args.sources)
    else:
        output = await generate_pptx(args.focus, args.idea, args.sources)
    
    print(f"\nDone! Output: {output}")


if __name__ == '__main__':
    asyncio.run(main())
