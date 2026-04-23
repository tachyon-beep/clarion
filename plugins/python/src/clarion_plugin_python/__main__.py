"""Entry point for the clarion-plugin-python executable.

Task 1 ships a bootstrap that writes a version stamp to stderr and exits 0,
proving the pip-installed binary is on ``$PATH``. Task 2 replaces the body
with the JSON-RPC server loop that speaks WP2's L4 method set.
"""

from __future__ import annotations

import sys

from clarion_plugin_python import __version__


def main() -> int:
    """Write the version stamp to stderr and exit cleanly."""
    sys.stderr.write(f"clarion-plugin-python {__version__}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
