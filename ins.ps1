# install.ps1
$ErrorActionPreference = "Stop"

########################################
# Konfigurácia
########################################
$AppName   = "Song Presenter"
$ExecName  = "jks_premietac.exe"
$ProjectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TargetDir  = Join-Path $ProjectDir "target\release"
$ExecPath   = Join-Path $TargetDir $ExecName
$DbSource   = Join-Path $ProjectDir "jks.db"
$DbTarget   = Join-Path $TargetDir "jks.db"
$DesktopDir = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = Join-Path $DesktopDir "Song Presenter.lnk"
$IconPath  = Join-Path $ProjectDir "icon.png"

Write-Host "==> Inštalujem $AppName na Windows"

########################################
# 1. Rust + Cargo cez rustup
########################################
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "==> Cargo/Rust nebol nájdený, inštalujem cez rustup..."

    $rustupUrl  = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"  # 64-bit
    $rustupExe  = Join-Path $env:TEMP "rustup-init.exe"

    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe
    & $rustupExe -y | Out-Null

    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (-not ($env:PATH -split ";" | Where-Object { $_ -eq $cargoBin })) {
        $env:PATH = "$cargoBin;$env:PATH"
    }
} else {
    Write-Host "==> Cargo je už nainštalované."
}

########################################
# 2. Build projektu
########################################
Write-Host "==> Kompilujem projekt v release móde..."
Set-Location $ProjectDir
cargo build --release

if (-not (Test-Path $ExecPath)) {
    Write-Host "!! Build zlyhal alebo binárka neexistuje: $ExecPath"
    exit 1
}

########################################
# 3. Skopírovanie jks.db k binárke
########################################
if (Test-Path $DbSource) {
    Write-Host "==> Kopírujem jks.db k binárke..."
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
    Copy-Item $DbSource $DbTarget -Force
} else {
    Write-Host "!! Súbor jks.db sa nenašiel v $ProjectDir, preskakujem kopírovanie."
}

########################################
# 4. Vytvorenie skratky na ploche
########################################
Write-Host ""
$answer = Read-Host "Chceš vytvoriť skratku na ploche? [Y/n]"
if ($answer -eq "" -or $answer -match "^[Yy]") {
    Write-Host "==> Vytváram skratku na ploche: $ShortcutPath"

    $WScriptShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WScriptShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $ExecPath
    $Shortcut.WorkingDirectory = $TargetDir
    $Shortcut.WindowStyle = 1
    if (Test-Path $IconPath) {
        $Shortcut.IconLocation = $IconPath
    } else {
        $Shortcut.IconLocation = $ExecPath
    }
    $Shortcut.Save()
} else {
    Write-Host "==> Skratka na ploche nebude vytvorená."
}

Write-Host ""
Write-Host "==> Hotovo."
Write-Host "Binárka: $ExecPath"
Write-Host "Databáza pri binárke: $DbTarget"
Write-Host "Spustiť môžeš aj z PowerShellu/CMD príkazom:"
Write-Host "  `"$ExecPath`""