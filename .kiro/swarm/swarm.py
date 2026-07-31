#!/usr/bin/env python3
"""
Swarm Orchestrator - Dispatches tasks to agent inboxes and processes events.

Usage:
  python swarm.py dispatch <agent> <task_file>
  python swarm.py process-events
  python swarm.py status
  python swarm.py start-feature <spec_path>
"""

import json
import sys
import os
from datetime import datetime
from pathlib import Path

SWARM_DIR = Path(__file__).parent
STATE_FILE = SWARM_DIR / "state.json"
EVENTS_DIR = SWARM_DIR / "events"
INBOX_DIR = SWARM_DIR / "inbox"
ARTIFACTS_DIR = SWARM_DIR / "artifacts"

def load_state():
    with open(STATE_FILE) as f:
        return json.load(f)

def save_state(state):
    with open(STATE_FILE, 'w') as f:
        json.dump(state, f, indent=2)

def timestamp():
    return datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")

def dispatch_task(agent: str, task: dict):
    """Write task to agent's inbox."""
    inbox = INBOX_DIR / agent
    inbox.mkdir(parents=True, exist_ok=True)
    
    task_id = task.get("id", f"task-{timestamp()}")
    task["timestamp"] = timestamp()
    task["from"] = "orchestrator"
    task["to"] = agent
    
    task_file = inbox / f"{task_id}.json"
    with open(task_file, 'w') as f:
        json.dump(task, f, indent=2)
    
    # Update state
    state = load_state()
    state["agents"][agent]["status"] = "assigned"
    state["agents"][agent]["current_task"] = task_id
    state["agents"][agent]["last_updated"] = timestamp()
    save_state(state)
    
    print(f"✓ Dispatched task {task_id} to {agent}")
    return task_file

def write_event(agent: str, event_type: str, payload: dict):
    """Write event from agent to events queue."""
    EVENTS_DIR.mkdir(parents=True, exist_ok=True)
    
    ts = datetime.utcnow().strftime("%Y%m%d-%H%M%S")
    event = {
        "timestamp": timestamp(),
        "agent": agent,
        "event": event_type,
        "payload": payload
    }
    
    event_file = EVENTS_DIR / f"{ts}-{agent}-{event_type}.json"
    with open(event_file, 'w') as f:
        json.dump(event, f, indent=2)
    
    print(f"✓ Event written: {event_file.name}")
    return event_file

def process_events():
    """Process pending events and dispatch next agents."""
    events = sorted(EVENTS_DIR.glob("*.json"))
    if not events:
        print("No pending events")
        return
    
    state = load_state()
    
    for event_file in events:
        with open(event_file) as f:
            event = json.load(f)
        
        agent = event["agent"]
        event_type = event["event"]
        payload = event.get("payload", {})
        
        print(f"Processing: {event_file.name}")
        
        # Update agent status
        state["agents"][agent]["status"] = "idle"
        state["agents"][agent]["current_task"] = None
        state["agents"][agent]["last_updated"] = timestamp()
        
        # Record in history
        state["history"].append({
            "timestamp": timestamp(),
            "agent": agent,
            "event": event_type,
            "result": payload.get("result", "unknown")
        })
        
        # Determine next agent based on workflow
        if event_type == "task_complete" and payload.get("result") == "success":
            next_agent = payload.get("next_suggested")
            if next_agent:
                # Auto-dispatch to next agent
                print(f"  → Next: {next_agent}")
                state["phase"] = f"{next_agent}_pending"
        
        # Update quality gates if applicable
        if "quality_gate" in payload:
            gate = payload["quality_gate"]
            result = payload.get("result")
            state["workflow"]["quality_gates"][gate] = result
        
        # Archive processed event
        archive_dir = EVENTS_DIR / "processed"
        archive_dir.mkdir(exist_ok=True)
        event_file.rename(archive_dir / event_file.name)
    
    save_state(state)
    print("✓ Events processed")

def start_feature(spec_path: str):
    """Initialize swarm workflow for a feature spec."""
    state = load_state()
    
    # Reset state for new feature
    state["current_feature"] = spec_path
    state["phase"] = "development"
    state["workflow"]["spec_path"] = spec_path
    state["workflow"]["branch"] = f"feature/{Path(spec_path).name}"
    state["workflow"]["tasks_completed"] = []
    state["workflow"]["tasks_pending"] = []
    state["workflow"]["blockers"] = []
    state["workflow"]["quality_gates"] = {
        "unit_tests": None,
        "lint": None,
        "security_scan": None,
        "e2e_tests": None
    }
    
    for agent in state["agents"]:
        state["agents"][agent]["status"] = "idle"
        state["agents"][agent]["current_task"] = None
    
    save_state(state)
    
    # Create initial task for developer
    task = {
        "id": f"dev-{Path(spec_path).name}",
        "type": "implement",
        "spec_path": spec_path,
        "instruction": f"Read spec at {spec_path}. Implement all tasks from tasks.md using TDD.",
        "context": {
            "branch": state["workflow"]["branch"],
            "constraints": ["TDD required", "Follow .kiro/steering/ standards"]
        }
    }
    
    dispatch_task("developer", task)
    print(f"✓ Feature workflow started: {spec_path}")

def show_status():
    """Print current swarm status."""
    state = load_state()
    
    print("\n=== JualanKu Swarm Status ===")
    print(f"Feature: {state['current_feature'] or 'None'}")
    print(f"Phase: {state['phase']}")
    print(f"Branch: {state['workflow'].get('branch', 'N/A')}")
    
    print("\nAgents:")
    for agent, info in state["agents"].items():
        status = info["status"]
        task = info.get("current_task", "-")
        emoji = "🟢" if status == "idle" else "🔵" if status == "assigned" else "🟡"
        print(f"  {emoji} {agent}: {status} (task: {task})")
    
    print("\nQuality Gates:")
    for gate, result in state["workflow"]["quality_gates"].items():
        emoji = "✅" if result == "pass" else "❌" if result == "fail" else "⬜"
        print(f"  {emoji} {gate}: {result or 'pending'}")
    
    # Check pending events
    events = list(EVENTS_DIR.glob("*.json"))
    if events:
        print(f"\nPending Events: {len(events)}")
        for e in events[:5]:
            print(f"  - {e.name}")

def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    
    cmd = sys.argv[1]
    
    if cmd == "status":
        show_status()
    elif cmd == "process-events":
        process_events()
    elif cmd == "start-feature" and len(sys.argv) >= 3:
        start_feature(sys.argv[2])
    elif cmd == "dispatch" and len(sys.argv) >= 4:
        agent = sys.argv[2]
        with open(sys.argv[3]) as f:
            task = json.load(f)
        dispatch_task(agent, task)
    elif cmd == "write-event" and len(sys.argv) >= 5:
        agent = sys.argv[2]
        event_type = sys.argv[3]
        payload = json.loads(sys.argv[4])
        write_event(agent, event_type, payload)
    else:
        print(__doc__)
        sys.exit(1)

if __name__ == "__main__":
    main()
