import subprocess
import os
import re
from PIL import Image, ImageDraw, ImageFont

def get_cmd_output(args, cwd='.', stdin_text='\n'):
    env = os.environ.copy()
    env['CLICOLOR_FORCE'] = '1'
    env['FORCE_COLOR'] = '1'
    p = subprocess.Popen(
        args,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        encoding='utf-8',
        env=env,
        cwd=cwd
    )
    out, err = p.communicate(input=stdin_text)
    return out

ANSI_COLORS = {
    '30': (110, 118, 129),   # Black / Dark Grey
    '31': (255, 123, 114),   # Red
    '32': (63, 185, 80),     # Green
    '33': (227, 179, 65),    # Yellow
    '34': (121, 192, 255),   # Blue
    '35': (210, 168, 255),   # Magenta
    '36': (56, 189, 248),    # Cyan
    '37': (230, 237, 243),   # White
    '90': (110, 118, 129),   # Bright Black / Dark Grey
    '91': (255, 160, 155),
    '92': (86, 211, 100),
    '93': (242, 204, 96),
    '94': (155, 210, 255),
    '95': (226, 196, 255),
    '96': (86, 212, 255),
}

DEFAULT_FG = (230, 237, 243)
DIM_FG = (125, 133, 144)
BOLD_WHITE = (255, 255, 255)

def render_terminal(text, title, output_path, max_lines=45):
    # Filter prompt out if present at end
    lines = text.strip('\r\n').split('\n')
    filtered_lines = []
    for line in lines:
        if "Enter folder # to open directly in File Explorer" in line:
            continue
        filtered_lines.append(line.rstrip('\r'))
    if len(filtered_lines) > max_lines:
        filtered_lines = filtered_lines[:max_lines]

    scale = 2  # 2x HiDPI
    font_size = 14 * scale
    font = ImageFont.truetype('C:/Windows/Fonts/CascadiaCode.ttf', font_size)
    
    # Calculate char width accurately
    char_w = font.getlength('M')
    line_h = int(font_size * 1.55)

    # Calculate max visual line length
    max_cols = 100
    for line in filtered_lines:
        plain_len = len(re.sub(r'\x1b\[[0-9;]*m', '', line))
        if plain_len > max_cols:
            max_cols = plain_len

    pad_x = 28 * scale
    header_h = 42 * scale
    pad_y = 20 * scale

    width = int(max_cols * char_w + pad_x * 2)
    height = int(header_h + len(filtered_lines) * line_h + pad_y * 2)

    # Create image
    img = Image.new('RGBA', (width, height), (15, 20, 28, 255))
    draw = ImageDraw.Draw(img)

    # Title bar background
    draw.rectangle([0, 0, width, header_h], fill=(22, 27, 34, 255))
    draw.line([0, header_h, width, header_h], fill=(48, 54, 61, 255), width=scale)

    # Window buttons
    dot_radius = 6 * scale
    dot_y = header_h // 2
    dots = [
        ((22 * scale, dot_y), (255, 95, 86)),   # Close
        ((38 * scale, dot_y), (255, 189, 46)),  # Minimize
        ((54 * scale, dot_y), (39, 201, 63)),   # Maximize
    ]
    for (cx, cy), color in dots:
        draw.ellipse([cx - dot_radius, cy - dot_radius, cx + dot_radius, cy + dot_radius], fill=color)

    # Title text
    title_font = ImageFont.truetype('C:/Windows/Fonts/CascadiaCode.ttf', int(12 * scale))
    title_w = title_font.getlength(title)
    title_x = (width - title_w) // 2
    draw.text((title_x, (header_h - int(12 * scale * 1.2)) // 2), title, font=title_font, fill=(139, 148, 158, 255))

    # Render lines
    y = header_h + pad_y
    for line in filtered_lines:
        tokens = re.split(r'(\x1b\[[0-9;]*m)', line)
        cur_fg = DEFAULT_FG
        cur_bold = False
        cur_dim = False
        x = pad_x

        for tok in tokens:
            if not tok:
                continue
            if tok.startswith('\x1b['):
                codes = tok[2:-1].split(';')
                for c in codes:
                    if c in ('0', ''):
                        cur_fg = DEFAULT_FG
                        cur_bold = False
                        cur_dim = False
                    elif c == '1':
                        cur_bold = True
                    elif c == '2':
                        cur_dim = True
                    elif c in ANSI_COLORS:
                        cur_fg = ANSI_COLORS[c]
            else:
                color = cur_fg
                if cur_bold and color == DEFAULT_FG:
                    color = BOLD_WHITE
                elif cur_dim and color == DEFAULT_FG:
                    color = DIM_FG
                draw.text((x, y), tok, font=font, fill=color + (255,))
                x += font.getlength(tok)
        y += line_h

    # Resize with LANCZOS to 1x for crisp downsampled antialiasing
    final_img = img.resize((width // scale, height // scale), Image.Resampling.LANCZOS)
    final_img.save(output_path, 'PNG', optimize=True)
    print(f"Generated {output_path} ({final_img.width}x{final_img.height})")

if __name__ == '__main__':
    print("Capturing bloathound scan (demo workspace) ..")
    scan_out = get_cmd_output(['cargo', 'run', '--quiet', '--example', 'demo_screenshot'])
    render_terminal(scan_out, "bloathound scan (Developer Projects Workspace)", "docs/images/scan_demo.png")

    print("Capturing bloathound analyze")
    analyze_out = get_cmd_output(['bloathound', 'analyze'])
    render_terminal(analyze_out, "bloathound analyze (Structural Integrity & Dependency Audit)", "docs/images/analyze_demo.png")

    print("Capturing bloathound inspect target")
    inspect_out = get_cmd_output(['bloathound', 'inspect', 'target', '--limit', '15'])
    render_terminal(inspect_out, "bloathound inspect target (File Lifecycle Analytics)", "docs/images/inspect_demo.png")
