#!/usr/bin/env python3
"""AICrypto Ops Tools — CLI for operational tasks."""

import argparse
import json
import sys
import urllib.request
import urllib.error
from datetime import datetime

API_BASE = "http://localhost:8080"


def api_get(path: str):
    try:
        with urllib.request.urlopen(f"{API_BASE}{path}", timeout=10) as resp:
            return json.loads(resp.read())
    except urllib.error.URLError as e:
        print(f"ERROR: Cannot reach API at {API_BASE}: {e}")
        sys.exit(1)


def api_post(path: str, body: dict | None = None):
    data = json.dumps(body or {}).encode()
    req = urllib.request.Request(
        f"{API_BASE}{path}", data=data,
        headers={"Content-Type": "application/json"}, method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read())
    except urllib.error.URLError as e:
        print(f"ERROR: API request failed: {e}")
        sys.exit(1)


def cmd_health(args):
    r = api_get("/health")
    print(f"Status:  {r['status']}")
    print(f"Version: {r['version']}")
    print(f"Uptime:  {r['uptime_secs']}s")


def cmd_skills(args):
    skills = api_get("/api/skills")
    if args.family:
        skills = [s for s in skills if s.get("skill_family") == args.family]
    print(f"{'ID':<40} {'Name':<25} {'Family':<15} {'Status':<20} {'Direction'}")
    print("-" * 110)
    for s in skills:
        print(f"{s['skill_id'][:40]:<40} {s['skill_name'][:25]:<25} {s['skill_family']:<15} {s['status']:<20} {s['direction']}")


def cmd_signals(args):
    signals = api_get("/api/signals")
    if not signals:
        print("No signals.")
        return
    print(f"{'ID':<40} {'Type':<10} {'Symbol':<12} {'Direction':<10} {'Confidence':<12} {'Reason'}")
    print("-" * 100)
    for s in signals:
        print(f"{s['signal_id'][:40]:<40} {s['signal_type']:<10} {s['symbol']:<12} {s['direction']:<10} {s['confidence']:<12.2%} {', '.join(s['reason_codes'])}")


def cmd_risk(args):
    rules = api_get("/api/risk/rules")
    print(f"{'ID':<8} {'Name':<30} {'Severity':<12} {'Threshold'}")
    print("-" * 65)
    for r in rules:
        print(f"{r['rule_id']:<8} {r['name']:<30} {r['severity']:<12} {r['threshold']}")


def cmd_pipeline(args):
    print("Running pipeline...")
    r = api_post("/api/run-pipeline")
    print(f"Scenarios:    {r['scenarios_run']}")
    print(f"Signals:      {r['total_signals']}")
    print(f"Intents:      {r['total_intents']}")
    print(f"Executed:     {r['total_executed']}")
    print(f"Risk rejects: {r['total_rejected_risk']}")
    print(f"Positions:    {r['open_positions']}")


def cmd_portfolio(args):
    p = api_get("/api/portfolio")
    print(f"Equity: ${p['equity']:,.2f}")
    positions = p.get("positions", [])
    if not positions:
        print("No open positions.")
        return
    print(f"\n{'Symbol':<12} {'Side':<8} {'Qty':<10} {'Entry':<15} {'Mark':<15} {'PnL':<12} {'Lev'}")
    print("-" * 85)
    for pos in positions:
        print(f"{pos['symbol']:<12} {pos['side']:<8} {pos['quantity']:<10} "
              f"${pos['entry_price']:>12,.2f} ${pos['mark_price']:>12,.2f} "
              f"${pos['unrealized_pnl']:>8,.2f} {pos['leverage']}x")


def main():
    parser = argparse.ArgumentParser(
        prog="aicrypto-ops",
        description="AICrypto operational CLI tools",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("health", help="Check API gateway health")

    sk = sub.add_parser("skills", help="List all skills")
    sk.add_argument("--family", help="Filter by family (trend/short/correlation/risk)")

    sub.add_parser("signals", help="List recent signals")
    sub.add_parser("risk", help="Show risk rules")
    sub.add_parser("pipeline", help="Run full pipeline")
    sub.add_parser("portfolio", help="Show portfolio and positions")

    args = parser.parse_args()

    commands = {
        "health": cmd_health,
        "skills": cmd_skills,
        "signals": cmd_signals,
        "risk": cmd_risk,
        "pipeline": cmd_pipeline,
        "portfolio": cmd_portfolio,
    }
    commands[args.command](args)


if __name__ == "__main__":
    main()
