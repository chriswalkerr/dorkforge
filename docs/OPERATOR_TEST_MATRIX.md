# Матрица ручной проверки операторов

Эта матрица предназначена для ручного запуска в интерфейсах поисковиков. DorkForge и обычные тесты проекта не выполняют сетевых поисковых запросов. Все строки пока `untested`; фактическое наблюдение и дату нужно заполнить после ручной проверки. Документированность оператора не означает, что его поведение прошло эмпирический тест.

`probe` и `example.com` используются как синтетические тестовые значения. Для сравнения проверяйте результат с тем же запросом без тестируемого оператора. Для неизвестного синтаксиса цель проверки — установить, применяется ли фильтр, игнорируется ли оператор или меняется ли разбор запроса.

| Провайдер | Оператор | Тестовый запрос | Ожидаемое поведение | Наблюдение | Дата | Статус |
| --- | --- | --- | --- | --- | --- | --- |
| Google | `site:` | `probe site:example.com` | Результаты ограничены указанным сайтом/доменом | Не проверено | - | `untested` |
| Google | `filetype:` | `probe filetype:pdf` | Результаты ограничены документами PDF | Не проверено | - | `untested` |
| Google | `ext:` | `probe ext:pdf` | Проверить, ограничиваются ли результаты расширением файла | Не проверено | - | `untested` |
| Google | `inurl:` | `probe inurl:example` | Проверить фильтрацию терма по URL | Не проверено | - | `untested` |
| Google | `allinurl:` | `probe allinurl:example test` | Проверить, ограничиваются ли URL все последующие термы | Не проверено | - | `untested` |
| Google | `intitle:` | `probe intitle:example` | Проверить фильтрацию терма по заголовку | Не проверено | - | `untested` |
| Google | `allintitle:` | `probe allintitle:example test` | Проверить, ограничиваются ли заголовки все последующие термы | Не проверено | - | `untested` |
| Google | `intext:` | `probe intext:example` | Проверить фильтрацию терма по тексту страницы | Не проверено | - | `untested` |
| Google | `allintext:` | `probe allintext:example test` | Проверить, ограничивается ли текст страницы всеми последующими термами | Не проверено | - | `untested` |
| Google | `OR` | `example OR sample` | Результаты соответствуют одной из альтернатив | Не проверено | - | `untested` |
| Google | `AND` | `example AND sample` | Проверить явное логическое пересечение | Не проверено | - | `untested` |
| Google | `-` | `example -sample` | Результаты исключают терм `sample` | Не проверено | - | `untested` |
| Google | `"` | `"exact phrase probe"` | Поиск точной фразы | Не проверено | - | `untested` |
| Bing | `site:` | `probe site:example.com` | Результаты ограничены сайтом/доменом | Не проверено | - | `untested` |
| Bing | `filetype:` | `probe filetype:pdf` | Результаты ограничены документами PDF | Не проверено | - | `untested` |
| Bing | `ext:` | `probe ext:pdf` | Результаты ограничены расширением файла | Не проверено | - | `untested` |
| Bing | `inurl:` | `probe inurl:example` | Официальное поведение не подтверждено; проверить фильтрацию по URL | Не проверено | - | `untested` |
| Bing | `intitle:` | `probe intitle:example` | Проверить фильтрацию терма по метаданным заголовка | Не проверено | - | `untested` |
| Bing | `inbody:` | `probe inbody:example` | Термин находится в тексте страницы; отдельно оценить надёжность | Не проверено | - | `untested` |
| Bing | `inanchor:` | `probe inanchor:example` | Термин находится в тексте ссылки | Не проверено | - | `untested` |
| Bing | `feed:` | `feed:probe` | Находит RSS/Atom-каналы для терма | Не проверено | - | `untested` |
| Bing | `hasfeed:` | `probe site:example.com hasfeed:probe` | Находит страницы сайта, содержащие RSS/Atom-канал | Не проверено | - | `untested` |
| Bing | `ip:` | `probe ip:8.8.8.8` | Результаты относятся к сайтам на указанном IPv4 | Не проверено | - | `untested` |
| Bing | `loc:` | `probe loc:US` | Результаты ограничены указанным регионом | Не проверено | - | `untested` |
| Bing | `language:` | `probe language:en` | Результаты ограничены указанным языком | Не проверено | - | `untested` |
| Bing | `contains:` | `probe contains:pdf` | Находит страницы со ссылками на указанный тип файла | Не проверено | - | `untested` |
| Bing | `url:` | `url:example.com` | Проверяет наличие домена/URL в индексе | Не проверено | - | `untested` |
| Bing | `OR` | `example OR sample` | Результаты соответствуют одной из альтернатив | Не проверено | - | `untested` |
| Bing | `AND` | `example AND sample` | Результаты содержат оба терма/условия | Не проверено | - | `untested` |
| Bing | `NOT` | `example NOT sample` | Результаты исключают условие `sample` | Не проверено | - | `untested` |
| Bing | `-` | `example -sample` | Результаты исключают терм `sample` | Не проверено | - | `untested` |
| Bing | `"` | `"exact phrase probe"` | Поиск точной фразы | Не проверено | - | `untested` |
| Yandex | `site:` | `probe site:example.com` | Поиск на сайте, его поддоменах и страницах | Не проверено | - | `untested` |
| Yandex | `host:` | `probe host:example.com` | Поиск на конкретном основном хосте | Не проверено | - | `untested` |
| Yandex | `rhost:` | `probe rhost:com.example` | Поиск на хосте с именем в обратном порядке | Не проверено | - | `untested` |
| Yandex | `url:` | `probe url:example.com/path` | Поиск страниц по адресу или префиксу URL | Не проверено | - | `untested` |
| Yandex | `domain:` | `probe domain:com` | Ограничение доменной зоной | Не проверено | - | `untested` |
| Yandex | `inurl:` | `probe inurl:example` | Официальное поведение не подтверждено текущей справкой; проверить разбор URL | Не проверено | - | `untested` |
| Yandex | `title:` | `probe title:example` | Официальное поведение не подтверждено текущей справкой; проверить фильтр заголовка | Не проверено | - | `untested` |
| Yandex | `mime:` | `probe mime:pdf` | Результаты ограничены документами поддерживаемого типа | Не проверено | - | `untested` |
| Yandex | `lang:` | `probe lang:en` | Результаты ограничены языком | Не проверено | - | `untested` |
| Yandex | `date:` | `probe date:20200101` | Результаты ограничены датой публикации | Не проверено | - | `untested` |
| Yandex | `\|` | `example \| sample` | Результаты содержат один или несколько альтернативных термов | Не проверено | - | `untested` |
| Yandex | `&&` | `example && sample` | Официальное поведение не подтверждено; проверить явное логическое пересечение | Не проверено | - | `untested` |
| Yandex | `~~` | `example ~~ sample` | Официальное поведение не подтверждено; проверить словный оператор отдельно от `site:` | Не проверено | - | `untested` |
| Yandex | `-` | `example -sample` | Исключение слова; проверить позиционное ограничение в конце запроса | Не проверено | - | `untested` |
| Yandex | `"` | `"exact phrase probe"` | Поиск фразы в заданной последовательности слов | Не проверено | - | `untested` |
| Brave | `site:` | `probe site:example.com` | Результаты ограничены сайтом | Не проверено | - | `untested` |
| Brave | `inurl:` | `probe inurl:example` | Не входит в подтверждённый allowlist; проверить, игнорируется ли или применяется | Не проверено | - | `untested` |
| Brave | `filetype:` | `probe filetype:pdf` | Результаты ограничены указанным типом файла | Не проверено | - | `untested` |
| Brave | `ext:` | `probe ext:pdf` | Результаты ограничены расширением файла | Не проверено | - | `untested` |
| Brave | `intitle:` | `probe intitle:example` | Термин присутствует в заголовке страницы | Не проверено | - | `untested` |
| Brave | `inbody:` | `probe inbody:example` | Термин присутствует в теле страницы | Не проверено | - | `untested` |
| Brave | `inpage:` | `probe inpage:example` | Термин присутствует в заголовке или теле страницы | Не проверено | - | `untested` |
| Brave | `loc:` | `probe loc:ca` | Результаты ограничены страной/регионом | Не проверено | - | `untested` |
| Brave | `lang:` | `probe lang:en` | Результаты ограничены языком | Не проверено | - | `untested` |
| Brave | `OR` | `example OR sample` | Результаты соответствуют одной из альтернатив | Не проверено | - | `untested` |
| Brave | `AND` | `example AND sample` | Результаты соответствуют обоим условиям | Не проверено | - | `untested` |
| Brave | `NOT` | `example NOT sample` | Результаты исключают условие `sample` | Не проверено | - | `untested` |
| Brave | `-` | `example -sample` | Исключает результаты с термом `sample` | Не проверено | - | `untested` |
| Brave | `+` | `example +sample` | Повышает приоритет страниц с термом `sample` | Не проверено | - | `untested` |
| Brave | `"` | `"exact phrase probe"` | Поиск точной фразы | Не проверено | - | `untested` |
| DuckDuckGo | `site:` | `probe site:example.com` | Результаты ограничены указанным сайтом | Не проверено | - | `untested` |
| DuckDuckGo | `filetype:` | `probe filetype:pdf` | Результаты ограничены документированными типами файлов | Не проверено | - | `untested` |
| DuckDuckGo | `inurl:` | `probe inurl:example` | URL содержит указанный терм; проверить ограничения на одно слово | Не проверено | - | `untested` |
| DuckDuckGo | `intitle:` | `probe intitle:example` | Заголовок страницы содержит указанный терм | Не проверено | - | `untested` |
| DuckDuckGo | `OR` | `example OR sample` | Не перечислен в официальной справке; проверить, не трактуется ли как обычный терм | Не проверено | - | `untested` |
| DuckDuckGo | `AND` | `example AND sample` | Не перечислен в официальной справке; проверить, не трактуется ли как обычный терм | Не проверено | - | `untested` |
| DuckDuckGo | `-` | `example -sample` | Результатов с термом `sample` становится меньше | Не проверено | - | `untested` |
| DuckDuckGo | `+` | `example +sample` | Повышается приоритет страниц с термом `sample` | Не проверено | - | `untested` |
| DuckDuckGo | `"` | `"exact phrase probe"` | Поиск точной фразы; проверить fallback похожих результатов | Не проверено | - | `untested` |

## Источники

- Google: <https://support.google.com/websearch/answer/2466433>
- Bing: <https://support.microsoft.com/ru-ru/bing/advanced-search-options> и <https://support.microsoft.com/ru-ru/bing/advanced-search-keywords>
- Yandex: <https://yandex.com/support/search/en/query-language/qlanguage.md>, <https://yandex.com/support/search/en/query-language/search-operators.md> и <https://yandex.com/support/search/en/query-language/search-context.md>
- Brave: <https://search.brave.com/help/operators>
- DuckDuckGo: <https://duckduckgo.com/duckduckgo-help-pages/results/syntax/>
