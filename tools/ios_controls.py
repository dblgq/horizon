#!/usr/bin/env python3
"""Download the touch controls' textures for the iOS build into prepared resources.

The joystick and button art of `horizon_client_render::touch` is Mojang's, from the
Mojang/bedrock-samples repository at the revision `ios/bedrock-controls.json` pins
(manifest from Vasysik/horizon-android). Mojang's EULA terms cover these files, not
Horizon's MIT license: they are downloaded on the player's machine, checked against the
manifest's SHA-256 and size, and never committed.

    python3 tools/ios_controls.py                       # into .local/horizon-client/26.3
    python3 tools/ios_controls.py --resources DIR
"""
import argparse
import hashlib
import json
import sys
import urllib.request
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "ios" / "bedrock-controls.json"
RAW = "https://raw.githubusercontent.com/Mojang/bedrock-samples/{revision}/{source}"
LIMIT = 1024 * 1024


def fetch(url: str, size: int) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": "horizon-ios-tools"})
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read(size + 1)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--resources", type=Path, default=ROOT / ".local/horizon-client/26.3")
    args = parser.parse_args()
    if not (args.resources / "horizon-resources.json").is_file():
        print(f"{args.resources} is not prepared: horizon-prepare --prepare first", file=sys.stderr)
        return 1
    manifest = json.loads(MANIFEST.read_text())
    revision = manifest["revision"]
    target = args.resources / "assets/minecraft/textures/gui/bedrock"
    fetched = kept = 0
    for entry in manifest["files"]:
        relative = PurePosixPath(entry["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise SystemExit(f"bad path in the manifest: {relative}")
        size, digest = int(entry["size"]), entry["sha256"]
        if size > LIMIT:
            raise SystemExit(f"{relative}: {size} bytes is more than a texture should be")
        file = target.joinpath(*relative.parts)
        if file.is_file():
            data = file.read_bytes()
            if len(data) == size and hashlib.sha256(data).hexdigest() == digest:
                kept += 1
                continue
        source = "LICENSE.md" if entry["path"] == "LICENSE.md" else f"resource_pack/textures/{entry['source']}"
        data = fetch(RAW.format(revision=revision, source=source), size)
        if len(data) != size or hashlib.sha256(data).hexdigest() != digest:
            raise SystemExit(f"{relative}: checksum mismatch, not written")
        file.parent.mkdir(parents=True, exist_ok=True)
        partial = file.with_suffix(file.suffix + ".partial")
        partial.write_bytes(data)
        partial.replace(file)
        fetched += 1
    print(f"Touch control textures in {target}: {fetched} downloaded, {kept} already there")
    return 0


if __name__ == "__main__":
    sys.exit(main())
