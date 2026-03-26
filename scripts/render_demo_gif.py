from pathlib import Path
from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "service-impact-demo.gif"
OUT.parent.mkdir(parents=True, exist_ok=True)

WIDTH = 1120
HEIGHT = 720
PADDING_X = 36
PADDING_Y = 28
LINE_HEIGHT = 28
FONT_SIZE = 21
BACKGROUND = "#0f1115"
PANEL = "#151922"
TEXT = "#e6edf3"
MUTED = "#8b949e"
GREEN = "#3fb950"
BLUE = "#79c0ff"

COMMAND_LINES = [
    "$ echo '{",
    '  "registry_path": "fixtures/sample/registry.json",',
    '  "service_id": "billing-api",',
    '  "changed_paths": ["src/events/publisher.rs"],',
    '  "mode": "strict"',
    "}' | cargo run --bin service-impact -- impact",
]

OUTPUT_LINES = [
    "{",
    '  "service_id": "billing-api",',
    '  "changed_paths": ["src/events/publisher.rs"],',
    '  "impacted_services": [',
    "    {",
    '      "service_id": "billing-worker",',
    '      "reasons": ["consumes event invoice-issued"],',
    '      "verification_hooks": ["worker-smoke"]',
    "    }",
    "  ],",
    '  "summary": "Found 1 impacted service and 1 hook"',
    "}",
]


def load_font(size: int):
    candidates = [
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/SFNSMono.ttf",
        "/Library/Fonts/Courier New.ttf",
    ]
    for candidate in candidates:
        path = Path(candidate)
        if path.exists():
            return ImageFont.truetype(str(path), size=size)
    return ImageFont.load_default()


FONT = load_font(FONT_SIZE)
SMALL_FONT = load_font(16)


def draw_frame(typed_chars: int, output_lines_count: int):
    image = Image.new("RGB", (WIDTH, HEIGHT), BACKGROUND)
    draw = ImageDraw.Draw(image)

    draw.rounded_rectangle((18, 18, WIDTH - 18, HEIGHT - 18), radius=18, fill=PANEL)
    draw.ellipse((36, 30, 50, 44), fill="#ff5f57")
    draw.ellipse((58, 30, 72, 44), fill="#febc2e")
    draw.ellipse((80, 30, 94, 44), fill="#28c840")
    draw.text((120, 28), "service-impact demo", font=SMALL_FONT, fill=MUTED)

    y = 78
    command_text = "\n".join(COMMAND_LINES)
    visible = command_text[:typed_chars]
    command_chunks = visible.split("\n")

    for idx, chunk in enumerate(command_chunks):
        color = GREEN if idx == 0 and chunk.startswith("$") else TEXT
        draw.text((PADDING_X, y), chunk, font=FONT, fill=color)
        y += LINE_HEIGHT

    if typed_chars < len(command_text):
        cursor_x = PADDING_X + draw.textlength(command_chunks[-1], font=FONT)
        cursor_y = 78 + (len(command_chunks) - 1) * LINE_HEIGHT
        draw.rectangle((cursor_x + 2, cursor_y + 4, cursor_x + 16, cursor_y + 24), fill=TEXT)

    if output_lines_count > 0:
        y += 18
        draw.text((PADDING_X, y), "output", font=SMALL_FONT, fill=BLUE)
        y += 24
        for line in OUTPUT_LINES[:output_lines_count]:
            color = BLUE if '"summary"' in line else TEXT
            draw.text((PADDING_X, y), line, font=FONT, fill=color)
            y += LINE_HEIGHT

    footer = "Give it changed files, get the smallest reasonable verification scope."
    draw.text((PADDING_X, HEIGHT - 56), footer, font=SMALL_FONT, fill=MUTED)
    return image


def build_frames():
    frames = []
    command_text = "\n".join(COMMAND_LINES)

    for i in range(1, len(command_text) + 1, 3):
        frames.append((draw_frame(i, 0), 45))
    for _ in range(8):
        frames.append((draw_frame(len(command_text), 0), 60))
    for count in range(1, len(OUTPUT_LINES) + 1):
        frames.append((draw_frame(len(command_text), count), 160))
    for _ in range(14):
        frames.append((draw_frame(len(command_text), len(OUTPUT_LINES)), 120))
    return frames


def main():
    frames = build_frames()
    first, first_duration = frames[0]
    rest = [frame for frame, _ in frames[1:]]
    durations = [first_duration] + [duration for _, duration in frames[1:]]
    first.save(
        OUT,
        save_all=True,
        append_images=rest,
        duration=durations,
        loop=0,
        optimize=True,
    )
    print(OUT)


if __name__ == "__main__":
    main()
