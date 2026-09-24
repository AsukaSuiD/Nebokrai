# Клиентский запрос входа на LoginServer

Эта страница описывает достигнутые ветви `0x2FD01` (обычный вход) и `0x2FD0B` (расширенный вход) после проверки [входного клиентского кадра](transport.md). Источник — `loginserver.exe + LoginServer.pdb`, исходный owner `applogin/message/logmessage.cpp`, сохранённый в [Rust `OnLogMessage`](../../server/rust/src/loginserver/applogin/message/logmessage.rs). Порядок чтения и обработанные ветви имеют статус `VERIFIED` по статически сохранённому EXE/PDB контракту; побайтовая сверка с живым исходным клиентом и поведение любого недостающего payload — `UNKNOWN`. Все поля ниже идут **после** [16-байтового `CBaseMessage` заголовка](message-header.md), хранятся в little-endian там, где числовые, и не включают внешние 12 байт и RLE.

## `0x2FD01`: порядок чтения

| Порядок | Поле | Wire-представление | Следствие |
| ---: | --- | --- | --- |
| 1 | Marker | `long`, 4 байта | Ожидается `6`; неверное значение даёт клиенту `0xAF501` с однобайтовым кодом `4`. |
| 2 | Версия клиента | `long`, 4 байта | Сверяется с версией настроек LoginServer **после** account и digest. Несовпадение также даёт код `4`. |
| 3 | Account | C-строка, bounded `GetStr(0x20)` | Все ASCII-пробелы удаляются. Пустая строка и длина `>=0x20` прекращают ветвь без постановки login-запроса. После marker/version проверяются запрещённые `'`, `=`, пробел; нарушение даёт код `5`. Затем ASCII `A..Z` приводятся к нижнему регистру. |
| 4 | Password digest | `long` со значением `0x10`, затем **ровно 16 сырых байт** | Неполный либо иной размер прекращает ветвь до проверки marker/version. Это length-prefixed raw bytes, не C-строка. |
| 5 | Client code | `short`, 2 байта | Передаётся вместе с запросом в CD-key FIFO. При нехватке байт базовый reader даёт default `0`. |
| 6 | Encryption key | `long`, 4 байта | Значение передаётся в CD-key FIFO. Само имя поля **не доказывает** шифрование TCP-кадра; его смысл дальше по очереди требует отдельной проверки. |
| 7 | Неиспользуемое здесь поле | C-строка, bounded `GetStr(0x40)` | Читается для сдвига курсора, но здесь не используется. Нельзя удалить его из wire layout. |
| 8 | World server | C-строка, bounded `GetStr(0x14)` | Передаётся в CD-key FIFO вместе с account, digest, client code, key и версией. |

После проверки LoginServer ставит запрос в [CD-key очередь](../../server/rust/src/loginserver/loginserver/loginqueue.rs) с socket ID и IP **из metadata соединения**, а не из клиентского payload. Доменный исход дальнейшей проверки и способ выдачи `0xAF503` нельзя выводить только из этого пакета. Важный порядок: account/digest могут завершить ветвь молча **до** ответа о неверной версии; перепроверка версии прежде разбора строк изменила бы наблюдаемое поведение. Источник: [обычная ветвь и helpers](../../server/rust/src/loginserver/applogin/message/logmessage.rs). `VERIFIED` для указанного порядка, полный end-to-end вход `PARTIAL`.

## `0x2FD0B`: отдельная расширенная ветвь

Формат после заголовка: `long marker`, `long version`, C-строка world server (граница `0x20`), C-строка account (`0x20`), C-строка password source (`0x104`). Пустой password прекращает обработку раньше проверки пустого account. Из account удаляются ASCII-пробелы, но **приведение к lowercase здесь отсутствует**. Затем исходный код считает длину password source без пробелов, однако берёт префикс **исходных** байт именно этой длины; пробелы внутри префикса могут остаться, а байты с хвоста отбрасываются. NUL добавляется к digest только если длина не сократилась. Это подтверждённая странность исходной ветви; «нормальная» очистка всех пробелов меняла бы digest.

Marker `6` и версия настроек проверяются после сборки digest. Отказ посылает `0xAF50A` с `long(6)` и двумя пустыми C-строками. При успехе в CD-key FIFO ставится вариант `1`, нулевые client code и encryption key и сохранённый world server. [Ветвь и ответ](../../server/rust/src/loginserver/applogin/message/logmessage.rs). `VERIFIED` для достигнутого кода; смысл каждого необработанного хвостового байта `UNKNOWN`.

## Проверка Login → Auth → Login

После клиентского разбора Login выбирает маршрут в `CLoginQueue::on_quest_cdkey`; Auth используется не во всех ветвях. Выбор GAS, локального пароля и прямого World, а также жизнь pending-записи описаны в [служебных процессах](../server/auth-login-and-services.md).

Для обычной Auth-проверки текущие writer и reader совпадают в таком порядке полей после общего заголовка:

| Тип и направление | Payload |
| --- | --- |
| `0xCF501`, Login → Auth | C-строка account; C-строка password с текстовым hex-digest; `u32 client_ip`; `i32 client_socket_id`. |
| `0xCF601`, Auth → Login | `i32 result`; C-строка account; `u32 client_ip`; `i32 client_socket_id`. |

Источники: Login [`send_quest_message` / `AuthManager::on_response_auth`](../../server/rust/src/loginserver/loginserver/authmanager.rs), Auth [`AuthMessageHandlers::on_auth_account`](../../server/rust/src/authserver/appauth/message/message_func.rs) и [`send_auth_result`](../../server/rust/src/authserver/src/cgame.rs). Это межсерверный [CRC-кадр без RLE](transport.md). В задании Auth `return_socket_id` хранится отдельно и берётся из принятого Login-соединения; `client_socket_id` в payload относится к конечному клиенту внутри Login.

Локальный timeout Login создаёт такую же форму `0xCF601` с результатом `4` и публикует её в очередь Auth events без отправки в сеть. Корреляция текущего `AuthManager` использует account; переданные IP/socket читаются из ответа. Поэтому эти поля не следует описывать как уникальный request ID.

Auth также реализует `0xCF502` → `0xCF602` с условными дополнительными данными. Это не автоматическое продолжение клиентского `0x2FD0B`: текущий `AuthManager::add_quest` строит `0xCF501`, а Login `AsMessageHandlers` отдельно обрабатывает `0xCF601`; `0xCF602` попадает в `Unknown`. Связь расширенной Auth-пары с конкретным внешним сценарием остаётся `UNKNOWN`.

## Ошибки, транспорт и предел знания

Login принимает эти типы только после внешних проверок длины/CRC/RLE и фильтра диапазона `0x2FD01..=0x3FBFF` ([receive owner](../../server/rust/realm/src/app/login_server_client.rs)). Неверный CRC, слишком длинный кадр и opcode вне диапазона приводят к diagnostic → forbid IP → `QUIT`; доменные ошибки account/password/version выше имеют **другую** реакцию. Не следует переносить сетевую политику запрета IP на обычный отказ логина. Текстовая кодировка account/world за пределами описанной ASCII-обработки и назначение `encryption key` после CD-key очереди пока `UNKNOWN`. Никакого криптографического алгоритма для этих полей эта страница не утверждает.
