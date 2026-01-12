#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "python-roborock>=0.24.1",
# ]
# ///
import asyncio
import enum
import json
import subprocess
import sys
import types
from pathlib import Path

HOST = "10.0.1.150"
MODEL = "roborock.vacuum.a27"
PRODUCT_ID = "product-id-123"
NAME = "Roborock"
QUEUE_TIMEOUT_SECONDS = 15
FORCE_LOCAL_PROTOCOL = "L01"
CACHE_FILES = ("~/.roborock", "~/.roborock.cache")
DUID_OP_REF = "op://private/vacuum roborock/username"
LOCAL_KEY_OP_REF = "op://private/vacuum roborock/credential"
EMAIL_OP_REF = "op://private/Roborock/username"
PASSWORD_OP_REF = "op://private/Roborock/password"

def _add_python_roborock_to_path() -> Path:
    repo_root = Path(__file__).resolve().parents[1]
    python_roborock = repo_root / ".tmp" / "python-roborock"
    if not python_roborock.exists():
        raise SystemExit(f"python-roborock not found at {python_roborock}")
    sys.path.insert(0, str(python_roborock))
    return python_roborock


def _load_roborock_minimal(python_roborock: Path) -> None:
    package = types.ModuleType("roborock")
    package.__path__ = [str(python_roborock / "roborock")]
    sys.modules.setdefault("roborock", package)

    import importlib

    exceptions = importlib.import_module("roborock.exceptions")
    containers = importlib.import_module("roborock.data.containers")
    code_mappings = importlib.import_module("roborock.data.code_mappings")
    v1_containers = importlib.import_module("roborock.data.v1.v1_containers")
    v1_code_mappings = importlib.import_module("roborock.data.v1.v1_code_mappings")
    roborock_typing = importlib.import_module("roborock.roborock_typing")

    package.CommandVacuumError = exceptions.CommandVacuumError
    package.RoborockException = exceptions.RoborockException
    package.UnknownMethodError = exceptions.UnknownMethodError
    package.VacuumError = exceptions.VacuumError
    package.DeviceData = containers.DeviceData
    package.RoborockCommand = roborock_typing.RoborockCommand
    package.RoborockEnum = code_mappings.RoborockEnum
    package.HomeDataSchedule = containers.HomeDataSchedule
    package.AppInitStatus = v1_containers.AppInitStatus
    package.RoborockDockTypeCode = v1_code_mappings.RoborockDockTypeCode
    package.DockSummary = roborock_typing.DockSummary
    package.DeviceProp = roborock_typing.DeviceProp


def _read_op_secret(ref: str, label: str) -> str:
    try:
        result = subprocess.run(
            ["op", "read", ref, "-n"],
            check=True,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as exc:
        raise SystemExit("1Password CLI (op) not found in PATH.") from exc
    except subprocess.CalledProcessError as exc:
        stderr = exc.stderr.strip() if exc.stderr else "unknown error"
        raise SystemExit(f"Failed to read {label} from 1Password: {stderr}") from exc

    value = result.stdout.strip()
    if not value:
        raise SystemExit(f"Empty {label} from 1Password.")
    return value


def _load_room_name_map() -> dict[str, str]:
    for cache_file in CACHE_FILES:
        cache_path = Path(cache_file).expanduser()
        if not cache_path.is_file():
            continue
        try:
            data = json.loads(cache_path.read_text())
        except json.JSONDecodeError:
            continue

        cache_data = data.get("cache_data") or {}
        home_data = cache_data.get("home_data") or {}
        rooms = home_data.get("rooms") or []
        name_map = {
            str(room.get("id")): str(room.get("name"))
            for room in rooms
            if room.get("id") is not None and room.get("name")
        }
        if name_map:
            return name_map
    return {}


def _load_connection_cache() -> tuple[Path, dict] | None:
    for cache_file in CACHE_FILES:
        cache_path = Path(cache_file).expanduser()
        if not cache_path.is_file():
            continue
        try:
            data = json.loads(cache_path.read_text())
        except json.JSONDecodeError:
            continue
        if data.get("user_data") and data.get("email"):
            return cache_path, data
    return None


async def _refresh_home_data_cache() -> None:
    cache = _load_connection_cache()
    from roborock.data.containers import UserData  # type: ignore
    from roborock.exceptions import RoborockException  # type: ignore
    from roborock.web_api import RoborockApiClient  # type: ignore
    import aiohttp

    cache_path = Path(CACHE_FILES[0]).expanduser()
    data = {}
    if cache is not None:
        cache_path, data = cache

    email = data.get("email")
    user_data_raw = data.get("user_data")
    user_data = None
    if email and isinstance(user_data_raw, dict):
        user_data = UserData.from_dict(user_data_raw)

    if user_data is None:
        email = _read_op_secret(EMAIL_OP_REF, "email")
        password = _read_op_secret(PASSWORD_OP_REF, "password")
        async with aiohttp.ClientSession() as session:
            client = RoborockApiClient(email, session=session)
            try:
                user_data = await client.pass_login(password)
            except RoborockException as exc:
                if "two step" not in str(exc).lower():
                    raise
                for attempt in range(3):
                    try:
                        await client.request_code()
                        break
                    except Exception as inner_exc:
                        if "RateLimit" not in type(inner_exc).__name__:
                            raise
                        if attempt == 2:
                            raise SystemExit(
                                "Roborock login rate limit hit. Please wait a minute and retry."
                            ) from inner_exc
                        await asyncio.sleep(1.5)
                code = input("Roborock email code: ").strip()
                if not code:
                    raise SystemExit("Email code is required for two-step validation.")
                user_data = await client.code_login(code)
            home_data = await client.get_home_data_v3(user_data)
    else:
        async with aiohttp.ClientSession() as session:
            client = RoborockApiClient(email, session=session)
            home_data = await client.get_home_data_v3(user_data)

    cache_data = data.get("cache_data") or {}
    cache_data["home_data"] = home_data.as_dict()
    data["cache_data"] = cache_data
    data["email"] = email
    data["user_data"] = user_data.as_dict()
    cache_path.write_text(json.dumps(data, indent=4))


async def _run() -> int:
    if not hasattr(enum, "StrEnum"):
        class StrEnum(str, enum.Enum):
            pass

        enum.StrEnum = StrEnum  # type: ignore[attr-defined]

    python_roborock = _add_python_roborock_to_path()
    _load_roborock_minimal(python_roborock)

    from roborock.data.containers import DeviceData, HomeDataDevice  # type: ignore
    from roborock.version_1_apis import RoborockLocalClientV1  # type: ignore

    duid = _read_op_secret(DUID_OP_REF, "DUID")
    local_key = _read_op_secret(LOCAL_KEY_OP_REF, "local key")

    await _refresh_home_data_cache()

    device = HomeDataDevice(
        duid=duid,
        name=NAME,
        local_key=local_key,
        product_id=PRODUCT_ID,
    )
    device_data = DeviceData(device, model=MODEL, host=HOST)

    from roborock.protocols.v1_protocol import LocalProtocolVersion  # type: ignore

    protocol_version = None
    if FORCE_LOCAL_PROTOCOL:
        protocol_version = LocalProtocolVersion[FORCE_LOCAL_PROTOCOL]

    client = RoborockLocalClientV1(
        device_data,
        queue_timeout=QUEUE_TIMEOUT_SECONDS,
        local_protocol_version=protocol_version,
    )
    try:
        mapping = await client.get_room_mapping()
    finally:
        await client.async_disconnect()

    if not mapping:
        print("No room mapping returned.")
        return 1

    name_map = _load_room_name_map()
    for room in mapping:
        name = name_map.get(str(room.iot_id))
        if name:
            print(f"{room.segment_id}\t{name}\t({room.iot_id})")
        else:
            print(f"{room.segment_id}\t{room.iot_id}")

    return 0


def main() -> None:
    raise SystemExit(asyncio.run(_run()))


if __name__ == "__main__":
    main()
