$token = "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"
$headers = @{
  Authorization = "token $token"
  Accept = "application/vnd.github.v3+json"
}

# Get current workflow
$url = "https://api.github.com/repos/hernandez42/APEX-AGI/contents/.github/workflows/check.yml"
$existing = Invoke-RestMethod -Uri $url -Headers $headers -Method Get
$yaml = [System.Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($existing.content))

# New workflow content
$newYaml = @'
name: Check
on: [push, workflow_dispatch]
jobs:
  check:
    runs-on: ubuntu-latest
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
        run: cargo test --workspace --no-fail-fast 2>&1
'@

$b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($newYaml))
$body = @{
  message = "Add cargo test to CI"
  content = $b64
  sha = $existing.sha
  branch = "master"
} | ConvertTo-Json -Compress

$result = Invoke-RestMethod -Uri $url -Headers $headers -Body $body -Method Put -ContentType 'application/json'
Write-Host "Updated: $($result.commit.sha)"
