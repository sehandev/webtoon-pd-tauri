from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from PIL import Image
from psd_tools import PSDImage

SUPPORTED_EXTENSIONS = {".clip", ".psd", ".psb", ".jpg", ".jpeg", ".png", ".webp", ".avif"}


def main() -> None:
    args = parse_args()
    source = args.input.resolve()
    if source.suffix.lower() not in SUPPORTED_EXTENSIONS:
        raise SystemExit(f"unsupported file format: {source.suffix}")

    args.output_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as temp_dir:
        image = load_source_image(source, Path(temp_dir))
        source_width, source_height = image.size
        rendered = resize_to_max_width(image, args.max_width)
        tiles = write_png_tiles(rendered, args.output_dir, args.tile_height)

    manifest = {
        "sourceWidth": source_width,
        "sourceHeight": source_height,
        "renderedWidth": rendered.width,
        "renderedHeight": rendered.height,
        "tiles": tiles,
    }
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(json.dumps(manifest, ensure_ascii=False), encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--max-width", required=True, type=int)
    parser.add_argument("--tile-height", required=True, type=int)
    parser.add_argument("--manifest", required=True, type=Path)
    return parser.parse_args()


def load_source_image(source: Path, temp_dir: Path) -> Image.Image:
    extension = source.suffix.lower()
    if extension == ".clip":
        psd_path = convert_clip_to_psb(source, temp_dir)
        return load_psd_image(psd_path)
    if extension in {".psd", ".psb"}:
        return load_psd_image(source)
    return load_raster_image(source)


def convert_clip_to_psb(source: Path, temp_dir: Path) -> Path:
    output_path = temp_dir / f"{source.stem}.psb"
    converter = resolve_clip_converter()
    command = [
        sys.executable,
        str(converter),
        str(source),
        "--output-psd",
        str(output_path),
        "--psd-version",
        "2",
    ]
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip() or "clip conversion failed"
        raise RuntimeError(f"manual_export_required: {message}")
    return output_path


def resolve_clip_converter() -> Path:
    env_path = os.environ.get("WEBTOON_PD_CLIP_TO_PSD")
    if env_path:
        return Path(env_path)

    local_path = Path(__file__).with_name("clip_to_psd.py")
    if local_path.exists():
        return local_path

    found = shutil.which("clip_to_psd.py") or shutil.which("clip_to_psd")
    if found:
        return Path(found)

    raise RuntimeError("manual_export_required: clip_to_psd converter was not found")


def load_psd_image(source: Path) -> Image.Image:
    psd = PSDImage.open(source)
    image = psd.composite()
    if image is None:
        raise RuntimeError("psd composite failed")
    return normalize_image(image)


def load_raster_image(source: Path) -> Image.Image:
    with Image.open(source) as image:
        image.load()
        return normalize_image(image)


def normalize_image(image: Image.Image) -> Image.Image:
    if image.mode in {"RGB", "RGBA"}:
        return image.copy()
    if "A" in image.getbands():
        return image.convert("RGBA")
    return image.convert("RGB")


def resize_to_max_width(image: Image.Image, max_width: int) -> Image.Image:
    if image.width <= max_width:
        return image
    resized_height = round(image.height * (max_width / image.width))
    return image.resize((max_width, resized_height), Image.Resampling.LANCZOS)


def write_png_tiles(image: Image.Image, output_dir: Path, tile_height: int) -> list[dict[str, Any]]:
    tiles = []
    for index, top in enumerate(range(0, image.height, tile_height)):
        bottom = min(top + tile_height, image.height)
        tile = image.crop((0, top, image.width, bottom))
        path = output_dir / f"tile_{index:04}.png"
        tile.save(path, format="PNG")
        tiles.append(
            {
                "index": index,
                "path": str(path),
                "x": 0,
                "y": top,
                "width": tile.width,
                "height": tile.height,
            }
        )
    return tiles


if __name__ == "__main__":
    main()
