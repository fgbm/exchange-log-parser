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

    # Поиск нужного ассета
    $asset = $latestRelease.assets | Where-Object { $_.name -like "*windows-$arch.exe" }
    if (-not $asset) {
        Write-Error "Не найден подходящий файл для windows-$arch в релизе $version"
        exit 1
    }

    Write-Host "Загрузка Exchange Log Parser $version для windows-$arch..."

    # Создание директории для установки
    $installDir = Join-Path $env:LOCALAPPDATA "ExchangeLogParser"
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null

    # Загрузка исполняемого файла
    $exePath = Join-Path $installDir "exchange-log-parser.exe"
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $exePath
    if (-not (Test-Path $exePath)) {
        Write-Error "Не удалось загрузить файл"
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