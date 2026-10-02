param([Parameter(Mandatory = $true)][string] $ArtifactPath)
$ErrorActionPreference = 'Stop'
$resolvedArtifact = (Resolve-Path -LiteralPath $ArtifactPath -ErrorAction Stop).Path
if (-not (Test-Path -LiteralPath $resolvedArtifact -PathType Leaf)) { throw 'Signing target is not a file.' }
$thumbprint = ($env:WINDOWS_CERTIFICATE_THUMBPRINT -replace '\s', '').ToUpperInvariant()
if (-not $thumbprint -or $thumbprint -notmatch '^[0-9A-F]{40}$') { throw 'Missing or invalid Windows signing thumbprint.' }
$tool = Get-Command signtool.exe -ErrorAction Stop | Select-Object -First 1

# PowerShell passes each argument as a distinct native argument, including paths with spaces.
& $tool.Source sign /fd SHA256 /tr 'http://timestamp.digicert.com' /td SHA256 /sha1 $thumbprint $resolvedArtifact
if ($LASTEXITCODE -ne 0) { throw "Windows signing failed with exit code $LASTEXITCODE." }
& $tool.Source verify /pa /all $resolvedArtifact
if ($LASTEXITCODE -ne 0) { throw "Windows signature verification failed with exit code $LASTEXITCODE." }

$signature = Get-AuthenticodeSignature -LiteralPath $resolvedArtifact
if ($signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
    throw "Authenticode verification failed: $($signature.Status)."
}
if (-not $signature.SignerCertificate -or $signature.SignerCertificate.Thumbprint -ne $thumbprint) {
    throw 'The verified Authenticode signer does not match the configured certificate thumbprint.'
}
