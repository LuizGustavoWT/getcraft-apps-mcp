#!/usr/bin/env python3
"""Validate the distributable plugin without third-party Python packages (3.11+)."""

from __future__ import annotations

import argparse
import json
from pathlib import Path, PurePosixPath
import re
import shlex
import sys
import tomllib


PLUGIN_NAME = "getcraft-apps-mcp"
MARKETPLACE_NAME = "getcraft-apps"
APPS = {
    "filmcraft", "photocraft", "pdfcraft", "designcraft",
    "effectcraft", "lightcraft", "vectorcraft",
}


class PackageError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PackageError(message)


def unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def read_json(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
    except (OSError, ValueError) as error:
        raise PackageError(f"{path}: {error}") from error
    require(isinstance(value, dict), f"{path}: expected a JSON object")
    require("[TODO:" not in json.dumps(value), f"{path}: unfinished scaffold placeholder")
    return value


def local_directory(root: Path, value: object, context: str) -> Path:
    require(isinstance(value, str) and bool(value), f"{context}: expected a relative path")
    relative = Path(value)
    require(not relative.is_absolute(), f"{context}: source must be repository-relative")
    target = (root / relative).resolve()
    require(target.is_relative_to(root), f"{context}: path escapes repository root")
    require(target.is_dir(), f"{context}: directory does not exist: {target}")
    return target


def scalar(value: str, continuation: list[str]) -> str:
    """Read the plain/quoted/block scalars used by SKILL frontmatter, not general YAML."""
    value = value.strip()
    if value in {">", ">-", ">+", "|", "|-", "|+"}:
        return "\n".join(line.strip() for line in continuation).strip()
    if value.startswith('"'):
        decoded = json.loads(value)
        require(isinstance(decoded, str), "frontmatter scalar must be a string")
        return decoded
    if value.startswith("'"):
        require(value.endswith("'") and len(value) >= 2, "unterminated frontmatter string")
        return value[1:-1].replace("''", "'")
    return value


def skill_fields(text: str) -> dict[str, str]:
    lines = text.splitlines()
    require(bool(lines) and lines[0] == "---", "SKILL.md must begin with YAML frontmatter")
    end = next((i for i in range(1, len(lines)) if lines[i] == "---"), None)
    require(end is not None, "SKILL.md frontmatter has no closing delimiter")
    require(any(line.strip() for line in lines[end + 1:]), "SKILL.md has no instructions")
    fields: dict[str, str] = {}
    frontmatter = lines[1:end]
    for index, line in enumerate(frontmatter):
        match = re.fullmatch(r"([A-Za-z_][\w-]*):\s*(.*)", line)
        if not match or match[1] not in {"name", "description"}:
            continue
        key = match[1]
        require(key not in fields, f"duplicate frontmatter field: {key}")
        continuation = []
        for following in frontmatter[index + 1:]:
            if following and not following[0].isspace():
                break
            continuation.append(following)
        fields[key] = scalar(match[2], continuation)
    for key in ("name", "description"):
        require(bool(fields.get(key, "").strip()), f"SKILL.md needs a nonempty {key}")
    return fields


def check_sparse_examples(path: Path, plugin_relative: Path) -> None:
    """Interpret shell examples so whitespace, quotes and continuations do not matter."""
    text = path.read_text(encoding="utf-8")
    blocks = re.findall(r"^```(?:sh|bash|shell|zsh)?\s*\n(.*?)^```", text, re.M | re.S)
    required = (PurePosixPath(".agents/plugins"), PurePosixPath(plugin_relative.as_posix()))
    found = 0
    for block in blocks:
        for line in re.sub(r"\\\r?\n", " ", block).splitlines():
            tokens = shlex.split(line, comments=True)
            if tokens[:1] == ["$"]:
                tokens = tokens[1:]
            if tokens[:4] != ["codex", "plugin", "marketplace", "add"]:
                continue
            found += 1
            sparse = []
            index = 4
            while index < len(tokens):
                token = tokens[index]
                if token == "--sparse":
                    index += 1
                    require(index < len(tokens), f"{path.name}: --sparse has no directory")
                    sparse.append(PurePosixPath(tokens[index]))
                elif token.startswith("--sparse="):
                    sparse.append(PurePosixPath(token.split("=", 1)[1]))
                index += 1
            # A full checkout is safe too. Only partial checkouts can omit plugin sources.
            if sparse:
                for directory in sparse:
                    require(not directory.is_absolute() and ".." not in directory.parts,
                            f"{path.name}: invalid sparse directory {directory}")
                for directory in required:
                    require(any(directory.is_relative_to(parent) for parent in sparse),
                            f"{path.name}: marketplace example omits {directory} from sparse checkout")
    require(found > 0, f"{path.name}: no Codex marketplace installation example found")


def check_package(root: Path) -> str:
    root = root.resolve()
    cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["package"]
    require(cargo.get("name") == PLUGIN_NAME, "Cargo package name does not match plugin name")
    version = cargo.get("version")
    require(isinstance(version, str) and bool(version), "Cargo package needs a version")
    locked = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))["package"]
    own = [item for item in locked if item.get("name") == PLUGIN_NAME]
    require(len(own) == 1 and own[0].get("version") == version,
            "Cargo.lock package version differs from Cargo.toml; regenerate the lockfile")

    plugin = local_directory(root, f"plugins/{PLUGIN_NAME}", "plugin")
    manifests = ("plugin.json", ".codex-plugin/plugin.json", ".claude-plugin/plugin.json")
    for relative in manifests:
        path = plugin / relative
        manifest = read_json(path)
        require(manifest.get("name") == PLUGIN_NAME, f"{relative}: wrong plugin name")
        require(manifest.get("version") == version, f"{relative}: version must be {version}")
        require(bool(manifest.get("description")), f"{relative}: description is required")
        if relative == ".codex-plugin/plugin.json":
            require(local_directory(plugin, manifest.get("skills"), relative) == plugin / "skills",
                    f"{relative}: skills must resolve to the packaged skills directory")
            require(manifest.get("mcpServers") == "./.mcp.json",
                    f"{relative}: mcpServers must reference the packaged .mcp.json")

    for relative in (".agents/plugins/marketplace.json", ".claude-plugin/marketplace.json"):
        marketplace = read_json(root / relative)
        require(marketplace.get("name") == MARKETPLACE_NAME, f"{relative}: wrong marketplace name")
        entries = marketplace.get("plugins")
        require(isinstance(entries, list) and bool(entries), f"{relative}: plugins must be a nonempty list")
        seen = set()
        for entry in entries:
            require(isinstance(entry, dict), f"{relative}: invalid plugin entry")
            name = entry.get("name")
            require(isinstance(name, str) and name not in seen, f"{relative}: invalid/duplicate plugin name")
            seen.add(name)
            source = entry.get("source")
            if relative.startswith(".agents/"):
                require(isinstance(source, dict) and source.get("source") == "local",
                        f"{relative}: expected a local source object")
                source = source.get("path")
            target = local_directory(root, source, f"{relative}: {name}")
            require(read_json(target / "plugin.json").get("name") == name,
                    f"{relative}: source manifest name does not match {name}")
            if name == PLUGIN_NAME:
                require(target == plugin, f"{relative}: source points to a different plugin")
                require(entry.get("version", version) == version, f"{relative}: stale version")
        require(PLUGIN_NAME in seen, f"{relative}: expected plugin is missing")

    configs = [read_json(plugin / relative) for relative in ("mcp.json", ".mcp.json")]
    require(configs[0] == configs[1], "mcp.json and .mcp.json differ")
    for config in configs:
        servers = config.get("mcpServers")
        require(isinstance(servers, dict) and set(servers) == {"getcraft_apps"},
                "MCP config must contain exactly the getcraft_apps server")
        server = servers["getcraft_apps"]
        require(isinstance(server, dict), "getcraft_apps MCP server must be an object")
        require(server.get("type") == "stdio", "getcraft_apps MCP transport must be stdio")
        require(server.get("command") == "getcraft-mcp", "wrong MCP command")
        require(server.get("args") == ["serve"], "MCP arguments must be ['serve']")

    skill_paths = list((plugin / "skills").glob("*/SKILL.md"))
    require({path.parent.name for path in skill_paths} == APPS and len(skill_paths) == len(APPS),
            "the plugin must package exactly the seven supported Craft skills")
    for path in skill_paths:
        require(path.resolve().is_relative_to(plugin), f"{path}: skill escapes plugin directory")
        fields = skill_fields(path.read_text(encoding="utf-8"))
        require(fields["name"] == path.parent.name, f"{path}: frontmatter name differs from directory")
        require("[TODO:" not in path.read_text(encoding="utf-8"), f"{path}: unfinished scaffold")

    for readme in ("README.md", "README.pt-BR.md"):
        check_sparse_examples(root / readme, plugin.relative_to(root))
    return version


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        version = check_package(args.root)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"package: FAIL: {error}", file=sys.stderr)
        return 1
    print(f"package: OK ({PLUGIN_NAME} {version}, two marketplaces, seven skills)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
