# Exchange Log Parser

Программа для парсинга лог-файлов Microsoft Exchange Server (SMTP Receive, SMTP Send, Message Tracking) и сохранения данных в базу данных PostgreSQL.

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

## Предварительные требования

*   **Rust:** Установленный Rust toolchain (компилятор `rustc` и менеджер пакетов `cargo`). [Инструкция по установке](https://www.rust-lang.org/tools/install)
*   **PostgreSQL:** Работающий сервер PostgreSQL.

## Сборка

1.  Клонируйте репозиторий (если вы этого еще не сделали):
    ```bash
    git clone <repository_url>
    cd exchange-log-parser
    ```
2.  Соберите проект:
    ```bash
    cargo build --release
    ```
    Исполняемый файл будет находиться в `target/release/exchange-log-parser`.

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