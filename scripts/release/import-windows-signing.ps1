param()
$ErrorActionPreference = 'Stop'

function Resolve-SignTool {
    $command = Get-Command signtool.exe -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($command) {
        return $command.Source
    }

    $programFilesX86 = ${env:ProgramFiles(x86)}
    if ($programFilesX86) {
        $kitsRoot = Join-Path $programFilesX86 'Windows Kits\10\bin'
        $candidates = @(
            Get-ChildItem -Path (Join-Path $kitsRoot '*\x64\signtool.exe') -File -ErrorAction SilentlyContinue |
                Sort-Object FullName -Descending
        )
        if ($candidates.Count -gt 0) {
            return $candidates[0].FullName
        }
    }

    throw 'signtool.exe was not found in PATH or the installed Windows SDK.'
}

foreach ($name in @('WINDOWS_CERTIFICATE', 'WINDOWS_CERTIFICATE_PASSWORD', 'WINDOWS_CERTIFICATE_THUMBPRINT')) {
    if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
        throw "$name is required in the production-release environment."
    }
}

$expectedThumbprint = ($env:WINDOWS_CERTIFICATE_THUMBPRINT -replace '\s', '').ToUpperInvariant()
if ($expectedThumbprint -notmatch '^[0-9A-F]{40}$') {
    throw 'WINDOWS_CERTIFICATE_THUMBPRINT must be a 40-character SHA-1 certificate thumbprint.'
}

$pfx = Join-Path $env:RUNNER_TEMP 'opendownloader-signing.pfx'
try {
    try {
        $base64 = $env:WINDOWS_CERTIFICATE -replace '\s', ''
        [IO.File]::WriteAllBytes($pfx, [Convert]::FromBase64String($base64))
    }
    catch {
        throw 'WINDOWS_CERTIFICATE must contain the base64 encoding of an exportable PFX/PKCS#12 certificate.'
    }

    $password = ConvertTo-SecureString $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
    Import-PfxCertificate -FilePath $pfx -CertStoreLocation Cert:\CurrentUser\My -Password $password | Out-Null

    $certificate = Get-Item "Cert:\CurrentUser\My\$expectedThumbprint" -ErrorAction SilentlyContinue
    if (-not $certificate) {
        throw 'The imported PFX does not contain the certificate identified by WINDOWS_CERTIFICATE_THUMBPRINT.'
    }
    if (-not $certificate.HasPrivateKey) {
        throw 'The Windows signing certificate was imported without its private key.'
    }

    $now = Get-Date
    if ($now -lt $certificate.NotBefore -or $now -gt $certificate.NotAfter) {
        throw 'The Windows signing certificate is not currently valid.'
    }

    $ekuOids = @()
    if ($certificate.PSObject.Properties.Name -contains 'EnhancedKeyUsageList') {
        $ekuOids = @($certificate.EnhancedKeyUsageList | ForEach-Object { $_.ObjectId.Value })
    }
    if ($ekuOids.Count -gt 0 -and $ekuOids -notcontains '1.3.6.1.5.5.7.3.3') {
        throw 'The Windows certificate does not permit code signing.'
    }

    $signTool = Resolve-SignTool
    Write-Host "Windows signing preflight passed. Certificate expires $($certificate.NotAfter.ToString('u')); signtool: $signTool"
}
finally {
    Remove-Item -LiteralPath $pfx -Force -ErrorAction SilentlyContinue
}
