#!/usr/bin/env python3
"""Smoke-test a built GetCraft MCP binary; --native also checks installed CLIs read-only."""

from __future__ import annotations

import argparse
from collections import deque
import json
import os
from pathlib import Path
from queue import Empty, Queue
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import uuid


APPS = {
    "filmcraft", "photocraft", "pdfcraft", "designcraft",
    "effectcraft", "lightcraft", "vectorcraft",
}
TOOLS = {
    "craft_list_apps", "craft_cli_help", "craft_command_catalog", "craft_run_cli",
    "craft_native_mcp_config", "craft_app_info", "craft_search_commands",
}


class SmokeError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SmokeError(message)


class Client:
    def __init__(self, binary: str, timeout: float, *, without_native: bool = False):
        self.timeout = timeout
        self.next_id = 0
        self.messages: Queue = Queue()
        self.stderr: deque[str] = deque(maxlen=16)
        environment = None
        if without_native:
            # Even on a developer machine, a broken rejection must not start an app.
            missing = unused_path()
            environment = os.environ.copy()
            for app in APPS:
                environment[f"GETCRAFT_{app.upper()}_CLI"] = str(missing / f"{app}-cli")
        self.process = subprocess.Popen(
            [binary, "serve"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, encoding="utf-8", bufsize=1, env=environment,
        )
        self.readers = [
            threading.Thread(target=self.read_stdout, daemon=True),
            threading.Thread(target=self.read_stderr, daemon=True),
        ]
        for reader in self.readers:
            reader.start()

    def read_stdout(self) -> None:
        try:
            while True:
                line = self.process.stdout.readline(4 * 1024 * 1024 + 1)
                if not line:
                    self.messages.put(SmokeError("MCP server closed stdout"))
                    return
                if not line.endswith("\n") or len(line) > 4 * 1024 * 1024:
                    raise SmokeError("MCP stdout line is incomplete or exceeds 4 MiB")
                self.messages.put(json.loads(line))
        except (OSError, ValueError, SmokeError) as error:
            self.messages.put(SmokeError(f"invalid MCP stdout: {error}"))

    def read_stderr(self) -> None:
        try:
            while chunk := self.process.stderr.read(2048):
                self.stderr.append(chunk)
        except (OSError, ValueError):
            pass

    def send(self, message: dict) -> None:
        try:
            self.process.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
            self.process.stdin.flush()
        except (OSError, ValueError) as error:
            raise SmokeError(f"could not send MCP message: {error}") from error

    def notify(self, method: str) -> None:
        self.send({"jsonrpc": "2.0", "method": method})

    def request(self, method: str, params: dict) -> dict:
        self.next_id += 1
        request_id = self.next_id
        self.send({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params})
        deadline = time.monotonic() + self.timeout
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise SmokeError(f"{method} timed out after {self.timeout:g}s")
            try:
                message = self.messages.get(timeout=remaining)
            except Empty as error:
                raise SmokeError(f"{method} timed out after {self.timeout:g}s") from error
            if isinstance(message, Exception):
                raise message
            require(isinstance(message, dict) and message.get("jsonrpc") == "2.0",
                    f"invalid JSON-RPC response: {message!r}")
            if "id" not in message and "method" in message:
                continue  # MCP progress/logging notifications may arrive between replies.
            require(message.get("id") == request_id, f"unexpected response id: {message!r}")
            return message

    def result(self, method: str, params: dict) -> dict:
        response = self.request(method, params)
        require("error" not in response, f"{method}: {response.get('error')}")
        result = response.get("result")
        require(isinstance(result, dict), f"{method}: expected a result object")
        return result

    def tool(self, name: str, arguments: dict, *, expect_error: bool = False):
        response = self.request("tools/call", {"name": name, "arguments": arguments})
        result = response.get("result", {})
        failed = "error" in response or (isinstance(result, dict) and result.get("isError") is True)
        if expect_error:
            require(failed, f"{name} accepted invalid arguments: {arguments}")
            return response
        require(not failed, f"{name} failed: {response}")
        require(isinstance(result, dict), f"{name}: expected a tool result object")
        content = result.get("content", [])
        require(isinstance(content, list), f"{name}: content must be an array")
        text = "\n".join(item.get("text", "") for item in content
                         if isinstance(item, dict) and item.get("type") == "text")
        try:
            return json.loads(text)
        except ValueError as error:
            raise SmokeError(f"{name}: tool text is not JSON: {text[:400]!r}") from error

    def close(self) -> None:
        try:
            self.process.stdin.close()
        except (OSError, ValueError):
            pass
        try:
            self.process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=3)
        for reader in self.readers:
            reader.join(timeout=1)
        self.process.stdout.close()
        self.process.stderr.close()


def check_invocation(result: object, context: str) -> dict:
    require(isinstance(result, dict), f"{context}: expected InvocationResult")
    require(result.get("success") is True, f"{context}: CLI failed: {result}")
    require(result.get("stdout_truncated") is not True, f"{context}: stdout was truncated")
    require(isinstance(result.get("stdout"), str) and bool(result["stdout"].strip()),
            f"{context}: missing CLI stdout")
    return result


def check_search(result: object, app: str, kind: str) -> None:
    fields = {"app", "kind", "total", "matched", "offset", "next_offset", "entries"}
    require(isinstance(result, dict) and fields <= result.keys(), f"{app}: incomplete search result")
    require(result["app"] == app and result["kind"] == kind, f"{app}: wrong search identity")
    for field in ("total", "matched", "offset"):
        require(type(result[field]) is int and result[field] >= 0, f"{app}: invalid {field}")
    require(result["matched"] <= result["total"], f"{app}: matched count exceeds total")
    entries = result["entries"]
    require(isinstance(entries, list) and len(entries) <= 3, f"{app}: search limit was ignored")
    require(result["offset"] == 0 and len(entries) <= result["matched"], f"{app}: invalid result page")
    expected_next = len(entries) if len(entries) < result["matched"] else None
    require(result["next_offset"] == expected_next, f"{app}: invalid next_offset")


def unused_path() -> Path:
    """A unique path for configuration-only checks; no directory or token is created."""
    path = Path(tempfile.gettempdir()).resolve() / f"getcraft-smoke-{uuid.uuid4().hex}"
    require(not path.exists(), "configuration-only test path unexpectedly exists")
    return path


def config_server(result: object, app: str) -> dict:
    require(isinstance(result, dict), f"{app}: native configuration must be an object")
    servers = result.get("mcpServers")
    require(isinstance(servers, dict) and app in servers, f"{app}: native server is missing")
    server = servers[app]
    require(isinstance(server, dict) and server.get("type") == "stdio", f"{app}: MCP transport must be stdio")
    require(isinstance(server.get("command"), str) and bool(server["command"]), f"{app}: executable is missing")
    args = server.get("args")
    require(isinstance(args, list) and args[:1] == ["mcp"], f"{app}: native MCP arguments are missing")
    return server


def flag_values(server: dict, flag: str) -> list[str]:
    args = server["args"]
    positions = [index for index, value in enumerate(args) if value == flag]
    require(all(index + 1 < len(args) for index in positions), f"{flag}: missing argument value")
    return [args[index + 1] for index in positions]


def check_config_options(client: Client) -> None:
    base = unused_path()
    for app, flag in (("pdfcraft", "--root"), ("lightcraft", "--library")):
        workspace = str(base / app)
        server = config_server(client.tool("craft_native_mcp_config", {
            "app": app, "mode": "headless", "workspace": workspace, "compact": True,
        }), app)
        require(server["args"].count("--compact") == 1, f"{app}: compact catalog option was lost")
        require(flag_values(server, flag) == [workspace], f"{app}: workspace was not preserved")

    # A nonexistent token proves configuration does not require reading its contents.
    token = base / "photocraft-control-token"
    server = config_server(client.tool("craft_native_mcp_config", {
        "app": "photocraft", "mode": "bridge", "connect": "127.0.0.1:9876",
        "control_token_file": str(token),
    }), "photocraft")
    require(flag_values(server, "--bridge") == ["127.0.0.1:9876"], "PhotoCraft bridge address was lost")
    require(flag_values(server, "--control-token-file") == [str(token)], "PhotoCraft token path was lost")
    require(not base.exists(), "configuration generation unexpectedly created files")


def check_persistent_rejections(client: Client, apps: list[dict]) -> None:
    require(all(app.get("available") is False for app in apps),
            "no-CLI smoke must isolate native executables before testing persistent commands")
    for app, args in (
        ("filmcraft", ["--demo", "mcp"]),
        ("filmcraft", ["--project", "example.fcproj", "mcp"]),
        ("effectcraft", ["--json", "example.ecproj", "mcp"]),
        ("lightcraft", ["denoise-serve"]),
    ):
        response = client.tool("craft_run_cli", {"app": app, "args": args}, expect_error=True)
        # Reject on the command policy, not merely because no executable was found.
        diagnostic = json.dumps(response)
        require("craft_run_cli" in diagnostic and "craft_native_mcp_config" in diagnostic,
                f"{app}: persistent subcommand was not rejected before executable lookup")


def check_app_info(info: object, app: str) -> str:
    require(isinstance(info, dict), f"{app}: craft_app_info must return an object")
    status = info.get("app")
    require(isinstance(status, dict) and status.get("id") == app and status.get("available") is True,
            f"{app}: installed app identity is missing or unavailable")
    version = info.get("installed_version")
    require(isinstance(version, str) and bool(version.strip()), f"{app}: installed version is empty")
    require("version_error" in info and info["version_error"] is None,
            f"{app}: version probe failed: {info.get('version_error', 'missing field')}")
    require(isinstance(info.get("version_output"), str) and bool(info["version_output"].strip()),
            f"{app}: version probe returned no output")
    audit = info.get("audit")
    require(isinstance(audit, dict) and bool(audit) and "error" not in audit,
            f"{app}: bundled upstream audit is missing or invalid")
    return version


def check_root_options(client: Client, app: str, help_result: dict) -> None:
    """Determine support from this binary's advertised flags, never its version number."""
    help_text = help_result["stdout"] + "\n" + help_result.get("stderr", "")
    base = unused_path()
    if app in {"pdfcraft", "designcraft"}:
        for field, flag in (("read_root", "--automation-read-root"), ("write_root", "--automation-write-root")):
            root = str(base / field)
            supported = flag in help_text
            result = client.tool("craft_native_mcp_config", {"app": app, field: root},
                                 expect_error=not supported)
            if supported:
                server = config_server(result, app)
                require(flag_values(server, flag) == [root], f"{app}: advertised root flag was lost")
    elif app == "effectcraft":
        reads = [str(base / "input-one"), str(base / "input-two")]
        writes = [str(base / "output-one"), str(base / "output-two")]
        supported = all(flag in help_text for flag in ("--automation-read-root", "--automation-write-root"))
        for mode in ("headless", "bridge"):
            arguments = {
                "app": app, "mode": mode, "read_root": reads[0], "read_roots": reads[1:],
                "write_root": writes[0], "write_roots": writes[1:],
            }
            if mode == "bridge":
                arguments["connect"] = "127.0.0.1:9877"
            result = client.tool("craft_native_mcp_config", arguments, expect_error=not supported)
            if supported:
                server = config_server(result, app)
                require(flag_values(server, "--automation-read-root") == reads,
                        f"EffectCraft {mode}: repeated read roots were lost")
                require(flag_values(server, "--automation-write-root") == writes,
                        f"EffectCraft {mode}: repeated write roots were lost")
    require(not base.exists(), f"{app}: root configuration unexpectedly created files")


def smoke(client: Client, native: bool) -> None:
    initialized = client.result("initialize", {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "getcraft-smoke", "version": "1.0.0"},
    })
    require(initialized.get("protocolVersion") == "2025-06-18", "wrong negotiated MCP version")
    require(initialized.get("serverInfo", {}).get("name") == "getcraft-apps-mcp", "wrong server identity")
    client.notify("notifications/initialized")
    listed = client.result("tools/list", {})
    names = [item.get("name") for item in listed.get("tools", [])]
    require(set(names) == TOOLS and len(names) == len(TOOLS), f"unexpected MCP tool surface: {names}")

    apps = client.tool("craft_list_apps", {})
    require(isinstance(apps, list) and len(apps) == len(APPS), "craft_list_apps must return seven apps")
    require({app.get("id") for app in apps} == APPS, "craft_list_apps returned different apps")

    config = client.tool("craft_native_mcp_config", {"app": "vectorcraft", "mode": "headless"})
    server = config_server(config, "vectorcraft")
    require("--headless" in server["args"], "VectorCraft headless config is missing --headless")
    client.tool("craft_native_mcp_config", {
        "app": "filmcraft", "read_root": str(Path.cwd().resolve()),
    }, expect_error=True)
    client.tool("craft_native_mcp_config", {"app": "filmcraft", "mode": "bridge"}, expect_error=True)
    check_config_options(client)
    if not native:
        check_persistent_rejections(client, apps)
    print("smoke: OK (handshake, seven tools, discovery, compact/token/headless configs, invalid requests)", flush=True)

    if native:
        missing = [app["id"] for app in apps if app.get("available") is not True]
        require(not missing, f"--native requires all seven CLIs; missing: {', '.join(missing)}")
        for app in sorted(APPS):
            info = client.tool("craft_app_info", {"app": app})
            version = check_app_info(info, app)
            help_result = check_invocation(client.tool("craft_cli_help", {"app": app, "command": "mcp"}), f"{app} help")
            if app in {"pdfcraft", "designcraft", "effectcraft"}:
                # The gateway's capability guard probes top-level help; so do we.
                if help_result.get("args") != ["--help"]:
                    help_result = check_invocation(client.tool("craft_cli_help", {"app": app}), f"{app} top-level help")
                check_root_options(client, app, help_result)
            catalog = check_invocation(client.tool("craft_command_catalog", {"app": app}), f"{app} catalog")
            decoded = json.loads(catalog["stdout"])
            require(isinstance(decoded, (dict, list)), f"{app}: catalog is not a JSON object/array")
            check_search(client.tool("craft_search_commands", {"app": app, "query": "export", "limit": 3}),
                         app, "commands")
            if app == "lightcraft":
                check_search(client.tool("craft_search_commands", {
                    "app": app, "kind": "controls", "query": "export", "limit": 3,
                }), app, "controls")
            print(f"smoke: OK ({app} {version}: version/audit, help, catalog, search, configuration guards)", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", help="path to the built getcraft-mcp executable")
    parser.add_argument("--native", action="store_true", help="also query all seven installed CLIs (read-only)")
    parser.add_argument("--timeout", type=float, default=45, help="maximum seconds per MCP reply (default: 45)")
    args = parser.parse_args()
    if not 0 < args.timeout <= 120:
        parser.error("--timeout must be between 0 and 120 seconds")
    binary = str(Path(args.binary).resolve()) if Path(args.binary).is_file() else shutil.which(args.binary)
    if binary is None:
        parser.error(f"binary not found: {args.binary}")
    client = None
    failure = None
    try:
        client = Client(binary, args.timeout, without_native=not args.native)
        smoke(client, args.native)
    except (OSError, ValueError, TypeError, KeyError, SmokeError) as error:
        failure = str(error)
    finally:
        if client is not None:
            client.close()
    if failure is None and client.process.returncode != 0:
        failure = f"server exited with status {client.process.returncode}"
    if failure is not None:
        print(f"smoke: FAIL: {failure}", file=sys.stderr)
        if client is not None and client.stderr:
            print("server stderr:\n" + "".join(client.stderr), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
