$ErrorActionPreference = "Stop"

Write-Host "Preparing installation for kill-tree..."

$binPath = Join-Path -Path $env:USERPROFILE -ChildPath "bin"
if (-not (Test-Path $binPath)) {
    New-Item -ItemType Directory -Path $binPath
    Write-Host "Created bin directory at $binPath"
}

if (-not ($env:PATH -split ";" -contains $binPath)) {
    $env:PATH += ";$binPath"
    [System.Environment]::SetEnvironmentVariable("PATH", $env:PATH, [System.EnvironmentVariableTarget]::User)
    Write-Host "Added $binPath to user's PATH environment variable"
}

Write-Host "Downloading and installing kill-tree..."

$tempDir = New-Item -ItemType Directory -Force -Path "$env:TEMP\kill-tree$(Get-Date -Format 'yyyyMMddHHmmss')"

try {
    $latestReleaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/oneofthezombies/kill-tree/releases/latest" -Headers @{"Accept"="application/vnd.github.v3+json"}
    $asset = $latestReleaseInfo.assets | Where-Object { $_.name -match "kill-tree-windows-x86_64" }

    if (-not $asset) {
        Write-Error "kill-tree asset not found in the latest release"
        exit 1
    }

    $downloadUrl = $asset.browser_download_url

    $localPath = Join-Path -Path $tempDir -ChildPath $asset.name
    Invoke-WebRequest -Uri $downloadUrl -OutFile $localPath

    $finalPath = Join-Path -Path $binPath -ChildPath "kill-tree.exe"
    Move-Item -Path $localPath -Destination $finalPath -Force

    Write-Host "kill-tree installed to $finalPath"

    Write-Host "Trying to print version..."
    & "$finalPath" --version

    Write-Host "kill-tree installed successfully."
}
catch {
    Write-Error "An error occurred: $_"
}
finally {
    Remove-Item -Path $tempDir -Recurse -Force
}
