# ADR-0008: безопасная граница startup-регионов без war-tail

Принято. Область: World → Game, начальный selector `0x0E`, типы региона `4` (`RT_NATION`) и `5` (`RT_GODSBATTLE`).

## Проблема

`VERIFIED` по совпадающим `WorldServer/Nworldserver.exe + WorldServer.pdb`: `CGame::LoadRegionList` создаёт для типов `4` и `5` обычный `CWorldRegion`. Его `CWorldRegion::AddToByteArray` (`0x004740D0`) заканчивает snapshot после `CWorldRegion::AddSetupToByteArray` и `m_Param[0x24]`. Три war-counter туда не входят. Они добавляются только `CWorldWarRegion::AddToByteArray` (`0x004DD3F0`).

`VERIFIED` по совпадающим `GameServer/gameserver.exe + GameServer.pdb`: startup selector `0x0E` создаёт для subtype `4` `ServerNationRegion`, для subtype `5` `CServerGodsBattleRegion`. Оба наследуют `CServerWarRegion`, а их decoder-slot указывает на `CServerWarRegion::DecordFromByteArray` (`0x005D3110`). После полного `CServerRegion` prefix он без проверки длины читает ещё `m_lSymbolTotalNum`, `m_lWinVicSymbolNum` и `m_lVicSymbolNum`.

Скрытого tail внутри base snapshot нет: парный virtual hook World — `CWorldRegion::AddSetupToByteArray` (`0x00474390`), Game — `CServerRegion::DecordSetupFromByteArray` (`0x0047EAC0`); оба обрабатывают setup и затем тот же `m_Param[0x24]`.

Это не trailing-данные transport. `CMessage::CreateMessageWithoutRLE` в World-варианте (`0x00422F20`; общий nets-код той же ветки) создаёт новый `CMessage` и через `CBaseMessage::Add(void const*, long)` копирует только logical message bytes. `std::vector<unsigned char>::_Insert_n` оставляет capacity сверх `_Mylast`, но эта память не инициализируется как часть сообщения. Чтение трёх `long` за `_Mylast` поэтому является поведением вне логического wire-контракта.

Исторические журналы оригинального Game показывают успешный `Start Region` для реальных регионов типов `4/5`, но не фиксируют значения трёх прочитанных DWORD. Значения этого out-of-range tail остаются `UNKNOWN`.

## Решение

Rust World сохраняет доказанный исходный wire и не дописывает три придуманных нуля.

Rust Game для selector `0x0E`, subtype `4/5`, декодирует только парный `CServerRegion` prefix. Производные owner-ы и их NPC side effects сохраняются. Общий `CServerWarRegion::DecordFromByteArray` остаётся для путей, где sender действительно сериализует war-tail.

У нового `CServerWarRegion` Rust остаётся детерминированное состояние `Default`. Для `m_lSymbolTotalNum` и `m_lWinVicSymbolNum` ноль подтверждён конструктором оригинала `0x005D3740`; `m_lVicSymbolNum` оригинальный конструктор не инициализирует, поэтому Rust zero для него является безопасным состоянием реализации, а не утверждением о точном значении оригинальной памяти.

## Последствия

Startup больше не должен останавливаться на `InitialRegionStartup(War(Input(UnexpectedEnd)))` для этих восьми регионов. Это требует отдельной runtime-проверки и до неё имеет статус `PARTIAL`.

Если gameplay типа `4/5` достигнет логики, зависящей от отсутствующих war-counter, нельзя выводить их значения из Rust `Default`. Для такого caller-а потребуется отдельное прямое основание или наблюдение оригинального процесса.
