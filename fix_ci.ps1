$token = "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"
$headers = @{
  Authorization = "token $token"
  Accept = "application/vnd.github.v3+json"
}

# Cancel stuck run
Write-Host "Cancelling stuck run..."
Invoke-RestMethod -Uri "https://api.github.com/repos/hernandez42/APEX-AGI/actions/runs/26493454267/cancel" -Headers $headers -Method Post | Out-Null
Invoke-RestMethod -Uri "https://api.github.com/repos/hernandez42/APEX-AGI/actions/runs/26493452239/cancel" -Headers $headers -Method Post | Out-Null
Write-Host "Cancelled"

# Update workflow
$url = "https://api.github.com/repos/hernandez42/APEX-AGI/contents/.github/workflows/check.yml"
$existing = Invoke-RestMethod -Uri $url -Headers $headers -Method Get

$newYaml = @'
name: Check
on: [push, workflow_dispatch]
jobs:
  check:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
          echo "$HOME/.cargo/bin" >> $GITHUB_PATH
      - name: cargo check
        working-directory: ./omega-agi
        run: cargo check --workspace 2>&1
      - name: cargo test
        working-directory: ./omega-agi
        run: cargo test --workspace --no-fail-fast -- --test-threads=1 2>&1
        timeout-minutes: 10
'@

$b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($newYaml))
$body = @{
  message = "Fix CI: add job timeout, single test thread"
  content = $b64
  sha = $existing.sha
  branch = "master"
} | ConvertTo-Json -Compress

$result = Invoke-RestMethod -Uri $url -Headers $headers -Body $body -Method Put -ContentType 'application/json'
Write-Host "Updated: $($result.commit.sha)"
