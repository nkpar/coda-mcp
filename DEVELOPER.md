# Coda MCP Server Specification

## Overview

MCP server для работы с Coda.io API. Позволяет читать и писать документы, таблицы, строки.

## Auth

- Переменная окружения: `CODA_API_TOKEN`
- Header: `Authorization: Bearer {token}`
- Base URL: `https://coda.io/apis/v1`

## Tools

### list_docs
Список доступных документов.
- `limit: int = 50` — макс количество
- `query: str = ""` — фильтр по имени

### get_doc
Метаданные документа.
- `doc_id: str` — ID документа

### list_pages
Страницы в документе.
- `doc_id: str`

### get_page
Контент страницы (HTML). Использует async export workflow для получения контента canvas-страниц.
- `doc_id: str`
- `page_id: str`

**Workflow:**
1. POST `/docs/{doc_id}/pages/{page_id}/export` с `{"outputFormat": "html"}`
2. Poll GET `/docs/{doc_id}/pages/{page_id}/export/{export_id}` до статуса `complete`
3. Download контент по `downloadLink`

Max polling: 30 attempts, 1s interval (30s timeout)

### list_tables
Таблицы в документе.
- `doc_id: str`

### get_table
Метаданные таблицы.
- `doc_id: str`
- `table_id: str`

### list_columns
Колонки таблицы.
- `doc_id: str`
- `table_id: str`

### get_rows
Строки таблицы.
- `doc_id: str`
- `table_id: str`
- `limit: int = 100`
- `query: str = ""` — фильтр в синтаксисе Coda формул
- Query param: `useColumnNames=true`

### get_row
Одна строка.
- `doc_id: str`
- `table_id: str`
- `row_id: str`

### add_row
Добавить строку.
- `doc_id: str`
- `table_id: str`
- `cells: dict` — `{column_name: value}`
- POST body: `{"rows": [{"cells": [{"column": k, "value": v}, ...]}]}`

### update_row
Обновить строку.
- `doc_id: str`
- `table_id: str`
- `row_id: str`
- `cells: dict`
- PUT body: `{"row": {"cells": [{"column": k, "value": v}, ...]}}`

### delete_row
Удалить строку.
- `doc_id: str`
- `table_id: str`
- `row_id: str`

### search_docs
Поиск по документам.
- `query: str`

### create_doc
Создать новый документ. Опционально можно указать папку, шаблон (исходный документ) или таймзону.
- `title: str` — название документа
- `folder_id: str = null` — ID папки (опционально)
- `source_doc: str = null` — ID документа-шаблона для копирования (опционально)
- `timezone: str = null` — таймзона (опционально, напр. "America/Los_Angeles")

### delete_doc
Удалить документ. Действие необратимо.
- `doc_id: str` — ID документа для удаления

### list_formulas
Именованные формулы в документе.
- `doc_id: str`

### get_formula
Значение формулы.
- `doc_id: str`
- `formula_id: str`

### list_controls
Контролы (кнопки, слайдеры).
- `doc_id: str`

## API Endpoints

```
GET  /docs
POST /docs
GET  /docs/{doc_id}
DELETE /docs/{doc_id}
GET  /docs/{doc_id}/pages
GET  /docs/{doc_id}/pages/{page_id}
POST /docs/{doc_id}/pages/{page_id}/export
GET  /docs/{doc_id}/pages/{page_id}/export/{export_id}
GET  /docs/{doc_id}/tables
GET  /docs/{doc_id}/tables/{table_id}
GET  /docs/{doc_id}/tables/{table_id}/columns
GET  /docs/{doc_id}/tables/{table_id}/rows
GET  /docs/{doc_id}/tables/{table_id}/rows/{row_id}
POST /docs/{doc_id}/tables/{table_id}/rows
PUT  /docs/{doc_id}/tables/{table_id}/rows/{row_id}
DELETE /docs/{doc_id}/tables/{table_id}/rows/{row_id}
GET  /docs/{doc_id}/formulas
GET  /docs/{doc_id}/formulas/{formula_id}
GET  /docs/{doc_id}/controls
```

## Stack

- языки со строгой типизацией

## Notes

- Все ответы JSON
- Row query syntax: `'ColumnName:"value"'`
- useColumnNames=true возвращает имена колонок вместо ID

## Developer Notes

### HTTP Client Configuration

The reqwest client is configured with specific settings to avoid Coda API issues:

```rust
Client::builder()
    .pool_max_idle_per_host(0)  // Disable connection pooling
    .timeout(Duration::from_secs(60))
    .connect_timeout(Duration::from_secs(30))
    .build()
```

**Why disable connection pooling (`pool_max_idle_per_host(0)`):**
- Investigation showed that curl requests work fine, but reqwest requests fail with 404
- Hypothesis: Coda API may mishandle HTTP/2 multiplexed requests on the same connection
- Disabling connection pooling forces each request to use a fresh connection

### Debugging

Run with verbose logging:

```bash
RUST_LOG=info CODA_API_TOKEN=xxx ./target/release/coda-mcp 2>&1 | tee /tmp/coda-debug.log
```

The client logs HTTP request URLs at INFO level when debugging. Response status is logged at DEBUG level.

### Security

The following security measures are implemented:

1. **Token redaction**: The `Config` struct implements custom `Debug` to redact the API token from log output
2. **No token logging**: HTTP client does not log authorization headers or token previews
3. **URL validation**: `download_raw()` validates that download URLs are from trusted hosts only:
   - `coda.io`
   - `codahosted.io`
   - `storage.googleapis.com`
4. **Limit bounds**: User-provided limits are capped at 1000 to prevent excessive resource usage
5. **Install script security**:
   - Token input is silent (`read -sp`)
   - Config file permissions set to 600 (owner read/write only)
