# uninstall.ps1
$ErrorActionPreference = "Stop"

########################################
# Konfigurácia (musí sedieť s install.ps1)
########################################
$AppName   = "Song Presenter"
$ExecName  = "jks_premietac.exe"
$ProjectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TargetDir  = Join-Path $ProjectDir "target\release"
$ExecPath   = Join-Path $TargetDir $ExecName
$DbTarget   = Join-Path $TargetDir "jks.db"
$DesktopDir = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = Join-Path $DesktopDir "Song Presenter.lnk"

Write-Host "==> Odinštalovávam $AppName"

########################################
# 1. Zmazanie skratky na ploche
########################################
if (Test-Path $ShortcutPath) {
    Write-Host "==> Mažem skratku na ploche: $ShortcutPath"
    Remove-Item $ShortcutPath -Force
} else {
    Write-Host "==> Skratku na ploche som nenašiel."
}

########################################
# 2. Zmazanie databázy pri binárke
########################################
if (Test-Path $DbTarget) {
    Write-Host "==> Mažem databázu pri binárke: $DbTarget"
    Remove-Item $DbTarget -Force
} else {
    Write-Host "==> jks.db pri binárke som nenašiel."
}

########################################
# 3. Vyčistenie build artefaktov (target/)
########################################
if (Test-Path (Join-Path $ProjectDir "target")) {
    Write-Host "==> Mažem build artefakty (cargo clean)..."
    Set-Location $ProjectDir
    cargo clean
} else {
    Write-Host "==> Žiadny target/ adresár, preskakujem cargo clean."
}

Write-Host "==> Odinštalovanie $AppName dokončené."
Write-Host "Zdrojové súbory projektu zostali zachované."