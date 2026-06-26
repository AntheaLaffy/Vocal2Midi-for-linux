from __future__ import annotations

import os
import platform
import shutil
from pathlib import Path


PORTABLE_ROOT_ENV = "V2M_PORTABLE_ROOT"


def get_portable_root(project_root: Path | None = None) -> Path | None:
    raw = os.environ.get(PORTABLE_ROOT_ENV, "").strip()
    if raw:
        return Path(raw).resolve()
    if project_root is None:
        return None
    return None


def resolve_settings_path(project_root: Path | None = None) -> Path | None:
    """Return the project-local settings INI path.

    On all platforms the canonical config file lives at
    ``<project_root>/settings/vocal2midi.ini``.
    """
    if project_root is not None:
        return project_root / "settings" / "vocal2midi.ini"
    # Fallback for callers that don't pass project_root (should not happen in
    # practice but keeps the return-type annotation honest).
    portable_root = get_portable_root(project_root)
    if portable_root is not None:
        return portable_root / "settings" / "vocal2midi.ini"
    return None


def _legacy_qsettings_path() -> Path | None:
    """Return the OS-native QSettings path used before unification.

    - Linux:   ``~/.config/GAME_Extractor/Vocal2Midi.conf``
    - macOS:   ``~/Library/Application Support/GAME_Extractor/Vocal2Midi.conf``
    - Windows: Registry-based (no file path) — nothing to migrate.

    Returns ``None`` on Windows or if the path cannot be determined.
    """
    system = platform.system().lower()
    if system == "windows":
        return None
    if system == "linux":
        base = Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config"))
    else:  # darwin / macOS
        base = Path.home() / "Library" / "Application Support"
    return base / "GAME_Extractor" / "Vocal2Midi.conf"


def migrate_legacy_qsettings(project_root: Path) -> Path:
    """Ensure the project-local INI exists and migrate old system settings.

    1. If the legacy OS-native config exists **and** the local INI is empty
       or missing, copy legacy settings into the local INI.
    2. Remove the legacy config file (and its parent directory if empty).

    Returns the resolved local INI path.
    """
    settings_path = resolve_settings_path(project_root)
    settings_path.parent.mkdir(parents=True, exist_ok=True)

    legacy_path = _legacy_qsettings_path()
    if legacy_path is not None and legacy_path.is_file():
        # Only migrate if the local INI doesn't already have user data.
        needs_migration = not settings_path.is_file() or settings_path.stat().st_size == 0
        if needs_migration:
            shutil.copy2(str(legacy_path), str(settings_path))
            print(f"[Settings] Migrated legacy config from {legacy_path}")
        # Remove legacy file regardless — avoid future confusion.
        legacy_path.unlink(missing_ok=True)
        parent = legacy_path.parent
        try:
            if parent.exists() and not any(parent.iterdir()):
                parent.rmdir()
        except OSError:
            pass

    return settings_path


def default_output_dir(project_root: Path | None = None) -> Path:
    portable_root = get_portable_root(project_root)
    if portable_root is not None:
        return portable_root / "outputs"
    return Path.home() / "Desktop"
