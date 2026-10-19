# DorkForge: примеры использования

Все команды работают локально: DorkForge генерирует запросы и ссылки для ручного открытия, но не выполняет поиск, сетевые подключения или сканирование.

В примерах предполагается, что бинарный файл доступен как `dorkforge`. Для локальной сборки используйте `cargo run --` перед командой или `target/release/dorkforge` после `cargo build --release`.

## Справочники и shell completion

```bash
# Поддерживаемые типы, категории и провайдеры
dorkforge types
dorkforge categories
dorkforge providers

# Сгенерировать completion для текущей оболочки
dorkforge completions zsh > ~/.zfunc/_dorkforge
dorkforge completions bash > ~/.local/share/bash-completion/completions/dorkforge
dorkforge completions fish > ~/.config/fish/completions/dorkforge.fish
```

## Быстрый старт

```bash
# Один идентификатор, читаемый TXT в stdout
dorkforge generate -t domain -v example.org

# Все провайдеры и все артефакты в каталог results/
dorkforge generate -t domain -v example.org -f all -o results

# Только Google и Brave, конкретные категории
dorkforge generate -t email -v analyst@example.org \
  -p google,brave -c exact_match,documents,code -f html -o report.html
```

`-p` принимает `google`, `bing`, `yandex`, `brave`, `duckduckgo` (также `ddg`). Канонические категории перечислены командой `dorkforge categories`; старые aliases `profiles`, `code`, `archives`, `company`, `security` и `tech` остаются допустимыми для фильтрации.

Провайдерские шаблоны дополняет общий слой [shared_templates.toml](../config/templates/shared_templates.toml). Общие запросы используют только синтаксис, подходящий всем указанным поисковикам; точные дубли каталогов провайдеров проверяются тестами. `high_recall` не является метрикой полноты, и генератор не обещает полный охват поисковых результатов или поддоменов.

## Все типы идентификаторов

Явно указывать `--type` рекомендуется всегда: это исключает неоднозначность автоматического определения.

```bash
dorkforge generate -t email         -v user@example.org
dorkforge generate -t wallet        -v 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --network eth
dorkforge generate -t username      -v @security_lead
dorkforge generate -t name          -v "Иван Иванов"
dorkforge generate -t domain        -v example.org
dorkforge generate -t ip            -v 2001:db8::1
dorkforge generate -t phone         -v "060 123 4567" --country RS
dorkforge generate -t license_plate -v "BG 123-AA" --country RS
dorkforge generate -t company       -v "Acme Corporation LLC"
dorkforge generate -t person_id     -v 7707083893 --country RU
dorkforge generate -t url           -v https://site.example/secret/passwords.txt
dorkforge generate -t vin           -v 1HGCR2F83HA000000
dorkforge generate -t file_hash     -v e4d909c290d0fb1ca068ffaddf22cbd0
dorkforge generate -t text_phrase   -v "CONFIDENTIAL PROJECT OMEGA"
dorkforge generate -t asn           -v AS15169
dorkforge generate -t certificate   -v 3B:A0:6B:7C:F4:A8:12:44:89:10:00:11:22:33:44:55
```

Подсказки `--country` и `--network` не являются сетевым обогащением: они только управляют локальной нормализацией и шаблонами.

## Источники входных данных

### Значение из аргумента

```bash
dorkforge generate --type email --value analyst@example.org
```

### Одно значение из stdin

```bash
printf '%s\n' 'analyst@example.org' | dorkforge generate -t email -v -
```

`--watch` со stdin недоступен, потому что stdin является одноразовым источником.

### Текстовый batch-файл

Одна строка — один идентификатор. При неоднозначных строках используйте JSON с типами.

```bash
dorkforge generate --input identifiers.txt -f both -o results
```

### JSON batch-файл

```json
[
  { "type": "email", "value": "analyst@example.org" },
  { "type": "domain", "value": "targetcorp.com" },
  { "type": "wallet", "value": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "network": "eth" },
  { "type": "license_plate", "value": "BG-123-AA", "country": "RS" }
]
```

```bash
dorkforge generate -i identifiers.json -f all -o results

# Остановить весь batch при первой ошибочной записи
dorkforge generate -i identifiers.json --strict -f json -o results
```

Без `--strict` неверные строки пропускаются с предупреждением. Для машинного сценария без диагностических сообщений используйте `--quiet`.

## Фильтрация, варианты и лимиты

```bash
# Категории с короткими алиасами
dorkforge generate -t domain -v example.org -c documents,profiles,code

# Ограничить поисковики
dorkforge generate -t username -v alice_dev -p google,duckduckgo

# Отключить дополнительные варианты записи
dorkforge generate -t phone -v "+381 60 1234567" --max-variants 1

# Ограничить объём результата для пакетного запуска
dorkforge generate -i identifiers.json \
  --max-variants 5 \
  --max-queries-per-identifier 30 \
  --max-queries-per-provider 100
```

Генерация остаётся детерминированной: одинаковые входные данные, шаблоны и параметры дают тот же порядок запросов. `fingerprint` — первые 8 шестнадцатеричных символов SHA-256 от текста запроса и служит короткой меткой. `record_id` (`dfq1_` + SHA-256) стабильно идентифицирует пару «провайдер + запрос»; прежний `query_id` сохраняет свою семантику.

Строгая дедупликация схлопывает повторные пробелы, но сохраняет регистр значения. Запросы разных провайдеров не удаляются друг из-за друга. В JSON `unique_query_texts` считает разные тексты, `unique_provider_queries` — пары «провайдер + текст», а `templates_used` — число использованных шаблонов.

## Форматы и места вывода

| Флаг | Содержимое | Без `-o` | В каталоге `-o results` |
| --- | --- | --- | --- |
| `-f txt` | TXT | stdout | `results/dorks.txt` |
| `-f json` | JSON | stdout | `results/dorks.json` |
| `-f html` | HTML с ссылками | stdout | `results/dorks.html` |
| `-f both` | TXT + JSON | TXT в stdout с предупреждением | `dorks.txt`, `dorks.json` |
| `-f all` | TXT + JSON + HTML | TXT в stdout с предупреждением | `dorks.txt`, `dorks.json`, `dorks.html` |

```bash
# Чистый TXT: один запрос в строке, удобен для pipe
dorkforge generate -t email -v analyst@example.org -f txt -m clean > dorks.txt

# Читаемый TXT: сгруппирован по поисковику, категории и варианту
dorkforge generate -t email -v analyst@example.org -f txt -m readable

# Только готовые поисковые URL для TXT
dorkforge generate -t domain -v example.org --urls -m clean > urls.txt

# HTML-отчёт: готовые кликабельные ссылки, без внешних скриптов и ресурсов
dorkforge generate -t domain -v example.org -f html -o report.html

# При прямом имени для нескольких форматов расширение заменяется
dorkforge generate -t domain -v example.org -f all -o artifacts/report.txt
# Создаст artifacts/report.txt, artifacts/report.json, artifacts/report.html
```

По умолчанию существующий файл не перезаписывается. Для осознанной перезаписи добавьте `--overwrite`.

```bash
dorkforge generate -t domain -v example.org -f all -o results --overwrite
```

`--urls` предназначен для TXT; HTML всегда содержит ссылки, а JSON сохраняет исходные структурированные запросы.

## Внешние шаблоны

```bash
# Полностью заменить встроенный набор шаблонов
dorkforge generate -t email -v analyst@example.org --config-dir ./templates -f all -o results

# Добавить шаблоны к встроенному набору
dorkforge generate -t email -v analyst@example.org --extra-templates ./templates -f all -o results
```

Минимальный TOML-шаблон:

```toml
[[template]]
id = "custom_email_mentions"
enabled = true
identifier_type = "email"
category = "mentions"
providers = ["google", "bing"]
template = '"{value}"'
tags = ["email", "custom"]
```

Доступные placeholders: `{value}`, `{domain}`, `{country}`. Провайдер и категория проходят проверку при загрузке шаблона. Необязательные поля `description`, `rationale`, `limitations`, `source_domains` и `template_version` сохраняются в JSON, если соответствующее поле входит в формат отчёта.

Проверка шаблона задаётся таблицей:

```toml
[template.validation]
status = "untested"
checked_at = "2026-10-16"
comment = "Ручной поисковый тест ещё не выполнялся"
```

Допустимые статусы: `untested`, `passed`, `partial`, `failed`, `stale`. `passed` можно указывать только после ручной проверки; обычные тесты проекта не обращаются к поисковикам. Старая строковая форма `validation = "untested"` остаётся совместимой.

Статусы операторов и официальные ссылки находятся в `config/providers.toml`. `documentation_status` принимает `documented`, `experimental`, `unknown` или `unsupported`; `test_status` хранит отдельный результат ручной проверки. Документированность сама по себе не означает, что поведение проверено.

## Hot reload шаблонов

`--watch` отслеживает TOML-файлы во внешнем каталоге и повторно генерирует артефакты при добавлении, удалении или изменении содержимого. Для первого запуска существующие файлы всё ещё защищены; добавьте `--overwrite`, если они уже существуют.

```bash
# Проверка раз в секунду
dorkforge generate -t email -v analyst@example.org \
  --extra-templates ./templates --watch -f all -o results

# Проверка раз в 3 секунды
dorkforge generate -t domain -v example.org \
  --config-dir ./templates --watch --watch-interval 3 -f html -o results
```

Если редактор на мгновение сохраняет невалидный TOML, DorkForge сообщает ошибку и сохраняет предыдущие артефакты. Следующее валидное изменение перегенерирует результат. Остановите режим сочетанием `Ctrl-C`.

## OSINT pivot graph

Команда `graph` принимает JSON, созданный через `generate -f json`, и выводит Graphviz DOT. В графе есть входные идентификаторы, запросы, категории и провайдеры. Узлы запросов имеют URL-атрибут, поэтому после рендеринга SVG становятся кликабельными.

```bash
dorkforge generate -t domain -v example.org -f json -o results
dorkforge graph results/dorks.json -o results/pivots.dot

# Необязательный рендеринг при установленном Graphviz
dot -Tsvg results/pivots.dot -o results/pivots.svg

# Вывести DOT в stdout
dorkforge graph results/dorks.json

# Перезаписать существующий файл
dorkforge graph results/dorks.json -o results/pivots.dot --overwrite
```

## Diff запусков

Diff сравнивает полную пару «провайдер + запрос», а не короткий fingerprint. Это надёжно и при теоретической коллизии 8-символьного отпечатка.

```bash
# Читаемый текстовый отчёт в stdout
dorkforge diff before/dorks.json after/dorks.json

# JSON-отчёт для автоматизации
dorkforge diff before/dorks.json after/dorks.json -f json -o changes.json

JSON diff содержит массивы `added` и `removed`, а также число `unchanged`.

## Комбинированные идентификаторы (Multi-Identifier OSINT)

DorkForge поддерживает одновременный поиск связок нескольких идентификаторов, когда шаблоны требуют нескольких типов входных данных (например, `username + domain`, `name + company`, `vin + license_plate`).

### Одиночная комбинация из CLI

```bash
# Поиск связки никнейма и корпоративного домена
dorkforge generate \
  --combine username:security_lead \
  --combine domain:corp.example.org \
  --format txt --mode readable

# Спецификация с явными лимитами декартова произведения вариантов
dorkforge generate \
  --combine name:"Иван Иванов" \
  --combine company:"ООО Вектор" \
  --max-combination-variants 4 \
  --max-variant-combinations 16 \
  --max-combination-queries 50 \
  --format json -o results
```

### Пакетный файл комбинаций (JSON)

Файл `examples/combinations.json` содержит массив сценариев связок:

```json
{
  "combinations": [
    {
      "id": "scenario_analyst_domain",
      "identifiers": [
        { "type": "username", "value": "analyst" },
        { "type": "domain", "value": "threatintel.org" }
      ]
    },
    {
      "id": "scenario_executive_company",
      "identifiers": [
        { "type": "name", "value": "John Doe" },
        { "type": "company", "value": "Acme Corp" }
      ]
    }
  ]
}
```

Запуск пакетной генерации комбинаций:

```bash
dorkforge generate -i examples/combinations.json -f all -o results
```

В итоговом JSON-файле каждый запрос комбинации содержит поле `identifiers` с полной трассировкой использованных типов и вариантов.

## Матрица покрытия источников (Coverage)

Команда `coverage` анализирует плотность покрытия категорий, провайдеров и источников данных OSINT (`config/sources.toml`).

```bash
# Вывод таблицы покрытия в stdout
dorkforge coverage

# Экспорт матрицы в JSON
dorkforge coverage --format json -o coverage.json

# Фильтрация по конкретному провайдеру, категории или типу идентификатора
dorkforge coverage --provider google --category public_documents
dorkforge coverage --type domain
```

## Линтинг и валидация шаблонов (Lint Templates)

Команда `lint-templates` производит офлайн-аудит TOML-каталогов шаблонов на предмет синтаксических ошибок, плейсхолдеров, совместимости операторов с базами поисковых систем и валидности синтаксиса URL документации:

```bash
# Проверка встроенных шаблонов
dorkforge lint-templates

# Строгий режим: любые замечания (warning/error) вызывают код возврата 1
dorkforge lint-templates --strict

# Проверка внешнего каталога пользовательских шаблонов
dorkforge lint-templates --path ./my-custom-templates --strict
```

