#!/bin/bash

set -e

# Определение архитектуры и ОС
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

if [ "$ARCH" = "x86_64" ]; then
    ARCH="amd64"
elif [ "$ARCH" = "aarch64" ]; then
    ARCH="arm64"
else
    echo "Неподдерживаемая архитектура: $ARCH"
    exit 1
fi

if [ "$OS" = "darwin" ]; then
    OS="macos"
fi

# Получение последней версии
VERSION=$(curl -s https://api.github.com/repos/fgbm/exchange-log-parser/releases/latest | grep '"tag_name":' | cut -d'"' -f4)

# Формирование URL для загрузки
BINARY_NAME="exchange-log-parser-${VERSION#v}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/fgbm/exchange-log-parser/releases/download/${VERSION}/${BINARY_NAME}"

echo "Загрузка Exchange Log Parser ${VERSION} для ${OS}-${ARCH}..."

# Создание временной директории
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Загрузка и распаковка архива
curl -L "$DOWNLOAD_URL" -o "$BINARY_NAME"
tar xzf "$BINARY_NAME"

# Установка бинарного файла
sudo mkdir -p /usr/local/bin
sudo mv exchange-log-parser /usr/local/bin/
sudo chmod +x /usr/local/bin/exchange-log-parser

# Очистка
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "Exchange Log Parser успешно установлен в /usr/local/bin/exchange-log-parser"
echo "Для проверки установки выполните: exchange-log-parser --help" 