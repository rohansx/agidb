# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues at `github.com/agidb/agidb` (to be created in [phase 0](../phases/phase-0-setup.md)). Use the `gh` CLI for all operations.

> **Transitional note:** until the GitHub remote is configured, treat issues as local-markdown drafts under `.scratch/<feature>/`. Re-run `setup-matt-pocock-skills` after the remote is set if you'd rather make that switch permanent.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. Use a heredoc for multi-line bodies.
- **Read an issue**: `gh issue view <number> --comments`, filtering comments by `jq` and also fetching labels.
- **List issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'` with appropriate `--label` and `--state` filters.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply / remove labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close**: `gh issue close <number> --comment "..."`

Infer the repo from `git remote -v` — `gh` does this automatically when run inside a clone.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.
