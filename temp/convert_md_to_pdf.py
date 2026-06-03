import os
import re
import sys
from reportlab.lib.pagesizes import letter
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, Preformatted, PageBreak
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib import colors

def escape_xml(text):
    return text.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')

def clean_text_for_helvetica(text):
    # Common replacements
    text = text.replace("🌟", "")
    text = text.replace("📐", "")
    text = text.replace("🎯", "")
    text = text.replace("🚀", "")
    text = text.replace("🔍", "")
    text = text.replace("📦", "")
    text = text.replace("🛠️", "")
    text = text.replace("⚡", "")
    text = text.replace("🛡️", "")
    text = text.replace("⚠️", "")
    text = text.replace("💡", "")
    text = text.replace("📝", "")
    text = text.replace("™", "")
    text = text.replace("•", "*")
    text = text.replace("—", "-")
    text = text.replace("–", "-")
    text = text.replace("”", '"').replace("“", '"')
    text = text.replace("’", "'").replace("‘", "'")
    
    # Filter out anything not in ISO-8859-1 (Latin-1)
    cleaned = []
    for char in text:
        if ord(char) < 256:
            cleaned.append(char)
        else:
            # Skip high unicode character to avoid ReportLab font crashes
            pass
    return "".join(cleaned)

def format_inline(text):
    # Clean text to remove emojis and non-latin1 characters
    text = clean_text_for_helvetica(text)
    
    # Escape XML
    escaped = escape_xml(text)
    
    # Bold **text**
    escaped = re.sub(r'\*\*(.*?)\*\*', r'<b>\1</b>', escaped)
    
    # Italic *text*
    escaped = re.sub(r'\*(.*?)\*', r'<i>\1</i>', escaped)
    
    # Inline code `code`
    escaped = re.sub(r'`(.*?)`', r'<font face="Courier" size="9" color="#D63384"><b>\1</b></font>', escaped)
    
    # Links [text](url)
    escaped = re.sub(r'\[(.*?)\]\((.*?)\)', r'<a href="\2" color="#0056b3"><u>\1</u></a>', escaped)
    
    return escaped

def parse_markdown(md_text):
    lines = md_text.splitlines()
    blocks = []
    in_code_block = False
    code_content = []
    code_lang = ""
    
    current_paragraph = []
    
    for line in lines:
        stripped = line.strip()
        
        # Check for code blocks
        if stripped.startswith("```"):
            if in_code_block:
                blocks.append({
                    "type": "code",
                    "content": "\n".join(code_content),
                    "lang": code_lang
                })
                code_content = []
                in_code_block = False
            else:
                if current_paragraph:
                    blocks.append({"type": "paragraph", "content": " ".join(current_paragraph)})
                    current_paragraph = []
                in_code_block = True
                code_lang = stripped[3:].strip()
            continue
            
        if in_code_block:
            code_content.append(line)
            continue
            
        # Check for horizontal rule
        if stripped == "---" or stripped == "***":
            if current_paragraph:
                blocks.append({"type": "paragraph", "content": " ".join(current_paragraph)})
                current_paragraph = []
            blocks.append({"type": "hr"})
            continue
            
        # Check for headings
        if stripped.startswith("#"):
            if current_paragraph:
                blocks.append({"type": "paragraph", "content": " ".join(current_paragraph)})
                current_paragraph = []
            
            level = 0
            for char in stripped:
                if char == '#':
                    level += 1
                else:
                    break
            content = stripped[level:].strip()
            blocks.append({"type": f"h{level}", "content": content})
            continue
            
        # Check for list items
        # Bullets
        bullet_match = re.match(r'^[\*\-\+]\s+(.*)', stripped)
        if bullet_match:
            if current_paragraph:
                blocks.append({"type": "paragraph", "content": " ".join(current_paragraph)})
                current_paragraph = []
            content = bullet_match.group(1).strip()
            # If list item has special syntax like "* #### filename.ts" or "* **Bold**", handle content
            blocks.append({"type": "bullet", "content": content})
            continue
            
        # Numbered lists
        num_match = re.match(r'^(\d+)\.\s+(.*)', stripped)
        if num_match:
            if current_paragraph:
                blocks.append({"type": "paragraph", "content": " ".join(current_paragraph)})
                current_paragraph = []
            num = num_match.group(1)
            content = num_match.group(2).strip()
            blocks.append({"type": "numbered", "num": num, "content": content})
            continue
            
        # Check for empty lines
        if not stripped:
            if current_paragraph:
                blocks.append({"type": "paragraph", "content": " ".join(current_paragraph)})
                current_paragraph = []
            continue
            
        # Regular text line
        current_paragraph.append(line)
        
    if current_paragraph:
        blocks.append({"type": "paragraph", "content": " ".join(current_paragraph)})
        
    return blocks

def main(input_path, output_path):
    print(f"Reading markdown from {input_path}...")
    with open(input_path, 'r', encoding='utf-8') as f:
        md_content = f.read()
        
    blocks = parse_markdown(md_content)
    
    print(f"Parsed {len(blocks)} blocks. Generating PDF at {output_path}...")
    
    # Setup document
    # Margins: 0.75 inch (54 points)
    doc = SimpleDocTemplate(
        output_path,
        pagesize=letter,
        leftMargin=54,
        rightMargin=54,
        topMargin=54,
        bottomMargin=54
    )
    
    # Width = 612 - 54*2 = 504 points
    printable_width = 504
    
    styles = getSampleStyleSheet()
    
    # Custom Styles
    title_style = ParagraphStyle(
        'DocTitle',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=20,
        leading=24,
        textColor=colors.HexColor('#0F172A'),
        spaceAfter=15,
    )
    
    h1_style = ParagraphStyle(
        'DocH1',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=15,
        leading=18,
        textColor=colors.HexColor('#1E293B'),
        spaceBefore=14,
        spaceAfter=8,
        keepWithNext=True
    )
    
    h2_style = ParagraphStyle(
        'DocH2',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=12,
        leading=15,
        textColor=colors.HexColor('#334155'),
        spaceBefore=10,
        spaceAfter=6,
        keepWithNext=True
    )
    
    h3_style = ParagraphStyle(
        'DocH3',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=11,
        leading=14,
        textColor=colors.HexColor('#475569'),
        spaceBefore=8,
        spaceAfter=4,
        keepWithNext=True
    )
    
    body_style = ParagraphStyle(
        'DocBody',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=9.5,
        leading=13.5,
        textColor=colors.HexColor('#334155'),
        spaceAfter=6,
    )
    
    bullet_style = ParagraphStyle(
        'DocBullet',
        parent=body_style,
        leftIndent=15,
        firstLineIndent=-10,
        spaceAfter=3,
    )
    
    numbered_style = ParagraphStyle(
        'DocNumbered',
        parent=body_style,
        leftIndent=18,
        firstLineIndent=-13,
        spaceAfter=3,
    )
    
    code_para_style = ParagraphStyle(
        'CodeParaStyle',
        fontName='Courier',
        fontSize=8.5,
        leading=11,
        textColor=colors.HexColor('#0F172A'),
    )
    
    story = []
    
    def draw_hr():
        t = Table([['']], colWidths=[printable_width], rowHeights=[2])
        t.setStyle(TableStyle([
            ('LINEABOVE', (0,0), (-1,-1), 0.5, colors.HexColor('#CBD5E1')),
            ('BOTTOMPADDING', (0,0), (-1,-1), 0),
            ('TOPPADDING', (0,0), (-1,-1), 0),
        ]))
        return t

    def make_code_block(code_text):
        escaped_code = clean_text_for_helvetica(code_text)
        # Preformatted keeps layout, but we need to escape XML characters
        escaped_code = escape_xml(escaped_code)
        p = Preformatted(escaped_code, code_para_style)
        t = Table([[p]], colWidths=[printable_width])
        t.setStyle(TableStyle([
            ('BACKGROUND', (0,0), (-1,-1), colors.HexColor('#F8FAFC')),
            ('BOX', (0,0), (-1,-1), 0.5, colors.HexColor('#E2E8F0')),
            ('PADDING', (0,0), (-1,-1), 8),
            ('BOTTOMPADDING', (0,0), (-1,-1), 6),
            ('TOPPADDING', (0,0), (-1,-1), 6),
        ]))
        return t

    first_h1 = True
    
    for block in blocks:
        b_type = block['type']
        
        if b_type == 'h1':
            formatted_text = format_inline(block['content'])
            if not first_h1:
                story.append(Spacer(1, 10))
            else:
                first_h1 = False
            story.append(Paragraph(formatted_text, title_style))
            
        elif b_type == 'h2':
            formatted_text = format_inline(block['content'])
            story.append(Paragraph(formatted_text, h1_style))
            
        elif b_type == 'h3':
            formatted_text = format_inline(block['content'])
            story.append(Paragraph(formatted_text, h2_style))
            
        elif b_type == 'h4' or b_type == 'h5' or b_type == 'h6':
            formatted_text = format_inline(block['content'])
            story.append(Paragraph(formatted_text, h3_style))
            
        elif b_type == 'paragraph':
            formatted_text = format_inline(block['content'])
            story.append(Paragraph(formatted_text, body_style))
            
        elif b_type == 'bullet':
            formatted_text = format_inline(block['content'])
            # Render with a Bullet
            bullet_text = f"&bull;&nbsp;&nbsp;{formatted_text}"
            story.append(Paragraph(bullet_text, bullet_style))
            
        elif b_type == 'numbered':
            formatted_text = format_inline(block['content'])
            num_text = f"{block['num']}.&nbsp;&nbsp;{formatted_text}"
            story.append(Paragraph(num_text, numbered_style))
            
        elif b_type == 'code':
            story.append(make_code_block(block['content']))
            story.append(Spacer(1, 4))
            
        elif b_type == 'hr':
            story.append(Spacer(1, 4))
            story.append(draw_hr())
            story.append(Spacer(1, 8))
            
    doc.build(story)
    print("PDF build successful!")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python convert_md_to_pdf.py <input.md> <output.pdf>")
        sys.exit(1)
    main(sys.argv[1], sys.argv[2])
