from pathlib import Path
from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "service-impact-demo.gif"
OUT.parent.mkdir(parents=True, exist_ok=True)

WIDTH = 1200
HEIGHT = 720
BACKGROUND = "#0b1020"
PANEL = "#121a2b"
PANEL_ALT = "#0f1625"
TEXT = "#e6edf3"
MUTED = "#94a3b8"
GREEN = "#3fb950"
RED = "#ff7b72"
BLUE = "#7cc7ff"
YELLOW = "#e3b341"
DIVIDER = "#263042"


def load_font(size: int):
    candidates = [
        "/System/Library/Fonts/Supplemental/Menlo.ttc",
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/SFNSMono.ttf",
    ]
    for candidate in candidates:
        path = Path(candidate)
        if path.exists():
            return ImageFont.truetype(str(path), size=size)
    return ImageFont.load_default()


FONT_14 = load_font(14)
FONT_18 = load_font(18)
FONT_22 = load_font(22)
FONT_26 = load_font(26)
FONT_34 = load_font(34)
FONT_44 = load_font(44)


def lerp(a, b, t):
    return a + (b - a) * t


def rounded(draw, box, fill, radius=24, outline=None, width=1):
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def draw_chip(draw, x, y, text, fill, color=TEXT):
    w = draw.textlength(text, font=FONT_18) + 24
    rounded(draw, (x, y, x + w, y + 34), fill=fill, radius=12)
    draw.text((x + 12, y + 7), text, font=FONT_18, fill=color)
    return x + w


def draw_frame(stage):
    image = Image.new("RGB", (WIDTH, HEIGHT), BACKGROUND)
    draw = ImageDraw.Draw(image)

    rounded(draw, (20, 20, WIDTH - 20, HEIGHT - 20), fill=PANEL, radius=28)
    draw.text((44, 40), "service-impact", font=FONT_34, fill=TEXT)
    draw.text((44, 84), "From changed files to the smallest reasonable verification scope", font=FONT_22, fill=MUTED)

    chip_x = 44
    chip_x = draw_chip(draw, chip_x, 124, "Changed: src/events/publisher.rs", fill="#1d293d")
    chip_x += 12
    draw_chip(draw, chip_x, 124, "Mode: strict", fill="#11243d", color=BLUE)

    left = (44, 190, 560, 610)
    right = (600, 190, 1156, 610)
    rounded(draw, left, fill=PANEL_ALT, radius=22, outline=DIVIDER, width=2)
    rounded(draw, right, fill=PANEL_ALT, radius=22, outline=DIVIDER, width=2)

    draw.text((70, 216), "Before", font=FONT_26, fill=RED)
    draw.text((626, 216), "After", font=FONT_26, fill=GREEN)

    draw.text((70, 258), "Default CI scope", font=FONT_18, fill=MUTED)
    draw.text((626, 258), "service-impact output", font=FONT_18, fill=MUTED)

    # Before panel
    services = ["billing-api", "billing-worker", "billing-web", "analytics"]
    draw.text((70, 304), "Run 4 services", font=FONT_44, fill=TEXT)
    draw.text((70, 362), "Estimated CI time: 11.0 min", font=FONT_22, fill=YELLOW)

    for idx, service in enumerate(services):
        y = 418 + idx * 40
        rounded(draw, (70, y, 300, y + 30), fill="#2b1620", radius=10)
        draw.text((84, y + 6), service, font=FONT_18, fill=TEXT)

    draw.text((70, 584), "Too broad. Low explainability.", font=FONT_18, fill=MUTED)

    # After panel stages
    if stage >= 1:
        draw.text((626, 304), "Run 1 service", font=FONT_44, fill=TEXT)
    if stage >= 2:
        draw.text((626, 362), "Estimated CI time: 2.75 min", font=FONT_22, fill=YELLOW)
        draw.text((626, 396), "Minutes saved: 8.25", font=FONT_22, fill=GREEN)
    if stage >= 3:
        rounded(draw, (626, 438, 918, 474), fill="#132a1d", radius=10)
        draw.text((642, 446), "billing-worker", font=FONT_18, fill=TEXT)
    if stage >= 4:
        rounded(draw, (626, 486, 1070, 522), fill="#11243d", radius=10)
        draw.text((642, 494), "reason: consumes event invoice-issued", font=FONT_18, fill=BLUE)
    if stage >= 5:
        rounded(draw, (626, 534, 880, 570), fill="#1d293d", radius=10)
        draw.text((642, 542), "hook: worker-smoke", font=FONT_18, fill=TEXT)
    if stage >= 6:
        draw.text((626, 584), "Smallest reasonable verification scope.", font=FONT_18, fill=MUTED)

    footer = "Run fewer checks, with explicit reasons."
    draw.text((44, 646), footer, font=FONT_22, fill=MUTED)
    return image


def build_frames():
    frames = []
    for _ in range(8):
        frames.append((draw_frame(0), 120))
    for stage in range(1, 7):
        for _ in range(6):
            frames.append((draw_frame(stage), 140))
    for _ in range(18):
        frames.append((draw_frame(6), 120))
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
