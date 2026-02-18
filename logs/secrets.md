# Secrets & Hygiene Scan (2026-02-18)

- Tooling: gitleaks 8.30.0, trufflehog 3.93.3 (Homebrew).
- Commands run:
  - `gitleaks detect --source . --no-git --report-path logs/secrets/gitleaks_worktree.json --redact`
  - `gitleaks detect --source . --report-path logs/secrets/gitleaks_history.json --redact`
  - `trufflehog filesystem --no-update --json . --exclude-paths logs/secrets/trufflehog_excludes.txt > logs/secrets/trufflehog_filesystem.json`
  - `trufflehog git --no-update --json file://$(pwd) --exclude-paths logs/secrets/trufflehog_excludes.txt > logs/secrets/trufflehog_git.json`
- Findings:
  - gitleaks (working tree) reported 2 `generic-api-key` matches: `.env` line 18 and `.ralphie/config.env` line 42. Both files are ignored/local; rotate or remove any real keys from these files.
  - gitleaks (git history): no leaks across 11 commits.
  - trufflehog (filesystem + git, excluding .git/, target/, logs/, .ralphie/, coverage/, node_modules/): no verified or unverified secrets.
- Artifacts: redacted gitleaks JSON reports saved under `logs/secrets/` (ignored by git). Trufflehog report files left empty to indicate clean runs; exclusion patterns recorded in `logs/secrets/trufflehog_excludes.txt` (ignored).
- Next step: rerun scans after clearing or rotating the local `.env` / `.ralphie/config.env` secrets.
