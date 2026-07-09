from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "src-tauri" / "icons" / "icon.ico"
FONT_CANDIDATES = [
    Path("C:/Windows/Fonts/seguisb.ttf"),
    Path("C:/Windows/Fonts/arialbd.ttf"),
]


def load_font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for candidate in FONT_CANDIDATES:
        if candidate.exists():
            return ImageFont.truetype(candidate, size=size)
    return ImageFont.load_default()


def build_icon(size: int) -> Image.Image:
    scale = size / 256
    icon = Image.new("RGBA", (size, size), (0, 0, 0, 0))

    rect = [
        round(32 * scale),
        round(28 * scale),
        round(224 * scale),
        round(228 * scale),
    ]
    radius = round(52 * scale)

    shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)
    shadow_draw.rounded_rectangle(rect, radius=radius, fill=(7, 16, 21, 58))
    shadow = shadow.filter(ImageFilter.GaussianBlur(max(1, round(10 * scale))))
    icon.alpha_composite(shadow, (0, round(8 * scale)))

    gradient = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    pixels = gradient.load()
    for y in range(size):
        for x in range(size):
            t = max(0, min(1, (x + y) / (2 * (size - 1))))
            red = round(0x43 * (1 - t) + 0x16 * t)
            green = round(0xE0 * (1 - t) + 0xA9 * t)
            blue = round(0xCF * (1 - t) + 0x9D * t)
            pixels[x, y] = (red, green, blue, 255)

    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(rect, radius=radius, fill=255)
    gradient.putalpha(mask)
    icon.alpha_composite(gradient)

    draw = ImageDraw.Draw(icon)
    font = load_font(max(10, round(84 * scale)))
    label = "AI"
    bbox = draw.textbbox((0, 0), label, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    draw.text(
        ((size - text_width) / 2, round(78 * scale) - text_height / 2 - bbox[1]),
        label,
        font=font,
        fill=(6, 17, 22, 255),
    )
    draw.rounded_rectangle(
        [
            round(55 * scale),
            round(188 * scale),
            round(201 * scale),
            round(202 * scale),
        ],
        radius=round(7 * scale),
        fill=(232, 255, 251, 224),
    )
    return icon


def main() -> None:
    sizes = [16, 24, 32, 48, 64, 128, 256]
    images = [build_icon(size) for size in sizes]
    images[-1].save(
        OUT,
        format="ICO",
        sizes=[(size, size) for size in sizes],
        append_images=images[:-1],
    )
    print(OUT)


if __name__ == "__main__":
    main()
