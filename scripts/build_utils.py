import os
import subprocess
from pathlib import Path


def run_and_check(
    cmd_list: list[str],
    cwd: Path = None,
    env_additions: dict[str, str] | None = None,
) -> None:
    print(" ".join(cmd_list), cwd)
    env = os.environ.copy()
    if env_additions:
        for k, v in env_additions.items():
            env[k] = v

    status = subprocess.run(
        cmd_list,
        cwd=cwd.as_posix() if cwd else None,
        env=env if env_additions else None,
    )
    if status.returncode != 0:
        raise RuntimeError(f'Running "{" ".join(cmd_list)}" failed with exit code {status.returncode}')
