#!/usr/bin/env python3
"""
Generate interactive HTML explainer from IDEA.md or research-output.json

Usage:
    python generate-explainer.py <input_file> [output.html]

    input_file: Either IDEA.md or research-output.json
    output.html: Optional output path (default: explainer-{topic}.html)

Example:
    python generate-explainer.py .aria/docs/IDEA.md
    python generate-explainer.py .aria/docs/research-output.json my-explainer.html
"""

import argparse
import json
import re
import sys
from pathlib import Path
from datetime import datetime

def slugify(text):
    """Convert text to URL-friendly slug."""
    text = text.lower()
    text = re.sub(r'[^\w\s-]', '', text)
    text = re.sub(r'[-\s]+', '-', text)
    return text.strip('-')

def parse_markdown_idea(content):
    """Parse IDEA.md format into structured data."""
    data = {
        'title': '',
        'subtitle': '',
        'summary': '',
        'concepts': [],
        'sections': [],
        'synthesis': [],
        'takeaways': [],
        'sources': []
    }

    lines = content.split('\n')
    current_section = None
    current_content = []
    in_code_block = False

    for i, line in enumerate(lines):
        # Track code blocks
        if line.strip().startswith('```'):
            in_code_block = not in_code_block
            if current_section:
                current_content.append(line)
            continue

        if in_code_block:
            if current_section:
                current_content.append(line)
            continue

        # Extract title (first h1)
        if line.startswith('# ') and not data['title']:
            data['title'] = line[2:].strip()
            continue

        # Extract subtitle (blockquote after title)
        if line.startswith('> ') and not data['subtitle'] and data['title']:
            data['subtitle'] = line[2:].strip()
            continue

        # H2 sections
        if line.startswith('## '):
            # Save previous section
            if current_section and current_content:
                section_text = '\n'.join(current_content).strip()
                if 'summary' in current_section.lower() or 'overview' in current_section.lower():
                    data['summary'] = section_text
                elif 'concept' in current_section.lower() or 'key idea' in current_section.lower():
                    data['concepts'] = parse_concepts(section_text)
                elif 'takeaway' in current_section.lower() or 'conclusion' in current_section.lower():
                    data['takeaways'] = parse_list(section_text)
                elif 'source' in current_section.lower() or 'reference' in current_section.lower():
                    data['sources'] = parse_list(section_text)
                elif 'synthesis' in current_section.lower() or 'matrix' in current_section.lower():
                    data['synthesis'] = parse_table(section_text)
                else:
                    data['sections'].append({
                        'title': current_section,
                        'content': section_text
                    })

            current_section = line[3:].strip()
            current_content = []
            continue

        # Collect content for current section
        if current_section:
            current_content.append(line)

    # Don't forget last section
    if current_section and current_content:
        section_text = '\n'.join(current_content).strip()
        if 'takeaway' in current_section.lower() or 'conclusion' in current_section.lower():
            data['takeaways'] = parse_list(section_text)
        elif 'source' in current_section.lower():
            data['sources'] = parse_list(section_text)
        else:
            data['sections'].append({
                'title': current_section,
                'content': section_text
            })

    return data

def parse_concepts(text):
    """Parse concept definitions from markdown."""
    concepts = []
    current_concept = None
    current_desc = []

    for line in text.split('\n'):
        # H3 or bold as concept name
        if line.startswith('### '):
            if current_concept:
                concepts.append({
                    'name': current_concept,
                    'description': '\n'.join(current_desc).strip()
                })
            current_concept = line[4:].strip()
            current_desc = []
        elif line.startswith('**') and line.endswith('**'):
            if current_concept:
                concepts.append({
                    'name': current_concept,
                    'description': '\n'.join(current_desc).strip()
                })
            current_concept = line.strip('*').strip()
            current_desc = []
        elif line.startswith('- **') and '**:' in line:
            # Format: - **Concept**: Description
            match = re.match(r'-\s*\*\*(.+?)\*\*:\s*(.+)', line)
            if match:
                concepts.append({
                    'name': match.group(1).strip(),
                    'description': match.group(2).strip()
                })
        elif current_concept:
            current_desc.append(line)

    if current_concept:
        concepts.append({
            'name': current_concept,
            'description': '\n'.join(current_desc).strip()
        })

    return concepts

def parse_list(text):
    """Parse bullet list from markdown."""
    items = []
    for line in text.split('\n'):
        line = line.strip()
        if line.startswith('- ') or line.startswith('* '):
            items.append(line[2:].strip())
        elif line.startswith(('1.', '2.', '3.', '4.', '5.', '6.', '7.', '8.', '9.')):
            items.append(re.sub(r'^\d+\.\s*', '', line).strip())
    return items

def parse_table(text):
    """Parse markdown table into list of dicts."""
    rows = []
    headers = []

    for line in text.split('\n'):
        line = line.strip()
        if not line.startswith('|'):
            continue
        if '---' in line:
            continue

        cells = [c.strip() for c in line.split('|')[1:-1]]

        if not headers:
            headers = cells
        else:
            row = {}
            for i, cell in enumerate(cells):
                if i < len(headers):
                    row[headers[i]] = cell
            if row:
                rows.append(row)

    return rows

def parse_json_research(content):
    """Parse research-output.json format."""
    data = json.loads(content)

    return {
        'title': data.get('topic', data.get('title', 'Research Output')),
        'subtitle': data.get('subtitle', ''),
        'summary': data.get('summary', data.get('executive_summary', '')),
        'concepts': [
            {'name': c.get('name', c.get('term', '')),
             'description': c.get('definition', c.get('description', ''))}
            for c in data.get('concepts', data.get('key_concepts', []))
        ],
        'sections': [
            {'title': s.get('title', s.get('heading', '')),
             'content': s.get('content', s.get('body', ''))}
            for s in data.get('sections', data.get('findings', []))
        ],
        'synthesis': data.get('synthesis_matrix', data.get('synthesis', [])),
        'takeaways': data.get('takeaways', data.get('key_takeaways', [])),
        'sources': data.get('sources', data.get('references', []))
    }

def markdown_to_html(text):
    """Simple markdown to HTML conversion."""
    if not text:
        return ''

    # Code blocks
    text = re.sub(r'```(\w+)?\n(.*?)```', r'<pre><code>\2</code></pre>', text, flags=re.DOTALL)

    # Inline code
    text = re.sub(r'`([^`]+)`', r'<code>\1</code>', text)

    # Bold
    text = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', text)

    # Italic
    text = re.sub(r'\*(.+?)\*', r'<em>\1</em>', text)

    # Links
    text = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', r'<a href="\2">\1</a>', text)

    # Line breaks to paragraphs
    paragraphs = text.split('\n\n')
    html_parts = []
    for p in paragraphs:
        p = p.strip()
        if p:
            if p.startswith('<pre>') or p.startswith('<ul>') or p.startswith('<ol>'):
                html_parts.append(p)
            elif p.startswith('- ') or p.startswith('* '):
                items = [f'<li>{line[2:].strip()}</li>' for line in p.split('\n') if line.strip().startswith(('- ', '* '))]
                html_parts.append(f'<ul>{"".join(items)}</ul>')
            else:
                html_parts.append(f'<p>{p}</p>')

    return '\n'.join(html_parts)

def generate_html(data):
    """Generate the interactive HTML explainer."""

    # Generate navigation items
    nav_items = ['overview']
    if data['concepts']:
        nav_items.append('concepts')
    for i, section in enumerate(data['sections']):
        nav_items.append(f'section-{i}')
    if data['synthesis']:
        nav_items.append('synthesis')
    if data['takeaways']:
        nav_items.append('takeaways')

    nav_html = '\n'.join([
        f'            <button class="nav-btn{" active" if i == 0 else ""}" data-section="{item}">'
        f'{item.replace("-", " ").replace("section ", "").title()}</button>'
        for i, item in enumerate(nav_items)
    ])

    # Generate concept cards
    concepts_html = ''
    if data['concepts']:
        concept_cards = []
        colors = ['blue', 'green', 'yellow', 'purple', 'red']
        for i, concept in enumerate(data['concepts']):
            color = colors[i % len(colors)]
            concept_cards.append(f'''
                <div class="card">
                    <div class="card-header">
                        <div class="card-icon {color}">{i + 1}</div>
                        <h3>{concept['name']}</h3>
                    </div>
                    {markdown_to_html(concept['description'])}
                </div>
            ''')
        concepts_html = '\n'.join(concept_cards)

    # Generate section content
    sections_html = ''
    for i, section in enumerate(data['sections']):
        sections_html += f'''
        <section id="section-{i}">
            <h2>{section['title']}</h2>
            {markdown_to_html(section['content'])}
        </section>
        '''

    # Generate synthesis table
    synthesis_html = ''
    if data['synthesis']:
        if isinstance(data['synthesis'], list) and len(data['synthesis']) > 0:
            headers = list(data['synthesis'][0].keys())
            header_row = ''.join([f'<th>{h}</th>' for h in headers])
            body_rows = ''
            for row in data['synthesis']:
                cells = ''.join([f'<td>{row.get(h, "")}</td>' for h in headers])
                body_rows += f'<tr>{cells}</tr>'
            synthesis_html = f'''
            <div class="table-container">
                <table>
                    <thead><tr>{header_row}</tr></thead>
                    <tbody>{body_rows}</tbody>
                </table>
            </div>
            '''

    # Generate takeaways
    takeaways_html = ''
    if data['takeaways']:
        items = ''.join([f'<li>{t}</li>' for t in data['takeaways']])
        takeaways_html = f'<ul class="takeaways-list">{items}</ul>'

    # Generate sources
    sources_html = ''
    if data['sources']:
        items = ''.join([f'<li>{s}</li>' for s in data['sources']])
        sources_html = f'''
        <div class="sources">
            <h3>Sources</h3>
            <ul>{items}</ul>
        </div>
        '''

    html = f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{data['title']}</title>
    <style>
        :root {{
            --bg-primary: #0d1117;
            --bg-secondary: #161b22;
            --bg-tertiary: #21262d;
            --text-primary: #e6edf3;
            --text-secondary: #8b949e;
            --accent-blue: #58a6ff;
            --accent-green: #3fb950;
            --accent-yellow: #d29922;
            --accent-red: #f85149;
            --accent-purple: #a371f7;
            --border-color: #30363d;
        }}

        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            min-height: 100vh;
        }}

        .container {{
            max-width: 1000px;
            margin: 0 auto;
            padding: 2rem;
        }}

        header {{
            text-align: center;
            padding: 3rem 0;
            border-bottom: 1px solid var(--border-color);
            margin-bottom: 2rem;
        }}

        h1 {{
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
            background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }}

        .subtitle {{
            color: var(--text-secondary);
            font-size: 1.2rem;
        }}

        nav {{
            display: flex;
            justify-content: center;
            gap: 0.75rem;
            margin: 2rem 0;
            flex-wrap: wrap;
        }}

        .nav-btn {{
            padding: 0.6rem 1.2rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            color: var(--text-primary);
            cursor: pointer;
            transition: all 0.2s;
            font-size: 0.9rem;
        }}

        .nav-btn:hover {{
            background: var(--bg-tertiary);
            border-color: var(--accent-blue);
        }}

        .nav-btn.active {{
            background: var(--accent-blue);
            border-color: var(--accent-blue);
            color: #fff;
        }}

        section {{
            display: none;
            animation: fadeIn 0.3s ease;
        }}

        section.active {{
            display: block;
        }}

        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(10px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}

        h2 {{
            font-size: 1.8rem;
            margin-bottom: 1.5rem;
            color: var(--accent-blue);
        }}

        h3 {{
            font-size: 1.3rem;
            margin: 1.5rem 0 1rem;
            color: var(--text-primary);
        }}

        p {{
            margin-bottom: 1rem;
            color: var(--text-secondary);
        }}

        .card {{
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
        }}

        .card-header {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
            margin-bottom: 1rem;
        }}

        .card-icon {{
            width: 36px;
            height: 36px;
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: bold;
            font-size: 1rem;
        }}

        .card-icon.blue {{ background: rgba(88, 166, 255, 0.2); color: var(--accent-blue); }}
        .card-icon.green {{ background: rgba(63, 185, 80, 0.2); color: var(--accent-green); }}
        .card-icon.yellow {{ background: rgba(210, 153, 34, 0.2); color: var(--accent-yellow); }}
        .card-icon.red {{ background: rgba(248, 81, 73, 0.2); color: var(--accent-red); }}
        .card-icon.purple {{ background: rgba(163, 113, 247, 0.2); color: var(--accent-purple); }}

        ul, ol {{
            margin: 1rem 0 1rem 1.5rem;
            color: var(--text-secondary);
        }}

        li {{
            margin-bottom: 0.5rem;
        }}

        pre {{
            background: var(--bg-tertiary);
            border-radius: 6px;
            padding: 1rem;
            overflow-x: auto;
            margin: 1rem 0;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 0.85rem;
        }}

        code {{
            font-family: 'Monaco', 'Menlo', monospace;
            background: var(--bg-tertiary);
            padding: 0.2rem 0.4rem;
            border-radius: 3px;
            font-size: 0.9em;
        }}

        pre code {{
            background: none;
            padding: 0;
        }}

        .table-container {{
            overflow-x: auto;
            margin: 1.5rem 0;
        }}

        table {{
            width: 100%;
            border-collapse: collapse;
            background: var(--bg-secondary);
            border-radius: 8px;
            overflow: hidden;
        }}

        th, td {{
            padding: 0.75rem 1rem;
            text-align: left;
            border-bottom: 1px solid var(--border-color);
        }}

        th {{
            background: var(--bg-tertiary);
            color: var(--text-primary);
            font-weight: 600;
        }}

        td {{
            color: var(--text-secondary);
        }}

        tr:last-child td {{
            border-bottom: none;
        }}

        .takeaways-list {{
            list-style: none;
            margin-left: 0;
        }}

        .takeaways-list li {{
            padding: 1rem;
            background: var(--bg-secondary);
            border-left: 3px solid var(--accent-green);
            margin-bottom: 0.75rem;
            border-radius: 0 6px 6px 0;
        }}

        .sources {{
            margin-top: 2rem;
            padding-top: 2rem;
            border-top: 1px solid var(--border-color);
        }}

        .sources h3 {{
            color: var(--text-secondary);
            font-size: 1rem;
        }}

        .sources ul {{
            font-size: 0.9rem;
        }}

        footer {{
            text-align: center;
            padding: 2rem;
            margin-top: 2rem;
            border-top: 1px solid var(--border-color);
            color: var(--text-secondary);
            font-size: 0.9rem;
        }}

        a {{
            color: var(--accent-blue);
            text-decoration: none;
        }}

        a:hover {{
            text-decoration: underline;
        }}

        strong {{
            color: var(--text-primary);
        }}

        @media (max-width: 768px) {{
            .container {{
                padding: 1rem;
            }}

            h1 {{
                font-size: 1.8rem;
            }}

            nav {{
                gap: 0.5rem;
            }}

            .nav-btn {{
                padding: 0.5rem 0.8rem;
                font-size: 0.8rem;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>{data['title']}</h1>
            {f'<p class="subtitle">{data["subtitle"]}</p>' if data['subtitle'] else ''}
        </header>

        <nav>
{nav_html}
        </nav>

        <section id="overview" class="active">
            <h2>Overview</h2>
            {markdown_to_html(data['summary']) if data['summary'] else '<p>An interactive explainer generated from research.</p>'}
        </section>

        {'<section id="concepts"><h2>Key Concepts</h2>' + concepts_html + '</section>' if data['concepts'] else ''}

{sections_html}

        {'<section id="synthesis"><h2>Synthesis</h2>' + synthesis_html + '</section>' if synthesis_html else ''}

        {'<section id="takeaways"><h2>Key Takeaways</h2>' + takeaways_html + sources_html + '</section>' if data['takeaways'] else ''}

        <footer>
            <p>Generated by ARIA &middot; {datetime.now().strftime('%Y-%m-%d')}</p>
        </footer>
    </div>

    <script>
        document.querySelectorAll('.nav-btn').forEach(btn => {{
            btn.addEventListener('click', () => {{
                document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
                document.querySelectorAll('section').forEach(s => s.classList.remove('active'));
                btn.classList.add('active');
                const section = document.getElementById(btn.dataset.section);
                if (section) section.classList.add('active');
            }});
        }});
    </script>
</body>
</html>
'''
    return html

def main():
    parser = argparse.ArgumentParser(
        description='Generate interactive HTML explainer from IDEA.md or research-output.json'
    )
    parser.add_argument('input_file', help='Input file (IDEA.md or research-output.json)')
    parser.add_argument('output_file', nargs='?', help='Output HTML file (optional)')

    args = parser.parse_args()

    input_path = Path(args.input_file)

    if not input_path.exists():
        print(f"Error: Input file not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    content = input_path.read_text(encoding='utf-8')

    # Detect format and parse
    if input_path.suffix == '.json':
        data = parse_json_research(content)
    else:
        data = parse_markdown_idea(content)

    # Generate output filename if not provided
    if args.output_file:
        output_path = Path(args.output_file)
    else:
        slug = slugify(data['title']) if data['title'] else 'explainer'
        output_path = Path(f'explainer-{slug}.html')

    # Generate HTML
    html = generate_html(data)

    # Write output
    output_path.write_text(html, encoding='utf-8')
    print(f"Generated: {output_path}")

if __name__ == '__main__':
    main()
