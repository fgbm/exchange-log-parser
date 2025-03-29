# Установка TLS 1.2
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# Определение архитектуры
$arch = "amd64"
if ([Environment]::Is64BitOperatingSystem -eq $false) {
    Write-Error "Неподдерживаемая архитектура. Требуется 64-битная версия Windows."
    exit 1
}

try {
    # Получение последней версии
    $latestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/fgbm/exchange-log-parser/releases/latest"
    if (-not $latestRelease) {
        Write-Error "Не удалось получить информацию о последнем релизе"
        exit 1
    }
    
    $version = $latestRelease.tag_name
    $downloadUrl = ($latestRelease.assets | Where-Object { $_.name -like "*windows-$arch.exe" }).browser_download_url
    
    if (-not $downloadUrl) {
        Write-Error "Не найден исполняемый файл для Windows ($arch) в релизе $version"
        exit 1
    }

    Write-Host "Загрузка Exchange Log Parser $version для Windows ($arch)..."

    # Создание директории для установки
    $installDir = Join-Path $env:LOCALAPPDATA "ExchangeLogParser"
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null

    # Загрузка файла
    $exePath = Join-Path $installDir "exchange-log-parser.exe"
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $exePath -ErrorAction Stop
    }
    catch {
        Write-Error "Ошибка при загрузке файла: $_"
        exit 1
    }

    if (-not (Test-Path $exePath)) {
        Write-Error "Файл не был загружен"
        exit 1
    }

    # Добавление в PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable(
            "Path",
            "$userPath;$installDir",
            "User"
        )
    }

    Write-Host "Exchange Log Parser успешно установлен в $installDir"
    Write-Host "Для проверки установки откройте новое окно PowerShell и выполните: exchange-log-parser --help"
}
catch {
    Write-Error "Произошла ошибка при установке: $_"
    exit 1
} 