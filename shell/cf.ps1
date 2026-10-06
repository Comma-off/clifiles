# clifiles shell integration for PowerShell.
# Dot-source this from your profile ($PROFILE), then run `cf` to browse and `cd` on exit.
function cf {
    $cwdFile = [System.IO.Path]::GetTempFileName()
    try {
        & clifiles --cwd-file $cwdFile @args
        if ((Get-Item $cwdFile).Length -gt 0) {
            Set-Location -LiteralPath (Get-Content -Raw $cwdFile).Trim()
        }
    } finally {
        Remove-Item -Force $cwdFile -ErrorAction SilentlyContinue
    }
}
