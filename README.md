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
*   Запись данных в базу данных PostgreSQL.
*   Предотвращение дублирования записей с помощью уникальных индексов в БД и `ON CONFLICT DO NOTHING`.
*   Отображение прогресса обработки файлов с помощью прогресс-бара.
*   Конфигурация через аргументы командной строки.

## Установка

### Linux и macOS (одной командой)

```bash
curl -sSL https://raw.githubusercontent.com/fgbm/exchange-log-parser/main/install.sh | sudo bash
```

### Windows (одной командой)

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
exchange-log-parser --logs-dir <путь_к_папке_с_логами> \
                    --db-host <хост_бд> \
                    --db-port <порт_бд> \
                    --db-user <пользователь_бд> \
                    --db-password <пароль_бд> \
                    --db-name <имя_бд>
```

### Аргументы командной строки

*   `--logs-dir` (обязательный): Путь к директории, содержащей лог-файлы Exchange. Программа рекурсивно обойдет эту директорию.
*   `--db-host`: Адрес хоста сервера PostgreSQL (по умолчанию: `localhost`).
*   `--db-port`: Порт сервера PostgreSQL (по умолчанию: `5432`).
*   `--db-user`: Имя пользователя для подключения к PostgreSQL (по умолчанию: `postgres`).
*   `--db-password`: Пароль пользователя для подключения к PostgreSQL (по умолчанию: пустая строка).
*   `--db-name`: Имя базы данных PostgreSQL (по умолчанию: `exchange_logs`).

**Пример:**

```bash
exchange-log-parser --logs-dir "/mnt/exchange_logs" \
                    --db-host "192.168.1.10" \
                    --db-port 5432 \
                    --db-user "exchange_user" \
                    --db-password "secret_password" \
                    --db-name "exchange_log_db"
```

## Схема базы данных

Приложение автоматически создает (если они не существуют) следующие таблицы в указанной базе данных:

*   `smtp_receive_logs`: Для данных из логов SMTP Receive.
    *   Уникальный ключ: `(date_time, session_id, sequence_number)`
*   `smtp_send_logs`: Для данных из логов SMTP Send.
    *   Уникальный ключ: `(date_time, session_id, sequence_number)`
*   `message_tracking_logs`: Для данных из логов Message Tracking.
    *   Уникальный ключ: `(date_time, internal_message_id, recipient_address, event_id)`

## Основные зависимости

*   `clap`: Парсинг аргументов командной строки.
*   `tokio`: Асинхронная среда выполнения.
*   `tokio-postgres` & `deadpool-postgres`: Работа с PostgreSQL (асинхронный драйвер и пул соединений).
*   `chrono`: Работа с датой и временем.
*   `regex` & `lazy_static`: Работа с регулярными выражениями.
*   `indicatif`: Отображение прогресс-бара.
*   `color-eyre`: Обработка ошибок.
*   `log` & `env_logger`: Логирование.
*   `encoding_rs`: Декодирование текста из разных кодировок.
*   `walkdir`: Рекурсивный обход директорий.