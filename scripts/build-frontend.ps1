$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$distDirectory = Join-Path $projectRoot "dist"
$publicSource = Join-Path $projectRoot "public"
$publicDestination = Join-Path $distDirectory "public"

New-Item -ItemType Directory -Path $distDirectory -Force | Out-Null
New-Item -ItemType Directory -Path $publicDestination -Force | Out-Null

Copy-Item -LiteralPath (Join-Path $projectRoot "index.html") -Destination $distDirectory -Force
Copy-Item -LiteralPath (Join-Path $projectRoot "styles.css") -Destination $distDirectory -Force
Copy-Item -Path (Join-Path $publicSource "*") -Destination $publicDestination -Recurse -Force

Write-Host "Static frontend prepared in $distDirectory"
