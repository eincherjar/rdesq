# Download plink.exe from official PuTTY website
$url = "https://the.earth.li/~sgtatham/putty/latest-w64/plink.exe"
$output = Join-Path $PSScriptRoot "plink.exe"
Write-Host "Downloading plink.exe from $url..."
Invoke-WebRequest -Uri $url -OutFile $output -UseBasicParsing
if (Test-Path $output) {
    Write-Host "Downloaded plink.exe ($((Get-Item $output).Length / 1KB) KB)"
} else {
    Write-Host "Failed to download plink.exe"
    exit 1
}
