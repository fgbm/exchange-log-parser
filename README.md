![GitHub release (latest by date)](https://img.shields.io/github/v/release/fgbm/exchange-log-parser?color=blue&label=latest%20release&style=flat-square)
![Rust](https://img.shields.io/badge/rust-1.70.0-orange?style=flat-square)
![MIT License](https://img.shields.io/badge/license-MIT-yellowgreen?style=flat-square)


# Exchange Log Parser

Инструмент для автоматического анализа лог-файлов Microsoft Exchange Server (SMTP Receive, SMTP Send, Message Tracking) и их загрузки в базу данных PostgreSQL для удобного анализа и поиска.

## Зачем это нужно?

Анализ текстовых логов Exchange вручную может быть трудоемким и подверженным ошибкам, особенно при больших объемах данных или при необходимости сопоставить события из разных файлов (например, отследить путь письма). Стандартные средства просмотра логов не всегда предоставляют гибкие возможности для фильтрации, агрегации и поиска информации.

**Exchange Log Parser** решает эти проблемы путем:

*   **Автоматизации:** читает и разбирает стандартные форматы логов Exchange.
*   **Структурирования данных:** сохраняет извлеченную информацию в четко определенные таблицы базы данных PostgreSQL.
*   **Централизации:** позволяет хранить логи с разных серверов Exchange в одном месте.
*   **Гибкого анализа:** дает возможность использовать всю мощь SQL-запросов для анализа данных, построения отчетов, поиска конкретных писем или событий.

## Возможности

*   Парсинг логов SMTP Receive (`*REC*.log`)
*   Парсинг логов SMTP Send (`*SND*.log`)
*   Парсинг логов Message Tracking (`MSGTRK*.log`)
*   Автоматическое определение типа лог-файла по заголовку `#Log-type`.
*   Обработка файлов в кодировке `WINDOWS-1251`.
*   Поддержка PostgreSQL и Microsoft SQL Server в качестве целевых СУБД.
*   Предотвращение дублирования записей с помощью уникальных индексов в БД.
*   Отображение прогресса обработки файлов с помощью прогресс-бара.
*   Конфигурация через аргументы командной строки.

## Установка

### Linux и macOS

```bash
curl -sSL https://raw.githubusercontent.com/fgbm/exchange-log-parser/main/install.sh | sudo bash
```

### Windows

```powershell
irm -useb https://raw.githubusercontent.com/fgbm/exchange-log-parser/main/install.ps1 | iex
```

### Сборка из исходного кода (для разработчиков)

Если вы хотите собрать приложение самостоятельно или внести изменения:

1.  **Установите Rust:** Если у вас еще нет Rust, следуйте [официальной инструкции](https://www.rust-lang.org/tools/install).
2.  **Клонируйте репозиторий:**
    ```bash
    git clone https://github.com/fgbm/exchange-log-parser.git
    cd exchange-log-parser
    ```
3.  **Соберите проект:**
    ```bash
    cargo build --release
    ```
    Готовый исполняемый файл будет находиться в `target/release/exchange-log-parser` (или `.exe` для Windows).

## Использование

Запустите приложение, указав необходимые параметры:

```bash
exchange-log-parser --db-type <тип_бд> \
                    --db-host <хост_бд> \
                    --db-port <порт_бд> \
                    --db-user <пользователь_бд> \
                    --db-password <пароль_бд> \
                    --db-name <имя_бд> \
                    --concurrent-files <количество_файлов> \
                    --table-prefix <префикс_таблиц> \
                    <путь_к_папке_с_логами>
```

### Аргументы командной строки

*   `[logs_dir]` (по умолчанию: текущая директория): Путь к директории, содержащей лог-файлы Exchange. Программа рекурсивно обойдет эту директорию.
*   `--db-type`: Тип базы данных (`postgres` или `mssql`, по умолчанию: `postgres`).
*   `--db-host`: Адрес хоста сервера БД (по умолчанию: `localhost`).
*   `--db-port`: Порт сервера БД (по умолчанию: `5432`).
*   `--db-user`: Имя пользователя для подключения к БД (по умолчанию: `postgres`).
*   `--db-password`: Пароль пользователя для подключения к БД (по умолчанию: пустая строка).
*   `--db-name`: Имя базы данных (по умолчанию: `exchange_logs`).
*   `--concurrent-files`: Количество одновременно обрабатываемых файлов (по умолчанию: `10`).
*   `--table-prefix`: Префикс для имен таблиц в базе данных (опционально).

**Пример для PostgreSQL:**

```bash
exchange-log-parser --db-type postgres \
                    --db-host "192.168.1.10" \
                    --db-port 5432 \
                    --db-user "exchange_user" \
                    --db-password "secret_password" \
                    --db-name "exchange_log_db" \
                    --concurrent-files 10 \
                    --table-prefix "ex_" \
                    "/mnt/exchange_logs"
```

**Пример для MS SQL:**

```bash
exchange-log-parser --db-type mssql \
                    --db-host "192.168.1.10" \
                    --db-port 1433 \
                    --db-user "exchange_user" \
                    --db-password "secret_password" \
                    --db-name "exchange_log_db" \
                    --concurrent-files 10 \
                    --table-prefix "ex_" \
                    "/mnt/exchange_logs"
```

## Схема базы данных

Приложение автоматически создает (если они не существуют) следующие таблицы в указанной базе данных:

*   `{prefix}smtp_receive_logs`: Для данных из логов SMTP Receive.
    *   Уникальный ключ: `(date_time, session_id, sequence_number)`
*   `{prefix}smtp_send_logs`: Для данных из логов SMTP Send.
    *   Уникальный ключ: `(date_time, session_id, sequence_number)`
*   `{prefix}message_tracking_logs`: Для данных из логов Message Tracking.
    *   Уникальный ключ: `(date_time, internal_message_id, recipient_address, event_id)`

Где `{prefix}` - опциональный префикс таблиц, указанный через параметр `--table-prefix`.

## Основные зависимости

*   `clap`: Парсинг аргументов командной строки.
*   `tokio`: Асинхронная среда выполнения.
*   `tokio-postgres` & `deadpool-postgres`: Работа с PostgreSQL (асинхронный драйвер и пул соединений).
*   `tiberius`: Асинхронный драйвер для MS SQL Server.
*   `bb8` & `bb8-tiberius`: Пул соединений для MS SQL Server.
*   `chrono`: Работа с датой и временем.
*   `regex` & `lazy_static`: Работа с регулярными выражениями.
*   `indicatif`: Отображение прогресс-бара.
*   `color-eyre`: Обработка ошибок.
*   `log` & `env_logger`: Логирование.
*   `encoding_rs`: Декодирование текста из разных кодировок.
*   `walkdir`: Рекурсивный обход директорий.