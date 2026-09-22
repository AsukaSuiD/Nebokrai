# Nebokrai: documentation / документация

Nebokrai восстанавливает серверное поведение «Поднебесья» (Miracle) в современной Rust/Linux-реализации. Здесь можно изучать устройство MMO, legacy interoperability и метод проверки реконструированных контрактов, даже не зная исходную игру.

**New to Nebokrai?** Read the [project overview](../README.md), then follow the guides below. The history, architecture, player journey, status, and reconstruction overviews have short English summaries. Detailed documentation is currently available primarily in Russian; Russian remains the working language.

| Путь / Reading path | С чего начать |
| --- | --- |
| New to Nebokrai | [История проекта / Project history](overview/history.md) → [архитектура / Architecture](overview/architecture.md) → [путь игрока / Player journey](overview/player-journey.md) → [текущий статус / Current status](overview/status.md). |
| Current status | [Краткий статус](overview/status.md), затем подробный [аудит](status/audit.md) конкретного сценария. |
| Development | [Руководство разработчика](development.md) → [сборка](operations/build.md) → [contribution workflow](../CONTRIBUTING.md). |
| Reconstruction | [Обзор метода](reconstruction/overview.md) → [реальный case study](reconstruction/case-study-region-entry.md) → [правила evidence](reconstruction/evidence-and-contracts.md). |
| Protocol | [Карта протокола](protocol/README.md) → каталог opcode и конкретное направление обмена. |
| Gameplay systems | [Симуляция](gameplay/simulation.md), затем нужная подсистема ниже. |
| Reference | [Карта исходников](architecture/workspace.md), [общие механизмы](architecture/shared-mechanisms.md), [ADR](decisions/README.md), [материалы анализа](reconstruction/sources.md). |
| Next steps | [Направления дальнейшей работы](overview/roadmap.md): проверяемые результаты без искусственных релизов и сроков. |

Подробные страницы ниже — основные места хранения контрактов. Обзоры объясняют маршрут чтения и не создают второй набор формул или спецификаций.

## Архитектура и код

| Вопрос | Где читать |
| --- | --- |
| Где что лежит и что входит в основной сервер? | [Карта проекта](architecture/workspace.md) |
| Кто отвечает за игрока, мир и данные? | [Владение состоянием](architecture/state-ownership.md) |
| По каким границам рефакторить существующий сервер? | [Realm, Zone и Shared: каталоги, владельцы, операции и причины разделения](architecture/realm-and-zone.md) |
| Какое готовое решение использовать и где границы его применения? | [Общие механизмы](architecture/shared-mechanisms.md) |
| Какие типы, кодировки, часы и случайные числа использовать? | [Значения и совместимость](architecture/values-and-compatibility.md) |
| Как запускается процесс и где его менять? | [Запуск и жизненный цикл](server/process-model.md) |
| Как устроены вход, оплата и аукцион? | [Auth, Login, Billing и Misc](server/auth-login-and-services.md) |
| Как взаимодействуют World и Game? | [Игровой и мировой серверы](server/world-and-game.md) |
| В каком порядке исполняются сообщения и таймеры? | [Серверные циклы](server/runtime-ordering.md) |
| Как TCP-запрос доходит до обработчика и ответа? | [Сетевой runtime](server/network-runtime.md) |
| Где выполняется SQL и как сохраняется персонаж? | [База данных](architecture/database.md) |
| Как состояние попадает в снимок и что происходит при ошибке записи? | [Сохранение и фоновые работники](architecture/persistence.md) |
| Как файл с настройками становится игровыми данными? | [Конфигурация, пакеты ресурсов и обновление Game](architecture/resources-and-configuration.md) |
| Почему выбраны эти технологии и границы? | [Архитектурные решения](decisions/README.md) |

## Протокол

[Навигация по протоколу](protocol/README.md) разделяет сообщение, TCP-кадр и состояние соединения.

- [Заголовок и сериализация](protocol/message-header.md), [TCP, CRC и RLE](protocol/transport.md), [соединения и очереди](protocol/connection-lifecycle.md).
- [Каталог opcode](protocol/opcode-catalog.md).
- [Вход в Login](protocol/login-auth.md), [вход в Game](protocol/game-login.md), [команды игрока](protocol/game-actions.md).

## Игровые системы

[Симуляция](gameplay/simulation.md) показывает общий путь команды игрока и помогает выбрать подсистему.

- [Движение](gameplay/movement.md), [бой](gameplay/combat.md), [навыки](gameplay/skills.md), [ИИ монстров и NPC](gameplay/npc-ai.md).
- [Живые объекты и время жизни](gameplay/objects-and-lifetimes.md), [атрибуты и состояния](gameplay/attributes-and-states.md).
- [Регионы, видимость и переходы Game](gameplay/regions-and-visibility.md), [смерть, награды и развитие](gameplay/death-and-progression.md).
- [Предметы и контейнеры](gameplay/items.md), [торговля](gameplay/trade.md), [задания](gameplay/quests.md).
- [Аукцион и применение платежей](gameplay/auction-and-payments.md).
- [Сценарии и их исполнение](gameplay/scripting.md).
- [Жизненный цикл персонажа](gameplay/player-lifecycle.md).
- [Регионы и организации](gameplay/regions-and-organizations.md), [сессии и рейтинги](gameplay/world-sessions-and-rankings.md), [войны](gameplay/world-wars.md).

## Сборка и эксплуатация

- [Сборка](operations/build.md) — платформа, toolchain и команды.
- [Локальный стенд](operations/hybrid-runtime.md) — данные, конфигурация, службы и порядок запуска.
- [Совместный запуск Rust-служб](operations/rust-runtime.md) — все шесть процессов, общий кэш и локальные клиентские порты.
- [Диагностика](operations/diagnostics.md) — доступные сообщения, поиск этапа отказа и ограничения журналирования.
- [Состояние проекта](status/audit.md) — реализованные части, результаты проверок и известные пробелы.

## Работа с реконструкцией

[Как проверять поведение оригинала](reconstruction/evidence-and-contracts.md) нужно при восстановлении неизвестной механики или спорного формата. [Материалы анализа](reconstruction/sources.md) описывает необязательные локальные данные.

Документацию меняют вместе с поведением. На странице объясняют назначение, устройство и причину решения; точные поля и формулы оставляют там, где ими пользуются. Принятые правила отделяют от текущей реализации и её расхождений. Подход закреплён в [ADR-0001](decisions/0001-system-specification.md) и [ADR-0007](decisions/0007-reuse-existing-mechanisms.md).


## Участие

[CONTRIBUTING](../CONTRIBUTING.md) и [SECURITY](../SECURITY.md). Собственный код и документация проекта распространяются по [AGPL-3.0-only](../LICENSE); условия внешнего вклада описаны в CONTRIBUTING.
