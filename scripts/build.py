import os
import shutil
from pathlib import Path

from build_utils import run_and_check


_REPO_ROOT = Path(__file__).parents[1]
_EFI_ROOT = _REPO_ROOT / "esp"
_ARCH_TARGET_NAME = "x86_64-unknown-uefi"


def run_in_qemu():
    run_and_check(
        [
            "arch",
            "-arm64",
            "sudo",
            "-S",
            "/opt/homebrew/bin/qemu-system-x86_64",
            # OVMF: Open source UEFI firmware for QEMU
            "-bios",
            (_REPO_ROOT / "ubuntu_OVMF.fd").as_posix(),
            "-monitor", "stdio",
            "-m", "4G",
            "-vga", "virtio",
            "-netdev", "vmnet-shared,id=vmnet",
            "-device", "rtl8139,netdev=vmnet",
            "-drive", "format=raw,file=fat:rw:esp",
            "-debugcon", "file:debug.log",
            "-global", "isa-debugcon.iobase=0x402",
            # Provide a VirtIO RNG peripheral
            "-device",
            "virtio-rng-pci",
            "-device",
            "virtio-mouse-pci",

            "-usb", "-device", "usb-mouse",
            # Connect a FAT filesystem that hosts the UEFI application
            "-drive",
            f"format=raw,file=fat:rw:{_EFI_ROOT.relative_to(_REPO_ROOT).as_posix()}",
        ],
        cwd=_REPO_ROOT,
    )


def run_hosted():
    run_and_check(
        [
            "cargo", "run",
        ]
    )


def compile_and_run():
    built_uefi_app_path = _REPO_ROOT / "target" / _ARCH_TARGET_NAME / "debug" / "uefirc.efi"
    staged_uefi_app_path = _EFI_ROOT / "efi" / "boot" / "bootx64.efi"

    # Remove the build products so we're sure they were rebuilt successfully
    build_products = [built_uefi_app_path, staged_uefi_app_path]
    for build_product in build_products:
        if build_product.exists():
            print(f'Removing {build_product.as_posix()} prior to build...')
            os.unlink(build_product.as_posix())

    run_and_check(
        [
            "cargo",
            "build",
            "--features", "run_in_uefi",
            #"--features", "run_hosted",
            "--target",
            _ARCH_TARGET_NAME,
        ],
        cwd=_REPO_ROOT,
    )
    if not built_uefi_app_path.exists():
        raise RuntimeError(f'Expected build product to exist: {built_uefi_app_path.as_posix()}')

    shutil.copy(
        built_uefi_app_path.as_posix(),
        staged_uefi_app_path.as_posix(),
    )
    if not staged_uefi_app_path.exists():
        raise RuntimeError(f'Expected staged app to exist: {staged_uefi_app_path.as_posix()}')

    run_in_qemu()
    # run_hosted()


if __name__ == '__main__':
    compile_and_run()
