---
title: Agent Context Compaction
description: "Design notes and implementation status for Zed's native agent context compaction work."
---

# Agent Context Compaction

This page records the work completed so far on Zed's native agent context compaction feature on the `feature/agent-context-compaction` branch.

## Goals

The work has focused on four practical goals:

- Keep long-running native agent threads usable after they approach the model context window.
- Match the high-level approach used by VS Code's local agent, where compaction happens in place on the same thread rather than by moving the user to a separate transcript.
- Preserve enough technical detail that the agent can continue working after compaction without losing the current task, recent tool results, or important implementation decisions.
- Make compaction visible and debuggable for both users and developers.

## Design Conclusions

Research against VS Code's bundled Copilot agent implementation showed that the correct architectural direction is same-thread, prompt-time compaction.

The important parity points were:

- Earlier history should be replaced by a summary that is injected back into future prompts.
- The current thread should remain the source of truth rather than forking into a new conversation.
- Summary quality depends on deterministic summarization and strong instructions about preserving recent operations, technical facts, and continuation context.
- Recoverability matters: if the summary omits a detail, the agent should have a path back to the compacted transcript.

That led to a design that keeps Zed's existing same-thread compaction model, while improving observability, prompt quality, token accounting, and transcript recovery.

## Implementation Completed

### Backend compaction flow

The main implementation lives in `crates/agent/src/thread.rs`.

Completed backend work includes:

- Manual and background compaction continue to operate on the same thread state.
- Future requests inject the compacted history as a `<conversation-summary>` block and then append only the newer unsummarized messages.
- Compaction now emits clearer backend logs for start, skip, completion, failure, and application paths.
- Manual compaction emits explicit thread events so the UI can react to start and completion.
- Compaction summaries now run with deterministic sampling (`temperature: 0.0`) instead of inheriting the user's general model temperature settings.

### Summary quality improvements

The summarization prompt in `crates/agent_settings/src/prompts/compact_context_prompt.txt` was expanded substantially.

It now requires the model to preserve:

- exact technical facts
- recent commands and tool results
- causality and validation state
- unresolved issues and follow-up work
- the exact active task at the point of compaction

This moves Zed closer to the summary contract used by VS Code's local agent.

### Transcript recoverability

One of the largest parity gaps was the lack of a recovery path back to the compacted history.

That is now addressed by persisting a markdown snapshot of the compacted portion of the thread under:

- `.zed/compaction-transcripts/`

When the snapshot write succeeds, the stored summary gets an explicit `read_file` hint that points back to the saved transcript. This gives the agent a way to recover exact code snippets, tool results, and other verbatim details that may have been compressed out of the summary.

### Token usage behavior

Before this work, compaction cleared the latest token usage and the context usage indicator could disappear immediately after a successful compaction.

That behavior was improved by estimating post-compaction token usage from the compacted request shape and exposing that estimate until the next real token usage update arrives. This keeps the context usage UI visible after compaction and makes it easier to confirm that compaction actually reduced the window.

### UI feedback and observability

The main UI work lives in `crates/agent_ui/src/conversation_view/thread_view.rs`.

Completed UI work includes:

- a status toast when manual compaction starts
- a success toast when manual compaction completes
- clearer visible feedback that the action actually ran

These changes make manual compaction much easier to verify during local testing.

## Files Changed So Far

The main files changed during this work are:

- `crates/agent/src/thread.rs`
- `crates/agent_ui/src/conversation_view/thread_view.rs`
- `crates/agent_settings/src/prompts/compact_context_prompt.txt`

These changes cover the core backend compaction flow, prompt contract, token usage behavior, transcript recovery, and manual compaction UI feedback.

## Validation Completed

Validation performed during this work includes:

- `cargo check -p agent -p agent_ui`
- `cargo check -p agent`
- local manual testing of manual compaction behavior
- log verification showing compaction preparation, completion, and application on the active thread

There is also a regression test in `crates/agent/src/thread.rs` covering transcript snapshot persistence and summary hint appending.

## Current Behavior Summary

At this point, the native agent compaction flow behaves as follows:

1. Zed detects that the thread should compact, either manually or automatically.
2. The compacted slice of the thread is rendered to a markdown transcript snapshot.
3. Zed asks the summarization model for a deterministic summary using the stronger compaction prompt.
4. If transcript snapshot persistence succeeds, the summary is augmented with a `read_file` recovery hint.
5. The thread stores the resulting compaction state.
6. Future requests include the `<conversation-summary>` plus only the newer unsummarized messages.
7. The UI keeps showing context usage using an estimated post-compaction value until fresh token usage arrives.

## Remaining Follow-up

The major implemented gaps have been closed, but there is still room for follow-up work if needed:

- compare Zed's background compaction triggering policy more closely against VS Code's warm and emergency summarization behavior
- continue evaluating summary quality on long multi-tool sessions
- add more end-to-end coverage around repeated compaction cycles

## Branch Status

This document is intended to serve as the running implementation record for the current branch work, including research findings, architectural decisions, code changes, and validation status.