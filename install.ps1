# Установка TLS 1.2
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# Определение архитектуры
$arch = "amd64"
if ([Environment]::Is64BitOperatingSystem -eq $false) {
    Write-Error "Неподдерживаемая архитектура. Требуется 64-битная версия Windows."
    exit 1
}

# Получение последней версии
$latestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/fgbm/exchange-log-parser/releases/latest"
$version = $latestRelease.tag_name

# Формирование URL для загрузки
$binaryName = "exchange-log-parser-$($version.TrimStart('v'))-windows-$arch.zip"
$downloadUrl = "https://github.com/fgbm/exchange-log-parser/releases/download/$version/$binaryName"

Write-Host "Загрузка Exchange Log Parser $version для windows-$arch..."

# Создание временной директории
$tmpDir = Join-Path $env:TEMP ([System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $tmpDir | Out-Null

# Загрузка архива
$zipPath = Join-Path $tmpDir $binaryName
Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath

# Создание директории для установки
$installDir = Join-Path $env:LOCALAPPDATA "ExchangeLogParser"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

# Распаковка архива
Expand-Archive -Path $zipPath -DestinationPath $installDir -Force

# Добавление в PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$userPath;$installDir",
        "User"
    )
}

# Очистка
Remove-Item -Path $tmpDir -Recurse -Force

Write-Host "Exchange Log Parser успешно установлен в $installDir"
Write-Host "Для проверки установки откройте новое окно PowerShell и выполните: exchange-log-parser --help" 