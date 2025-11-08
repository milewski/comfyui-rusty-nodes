#!/usr/bin/env python3
"""
Build a Python wheel from a Rust crate and install it.

The script now works on Windows (and Unix) by:

* Using the Windows installer (`rustup-init.exe`) instead of `sh`/`curl`
* Replacing Unix‐only commands (`sh`, `rm`) with portable Python equivalents
* Detecting the current platform and adjusting paths accordingly
"""

import os
import sys
import subprocess
import platform
import glob
import shutil
import urllib.request

# --------------------------------------------------------------------------- #
#  Optional: use the stdlib tomllib (Python 3.11+), fallback to toml
try:
    import tomllib
except ImportError:
    import toml as tomllib

# --------------------------------------------------------------------------- #
def run_command(cmd, **kwargs):
    """
    Wrapper around subprocess.check_call that prints the command.
    """
    print(f"📡 Running: {' '.join(map(str, cmd))}")
    subprocess.check_call(cmd, **kwargs)

# --------------------------------------------------------------------------- #
def install_rust():
    """
    Make sure Rust (rustc + cargo) is available.
    On Windows we download and run `rustup-init.exe`.
    On Unix we fall back to the standard shell installer.
    """
    # Helper: test if a command is already on PATH
    def has_command(name):
        return shutil.which(name) is not None

    if has_command("rustc"):
        print("✅ Rust toolchain already installed")
        return

    print("📦 Installing Rust toolchain via rustup...")

    system = platform.system()
    if system == "Windows":
        # Windows installer
        installer = "rustup-init.exe"
        url = "https://win.rustup.rs"

        # Download the installer if it doesn't exist
        if not os.path.exists(installer):
            print(f"🔽 Downloading Rust installer from {url}")
            urllib.request.urlretrieve(url, installer)

        # Run the installer non‑interactive
        run_command([installer, "-y"])

        # Clean up the installer
        os.remove(installer)

        # Ensure the cargo bin directory is on PATH for this process
        cargo_bin = os.path.join(os.path.expanduser("~"), ".cargo", "bin")
        if cargo_bin not in os.environ["PATH"]:
            os.environ["PATH"] += os.pathsep + cargo_bin

    else:
        # Unix‑like (Linux, macOS, WSL, etc.)
        # We use the official rustup bootstrap script
        # First check if `curl` is available
        if has_command("curl"):
            run_command(["curl", "-sSf", "https://sh.rustup.rs", "-o", "rustup-init.sh"])
            run_command(["sh", "rustup-init.sh", "-y"])
            os.remove("rustup-init.sh")
        elif has_command("wget"):
            run_command(["wget", "-qO", "rustup-init.sh", "https://sh.rustup.rs"])
            run_command(["sh", "rustup-init.sh", "-y"])
            os.remove("rustup-init.sh")
        else:
            raise RuntimeError("No downloader found (curl or wget). Install one or run rustup manually.")

        # Add cargo bin to PATH for this process
        cargo_bin = os.path.expanduser("~/.cargo/bin")
        os.environ["PATH"] += os.pathsep + cargo_bin

    # Verify we now have rustc
    if not has_command("rustc"):
        raise RuntimeError("Rust installation failed – rustc not found after install.")


# --------------------------------------------------------------------------- #
def install_msvc_linker():
    """
    On Windows, install Visual Studio Build Tools if linker is missing.
    """
    if platform.system() != "Windows":
        return

    def has_linker():
        return shutil.which("link.exe") is not None

    if has_linker():
        print("✅ MSVC linker already installed")
        return

    print("📦 Installing Visual Studio Build Tools (for linker)...")

    # Download and run the Build Tools installer
    installer_url = "https://aka.ms/vs/17/release/vs_buildtools.exe"
    installer_path = "vs_buildtools.exe"

    if not os.path.exists(installer_path):
        print(f"🔽 Downloading Visual Studio Build Tools from {installer_url}")
        urllib.request.urlretrieve(installer_url, installer_path)

    # Install with minimal C++ build tools
    run_command([
        installer_path,
        "--quiet",
        "--wait",
        "--norestart",
        "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "--add", "Microsoft.VisualStudio.Component.Windows10SDK.19041"
    ])

    # Clean up installer
    os.remove(installer_path)

# --------------------------------------------------------------------------- #
def build():
    """
    Build the wheel using maturin.  `maturin` is installed automatically
    below if missing.
    """
    run_command([sys.executable, "-m", "maturin", "build", "--release", "--features", "extension-module"])

# --------------------------------------------------------------------------- #
def install_wheels():
    """
    Install all wheels that were just built into the current Python environment.
    """
    wheels = glob.glob("target/wheels/*.whl")
    if not wheels:
        print("⚠️  No wheel files found in target/wheels")
        return

    for wheel in wheels:
        print(f"📦 Installing {wheel} …")
        run_command([sys.executable, "-m", "pip", "install", "--force-reinstall", wheel])


# --------------------------------------------------------------------------- #
def clean_up():
    """
    Run cargo clean to trim dependencies and remove temporary installers.
    """
    print("🧹 Running cargo clean to trim dependencies…")
    try:
        run_command(["cargo", "clean"])
    except FileNotFoundError:
        print("⚠️  cargo not found (is Rust installed?)")

    # Remove the temporary rustup installer if it survived
    system = platform.system()
    installer = "rustup-init.exe" if system == "Windows" else "rustup-init.sh"
    if os.path.exists(installer):
        try:
            os.remove(installer)
            print(f"✅ Removed temporary installer {installer}")
        except Exception:
            pass


# --------------------------------------------------------------------------- #
def generate_init():
    """
    Create a minimal __init__.py that re‑exports the Rust crate's symbols.
    """
    print("📝 Generating __init__.py …")
    cargo_toml = "Cargo.toml"

    try:
        # tomllib (stdlib, Python ≥ 3.11) expects a binary file; the third‑party
        # toml library expects a text file. Handle both cleanly.
        if getattr(tomllib, "__name__", "") == "tomllib":
            with open(cargo_toml, "rb") as f:
                data = tomllib.load(f)
        else:
            with open(cargo_toml, "r", encoding="utf-8") as f:
                data = tomllib.load(f)
        crate_name = data.get("lib", {}).get("name")
    except Exception as e:
        print(f"⚠️  Failed to read {cargo_toml}: {e}")
        return

    if not crate_name:
        print("⚠️  Could not find [lib] name in Cargo.toml")
        return

    init_path = "__init__.py"
    with open(init_path, "w", encoding="utf-8") as f:
        f.write(f"# Auto-generated __init__.py for crate {crate_name}\n")
        f.write(f"from {crate_name} import *\n")

    print(f"✅ {init_path} generated for crate '{crate_name}'")


# --------------------------------------------------------------------------- #
if __name__ == "__main__":
    # Ensure maturin is available (install it if missing)
    try:
        import maturin
    except ImportError:
        run_command([sys.executable, "-m", "pip", "install", "maturin"])

    # Install Rust
    install_rust()

    # Install MSVC linker on Windows if needed
    install_msvc_linker()

    # Build
    build()
    install_wheels()
    clean_up()
    generate_init()
