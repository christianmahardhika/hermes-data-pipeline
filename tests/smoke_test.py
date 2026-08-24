#!/usr/bin/env python3
"""
Smoke tests for additive Hermes data pipeline scripts.
Runs WITHOUT pytest - pure stdlib assert, no framework.
Verifies: arango_storage, commodity_collector, correlation_analysis.
"""

import json
import os
import subprocess
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRIPT_DIR = os.path.join(REPO_ROOT, "scripts", "hermes-config")
sys.path.insert(0, SCRIPT_DIR)

FAILURES = []


def check(name: str, fn):
    try:
        fn()
        print(f"  ✅ {name}")
    except Exception as e:
        print(f"  ❌ {name}: {e}")
        FAILURES.append(name)


def test_arango_storage_import():
    import arango_storage
    assert hasattr(arango_storage, "get_arango_storage"), "get_arango_storage missing"


def test_commodity_collector_import():
    import commodity_collector
    assert hasattr(commodity_collector, "CommodityCollector"), "CommodityCollector missing"


def test_correlation_map():
    import correlation_analysis
    assert isinstance(correlation_analysis.CORRELATION_MAP, dict)
    assert "Gold" in correlation_analysis.CORRELATION_MAP


def test_correlation_cli_summary():
    r = subprocess.run(
        [sys.executable, os.path.join(SCRIPT_DIR, "correlation_analysis.py"), "summary"],
        capture_output=True, text=True, timeout=60,
    )
    # Summary mode should not crash; exit 0 even if ArangoDB empty
    assert r.returncode == 0, f"exit {r.returncode}: {r.stderr}"


def main():
    print("🧪 Hermes pipeline smoke tests (additive scripts)")
    check("arango_storage imports", test_arango_storage_import)
    check("commodity_collector imports", test_commodity_collector_import)
    check("correlation map", test_correlation_map)
    check("correlation CLI summary", test_correlation_cli_summary)

    if FAILURES:
        print(f"\n❌ {len(FAILURES)} FAILED: {FAILURES}")
        sys.exit(1)
    print("\n✅ All smoke tests passed")


if __name__ == "__main__":
    main()
