---
inclusion: manual
---

# Lavish Editor — Rich HTML Artifacts for Agent-Human Review

## Role

Kamu adalah agent yang menggunakan Lavish Editor untuk mengubah complex atau visual responses menjadi rich, reviewable HTML artifacts. Gunakan ketika akan memberikan plan, comparison, diagram, table, code diff, report, atau apapun yang lebih mudah dipahami secara visual daripada prose.

## Overview

Lavish Editor helps agents turn rich HTML artifacts into collaborative human review surfaces. Whenever you are about to give user a complex response that will be easier to understand via a rich / interactive page, consider using Lavish Editor.

## Core Workflow

1. **Create HTML artifact** — Default location `.lavish/<name>.html` in working directory
2. **Open session** — Run `npx -y lavish-axi <html-file>` to open review in browser
3. **Poll for feedback** — Run poll as BACKGROUND process (see "Polling Rules" below)
4. **Handle layout warnings** — If poll returns `layout_warnings`, repair the failure before involving human
5. **Apply feedback & loop** — Use `--agent-reply "<message>"` to continue conversation
6. **End session** — Run `npx -y lavish-axi end <html-file>` when done

## When to Use

Use lavish-axi when user asks for:
- Visual artifacts or HTML explainers
- Interactive prototypes
- Review surfaces
- Product or technical plans
- Comparisons or reports
- Browser-based feedback loops

## Playbooks

Run `npx -y lavish-axi playbook <id>` for focused guidance. One artifact often combines several playbooks.

| Playbook | Purpose |
|----------|---------|
| `diagram` | Maps relationships, flows, state, architecture |
| `table` | Turn dense records into scan-friendly surfaces |
| `comparison` | Show options, tradeoffs, current vs target |
| `plan` | Explain product/technical plan before implementation |
| `code` | Render source code, patches, PR diffs, before/after |
| `input` | Collect user input on decisions, choices, preferences |
| `slides` | Create deliberate presentations when requested |

**Important**: MUST open each matching playbook before writing HTML.

## Commands Reference

```bash
# Open/resume session
npx -y lavish-axi <html-file>

# Poll for feedback (long-running, don't kill)
npx -y lavish-axi poll <html-file>
npx -y lavish-axi poll <html-file> --agent-reply "<message>"

# End session
npx -y lavish-axi end <html-file>

# Export portable HTML
npx -y lavish-axi export <html-file> [--out <path>]

# Share publicly on ht-ml.app
npx -y lavish-axi share <html-file> [--password <pw>]

# Get design guidance
npx -y lavish-axi design

# Playbook guidance
npx -y lavish-axi playbook <playbook_id>

# Stop background server
npx -y lavish-axi stop
```

## Visual Guidance

- Use visual hierarchy to make important decisions, risks, tradeoffs, and next actions obvious at a glance
- Use visual structure: sections, cards, tables, diagrams, annotated snippets, side-by-side comparisons instead of long prose
- Choose typography, spacing, color, and layout deliberately with clear point of view
- **Prevent horizontal overflow** at every nesting level:
  - Nested grid/flex children need `minmax(0, 1fr)` tracks and `min-width: 0`
  - Wrap, truncate, or contain long unbreakable text deliberately
- When describing existing UI/state — **show it instead**: capture real screenshots rather than explaining in prose

## Design System Priority

Before writing HTML, decide design direction in this strict order:

1. **User specified** — Use the specific look or named design system user asked for
2. **Match project** — Inspect the project the artifact is about and match its design system (Tailwind/theme config, CSS variables, component library, brand assets)
3. **Lavish default** — Only when both above are empty, use Tailwind CSS v4 + DaisyUI v5 via CDN

Run `npx -y lavish-axi design` for CDN snippets and component reference.

## Mermaid Diagrams

For flows, architecture, state, or sequence diagrams:
- Use theme-aware Mermaid snippet from `npx -y lavish-axi design`
- Rendered diagrams in `.mermaid` containers become editable Excalidraw whiteboards in browser
- flowchart, sequence, class, ER, state diagrams convert to editable shapes
- Other types embed as image to draw on

## Polling Rules

Poll stays silent until user acts or browser proves layout failure. Rules:

- **IMPORTANT**: Poll is a long-running command — MUST use Kiro's `control_bash_process` with `action: start`
- Never use synchronous `execute_bash` for poll — it will timeout (poll waits indefinitely for feedback)
- Check poll output periodically with `get_process_output`
- Timeout from poll is NOT a failure — it means no user feedback yet
- Keep poll in foreground by default
- Never use `nohup`, shell `&`, `disown`, or fire-and-forget processes
- Background poll only allowed through harness-native tracked background-job facility
- If poll gets killed or times out, just re-run it — queued feedback is never lost
- `Send & End` from browser ends session; agent must not reopen uninvited

## Asset References

Lavish serves HTML through local Express server:
- Copy local assets (images, CSS, fonts) into same directory as HTML file
- Reference with relative paths (no `/` prefix — root paths won't work)
- Remote CDN/font references are kept as links

## Example: Creating a Technical Plan

```html
<!-- .lavish/architecture-plan.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Architecture Plan</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <link href="https://cdn.jsdelivr.net/npm/daisyui@5/dist/full.min.css" rel="stylesheet">
</head>
<body class="bg-base-200 p-8">
  <div class="max-w-4xl mx-auto">
    <h1 class="text-3xl font-bold mb-6">System Architecture Plan</h1>
    
    <!-- Use cards for sections -->
    <div class="card bg-base-100 shadow-xl mb-6">
      <div class="card-body">
        <h2 class="card-title">Current State</h2>
        <!-- Embed real screenshot here -->
        <img src="current-arch.png" alt="Current architecture">
      </div>
    </div>
    
    <!-- Mermaid diagram -->
    <div class="mermaid">
      flowchart TD
        A[Client] --> B[API Gateway]
        B --> C[Service A]
        B --> D[Service B]
    </div>
    
    <!-- Comparison table -->
    <div class="overflow-x-auto">
      <table class="table">
        <thead>
          <tr>
            <th>Option</th>
            <th>Pros</th>
            <th>Cons</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Option A</td>
            <td>Fast, simple</td>
            <td>Limited scale</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</body>
</html>
```

Then run:
```bash
npx -y lavish-axi .lavish/architecture-plan.html
npx -y lavish-axi poll .lavish/architecture-plan.html --agent-reply "Here's the architecture plan. Please review the proposed changes and annotate any concerns."
```

## When NOT to Use

- Simple text responses that don't benefit from visual layout
- Quick Q&A that needs immediate inline answer
- When user explicitly asks for text/markdown output
- Internal dashboard UIs (use for review surfaces, not production UIs)

## Resources

- **npm package**: `lavish-axi`
- **Hosting**: ht-ml.app (for sharing)
- Run `npx -y lavish-axi design` for full design reference
