# GameServer: NPC, регионы, AI, бой и прикладные владельцы Zone

Машинное свидетельство для production-файлов `server/rust/zone/src/` (кроме
`skills/`): таблицы pub/RVA, регистровые и стековые подробности и точные
ветки, вынесенные из верхних комментариев исходников при их нормализации.
Правила оформления статусов — [evidence-and-contracts.md](evidence-and-contracts.md).

## Идентификаторы сборки

Если у строки раздела не сказано иное, сверка выполнена по паре:

- EXE: `original/server/Miracle_server/GameServer/gameserver.exe`, SHA-256
  `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
  ImageBase `0x400000`, PE timestamp `0x53FAFF2D`.
- PDB: `original/server/Miracle_server/GameServer/GameServer.pdb`, SHA-256
  `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`, CodeView
  RSDS GUID `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2 (match, проверено
  `.local/evidence/symbols.py identity`).
- Карта pubs: `server/rust/src/manifest/_gameserver_export_manifest.toml`.
- Нотация `N:00xxxxxx` — section:offset как в Ghidra pubs; `.text` начинается
  с RVA `0x1000`, т.е. `1:001d2e90` = RVA `0x001D3E90` = VA `0x005D3E90`;
  «истинный RVA» = pub offset + `0x1000`, а VA − `0x400000` даёт RVA.

Второй артефакт упоминается у войны четырёх стран: пара `Nworldserver.exe` +
`WorldServer.pdb` из того же комплекта `original/server/Miracle_server/` —
World serializer использовался как парная сторона wire-формата.

## NPC и базовые фигуры

Реализация: `zone::regions::{npc, baseobject, shape, moveshape, skillregistry, monster, build, citygate, summonedcreature}`;
hub-агрегаты — `appserver/` одноимённых файлов старого пакета.

### CNpc (type 500)

| функция | pub/RVA | статус | суть факта |
|---|---|---|---|
| ctor `CNpc` | `1:001d2e90` (VA `0x005D3E90`) | VERIFIED_DISASSEMBLY | вызывает базовый `CMoveShape` (`CALL 0x4D0C60`), вешает vtable `0x0065DA1C`, пишет только type `500` (`+0x4`), пустую script-строку (MSVC `std::string +0x1D4`, capacity `+0x1EC = 0xF`) и `m_bShowList = 1` (`+0x1F0`); live `+0x1F4`/born `+0x1F8` ctor не трогает — Rust хранит `Option<u32>` |
| script-колонка | inline в `CServerRegion::AddNpc` VA `0x00480A40` | VERIFIED_DISASSEMBLY | отдельного pub `CNpc::SetScriptFile` нет (соседний `SetScriptFile` VA `0x005CBAC0` принадлежит `CBuild`); запись `m_strScriptFile` — strlen-цикл до NUL + `std::string::assign` по `+0x1D4`, та же strcpy-семантика, что у `CMonster::SetScriptFile` RVA `0x00038C70` и `CBuild::SetScriptFile` VA `0x005CBAC0` |
| lifetime-предикат из `AI` | `1:001d2dd0` (VA `0x005D3DD0`) | VERIFIED_DISASSEMBLY | `live == 0` отключает срок; `EAX = GetTickCount() - born` с wrapping; unsigned `CMP; JNC` — истечение при `live < now - born`; дальше AI шлёт around `0xBF504(type,id,0)` и удаляет объект виртуальным слотом `+0x24` |
| `LossHP` | `1:001d2ed0` (VA `0x005D3ED0`) | VERIFIED_DISASSEMBLY | `XOR AX,AX; RET 8` — всегда ноль, аргументы не читаются; новый виртуал `CNpc` (расширение `+0x1A4..+0x1B8`, слот `+0x1B4`); остальные пять виртуалов расширения, включая `IsDied`, свёрнуты ICF в funclet RVA `0x00201200` (`XOR EAX,EAX; RET`) |
| `DecordFromByteArray` | `1:001e8840` (VA `0x005E9840`) | VERIFIED_DISASSEMBLY | `MOV AL,1; RET 0xC` — ничего не читает, возвращает true |
| `Talk` | `1:001d2f50` (VA `0x005D3F50`) | VERIFIED_DISASSEMBLY | девять area по таблице `_area` `0x006A3B98..0x006A3BE0`, только type `0x190`, строгие фильтры `abs(dx) < AREA_WIDTH`/`abs(dy) < AREA_HEIGHT` (глобалы `0x0069EFC8`/`0x0069EFCC`), кадр `0xBF801`: long 0, long 500, long id, строки имени и текста; кадр формирует hub `CMessage` |
| `AddToByteArray` | `1:001d2dc0` (VA `0x005D3DC0`) | VERIFIED_DISASSEMBLY | `JMP 0x0045B250` — tail-jump в `CShape::AddToByteArray` без NPC-полей |
| vtable `CNpc` / `CMoveShape` | `2:00012a1c` = RVA `0x0025DA1C` / `2:00006e7c` = RVA `0x00251E7C` | VERIFIED_DISASSEMBLY | 111/105 слотов; отличия `CNpc` — dtor, codec-пара, `AI`, `IsAttackAble` (`+0x134`, у базы `_purecall`) и шесть виртуалов конца таблицы; сырой дамп — `.local/evidence/audit-npc-vtable.txt` |

Figure/HP-факты: `GetFigure` — слот `+0x8C` → общий ICF-funclet RVA `0x000856A0`
(`XOR AL,AL; RET 4`, ICF x7; привязка слота — `CShape::SetBlock` RVA `0x0005BA60`
вызывает `[this+0x8C]` с аргументами 2 и 0; совпадение слота `+0x18` с тем же
funclet — ICF-свёртка другого тела, не `GetFigure`) — figure по всем направлениям
нулевая виртуально-константная. `GetHP` — слот `+0xD0` → ICF-funclet RVA
`0x00201200` (ICF x55; привязка — невиртуальный `CMoveShape::IsDied` RVA
`0x000CCF20` делает `CALL [vftable+0xD0]`). Условие записи блока в `SetPosXY` RVA
`0x000CD050` — `CALL [+0xD0]; TEST; JA`, иначе `CMP [ESI+4],0x1F4; JNE` — в
точности `hp != 0 || type == 500`. `GetBeAttackedPoint` — слот `+0xAC` → RVA
`0x0004A250` (реальная base-impl, не thunk; NPC отвечает своей клеткой через
`GetTileX/Y` `0x0005B110/0x0005B140`; figure там не читается), overrides только у
`CBuild` (`0x001DD350`) и `CMonster` (`0x000E6AA0`). Два раздельных pub `IsDied`:
невиртуальный `CMoveShape` — `return virtual GetHP() == 0`; виртуальный `CNpc` —
ICF-ноль, точный слот внутри пятёрки return-0 (`+0x1A4..+0x1B8`) машинно
неразличим, ординал не ставится. `current_area: None` подтверждён ctor `CShape`
RVA `0x0005B9A0` (`[+0x60] = 0`, `[+0x80] = 0`) и NULL-branch `SetPosXY`
(change-state `SHAPE_CHANGE_NONE`, next-area не пишется).

### CBaseObject и CShape

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CBaseObject::AddToByteArray` | `0x000FC300` | VERIFIED_DISASSEMBLY | три little-endian long + имя с NUL |
| ctor scalar/name часть | `0x000FC3C0` | VERIFIED_DISASSEMBLY | signed `+0x4 type`, `+0x8 ID`, нулевой `CGUID +0xC`, `+0x1C GraphicsID`, byte-string `+0x20`, include-child `+0x3C == true`, null father `+0x40` |
| `CBaseObject::DecordFromByteArray` | `0x000FC440` | VERIFIED_DISASSEMBLY | локальный `char[256]`, тот же порядок, на normal return явное `AL=1` по `0x004FC4BD` |
| identity helpers `GetHashValue/CalculateType/CalculateID` | `0x000FC0C0/0x000FC0E0/0x000FC0F0` | VERIFIED_DISASSEMBLY | верхний DWORD хранит type с sign-extension отрицательного ID, нижний — битовый образ ID |
| ctor `CShape` | `0x0005B9A0` | VERIFIED_DISASSEMBLY | region-link `+0x40`, region ID `+0x44`, float X/Y, area-links `+0x60/+0x64`, next-area `+0x68/+0x6C`, нулевые direction/position/state/action, speed `2000.0` |
| `CShape::SetPosXY` (base) | `0x0002ABE0` | VERIFIED_DISASSEMBLY | — |
| `GetTileX/GetTileY/SetTileXY` | `0x0005B110/0x0005B140/0x0005B170` | VERIFIED_DISASSEMBLY | x87 truncation toward zero; неопределённый результат для non-finite/out-of-range → `BLOCKED_MISSING_FACT` |
| direction/geometry | `0x0004A1C0/0x000FC5C0/0x0005B2B0..0x0005B380` | VERIFIED_DISASSEMBLY | — |
| `CShape::SetBlock` | `0x0005BA60` | VERIFIED_DISASSEMBLY | меняет только клетки с block `3 -> 0` либо `0 -> 3` в прямоугольнике virtual figure `DIR 2/DIR 0`; вызывает слот `+0x8C` с аргументами 2 и 0 как half-extents; out-of-bounds footprint пропускается |
| `IsInAround` | `0x0005BCE0` | VERIFIED_DISASSEMBLY | равенство region ID, обе area-ссылки существуют, abs-разница X и Y < 2 |
| `Distance(CShape*)` | `0x0005B390` | VERIFIED_DISASSEMBLY | truncation позиций, virtual figure extents, wrapping subtraction, signed max без clamp к нулю |
| `RealDistance(CShape*)` | `0x0005B780` | VERIFIED_DISASSEMBLY | большая clearance-ось, вычитание figure extent обеих сторон, евклидово масштабирование; null → `LONG_MAX` |
| координатные `Distance`/`RealDistance` | `0x0005B580..0x0005B730` | VERIFIED_DISASSEMBLY | — |
| `InitMoveCheckCellList` | `0x0005BE60` | VERIFIED_DISASSEMBLY | ровно 96 offsets по трём figure × восемь direction, insertion-order и повторный append сохранены |
| persistence decode | `0x0005B280/0x0005BC30` | VERIFIED_DISASSEMBLY | сериализованная position читается, но live `m_lPos` становится нулём (сохранённая странность) |

### CMoveShape

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CMoveShape::SetPosXY` | `0x000CD050` | VERIFIED_DISASSEMBLY | условие записи блока — см. текст CNpc выше |
| `SetMoveable/SetFightable` | `0x000CCEE0/0x000CCE10` | VERIFIED_DISASSEMBLY | счётчики запрета движения и боя |
| `God/SetKilledMeAttackInfo/IsDied/GetDestDir` | `0x0002ACB0/0x000CCE50/0x000CCF20/0x000CCF60` | VERIFIED_DISASSEMBLY | единственная запись убийцы после пакета смерти `0xBF60B` |
| `ForceMove/OnMove/OnSetPosition/OnEnterRegion` | `0x000CD1A0/0x000CD490/0x000CD5C0/0x000CEF40` | VERIFIED_DISASSEMBLY | quirk `ForceMove`: верхняя граница Y записывает `width - 1`; wire `0xBF603/604/605` сохраняет исходный порядок полей |
| `CMoveShape::Stiffen` | `0x000CD2F0` | VERIFIED_DISASSEMBLY | второй замер часов при просроченном окне, damage ratio по unsigned HP, скан thresholds/probabilities сверху вниз, вычитание `GetReAnk` перед signed-сравнением с `random(100)`, возврат delay |

PDB `tagProperties` (type `0x6D94`, fieldlist `0x6D93`) задаёт 25 signed LONG
размером `0x64` по полю `CMoveShape +0x84` — это не wire-layout и не копия
свойств монстра.

### Реестр навыков CMoveShape

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `AddSkill` (ID-вариант) | `0x000D1C70` | VERIFIED_DISASSEMBLY | категория по `CSkillFactory`; повторный ID допустим при нулевом уровне первого найденного; повышение ненулевого уровня удаляет первое совпадение и добавляет в хвост |
| `AddSkill`/`DelSkill` name-варианты | `0x000D3C70/0x000CF560` | не перенесены | — |
| `GetSkill`/`DelSkill` (ID) | `0x000CF320` | VERIFIED_DISASSEMBLY | категория через `QuerySkillType(ID, 1)`; UNKNOWN отвергается до current cleanup; ID 0 проходит cleanup и только потом category lookup; удаляется только первый экземпляр |
| `ClearSkills` | `0x000CDCE0` | VERIFIED_DISASSEMBLY | неразрешённый current ID сохраняется; очистка Attack → Defense → Summon → State |
| `GetCurrentSkill` | `0x000CDC10` | VERIFIED_DISASSEMBLY | выбранный ID разрешается реестром; сам ID наличие не доказывает |
| `SetCurrentSkill`/`SetItemSkill` | `0x000CEEE0/0x000D1570` | VERIFIED_DISASSEMBLY | — |
| `GetDefaultAttackSkillID` | `0x000CE240` | VERIFIED_DISASSEMBLY | ID 2 только по attack-вектору `+0x130`, иначе ID 3 по summon-вектору `+0x150`, иначе ID 1; скан поля ID `CSkill +0x04` без `QuerySkillType` и без раннего выхода |

### CMonster / CBuild / CCityGate / CSummonedCreature

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CMonster::GetScriptFile/SetScriptFile` | `0x00031410/0x00038C70` | VERIFIED_DISASSEMBLY | `+0x284` std::string, копия до NUL без завершающего нуля |
| `SetMasterInfo/GetMasterInfo` | `0x000E63E0/0x000E63F0` | VERIFIED_DISASSEMBLY | десять DWORD `+0x22C..+0x254`, совпадает с `zone::combat::masterinfo` |
| `DoesCreatureBeenTamed` | `0x000E6460` | VERIFIED_DISASSEMBLY | знак приручения + master type 400 + ненулевой master id (`+0x224/+0x22C/+0x230`) |
| `IncreaseTameAttemptCount` | `0x000E6490` | VERIFIED_DISASSEMBLY | DWORD-инкремент `+0x228` |
| `SetTamedSign` | `0x000E64A0` | VERIFIED_DISASSEMBLY | встроенный `IsTamable`: `dwTamable == 1 && tameAttemptCount < dwMaxTameAttemptCount`; отдельного pub `IsTamable` нет; tamable-проверку и отказ перезаписи живой master-связи исполняет `skills/monstertaming.rs` |
| `GetPetLevel/GetPetExperience/SetPetLevel/SetPetExperience` | `0x000E6CE0..0x000E6D20` | VERIFIED_DISASSEMBLY | смежные DWORD `+0x254/+0x258`; родной `SetPetLevel` отсекает запись уровня 10+, а переходный владелец намеренно сохраняет единое поведение setter без отсечения — известное расхождение, отложено для pet-поведения |
| `CSummonedCreature` ctor | `1:1BA1B0` (VA `0x005BB1B0`) | VERIFIED_DISASSEMBLY | base `CMonster` ctor `0x4E7E70`, DWORD `+0x2A8/+0x2AC` обнуляются, vtable `0x65CFE4` |
| `CBuild` семья | ctor `0x001DD570`, dtor `0x001DD640/0x001DD690`, `AddToByteArray 0x001DD140`, `DecordFromByteArray 0x001DD180`, `SetAction 0x001DD1C0`, `AI 0x001DD210`, `GetFigure 0x001DD280`, `SetTileXY 0x001DD2B0`, `GetBeAttackedPoint 0x001DD350`, `IsAttackAble 0x001DD520`, `ApplyFinalDamage 0x001DD270`, `GetHP…GetElementResistant 0x001DD5F0..0x001DD630`, `SetScriptFile 0x001DBAC0`, vtable `0x0065E704` | VERIFIED_DISASSEMBLY | 54 pub-символа `symbols.py pubs`; общие empty-thunk `OnBeenHurted/OnBeenMurdered 0x000A8750` (`ret 8`) и `EnterCombatState/EnterPeaceState/UpdateCurrentState 0x00085540` |
| `CBuild::GetAttackerDir` | `0x001DD300`, slot `+0x0B0` | VERIFIED_DISASSEMBLY, недостижим | 8-way таблица знаков `GetDestDir`, quirk: `param_2` дважды в точку footprint; живые вызыватели с receiver CMoveShape не доказаны |
| `CBuild::OnBeenAttacked` | `0x001DD6B0`, slot `+0x1B0` | VERIFIED_DISASSEMBLY, недостижим | пакеты `0xBF60A/0xBF60B`, death-ветвь с `SetKilledMeAttackInfo` и `SetAction(6)`; ни одного call-сайта в `.text` этой сборки |
| `CBuild::OnDied` | `0x001DD9D0`, slot `+0x178` | VERIFIED_DISASSEMBLY, недостижим из death-pipeline | guard убийцы-игрока `0x190`, пустой `OnSymbolDestroy`, script через `stRunScript/RunScript`; единственный вызыватель — `CBaseAI::OnBeenKilled`, а у постройки `CBaseAI` нет; победу country-war обрабатывает `CountryWarSys` (`on_flag_destroy 0x000EBE60`), не это тело |
| `CCityGate` семья | ctor `1:001dcb00`, dtor `1:001dcb20/1:001dcb90`, `SetAction 1:001dcb30`, `AI_Stand/AI_BeAttack/AI_Died 1:001dcb80`, `AI 1:001dcbb0`, `IsAttackAble 1:001dcc10`, `OnBeenHurted 1:001dcd80`, vtable `2:000138c4` | статусы унаследованы от шапки старого владельца, без повышения | 41 pub-символ `symbols.py pubs`; региональное гейтовое тело — см. раздел «War-регионы» |
| `CCityGate` vtable slot `+0x178` | → no-op thunk `0x00485540` | VERIFIED_DISASSEMBLY | указывает на общий no-op вместо inherited `CBuild::OnDied`; сохранённое script-поле ворот из death-пути не исполняется — зафиксировано как поведение оригинала и не «улучшается»; death-контракт ворот остаётся открытым вопросом |

## Региональное пространство

Реализация: `zone::regions::{region, area, serverregion::*}`; hub — `appserver/serverregion.rs`.

### CRegion (клетки, switches, ресурс)

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `GetCell` | `0x0002AC10` | VERIFIED_DISASSEMBLY | signed width/height `+0x6C/+0x70`, cell pointer `+0x84`, row-major index, `sizeof(tagCell) == 4` (PDB+EXE) |
| `SetBlock`/`GetBlock` | `0x0002AC50/0x0007BCC0` | VERIFIED_DISASSEMBLY | `SetBlock` меняет только три младших бита 16-bit word |
| virtual `GetSecurity` | `0x000854F0` | VERIFIED_DISASSEMBLY | 3-bit значения включая `SAFE=2` и `CITYWAR=3`; единственная null-pointer ветка при нулевом индексе возвращает `SAFE`, у `GetBlock/SetBlock` такого guard-а нет |
| базовый `GetReturnPoint` | `0x000F0280` | VERIFIED_DISASSEMBLY | zero-write шести выходов |
| random-position family | `0x000F02C0/0x000F04D0` | VERIFIED_DISASSEMBLY | нормализация/расширение прямоугольника, ровно 1000 random-попыток, затем x-major linear scan и финальная random-позиция с `false`; `lSwitch` — little-endian word `+2` |
| serializer | `0x000F0540` | VERIFIED_DISASSEMBLY | region намеренно НЕ round-trip симметричен: serializer пишет region type, width/height, country/notify, cells и только 4-байтовый `lState` каждого switch, decoder читает resource ID и IEEE-754 scale перед размерами, а switches целиком по `0x14` — разница подтверждена EXE и сохранена |
| обе `GetSwitch` | `0x000F0620/0x000F0670` | VERIFIED_DISASSEMBLY | — |
| `New` | `0x000F06C0` | VERIFIED_DISASSEMBLY | сначала освобождает старые cells/switches, затем zero-filled block, возврат 1; ошибочные ранние `return` raw-декомпилята опровергнуты control flow `0x004F06C0..0x004F0742` |
| resource `Save/Load` + decoder | `0x000F0830/0x000F0EA0/0x000F1070` | VERIFIED_DISASSEMBLY | `regions/{ID}.rgn` = `CLS-RGN`, signed version 1, region type, width/height, `width*height*4` cell-байт, signed count и 20-байтовые switches; ignored short `fread` после заголовка → локальная typed-граница; старый invalid-header путь не закрывал `FILE` (ненаблюдаемая утечка не сохраняется) |
| constructor/destructor | `0x000F0D90/0x000F0750` | VERIFIED_DISASSEMBLY | object type 200, notify timestamps 0, scale 1.0; country/notify не инициализируются — Rust prefix хранит `Option` |

Посторонний domain destructor `CPlayerList::tagPropertiesUpgrade`, который
дизассемблер приписал этому translation unit, не является частью `CRegion`.

### CArea (области региона)

| функция | RVA | статус | суть факта |
|---|---|---|---|
| constructor | `0x00075400` | VERIFIED_DISASSEMBLY | размер объекта `0x118` по allocation stride; base-state type 300, координаты −1/−1, девять vector-storage, три ordered map, один critical section |
| `CServerRegion::CreateAreaArray` | `0x0007BE10` | VERIFIED_DISASSEMBLY | после построения массива пишет в inherited father-slot `+0x40` один и тот же `CServerRegion*`, затем X/Y по `+0x44/+0x48` — выражено структурным `AreaParentLink::OwningServerRegion` без self-reference |
| членство `GetNumShapes`/`AddObject`/`RemoveObject`/`FindShapes`/`GetAllShapes` | `0x00070B80/0x00073B70/0x000721F0/0x00073DE0/0x000743A0` | VERIFIED_DISASSEMBLY | `RemoveObject` буквально не удаляет unknown-type hash из `m_vOtherShapes` — подтверждённая странность; goods timestamp/protection удаляются только при успешном `CGoods` RTTI, а type 700 membership — по base GUID независимо от RTTI |
| inline `CSession::GetPlugList` | `0x00070910` | IMPLEMENTED | ordered plug-list без копии; материализация — у владельца Zone sessions |
| `PlayerEnter` | `0x00075580` | VERIFIED_DISASSEMBLY | точный девяти-area traversal |
| `WakeUpMonsters` | `0x00073A70` | VERIFIED_DISASSEMBLY | атомарно забирает sleeping storage; `CGame` выполняет `CMonsterAI::WakeUp`, публикует HP и возвращает монстра в список активных/питомцев/повозок либо оставляет вне области при устаревшем владельце или отсутствующем AI |
| `AI` | `0x00074500` | VERIFIED_DISASSEMBLY | отдельный `timeGetTime` для каждой ordered goods/protection записи, around-delete с `CS_DELETE`, active→sleep/pet/carriage переклассификация только в area без игроков |
| `OnRefreshMonster` | `0x00101A70` | VERIFIED_DISASSEMBLY | намеренный no-op (`ret 4`); метод оставлен явным ради границы owner-а и аргумента refresh index |
| `AddWarSoul/DelWarSoul/FindWarSoul` | `0x00072F60/0x000710D0/0x00072EA0` | VERIFIED_DISASSEMBLY | ordered-map semantics: delete помечает point `(-1,-1)`, find не публикует такие записи и не перезаписывает output key |

Девять однородных `Unwind@00635210..00635280` делали только `operator_delete`
временного allocation при exception и эквивалентно покрыты RAII Rust-контейнеров.

### CServerRegion: membership и transition

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `add_object_with_area_entry` | `0x00083270` | VERIFIED_DISASSEMBLY | тело `0x00483270-0x00483570`, vtable `+0x38`: RTTI-downcast `0x004832A0`, owner-link `[+0x40]`, tile X до tile Y, границы по `[+0x6C]/[+0x70]`, random-fallback только при ненулевом cell-array `[+0x84]` через `GetRandomPos 0x000F04D0`, virtual `SetTileXY +0x88`, registry-add switch по типам `0x190/0x1F4/0x258/0x2BC`, индекс области `idiv` глобалями area-span 15 из `0x69EFC8/0x69EFCC`, `GetArea 0x0007BB60`, virtual `AddObject +0x38` области, запись `m_pArea [+0x60]`, player-only `PlayerEnter 0x00075580`, entry-hook virtual `+0x150` либо fallback — virtual `RemoveObject +0x34` на себе |
| `remove_object` | `0x0007CE60` | VERIFIED_DISASSEMBLY | тело `0x0047CE60-0x0047D08C`, vtable `+0x34`: area-remove virtual `+0x34` области и `m_pArea = 0` ДО стирания блока клетки; player-ветка вызывает `SetBlock(x,y,0)` virtual `+0x90` и player-leave virtual `0x00601A70` — точный `ret 4`; NPC/monster-ветка — тот же `SetBlock` без stub-вызова; прочие типы блок не стирают; registry-erase по типам в конце |
| `plan_area_transition`/`commit_area_transition` (`OnShapeChangeArea`) | `0x000802A0` | VERIFIED_DISASSEMBLY | тело `0x004802A0-0x004808A2`: gate null-объекта и отсутствующего owner-link `[+0x60]`, gate совпавшей области по `[+0x68]/[+0x6C]` против `[+0x44]/[+0x48]`, обход девяти-area окружения по таблице `0x69FC38`/границе `0x69FC80` (center-first row-major, совпадает с `NEIGHBOR_AREAS`), list-разность «новые без старых», skip пустых областей по `GetNumShapes`, audience только для moving player (`[+0x4] == 0x190`) с исключением самой фигуры, ordered per-area snapshots до membership mutation, commit `RemoveObject -> GetArea(next) -> AddObject -> m_pArea`, player-only `PlayerEnter` |
| частичный эффект пути ненайденной цели | внутри `0x000802A0` | VERIFIED_DISASSEMBLY | `RemoveObject` прежней области до bounds-check цели, завершение без `AddObject` и без сброса `m_pArea` — ядро докладывает `AreaTransitionOutcome::RemovedWithoutTargetArea`, обвязка отображает в булево «отказ» |
| staging-очереди transition | — | PARTIAL | pointer-unique append исходного region AI; staging-сайт исходного region AI отдельным телом не дизассемблировался; marker сбрасывается caller-ом только после добавления, snapshot без очистки списка, cleanup — вызывающая сторона |
| `GetRandomPos` internals | `0x000F04D0` | не дизассемблировался | при неудаче оригинал молча продолжает с записанными out-координатами; ядро поднимает typed-границу — отличие зафиксировано |

### CServerRegion: налоги

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `add_tax_money` | `0x000826E0` | VERIFIED_DISASSEMBLY | тело `0x004826E0-0x00482816`, vtable `+0xC8`: superior-gate по `[+0x228] > 0`, signed `imul` low-32 доли, делитель через x87 `fmul` константой `0.01f` (`0x64DBD0`) и `fnstcw`-переключение rounding в truncate, вычитание доли до доставки, clamp `0xEE6B2800` обеими ветками `cmp/jae`, хвостовой `UpdateTaxToWorldServer 0x0007C0C0` |
| машинная странность glue | `0x0048277E` | VERIFIED_DISASSEMBLY | сумма рекурсивного `AddTaxMoney` superior-а и поле #2 сообщения доли `0x6012E` — неинициализированный стековый DWORD `[esp+0xC]`; клей пересылает вычисленную долю и не воспроизводит бесконечную рекурсию на циклическом superior-графе (visited-guard + escape `0x6012E`) |
| точность расчёта | — | оговорка | оригинал считает `rate*amount*0.01f` в 80-битном x87, ядро — в `f64`; расхождение возможно лишь на границе целого при значениях вне 53-битной мантиссы |
| `collect_today_tax` | `0x0007D9B0` | VERIFIED_DISASSEMBLY | тело `0x0047D9B0-0x0047DAE2`, vtable `+0xCC`: `total = clamp(total + today, 0xEE6B2800)` → `today = 0`, `GS0236` аргументами `(имя, collected, today = 0)`, `PutStringToFile("war") 0x0001CEB0`, хвостовой `UpdateTaxToWorldServer` |
| `UpdateTaxToWorldServer` | `0x0007C0C0` | VERIFIED_DISASSEMBLY | `0x6012D` с полями region ID `[+8]`, today `[+0x224]`, total `[+0x220]`, ставка через virtual `GetTaxRate +0xB8`, `Send(false)` |

### CServerRegion: return-setup, war-фазы, spawn, startup

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `GetReturnPoint` | `0x000814F0` | VERIFIED_DISASSEMBLY | при null-игроке zero-write шести выходов (инлайн-порт базового `CRegion::GetReturnPoint 0x000F0280`); локальный `m_stSetup` (`+0x170..+0x18C`, восемь signed DWORD) имеет приоритет при `use_return != 0` и пишет direction константой `-1`; иначе fallback в три mutating country-default карты `CCountryParam::main_return_point` (`operator[]`-семантика); `does_recall +0x184` и `use_return +0x18C` machine-verified, поле `+0x188` — INFERRED (не читается), ctor тела `??0CServerRegion`/`??0CRegion` setup не записывает |
| `DoesRecallWhenLost` | `0x0007BAC0` | VERIFIED_DISASSEMBLY | читает `+0x184` |
| war-фазы `OnWarDeclare/Start/End/Mass` | `0x00085560..0x000855B0` | VERIFIED_DISASSEMBLY | `OnWarDeclare`/`OnWarEnd` пишут сначала `+0x23C`, затем `+0x238` (обратный адресному порядок), `ReSetWarState` — прямой; имена полей INFERRED по именам методов, числа смещений машинные |
| ownership/state accessors | `0x000855D0..0x00085680` | VERIFIED_DISASSEMBLY | ownership-пара в `RegionParamState` (`+0x230/+0x234`) |
| общий пустой символ | `0x00201A70` (`ret 4`) | VERIFIED_DISASSEMBLY | ICF-свёртка ЧЕТЫРЁХ методов: `OnWarTimeOut`, `OnClearOtherPlayer`, `OnRefreshRegion`, `UpdateCityGateToClient` — наблюдаемых действий нет |
| enum `CITY_STATE_*` (0..3) | pub-офсеты `845F0/84600/84650/84660` (RVA +0x1000) | INFERRED по именам методов | сам enum в пабах не раскрыт, числа и офсеты машинные |
| pub `AddNpc@CServerRegion` | VA `0x00480A40` (RVA `0x00080A40`, `serverregion.cpp:1003`) | из карты стены + inline-сверка script-записи | prototype `int __thiscall AddNpc(tagNpc*, bool, bool)`; ctor-инициализация `CNpc` применяется через generic trait-шов без нового утверждения |
| `AddMonsterRect` | RVA `0x00084040` (`serverregion.cpp:672`) | из карты стены | batch-тела перенесены сверкой statement-в-statement с телом переходного владельца; статусы не повышались |
| `DecordFromByteArray`/`DecordSetupFromByteArray`/`FindForbidGood` | RVA `0x000858F0`/`0x0007EAC0`/`0x0007D6A0` | из карты стены; машинная сверка тел не выполнялась | перенесённые тела сверены statement-в-statement с переходным владельцем; ANSI-преобразование WINDOWS_1251 имени остаётся у переходного владельца (`encoding_rs`), zone получает `name_bytes` byte-exact |
| `AddToByteArray`/`AddRegionParamToByteArray` | `0x00019120` | не перенесены | subtype wire-tail декодеры War/City/Country — у переходного владельца |

## War-регионы

Реализация: `zone::regions::{serverwarregion, servercityregion, servercountryregion, servervillageregion, servergodsbattleregion}`;
hub-агрегаты — одноимённые файлы `appserver/` старого пакета. Статусы унаследованы
от шапок старых владельцев без повышения, где ниже указано `VERIFIED_DISASSEMBLY`.

### CServerWarRegion (общий)

| функция | RVA | статус | суть факта |
|---|---|---|---|
| contender lifecycle: `OnEnterContend`, `DecContendTime`, `CancelContendByPlayerID`, `AddContend`, `AI`, `SetFacWinSymbol`, `CancelContendBySymbolID`, `OnContendTimeOver` | `0x001D26F0`, `0x001D25F0`, `0x001D2C40`, `0x001D2D80`, `0x001D31A0`, `0x001D33A0`, `0x001D3460`, `0x001D3820` | VERIFIED_DISASSEMBLY | PDB: `tagContend` размером `0x34`, ordered `m_listContend +0x24C`, `m_FacWinSymbol +0x258`, три signed counter `+0x264..+0x26C` |
| decoder `DecordFromByteArray` | `0x001D3110` | VERIFIED_DISASSEMBLY | три signed little-endian DWORD после base-prefix; обновляются только keys `0..total`: старые map-keys за новым total оригинал не очищает |
| `UpdateContendPlayer` | `0x001D3530` | VERIFIED_DISASSEMBLY | — |
| арифметические края | — | BLOCKED_MISSING_FACT | `INT_MIN / -1` → `ContendArithmeticBlock`; недоказанная реакция x87 `fistp i32` на NaN/inf/out-of-range → `WarDamageArithmeticBlock` (safe Rust не назначает saturating cast) |
| ctor `m_lVicSymbolNum` | — | UNKNOWN | записи в constructor-е нет; исходное значение неинициализированной памяти не утверждается (zero в `Default` — не утверждение о памяти оригинала) |

### CServerCityRegion

| функция | RVA | статус | суть факта |
|---|---|---|---|
| фазовые callbacks | `0x001CF730`, `0x001CFA00..0x001CFD70`, `0x001D09F0` | VERIFIED_DISASSEMBLY | фазовый call order |
| ownership | `0x001CED70/0x001CEF40` | VERIFIED_DISASSEMBLY | — |
| victory | `0x001CF1A0` | VERIFIED_DISASSEMBLY | timeout агрегирует владельцев symbols по faction ID и выбирает первый достаточный ID в map-order; при отсутствии победителя сохраняется действующий owner faction/union; `OnWinSymbol` внутри `OnFactionVictory` у этой сборки — точный no-op `0x004A8750` |
| spatial | `0x001CEE80/0x001CEF60` | VERIFIED_DISASSEMBLY | два state-read, defender-only return setup, fallback в базовый/country owner у calling aggregate, random bool игнорируется |
| virtual security/guard attackability | `0x001CF0E0/0x001CF050` | VERIFIED_DISASSEMBLY | `GetSecurity` сначала возвращает `SAFE=2` для mass-state `2` (pre-query gate выполняет calling aggregate до cell lookup) |
| gate runtime | `0x001CAAA0`, `0x001CF370..0x001CF640` | VERIFIED_DISASSEMBLY | layout gate-полей, общий x-major footprint scan `0x001CAAA0` |
| clear / guard refresh / timeout-forwarding | `0x001CF970`, `0x001CF7C0`, `0x001CFEB0` | VERIFIED_DISASSEMBLY | decoder/factory returns, child-ID `+0x158`, base-region registration |

### ServerCountryRegion

| функция | RVA | статус | суть факта |
|---|---|---|---|
| subtype decoder | `0x001CD3F0` | VERIFIED_DISASSEMBLY | overload return |
| gate runtime | `0x001CAC80/0x001CADD0/0x001CB1E0..0x001CB310` | VERIFIED_DISASSEMBLY | gate/camp/flag layout |
| refresh / `ClearRegion` | `0x001CB750/0x001CB880`, `0x001CBA50` | VERIFIED_DISASSEMBLY | refresh/clear order; flags — обычные `CBuild` type `0x44C`: wire `field_24` не применяется, initial action 0, refresh меняет только HP |
| phase callbacks | `0x001CABC0..0x001CAC60` | VERIFIED_DISASSEMBLY | исходная странность `OnPrepareBegin/End`: оба проверяют `_state_prepare`, но меняют `_state_declare`; безопасный исходный gate закрыт |
| spatial / virtual security / `GetCamp` | `0x001CA9F0/0x001CE010`, `0x001CAD70`, `0x001CE540` | VERIFIED_DISASSEMBLY | `random(map.size())` и исходный mutating `operator[]` return-point семейства; `INT_MIN / -1` и недоказанные x87 conversions — локальные typed-границы |
| guard ownership/refresh | `0x001CB370`, `0x001CC960..0x001CCA30` | VERIFIED_DISASSEMBLY | area selection |
| attackability / contender damage/time / enter/list/symbol / AI/victory / `IsPlayerContendSymbol` | `0x001CE830..0x001CE970`, `0x001CA980/0x001CAFF0/0x001CB0D0`, `0x001CB710/0x001CCAC0/0x001CCB50/0x001CE190/0x001CE2B0`, `0x001CE380/0x001CEA10`, `0x001D24E0` | VERIFIED_DISASSEMBLY | — |
| `OnFlagDestroy` | `0x001CAD50` | VERIFIED_DISASSEMBLY | независимо от исходного war-byte оставляет false при совпавшем region ID; переданный country long не читается |
| `CancelContendByPlayer` | — | подтверждённый дефект | инвертированное условие: любой реальный player получает `false`, null-ветка читает absolute `0x8` и вызывает метод с null; safe Rust не придумывает ей результат |

### CServerVillageRegion

| функция | RVA | статус | суть факта |
|---|---|---|---|
| фазовые callbacks / victory / clear / membership / timeout-forwarding / `AddNeedGood` | `0x001D1310`, `0x001D13C0`, `0x001D1590..0x001D16D0`, `0x001D12C0`, `0x001D1370`, `0x001D12F0`, `0x001D14F0`, `0x001D1900` | IMPLEMENTED | PDB: наследование `CServerWarRegion`, ordered goods-list `+0x270`, `m_lFlagOwnerFacID +0x27C` |
| decoder-forwarding / ownership query | `0x001D1280`, `0x001D1980` | IMPLEMENTED, VERIFIED_DISASSEMBLY (унаследовано) | timeout намеренно игнорирует аргумент и шлёт текущие `(war, region, flag-owner, 0)` как `0x60136` |
| `OnFactionWinOneSymbol` | — | IMPLEMENTED | flag owner переносится только по symbol `0`, остальные символы колонку не меняют; `AddNeedGood` игнорирует пустое имя и сохраняет порядок списка |

### CGodsBattleMgr / CServerGodsBattleRegion

| функция | RVA | статус | суть факта |
|---|---|---|---|
| проверка first-contender | `0x000A9270` | VERIFIED_DISASSEMBLY | намеренное сравнение normal `m_lFactionID` с сохранённым GodsBattle faction — несовпадение подтверждено, не исправлено по позднему C++-донору |
| `CGodsBattleMgr::OnEnterContend` | — | IMPLEMENTED | индекс NPC-set `0..2` против player faction `5/6`: для допустимого игрока ветвь `SZLGS7` недостижима, поэтому после гибели всех стражей собственный символ тоже начинает contend — подтверждённое различие представлений не нормализуется |
| top-ten SZL exchange | — | IMPLEMENTED | безразмерный pointer и 256-байтный C-string buffer заменены bounded slice/cursor и owned bytes; обрыв возвращает typed error после завершённого prefix-а вместо неназначаемого legacy UB; точный script case `11130` — живой caller; byte-owned имя contender сохраняет исходную GBK |

Реализация — `zone::regions::servergodsbattleregion`; GodsBattle-contend и
NPC guard/counter lifecycle исполняет `CGame`.

## AI расписаний и поведение

Реализация: `zone::ai::*`; hub — делегаты `appserver/ai/*.rs` и `appserver/skills/monsterbaseattack.rs`.
Контракт поведения: `docs/gameplay/npc-ai.md`.

### CBaseAI

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `AddAIEvent` | `0x0C8F90` | VERIFIED_DISASSEMBLY | кладёт `{action, beginning, delay, handling=0} = {+0,+4,+8,+C}` с замером часов в момент вызова; `STIFFEN`/`DIED`/`OPEN`/`DEFENSE` всегда в `passive_actions`, остальные — в `active_war_soul_actions` при любом ненулевом флаге и иначе в `active_actions`; PDB подтверждает numeric `AI_SHAPE_ACTION 0..8`, `ASA_FORCE_DWROD = 0xFF` и три очереди |
| `GetCurrentActiveAction` | `0x0C81B0` | VERIFIED_DISASSEMBLY | читает только action первого элемента без проверки handling; пролог `or eax,-1` + `cmp [ecx+0x14],0` — пустая очередь есть native `-1`, в Rust `None` |
| `Run` | `0x0C7D10` | VERIFIED_DISASSEMBLY | guard owner `[+0x68]` и hibernate `[+0x6c]` → OnSchedule → background → passive → active → хвостовой OnIdle только при AES_IDLE всех основных фаз и отсутствии цели, затем WarSoul-хвост; сам факт вызова OnSchedule не блокирует OnIdle |
| `ProcessActiveAction` / `ProcessPassiveAction` / WarSoul-вариант | `0x0C81D0` / `0x0C84F0` / `0x0C8390` | VERIFIED_DISASSEMBLY | различает ноль, единицу и прочие знаковые `handling`; снятие по deadline только при `handling == 1`, ожидание любого action кроме `Move` даёт HUNG_UP и блокирует следующую фазу; пробы deadline читают часы лениво, после записи handling; снятие головы завершает текущий проход |
| `Stand` из `ProcessActiveAction` / `OnIdle` | — | VERIFIED_DISASSEMBLY | первый вызов обработчика всегда считается исполнением и удерживает расписание до срока; `OnIdle` ставит следующий `Stand` на 1000 мс только при пустом результате и отсутствии цели (базовый обработчик; слот `+0x48` `CPlayerAI` — пустой RET `0x00485540`) |
| `SetTarget`/`GetTarget`/`HasTarget`/`OnLoseTarget` | — | VERIFIED_DISASSEMBLY | identity `{type, id}`, нулевые type/id не образуют lookup, положительные обязательны для HasTarget |
| `Clear` | `0x0C7F70` | VERIFIED_DISASSEMBLY | очищает обычные active/passive очереди, object-цель и флаг сна; war-soul FIFO и сохранённые времена сна не затрагиваются |
| `WhenAddBackStageSkill` / `OnExecuteBackStageSkills` | `0x0C94B0` / `0x0C88E0` | VERIFIED_DISASSEMBLY | post-condition `owner != null` и `ID != SKILL_UNKNOW` (`cmp [esp+4], 0x7FFFFFFF`); добавление без дедупликации; исполнение сначала удаляет старые UNKNOWN, затем разрешает каждый ID у `CMoveShape`, отсутствующий/завершённый навык лишь помечается UNKNOWN |
| destructor | `0x0C8890` | VERIFIED_DISASSEMBLY | запись vtable `0x651D24` в прологе; back-stage список уничтожается вместе с AI |
| `MoveTo` / `Slip` | `0x0C9020` (пролог `83 ec 18`) / `0x0C7FF0` | VERIFIED_DISASSEMBLY | один Slip для ходьбы, два для ненулевого run с исходным желаемым направлением; отказ Slip не меняет очередь; Slip пробует статическую dword-таблицу 8×8 `_slip_order` (`.data` raw `0x29EDE0`, побайтово = `SLIP_ORDER`) и построчно `s_listMoveCheckCell[figure][direction]` (база VA `0xEF670C`); `CRegion::get_block` возвращает младшие три бита клетки (`& 7`); адрес `0x0C8020` — середина `Slip`, не вход MoveTo |
| задержка одного шага | по телу `0x0C9020` | VERIFIED_DISASSEMBLY | x87: `fsubr 1000.0` (raw `0x251DB8`) от `fild(4*g_ms)`, `fmul 0.001` (raw `0x24E9FC`), `fdivr` по скорости (vt `+0x6C`), `fmul` по `1_000_000.0` (чётное направление, raw `0x251DB4`) либо `1_414_000.0` (нечётное, raw `0x251DB0`), `fadd fild` stop-frame (vt `+0xB8`), `fistp` под truncating control word (`fstcw`/`or ah,0x0C`); промежуточные значения округляются до float через `fstp dword`; `g_ms` = 80 → коэффициент кадра `(1000 − 4·80)·0.001 = 0.68`; те же константы у `CPlayerAI::MoveTo` (refs raw `0x109031/0x109092`, `0x109046/0x1090A7`) |
| passive-реакции `OnBeenHurted`/`OnStiffen`/`OnBeenKilled` | `0x0C8700`/`0x0C8770`/`0x0C9220` | VERIFIED_DISASSEMBLY | `OnBeenHurted` удаляет только префикс active-очереди до первого `Attack`/`Move`; `OnStiffen` вызывает `CSkill::End(4)` только для `Attack` с `handling == 0`; `OnBeenKilled` отбрасывает active-события до первого `Move` и повторяет обработчик, пока движение не завершится; каждый `Defense` обрабатывается до pop — следующий видит мутации; war-soul FIFO никогда не затрагивается |

UNKNOWN: указатель владельца `[+0x68]` и обработчики вне достигнутых участков;
точная семантика virtual-вызовов (`OnLoseTarget` из `OnStiffen`, ответ `End(4)`)
— политики разрешения у hub-владельцев.

### CMonsterAI и реестр ai_type

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CMonsterAI::OnSchedule` | `1:0x1dbf80` → RVA `0x1DCF80` | VERIFIED_DISASSEMBLY | пустой базовый hook; owner `[+0x68]` обязателен; очереди пусты `[+0x14] == 0 ∧ [+0x28] == 0`; `can_fight` — `[owner+0x170] != 0` (INFERRED name), иначе только virtual `OnLoseTarget`; цель жива и `IsAttackAble` (vt `+0x134`); `GetCurrentSkill()` null → `OnChangeSkill` → повторный null → `OnLoseTarget`; Tracing держит дистанцию в `[min..max]` (vt `+0x74` max, vt `+0x70` — RET1-заглушка минимума), backoff — UNKNOWN; интервал: `timeGetTime >= [this+0x78] + (word)GetAtcInterval`, затем `[+0x78] = now` и только после этого `skill->vt[+0x08](owner, target)`; ok → `AddAIEvent(2)`, отказ → `OnLoseTarget + AddAIEvent(5)` |
| очередь `AI_EVENT` | — | VERIFIED_DISASSEMBLY | `{+0 action, +4 param, +8 время, +0xC состояние}`; `state == 1` ждёт `timeGetTime >= ev.time + prev.param`; `2 → OnFighting`, `5 → OnSearchEnemy` (у CPet собственный, у монстра пустой RET1), `1 → OnMoving` у CPet |
| `Run` | thunk `0x1DCBB0` → `CBaseAI::Run 0x0C7D10` | VERIFIED_DISASSEMBLY | guard → OnSchedule → ProcessBackStageAction → != 2 → passive → active → все 0 && !HasTarget → OnIdle → vt+0x44 hook → ProcessActiveActionWarSoul; возврат max-состояния |
| `OnChangeSkill` | `0x1DCBC0` | VERIFIED_DISASSEMBLY | `SelectAttackSkill` → уже выбранный + `CSkill::IsRestored` (vt `+0x80`: `QueryProperty(10005) + [+0x40] < now`; null-props → 1) → готово; иначе `SetCurrentSkill(GetDefaultAttackSkillID())`; возврат всегда 1 |
| `SelectAttackSkill` | `0x1DD0B0` | VERIFIED_DISASSEMBLY | `dynamic_cast CMonster`, один `random(10000)`, обход `std::list` (`[CMonster+0x210]`; узел `word[+8] = id`, `word[+0xC] = odds`), первый с префикс-суммой ≥ r; иначе default; `Attack(id, shape)` (`0x1DCEC0`) игнорирует ID — только `SetTarget` |
| `CBaseAI::MoveTo` контракт monster walk/run | `0x0C9020` | VERIFIED_DISASSEMBLY | один Slip для ходьбы, два для бега с исходным направлением; после Move timestamp берётся заново |
| `CAIFactory::CreateAI` | RVA `0x1DC550`, byte-map `0x5DCB08`, jump-table `0x5DCA94`, default-case `0x5DCA32` | VERIFIED_DISASSEMBLY | точная таблица классов по ai_type: 0 CGladiator, 1 CPassiveGladiator, 2 CSmartGladiator, 3 CStupidGladiator, 4 CArcher, 5 CFixedPositionArcher, 6 CStupidArcher, 7 CPuninessCreature, 8 CGuardWithBow, 9 CGuardWithSword, 10 CCityGuardWithSword, 11 CCityGuardWithBow, 12 CCarriage, 13 CGuardCountry, 14 CGuardCountry2, 15 CVilCouGuardWithSword, 16 CVilCouGuardWithBow, 17 CWarDeffendMonster, 18 CWarAttackMonster, 19 CNationCouGuardWithSword, 20 CGuardCountry, 21 CNationGladiator, 23 CGBGuardWithSward, 24 CGodsBattleMonsterAI, 100 CLord, 101 CJiuMai, 103 CBossBlue, 104 CBossFiend; 22, 25..=99, 102 и >104 — default `CMonsterAI` |
| стационарное расписание | собственный `OnSchedule 0x0020B890` | VERIFIED_DISASSEMBLY | ровно ai_type {5,8,11,13,14,16,20}; интервальный `[+0x78]` gate имеют `CMonsterAI` и thunk-наследники {0,1,3,4,6,9,10,15,17,18,19,21,23,24,100} вместе с default; собственные расписания без интервала — {2,7,12,101,103,104} и стационарные |

UNKNOWN: сайт вызова `Run` и каденсия AI глобальным циклом; форма backoff-шага
преследования; семантика поля `owner+0x170` (`can_fight` по INFERRED name);
маскировка vt `+0x12C` вокруг Tracing; массивы default-ID навыков; поле `tdI[2]`;
type шаблона списка навыков монстра.

### CPet

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CPet::OnSchedule` | `0x0E9DC0` | VERIFIED_DISASSEMBLY | ветви по режиму `[+0x80]` — 0 Attack / 1 Follow (`OnFallowingSchedule 0x0E9BB0`) / 2 Stay (`OnStayingSchedule 0x0E9650`); watchdog мастера `Distance(master) > 32` клеток + глобал `0xEF44E8` → `GS0012` и `Evanish` (vt `+0x188`); авто-отзыв при полных слотах (`CheckSkill(0xD4)`/`AddPet`, `GS0011`); шестичасовые (`0x1499700` мс) уведомления со счётчиком `>= 4`; master — `dynamic_cast<CPlayer*>`, region — `dynamic_cast<CServerRegion*>`; lifecycle-хвост (`0x0E9E4E`) выполняется после ветви действия и до background/passive даже при занятых FIFO |
| `OnAttackingSchedule` | `0x0E9A20` | VERIFIED_DISASSEMBLY | дистанционный гейт `CShape::Distance <` BSS `0xEF44EC` от мастера (иначе от питомца); отказ `IsAttackAble` живой цели: если AI цели целится в питомца — встречный `OnLoseTarget` цели, затем свой `OnLoseTarget + SearchEnemy`; БЕЗ `GetAtcInterval`-гейта |
| `OnStayingSchedule` | `0x0E9650` | VERIFIED_DISASSEMBLY | не вызывает Tracing; min/max проверяет общий dispatcher перед Begin, при выходе — `OnLoseTarget → SearchEnemy` |
| `OnLoseTarget` | `0x0E95F0` | VERIFIED_DISASSEMBLY | базовая очистка, затем для ATTACKING проверка HasTarget (`+0x50`), повторный OnLoseTarget (`+0x2C`), запись FOLLOWING |
| `SetTarget` | `0x0E9630` | VERIFIED_DISASSEMBLY | FOLLOWING → ATTACKING и `CBaseAI::SetTarget` (`0x004C7C60`); cast и Move не отменяются |
| `SetPetCurrentAction` / `SetPetCurrentAIMode` | `0x0E94B0` / `0x0E9460` | VERIFIED_DISASSEMBLY | OnLoseTarget только для нового FOLLOWING при HasTarget; смена режима обнуляет `invalid_master_ms (+0x84)` и `seek_master_ms (+0x88)`, не трогая шестичасовой счётчик и master_logout |
| ctor | `0x0E9400` (записи `0x0E942E/0x0E9435`) | VERIFIED_DISASSEMBLY | mode = 2, action = 1 при нулевых lifecycle-таймерах |
| режимы/события | `OnSearchEnemy 0x0E98CD`, `OnBeenHurted 0x0E94F9` | VERIFIED_DISASSEMBLY | PET_MODE: 0 активный, 1 защитный, 2 пассивный; `OnMoving` живого питомца без навыка ставит отдельный `ASA_SEARCH_ENEMY`; `OnIdle` сохраняет `ChangeSkill? → Stand → SearchEnemy`; runtime-входы по `CMonster::GetAI` (`0x004E6D80`), не по знаку приручения |

UNKNOWN: семантика BSS-глобалов `0xEF44E8` (watchdog) и `0xEF44EC` (предел
преследования) — значения записывает внешний lifecycle `CGame`; нить вызова
`Evanish` и отзыва из watchdog (выполняет hub по результату `tick`).

### CPlayerAI

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `OnSchedule` | `0x1098D0`; допуск `0x50993E..0x50999D` | VERIFIED_DISASSEMBLY | извлекает обычный запрос до Begin; End/OnLoseTarget не подменяют текущую команду головой pending FIFO; повторное завершение отсутствующего исполнения не удаляет будущий запрос; при свободных очередях новый запрос заменяет текущую цель; riding-отказ вызывает OnLoseTarget до pop и не ставит цель (`0x50998E..0x50999D`); pop сам цель не меняет |
| `HasTarget` / null owner ветви | `0x0C7DD0`; `0x509A34..0x509A60`; `0x509804`; `0x509780..0x509789` | VERIFIED_DISASSEMBLY | Begin не допускается при нулевой координате или неположительном type/id; при null owner после разрешения — virtual default игрока с прежней целью; WarSoul не выбирает default при null GetSkill и очищает цель до проверки пустоты pending FIFO |
| `Attack` обеих перегрузок | `0x109FF0`/`0x10A230`; WarSoul `0x10A334..0x10A3BF` | VERIFIED_DISASSEMBLY | до изменения FIFO требует существующий GetSkill и заменяет только ожидающую команду (`m_qTarget`), не текущее исполнение и не выбранный ID; WarSoul point-ветвь сравнивает голову, удаляет старые и добавляет новый без Reject, обычная очередь шлёт Reject для каждой замены (`0x10A43C`); у подключённых WarSoul нет записи prepared — проверка повторного prepared Attack не даёт им дополнительный End |
| снятия active-фазы | `OnFighting 0x1092B0`, `OnFightingWithWarSoul 0x109230`, `OnChangeSkill 0x108E40`, `OnChangeSkillWithWarSoul 0x109310` (`playerai.cpp:629`) | VERIFIED_DISASSEMBLY | Attack снимается лишь после подтверждённого завершения concrete навыка и IsEnded/IsPrepared-проверки; prepared переносится в back-stage до ChangeSkill; WarSoul-путь проверяет IsEnded до AI и не ставит ChangeSkill; очередь WarSoul Attack ставится после Begin и переживает End навыка; `OnChangeSkill` игрока возвращает default и снимает цель до завершения такта; `OnChangeSkillWithWarSoul` выполняет End(1) и выбирает `0x224`; `OnLoseTargetWarSoul` повторяет тот же side effect после полного End(1) |
| трёхаргументный virtual `MoveTo` | `0x10A480` | VERIFIED_DISASSEMBLY (пролог) | для живого игрока: очистка эмоции, удаление старейших назначений до длины ≤ 3, добавление `(direction, is_run)` — очередь ≤ 4 элементов; изъятие front после попытки MoveTo выполняет `OnSchedule` независимо от результата |
| четырёхаргументный `MoveTo` | `0x508F10`/`0x5090FB` | VERIFIED_DISASSEMBLY | общий `Slip` один/два/три раза по `is_run`/riding с исходным направлением; всегда `ASA_MOVE` после начатой spatial-мутации (`CMoveShape::OnMove` — void); задержка шага — общая формула CBaseAI |
| хвост `CPlayerAI::Run` (авто-прирост) | — | VERIFIED_DISASSEMBLY | `level³ · auto_exp_2 · (1 + faction_level·0.05) + auto_exp_1`, масштаб f32 `0.00011574074` (raw `0x2540E8`, единственная ссылка raw `0x1094D4`); `vigour = trunc(x·log10(exp+600) − y)` с cap; беззнаковая проверка срока `last >= now - interval` намеренно не wrap-safe; регенерация `trunc((level·0.1 − 1)·5 + faction_bonus + 10)`, faction bonus `max(1, min(level·0.01, 1) · (faction_level·0.5))` |
| базовый idle | слот `+0x48` vtable → `0x00485540` | VERIFIED_DISASSEMBLY | пустой RET (`c3`+padding): idle игрока не ставит базовый `Stand` на 1000 мс |
| форма цели и чтение ожидающей команды навыка | `HasTarget` VA `0x004C7DD0`; сравнение object/point VA `0x0050A13A–0x0050A15C`, `0x0050A349–0x0050A367`; чтение ID команды и очистка старшего бита VA `0x00488BAD–0x00488BE4`, `0x00488E73–0x00488EAE`, `0x004892DF–0x00489320`; выбор формы цели VA `0x00488D20–0x00488E20`, `0x00488FE6–0x00489109`, `0x0048949D–0x00489547` | MATCH (по шапке владельца) | enum и полный Eq — внутренняя модель Rust; ключ ожидающего запроса не включает уровень боевого духа и GUID цели; реализация `zone::skills::dispatch` |

UNKNOWN: источник события `ChangeSkillWithWarSoul` в этой паре не найден (из 82
прямых `AddAIEvent` только `0x509877` с WarSoul=1/Attack; все 13 постановок
ChangeSkill — WarSoul=0); полный registered `CSkill::End`; поведение слота `+0x8C`
`CPlayer` на пути MoveTo (`0x004856A0`).

### CLord / CJiuMai / боссы / гладиаторы / охрана / повозка

| класс/функция | якорь VA | статус | суть факта |
|---|---|---|---|
| `CLord::SelectAttackSkill` | `0x0060AF60` | MATCH | один RNG-бросок caller-а, HP как `f32`, `[20%, 50%)` → `TRUNC(roll·0.6666667)`, `< 20%` → `roll/2`; исключённые ID `1`/`2` продолжают копить odds |
| `CLord::WhenBeenHurted` | `0x0060B0B0` | MATCH | общая Defense-ветвь → скан квадрата младшего байта figure `x → y` через `CServerRegion::GetShape`, первая `CSummonShape` → направленный беговой отход (`0x0060B259` → MoveTo `0x004C7CB0` → общий `0x004C9020`: два Slip, Move(run=1), отказ любого Slip не публикует частичный шаг); атакующий принимается только при пустой прежней цели |
| `CLord::OnSearchEnemy` | — | MATCH (по прежней шапке владельца) | игроки перед питомцами, равная дистанция заменяет предыдущую запись |
| `CJiuMai::OnIdle` | `0x0060A5F0` | MATCH; PARTIAL в ветке отказа позиции | близнец берёт `m_lTwinsID` из `GetMasterInfo()->lMasterID`; обычный монстр один раз ставит `tagMasterInfo{[owner+4],[owner+8]}`, `GetRandomPosInRange(x−5,y−5,10,10)`, `AddSummonedCreature(…, −1, 0xFFFFFFFF)`, пишет ID близнеца, при отказе spawn — `−1`, затем базовый `CMonsterAI::OnIdle` (`0x005DCC60`); машинный код спавнит с уже записанным выходом позиции и ставит `−1` только при отказе `AddSummonedCreature`, hub-форма `position.ok().and_then(spawn)` даёт `−1` и при отказе позиции |
| `CJiuMai::OnSchedule` | `0x0060AB10` | MATCH | живой владелец с пустыми очередями; живой близнец дальше пяти клеток при неближней цели получает `GetRandomPosInRange(twin.x−5,…)` + `ForceMove(run=0)`; боевой хвост без `GetAtcInterval`-гейта |
| `CJiuMai::OnSearchEnemy` / `SetTarget` / `OnLoseTarget` | `0x0060AD10` / `0x0060AA50` / `0x0060A990` | MATCH | min-HP (vt `+0xD0`) внутри `GetGuardRange` (vt `+0x138`), равный HP сохраняет первую запись; цель дублируется живому близнецу без собственной цели; `OnLoseTarget` синхронизируется с близнецом |
| `CJiuMai::WhenBeenHurted` | `0x0060A750` | MATCH | ориентир задаёт запись направления формы `CShape::SetDir` (owner vt `+0x60`, `0x0044A1C0`) перед `GetDirPos` и общим `MoveTo(run=0)` (`0x0060A89F/0x0060A8E7`); прежний hub эту запись не выполнял — расхождение устранено |
| `CBossBlue` ctor / `WakeUp` / `SelectAttackSkill` | `0x00609CB0` / `0x00609CD0` (ICF с `CBossFiend::WakeUp`) / `0x0060A0E0` | MATCH | восемь одноразовых HP-порогов ярости; WakeUp после общего восстановления HP открывает только непройденные пороги текущей фазы; один RNG-бросок, повтор ниже 8% HP при отсутствии fury-состояния; odds учитывают исключённые ID `2` и `0x1f7`; исход — default-навык (в отличие от fallthrough CBossFiend) |
| `CBossFiend` ctor / `SelectAttackSkill` | `0x00609310` / `0x00609670` | MATCH | ctor записывает `timeGetTime` и открывает восемь HP-порогов в момент создания; повторный призыв ниже 8% HP по строгой `last + persist < now`; odds через исключённые ID `1`, `2`, `0x1f9`; без совпадения — возврат без назначения (в отличие от default CBossBlue); проверка таймера и запись момента — два отдельных чтения часов |
| `CBossBlue`/`CBossFiend` `OnSearchEnemy` | — | MATCH (по прежней шапке владельца) | общий проход игроков, затем питомцев; у Fiend особое предпочтение целей не ближе min-distance текущего навыка |
| `CSmartGladiator` ctor / `OnIdle` / `OnSchedule` | `0x00610850` / `0x00610660` / `0x006106E0` | MATCH | `m_qTarget` пустой deque (`[+0x7C..+0x8F]`); базовый idle только при живом owner и исчерпанной очереди шагов; без цели `front tagCell` идёт в `MoveTo(run=0)` (`0x0061083D`), затем `pop` всегда, включая неуспешное движение |
| `CSmartGladiator::OnSearchEnemy` / `WhenBeenHurted` | `0x00610AC0` / `0x006103B0` | MATCH | живые внутри `GetGuardRange` (vt `+0x138`); ближайшая угроза (`<=` заменяет) и цель с HP (vt `+0xD0`/`+0xD8`) < 0.4 с min абсолютным HP; vulnerable → virtual `SetTarget`, иначе отход; hurt: тип 400 при HP владельца < 0.75 (double `0x3FE8000000000000`) принимается целью, иначе отход; ориентир пишет `SetDir` перед `GetDirPos`/`MoveTo` (`0x006105A9`); прежний hub шаг к ближайшему монстру разворачивал отходом и `SetDir` не выполнял — расхождения устранены |
| `CCityGuardWithSword::OnSearchEnemy` (с целью) | `0x0060E290` | MATCH | при `HasTarget ≠ 0` и посте `≠ −1` сравнивается только `RealDistance` до поста с `GetChaseRange` (owners vt `+0x13C`) — при превышении virtual `OnLoseTarget`; selector в этой ветви не исполняется |
| городской selector (`SearchEnemyGuildMember/Pet`) | `0x0060E290`, `0x0060DB10`, `0x0060E350`, `0x0060E510`, vtable `0x00662BCC` | MATCH | ближайший из двух, игрок при равной дистанции; **`SearchEnemyGuildCarriage` (vt `+0x98`) эти тела не вызывают** — единственные caller-ы по статическому скану `CALL [reg+0x98]`: `CVilCouGuardWithBow::WhenBeenHurted` (`0x0060C8F6`) и `::OnSearchEnemy` (`0x0060C972`) AI16; прежний hub комбинировал проход повозок 603 — установленное расхождение hub, здесь проход удалён; фильтры: ненулевые `m_lFactionID == GetFactionID` (region vt `+0xB0`) либо `m_lUnionID == GetUnionID` (`+0xB4`) исключают; min-distance навыка vt `+0x70` внутри каждого прохода; живость — фильтром `CServerRegion::FindAroundObject` (`IsDied` в `0x00480E70`) |
| `OnLoseTarget` AI9/10/15/19 / `Tracing` AI10 | `0x0060D020` / `0x0060D0E0` | MATCH (по прежней шапке владельца, тело не перечитано) | базовый `CMonsterAI::OnLoseTarget` (`0x005DCC30`), возврат к посту, заблокированная клетка заменяется одним `GetRandomPosInRange` 3×3; Tracing: шаг назад от близкой цели, `ForceMove` в клетку 3×3 около далёкой (+ отдельный `Move(0)`), сброс за `chase_range` |
| стационарные `OnSchedule`/`OnIdle` семьи | `0x0020B890` | MATCH (зафиксированный факт) | `OnIdle` один раз фиксирует пост (`m_lX/m_lY`) и продолжает общий idle FIFO |
| `CCityGuardWithBow::WhenBeenHurted` | `0x0060DA90`, selector-ы `0x0060DB90/0x0060DD50` | MATCH; selector-форма PARTIAL (предполагает sword-форму по единому слоту vtable-пары и телам `0x0060E350/0x0060E510`) | базовый hurt всегда; `HasTarget != 0` → выход; иначе пара selector-ов, ближайший, назначение через virtual `SetTarget`; `SearchEnemyGuildCarriage` hurt-путь не вызывает — расхождение hub исправлено; маршрут min-distance через setup setup-уровень — прежний машинный вывод (PARTIAL) |
| `CFixedPositionArcher::OnChangeSkill` | `0x0060F9F0` | MATCH | virtual `SelectAttackSkill` (vt `+0x8C`); не разрешился → `SetCurrentSkill(GetDefaultAttackSkillID())`, возврат 1; не восстановленный (`IsRestored` vt `+0x80` == 0) ставит `AddAIEvent(Stand, GetRestoreTime(vt +0x84), 0)`; расширение ограничено набором {5, 23} — собственный конструктор `CVilCouGuardWithBow` строит `CMonsterAI` напрямую |
| `CFixedPositionArcher::OnFighting` / `OnIdle` / `OnSearchEnemy` | `0x0060FA70` / `0x0060FAA0` / `0x0060FBC0` | MATCH (варианты `0x0060D930`/`0x0060F6F0` — по прежней таблице владельца) | базовый `CBaseAI::OnFighting` (`0x004C9320`) + его успех `AddAIEvent(5)`; строгая очередь `ChangeSkill(6) → Stand(stop_frame, [monster+0x210]+0x94) → SearchEnemy(5)`; без owner — девять областей `::_area` непустым plug-list до `Hibernate` (vt `+0x64`); ближайшая цель не ближе min-distance навыка, слишком близкая заменяется следующей допустимой |
| `CPassiveGladiator::WhenBeenHurted` / `OnSearchEnemy` / `OnBeenHurted` / `Clear` | `0x211140` / `0x210f00` / `0x210e40` / `0x210ec0` | VERIFIED_DISASSEMBLY | упорядоченный список ≤ 10 уникальных ID нападавших игроков `m_vEnemy`: дубликат пропускает `push_back`, при счёте `> 0xa` вытесняется первый (`memmove` + `end -= 4`); `IndexSet` сохраняет порядок: новичок всегда в конец, дубликат — прежняя позиция; `OnSearchEnemy` удаляет записи на месте `memmove`-циклом (неразрешённый игрок, `RealDistance` > `GetChaseRange` vt `+0x13c` или `IsDied`), равенство дистанций сохраняет раннюю запись (`jge`); tamed-существо (`0x0E6460`) или повозка (`0x0E6D30`) назначается целью немедленно только вне боя; `OnBeenHurted` после базового ставит `ASA_SEARCH_ENEMY` — сам поиск выполняет caller через passive-реакцию Defense |
| `CPuninessCreature::OnSchedule` / `Tracing` / `OnSearchEnemy` | `0x0060F4B0` / `0x0060F390` / `0x0060F4E0` | MATCH | пустой общий hook, owner ≠ 0, `HasTarget == 1`, пустые очереди → tail-call virtual `Tracing` (vt `+0x4C`); Tracing: цель умерла → `OnLoseTarget + AddAIEvent(5)`; внутри `GetGuardRange` — `GetLineDir(target→owner)`, `GetDirPos`, общий `MoveTo(run=0)` (`0x0060F45B`) БЕЗ записи `SetDir`; за дальностью — проверка `GetChaseRange`; поиск: игроки перед питомцами, ближайший с заменой при `≤` (равная дистанция побеждает позднюю запись) |
| `CCarriage::SetCurrentAction` / `OnSchedule` / `OnFallowingSchedule` / `OnStayingSchedule` / master-проверка | `0x00506710` / `0x00506A20` (хвост `0x00506CFE`/`0x00506D04`) / `0x00506720` / `0x005068F0` | MATCH | прямая запись `m_caAction [+0x7C]`; умерший владелец (`GetState != 1`, vt `+0x74`) — журнал и `Evanish` (vt `+0x188`); follow: очереди пусты и `IsMoveable` (vt `+0x148`), хозяин `CPet::GetPetMaster`, тот же регион (vt `+0x44`), знаковая дистанция > 2, `FindPositionForCarriage` (`0x004CCFD0`) даёт клетку ровно в двух шагах сзади; затем повторная проверка региона и беззнаковая дистанция `≤ dwCarriageStopDistance` (`0x00EF45D0`) → `MoveTo(run=0)`, иначе GS0008 и STAYING; staying обратно → GS0009 и FALLOWING; master-проверка: первое пропадание → STAYING + таймер; дубликат `m_nCarriageID` вызывает Evanish до distance-хвоста; возвращение хозяина — прямая запись и FALLOWING до таймера; `disappear` — GS0007 и Evanish; ctor/dtor `0x005066D0`/`0x00506700` построчно не читались |
| точка поста стражника (`GuardStationState`) | `m_lX`/`m_lY` владельца в `0x0060E290` (`OnSearchEnemy`) | MATCH (по прежнему владельцу) | фиксируется один раз при первом проходе и далее только читается; общее дистанционное ядро `zone::ai::guardtarget` разобрано построчно по `0x0060E350`/`0x0060E510`/`0x0060E290` (категорийный выбор — ближайший не ближе min-distance навыка, слишком близкая сохранённая запись заменяется следующей; первый категорийный список выигрывает при равной дистанции, также `0x0060DA90`/`0x0060DB10`) |

Общие факты семейства: `GetDirPos` (`0x0045B330`) безотказен для восьми
направлений (табличное сложение дельт); входной state-вопрос `0x0047B150` и
пустой общий hook `0x00485540` — `RET1`-эквиваленты без наблюдаемого эффекта.

## Боевые формулы

Реализация: `zone::combat::{weaponattack, fightdefense, monsterformula, rounding}`.

### Оружейный roll (`CalculateAttackPower` семейство)

Живой vtable-канал тел: уровень цели `+0x110`, CCH `+0x114` (movzx WORD), живой
AddElementAtk `+0x118`, AddSoulAtk `+0x11C` (movzx WORD), GetMinAttack `+0xE4`,
GetMaxAttack `+0xE8`, weapon modifier `+0x184`; живая ловкость CPlayer —
`[+0x3B8]` через хелпер разрешения `0x619319`. RNG — глобальный
`?random@@YAHH@Z` (VA `0x41CBA0`; его `rand` на `0x61903F` повторяет MSVC-формулу
`state = state * 214013 + 2531011`, `(state >> 16) & 0x7FFF`); выделение
компонента — ctor `tagAttackPower 0x5D3C80`, push_back `0x4AF200/0x4AEFF0`;
критический множитель — изменяемая BSS-глобаль `fmul dword [0xEF3E5C]` (читается
владельцем и передаётся `critical_rate`); константа `0x64DBD0` = 0.01f. Тела
исходных владельцев PDB: `appserver/skills/chuckstone.cpp`,
`skeletonarchery.cpp`, `yakshaslash.cpp`, `ignition.cpp`, `scorpion.cpp`,
`ghostcut*.cpp`, `strike.cpp`, `lightingarrow2.cpp`, `poisonmoth.cpp`,
`bloodrose.cpp`, `explosivearrow*.cpp`, `heartlessarrow.cpp`,
`heartlessarrowphalanx2.cpp/.3`, `jucut.cpp`, `lightningsword*.cpp`,
`inversechopped.cpp` и др.

| вид ширины | якорь | статус | суть факта |
|---|---|---|---|
| RawRange | `CChuckStone::CalculateAttackPower` pub `1:0013c6e0` (RVA `0x13D6E0`); порядок зафиксирован шапками `CStrike 1:00169eb0`, `CGhostCut 1:0019c7e0` | VERIFIED_DISASSEMBLY | чтения MIN (`[esi]+0xE4`) → MAX (`+0xE8`), сырая DWORD-ширина `1 − min + max` (`mov ebx,1; sub; add` — без abs и без нижней границы), RNG, повторное MIN после RNG, сумма и `jns`-нижняя граница нуля; затем Element (`+0x118`) с `jge`-границей, Soul (`+0x11C`, movzx WORD) и общий критический хвост |
| AbsoluteRange | `CYakshaSlash` pub `1:00141dc0` (RVA `0x142DC0`); контрольно `CJuCut 1:00194060` (RVA `0x195060`), `CInverseChopped 1:00148230` (RVA `0x149230`) | VERIFIED_DISASSEMBLY | чтения MAX → MIN, ширина `abs(max − min) + 1` (`cdq; xor eax,edx; sub eax,edx` + `add eax,1`), RNG, повторное MIN после RNG, сумма, `jns`-граница; отличие от RawRange — порядок первых двух чтений и abs |
| CapturedMinimumAbsoluteRange | `CHeartLessArrow` pub `1:001915f0` (RVA `0x1925F0`) | VERIFIED_DISASSEMBLY | чтения MIN → MAX, та же abs-ширина с `+1`, RNG, сумма с СОХРАНЁННЫМ первым MIN (повторного чтения нет); далее Element/Soul и живой CCH (`+0x114`, movzx WORD) |
| знаковый CCH снимка | `CHeartLessArrowPhalanx2` pub `1:001ec910` (RVA `0x1ED910`) | VERIFIED_DISASSEMBLY | hit = 0 и damage_factor = factor·0.01f до компонентов, тот же captured-ролл, затем RNG(100) ПЕРВЫМ и знаковое `jge` с сохранённым DWORD `+0xC4` конструктора — повторного чтения живого CCH нет; NULL источника допускается до SOUL (physical → RNG(1) → 0, element нулевой), затем разыменование NULL — старый владелец отсекает этот случай до вызова формулы |
| фронтальное усиление | JuCut / InverseChopped | VERIFIED_DISASSEMBLY | после суммы со вторым MIN добавляется живая ловкость `[player+0x3B8]`; NULL-результат хелпера пропускает добавку без обрыва (в Rust намеренно оставлен недостижимый обрыв — цепочка уже разрешила игрока выше); InverseChopped до roll расходует состояние ID `0x89` (End+destroy) и умножает все ТРИ компонента f64-множителем: `fild; fmul qword; fistp qword` с `or ah,0xC` и записью МЛАДШЕГО DWORD (`truncate_original_i64_low`) |
| критический хвост | ChuckStone `0x53D888`, HeartLessArrow `0x59286B`, HeartLessArrowPhalanx2 `0x5EDAF3` | VERIFIED_DISASSEMBLY | фильтр видов `cmp [kind],1/3/4; jne skip`, затем `fnstcw; fild; fmul dword [0xEF3E5C]; fldcw (or ah,0xC); fistp; fldcw` только для Physical/Element/Soul; прежний Rust-проход масштабировал без фильтра (наблюдаемого различия не было — вызывающие кладут только виды 1/3/4); zone-форма фиксирует фильтр, симметричный `skills/projectile.rs::apply_projectile_critical` |
| `source_property` по типу владельца | vtable `??_7CPlayer@@6BCMoveShape@@@` RVA `0x24DF4C` + тела getter-ов + нулевые тела `0x201200`/`0xE69B0` + `combat/monsterformula` | VERIFIED_DISASSEMBLY | тип 400 — getter-ы CPlayer прямые чтения полей без поправок: +0xE4 `GetMinAtk` RVA `0x4A350` (`mov eax,[ecx+0x3C4]; ret`), +0xE8 `GetMaxAtk` `0x4A360` (`[+0x3C8]`), +0x114 `GetCCH` `0x4A380` (`mov ax,[ecx+0x3D0]`, живые caller-ы — movzx WORD), +0x118 `GetAddElementAtk` `0x4A420` (`[+0x3E8]`), +0x11C `GetAddSoulAtk` `0x4A430` (`mov ax,[ecx+0x3EC]`); типы 500/1100/1200 — унаследованные нулевые folded тела: +0xE4/+0xE8/+0x118 → `0x201200` (`xor eax,eax; ret`), +0x114/+0x11C → `0xE69B0` (`xor ax,ax; ret`), общие адреса у CNpc/CBuild/CCityGate/CMoveShape; тип 600 — state-границы MIN/MAX и SOUL монстра (`combat/monsterformula`), Element (`0xE7950`, pet factor умножается на ноль) и CCH (+0x114 → `0xE69B0`) всегда 0 |
| Archery | — | не перенесён сюда | мёртвая сырая ветвь без callsites; живой Archery-roll (`max(max-min, 0)` БЕЗ `+1`, два чтения MIN до RNG) — в `skills/projectile.rs` со статусом VERIFIED_DISASSEMBLY; RawRange и оба abs-вида прибавляют `+1` к ширине, Archery — нет |
| RNG при неположительной ширине | `game_legacy_random` старого пакета | наблюдение к владельцу RNG | возвращает 0 без расхода состояния; машинное `random(int)` при bound == 0 вызывает `rand` и возвращает 0 — формула ширину передаёт дословно |

Реализация — `zone::combat::weaponattack`; live-поля приходят обратным вызовом
`WeaponDamageLiveField`, RNG-состояние остаётся у владельца.

### Базовая защита `CFightDefense` (SKILL_BASE_DEFENSE)

| функция | RVA | статус | суть факта |
|---|---|---|---|
| ctor / dtor | `0x001B0970` / `0x001B09D0` | VERIFIED_DISASSEMBLY | id `0xA` = `SKILL_BASE_DEFENSE`; sentinel = `UNKNOWN_SKILL_ID` |
| `PreDefense` | `0x001B0A50` | VERIFIED_DISASSEMBLY | skip-правила `530..=545 && != 544`; диспетчер щитов и само правило — `effects/defenseshield.rs` |
| `Defense` | `0x001B10E0` | VERIFIED_DISASSEMBLY | hit-таблицы по occupation; level-модификатор `(Δlevel − 3)·15` через `shl 4` + `sub`; full-miss x87-ветвь без f32-промежутка (сравнивает целый RNG с `FILD u16 · FMUL f32`, дробная часть порога не усекается); порядок RNG hit → blast; обнуление видов 1/3/4 при miss с сохранением Poison (`full_miss = 2`); логический SHR половины unsigned defense до float (старший бит корректируется `+2^32`); критический множитель `−0.5f32`; один FISTP уклонения (`0.01_f32` в x87, усечение только итогового произведения); `Promotion` перемножает два целых и применяет `0.001_f32` с усечением конечного результата; PvP factor после clamp без повторного clamp |
| `PostDefense` | `0x001B1050` | VERIFIED_DISASSEMBLY | Pillar factor: произведение целого урона и сохранённого f32-коэффициента усекается к нулю при записи обратно в целое поле |

Общий FISTP DWORD-адаптер (Zone `combat/rounding`) сначала усекает к нулю и лишь
затем проверяет signed range: дробная часть выше INT_MAX ещё может усечься в
INT_MAX; NaN/inf/переполнение дают исходный indefinite INT_MIN. Safe/city-war
гейты — у caller (`game/periodicattack.rs` старого пакета), маркер city-war
совпадает. Установленное расхождение f32-округления mp-фактора закрыто в Zone
`effects/{shieldabsorption, lifeshield}` и формул этого файла не касается.

## Войны и расписания

Реализация: `zone::activities::{attackcitysys, villagewarsys, countrywarsys, fournationwarsys}`.

### CAttackCitySys

| функция | RVA | статус | суть факта |
|---|---|---|---|
| snapshot/setup, initial state, faction update, фазовые callbacks | `0x00060E40`, `0x000600F0`, `0x000601F0`, `0x00060320`, `0x00061040..0x00061420` | IMPLEMENTED, VERIFIED_DISASSEMBLY | PDB: восемь пар `event ID + tagTime`, signed schedule/city/state и `std::list` с offset `0xAC`; weekly flag `+0xB8` в snapshot не входит; `tagAttackCityTime` prefix `0xB0` |
| schedule queries | `0x0005FAD0..0x0005FBC0`, `0x000603C0` | IMPLEMENTED | map-order, strict `now < end`, packed minute-формула с x86 wrapping, main-then-proxy lookup имени |
| внешний цикл decoder | `0x00461009..0x00461015`, `mov al,1` по `0x00461025` | VERIFIED_DISASSEMBLY | — |
| vtable фаз Регионов | slots `+0x78..+0x90` | сверены с точными сигнатурами `CServerRegion` из PDB | declare/start/timeout/end/mass допускают proxy, clear/refresh — только основной map; timeout действует лишь в `Fight`, clear — лишь в `Mass`; `DUTH=1`, `Mass=2`, `Fight=3` |
| initial-state / faction-update | — | VERIFIED_DISASSEMBLY | closed interval `DeclarWarTime <= now <= AttackCityEndTime`; два stack-аргумента у vtable slot `+0xE8`; faction-update ищет city region только в `s_mapRegion` без proxy fallback, virtual `UpdateContendPlayer()` slot `+0x104` |
| membership queries | — | подтверждённый UB, не воспроизводится | оба query проверяли `bIsEveryWeek`, но setup кодирует только `0xB0`-префикс: constructor не задаёт DWORD `+0xB8`, assignment копирует неинициализированный stack — оба query используют переданный список фракций |

### CVillageWarSys

| функция | RVA | статус | суть факта |
|---|---|---|---|
| snapshot/setup, initial state, faction update, фазовые callbacks | `0x0005F320`, `0x0005E310`, `0x0005E480`, `0x0005E5B0..0x0005E640`, `0x0005F510..0x0005F740` | IMPLEMENTED, VERIFIED_DISASSEMBLY | PDB: шесть пар `event ID + tagTime`, signed schedule/region/state и `std::list` с offset `0x88`; weekly flag `+0x94` в snapshot не входит; prefix `tagVilWarSetup` `0x8C` |
| schedule queries | `0x0005DA90..0x0005DB80`, `0x0005E830` | IMPLEMENTED | — |
| внешний цикл decoder | `0x0045F4D3..0x0045F4DF`, `mov al,1` по `0x0045F4EF` | VERIFIED_DISASSEMBLY | — |
| initial-state / фазы | — | VERIFIED_DISASSEMBLY | closed interval `DeclarWarTime <= now <= WarEndTime`; byte `CRegion::m_btCountry +0x74` копируется из village в war region после reset; два stack-аргумента у slot `+0xE8`; start/end допускают proxy, timeout ищет только основной war-region; clear шлёт `0xBF806(0xFFDAEDFE, 0, GS0127)` и запускает вытеснение через 60000 ms; faction-update — без proxy fallback, slot `+0x104` |

### CountryWarSys

| функция | RVA | статус | суть факта |
|---|---|---|---|
| layout `CountryWarRegion` | — | PDB + точечный дизассембл | `{ defend:i32, attack:i32, clear:bool+padding }`, map-order, signed IDs, writer-before-region-callback |
| `init_country_region_state` | — | VERIFIED_DISASSEMBLY | вопреки имени только lookup: после успешного `find` вызывается не меняющий entry `operator[]`, side state не переносится |
| `update_apply_war` | — | VERIFIED_DISASSEMBLY | сначала меняется запись, затем найденный region и только после двух country-полей — `UpdateContendPlayer()` slot `+0x104` |
| phase chain | vtable slots `+0x138..+0x150` | VERIFIED_DISASSEMBLY | declare обнуляет country result `1..4`; start/timeout и prepare — только для записей с двумя ненулевыми сторонами; end сбрасывает три global-флага и стороны; timeout после callback читает low byte сторон именно из region |
| victory chain `on_flag_destroy` | `0x000EBE60` | VERIFIED_DISASSEMBLY | без найденного non-null region не делает ничего; входная страна получает result 2, найденная через `GetOtherCountry` — result 1; обе country-map keys усекаются до low byte; потерянные декомпилятором stack-присваивания и порядок двух writes подтверждены дизассемблированием |

### CFourNationWarSys (пара сторон Game ↔ World)

| функция | RVA | статус | суть факта |
|---|---|---|---|
| snapshot wire | `0x7FE3C..0x7FE45` (фазовая цепочка), direct morale `0x7FE49`, replacement player-war-time `0x7FE47` | IMPLEMENTED, VERIFIED_DISASSEMBLY | signed count + 196-байтные (`0xC4`) setup records, затем signed count и 16-байтные `tagRECT`; World serializer (пара `Nworldserver.exe + WorldServer.pdb`) и Game decoder используют одинаковый порядок; повторный snapshot принимается, пока setup vector пуст; dangling pointers оригинала выражены typed error; rect count > 5 блокируется после допустимого префикса; `InitWarState` — vector-order, main region с proxy fallback, только nation region, пять relive rectangles; фазовая цепочка сохраняет DUTH/Mass/Fight, local/proxy lookup, очистку process-wide времени и morale, пятизначный signup payload и atomic take пяти результатов |

Остальные player-war-time queries этого владельца сохраняют статус RAW.

## Торговля и деньги

Реализация: `zone::trade::{ctrader, auction, session, currency, ground, audit}`;
контракт — `docs/gameplay/trade.md` (двусторонний обмен).

### CTrader (рамка обмена)

Машинный статус — `MATCH` по подсемейству trade дизассембла тел точной пары:
CTrader трёх shadow-контейнеров; reset ready у обоих участников при любой смене
рамки `OnObjectAdded`; commit `0x1B9470` — 1241 insn, per-goods AmountChange →
packet Add → move publish; частичный отказ → Clear(temp) + RollBack(map)
полностью и немедленно.

| правило | машинный факт | статус |
|---|---|---|
| extend-id рамки | `plug << 8 \| index` (`0/1/2` = goods/Gold/YuanBao) | семья MATCH |
| приёмка goods | currency в goods → NoTrade flag `0x20` → позиция за объёмом → занятость ячейки | семья MATCH |
| приёмка валюты | несовпадение index ожидаемой валюты, единственная позиция `0` | семья MATCH |
| источник предложения | goods ← packet `1`/equipment `2`, Gold ← wallet `4`, YuanBao ← increment `5` | семья MATCH |
| сверка количества | packet снимает часть стека (`>=`), экипировка целиком (`==`) | семья MATCH |
| rollback stack-merge | полный merge (`amount == original`) обратим вычитанием; иначе `None` | семья MATCH |

Риск-ноты: между локальным сбросом собственного ready и внешним сбросом
готовности второго участника есть micro-окно исходного порядка — оно является
контрактом очерёдности оригинала и сознательно не уплотняется. При частичном
слиянии доставленного предмета со стеком получателя обратного удаления части
стека в commit-проходе нет — как и у release-формы оригинала (`None` caller-а).

### Player-аукцион

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CPlayer::AuctionLimit` | `0x0002EF00` | VERIFIED_DISASSEMBLY | побитовая логика GAP13: `&0x20→2`, `&0x04→−1`, attr `0xE5`→`−1`, иначе `3` |
| `CPlayer::CheckAuctionMoneyMove` | `0x000369B0` | VERIFIED_DISASSEMBLY | checked unsigned sum обоих wallet против max stack основного; notify `GPM015` у caller-а |
| `CPlayer::IsAollowAuction` | `0x00036200` | VERIFIED_DISASSEMBLY | 1-сек gate `[+0xCD0]`: timestamp до limit-queries; owner count `CAuctionRoom@CGame+0x1B8`; bonus slot1 `GAP 0xEA == 3 → value2`; float-compare; отказ поглощает попытку + notify |
| `CPlayer::BuyItemFromAauction` | `0x00030BC0` | VERIFIED_DISASSEMBLY | reject при `sampled ≤ last + 5000` — signed `jbe` после wrapping-сложения; второй сэмпл записывается до GUID decode |
| `CPlayer::MakeCurAucNode` | `0x00045A50` | VERIFIED_DISASSEMBLY | 5-сек двухсэмпловый gate; pipeline `IsAollow → IsMoney → IsCurAucNodeOK → DeleteGoods → узел` у caller-а |
| `CPlayer::ReFlushSelfGoods` | `0x00030D90` | VERIFIED_DISASSEMBLY | тот же 5-сек gate |
| `CPlayer::OpenAuction` | `0x000367B0` | VERIFIED_DISASSEMBLY | клиент `0xC0706` 11 dword → World `0x60810` 1 dword |
| `CPlayer::AutoAddAuctionGoods` | `0x000466A0` | VERIFIED_DISASSEMBLY | каждый созданный предмет безусловно заменяет `m_CurrentAucNode` |
| `CPlayer::AddItemToAuction` | `0x00045910` | VERIFIED_DISASSEMBLY | порядок `node Serialize → World 0x60801 → SendSaleLog → AddByteGS2WS → TellClietAuctionOK → Clear` у caller-а |
| `IsGoodAllowedInAuction` | `0x0002EA50` | UNKNOWN → INFERRED заполнение | линейный скан 256 dword таблицы `0xEF4AC8`; таблица в образе нулевая — Rust использует конфиг-список `GlobeSetupSnapshot::auction_goods_allowed` (Shared resources), поведенчески совпадающий при заполненном конфиге |

Соседние владельцы (не вошли в Zone-файл): wallet/container мутации
`ModifyAuctionSpace 0x0003F7A0`, `GetAuctionMoney 0x0002EEF0` (tail-jmp amount
`[+0x8F0]`), `SetAuctionMoney 0x00030E60`; message runtime caller-ы
`TellClietAuctionOK 0x0003ED50`, `WriteBuyAuctionLog 0x00036370` (`0x60214`),
`SendToGSBaiTan 0x0002F1C0`/`NoticyWS_BaiTan_Over 0x0002F140`
(`0x60811`/`0x60812`); fee-формулы `GetOptMoneyJin/Yuan` (x87) не реализованы.

### Денежное ядро и оркестрация обмена

| правило | машинный факт | статус |
|---|---|---|
| merge денег commit `0x1B9470` | 64-бит `sub eax,edx; sbb esi,0` с clamp к 0 и `add/adc` (НЕ i64-saturating); валидированное предусловие делает wrap/clamp недостижимым | семья MATCH |
| маршруты банка | `4→8`, `8→4`, `15→4`; позиции строго `0` | семья MATCH |
| auction-return `15→4` | `CheckAuctionMoneyMove` ДО remove, повторно после remove → RollBack + `GPM015`/`GPM019` у caller-а | семья MATCH |
| установка баланса | `SetYuanBao`/`SetAuctionMoney`: increase на `current − previous`, decrease на `previous − current`, иначе без изменения | семья MATCH |
| ground-источник валюты | wallet `4` / increment `5` | семья MATCH |
| bank-источник | wallet `4` / bank `8` / auction wallet `15` | семья MATCH |
| порядок `CheckTradeCondition` | contrary trader + owners → ёмкости/весы/деньги → IsSpaceEnough; 8 уведомлений | семья MATCH |
| ёмкость валют | `maximum < balance − outgoing + incoming` (wrapping DWORD) | семья MATCH |
| вес | burden после замены `own → incoming` строго превышает max | семья MATCH |
| Billing-вилка | signed разность YuanBao → payer/receiver/amount | семья MATCH |
| запрос `0xEF203` | кадр type 2 + SendToBS (вызов `0x001BB036` → `0x00013BE0`) | семья MATCH (кадр по caller-у прежнего owner) |

Риск-нота Billing-complete: завершение обмена по ответу Billing исполняет commit
БЕЗ повторной дистанционной проверки участников — оригинал доверяет уже
зафиксированной рамке; гейты сверх исходного сознательно не добавляются.

### Наземное перемещение и журналы

| правило | машинный факт | статус |
|---|---|---|
| drop-запрет | particular flag `0x100` (`GAP 0x13`) отклоняет drop | семья MATCH |
| сумма drop | `0 < amount ≤ source`, equipment — строго целиком | семья MATCH |
| частичный drop | busy progress блокирует частичный drop НЕ-валюты | семья MATCH |
| дистанция pickup | отказ при `\|dx\| ≥ 2` или `\|dy\| ≥ 2` | семья MATCH |
| currency назначение | Gold → `(4, 0)`, YuanBao → `(5, 0)` | семья MATCH |
| цена move-журнала | валюта пишет amount, остальные — price | семья MATCH |
| кадр `0x60201` предмет обмена | byte 0, обе party-записи, GUID, price, amount, name, оба IPv4 | семья MATCH |
| кадр `0x60202` move-журнал | reason byte, actor-запись, GUID, price/amount rule, name, region/tile/IP | семья MATCH |
| кадр `0x6020D` валюта обмена | kind byte, transaction, amount, текст, owner, balance | UNKNOWN (состав машинно не переоткрывался; форма по прежнему owner) |

## Сессии игрока

Реализация: `zone::interactions::csessionfactory`, `zone::sessions::{cequipmentupgrade, cequipmentdakong, cequipmentcompose}`.

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CSessionFactory::QuerySession` | `0x000780C0` | VERIFIED_DISASSEMBLY | возвращает null при отсутствии ключа; PDB: статическая `hash_map<long, CSession*>` |
| `CSessionFactory::QueryPlug` | `0x00078190` | VERIFIED_DISASSEMBLY | то же для plug (`hash_map<long, CPlug*>`) |
| `CTeam::GetTeamatesAmount` | VA `0x00507590` | VERIFIED_DISASSEMBLY | считает только разрешённые plug ID своего списка через `QueryPlugByID` (VA `0x0047B6C0`), не сырую длину; повторные ID учитываются повторно; ended/type/локальность не фильтруются |
| `CEquipmentCompose::OnSessionEnded` | — | VERIFIED | compose-plug только проверяет наличие player/region и возвращает bool, который `CSession::End` не использует; отдельной mutation/publication при завершении сессии нет (`zone::sessions::cequipmentcompose`) |

Остальное у сессий экипировки — прежняя форма старого пакета
(`src/gameserver/appserver/session/cequipmentupgrade.rs` /
`cequipmentdakong.rs`), материализованная в Zone поверх типов Zone items/content;
машинные таблицы не велись, статусы не повышались. Половинное свойство седьмого
слота DaKong сохраняет x87-усечение к нулю, включая отрицательные значения
снятия камня.

## Квестовые записи и wire-снимки CPlayer

Реализация: `zone::quests::{availability, client, progress}`; исходный владелец
PDB — `server/gameserver/appserver/player.cpp`, точная пара
`GameServer/gameserver.exe + GameServer/GameServer.pdb`.

| функция | VA | статус | суть факта |
|---|---|---|---|
| `QuestTimeBegin` / `QuestTimeClear` / `SetQuestOn` | `0x0042DA20–0x0042DAAF` / `0x0042DAC0–0x0042DB32` / `0x0042DB40–0x0042DBB4` | MATCH (по шапке владельца) | записывают поля игрока до создания клиентского сообщения |
| script `GetQuestTime` | `0x004BA553–0x004BA59A` и `0x004B81ED` | MATCH (по шапке владельца) | проверяет нули и обнуляет отрицательный остаток |
| client `0x8FA12` | `0x004FAB19–0x004FAB6A` | MATCH (по шапке владельца) | отправляет сырую 32-битную разность |
| `AddQuestDataByteArray` | `0x00433310–0x0043339F` | MATCH (по шапке владельца) | count, затем упорядоченные u16 ID и u8 state |
| `AddQuestDataByteArray_ForClient` — фильтр и снимок входа | `0x0043E204–0x0043E229`, `0x0043E229–0x0043E339` | MATCH (по шапке владельца) | фильтр пропускает state 1 и неизвестные каталогу ID; снимок пишет u16 ID, пять u32 (old, type, level, difficulty, track), три C-строки (short description, name, description), u8 display и четыре i32 (region, x, y, effect); последний i32 читается со смещения +0x74 после +0x78/+0x7C/+0x80 |
| `CPlayer::AddQuest` и уведомление `0xBFF2C` | `0x00445397–0x00445531`, `0x004453F4–0x004454FE` | MATCH (по шапке владельца) | удаляет прежнее ненулевое состояние до поиска определения и только затем вставляет новый ноль; уведомление пишет тот же набор полей в том же порядке |
| `CPlayer::RunQuestCompleteScript` | `0x00458940–0x004589AA` | MATCH (по шапке владельца) | допускает только запись с нулевым byte state, затем ищет определение и передаёт его complete-script в общий исполнитель |
| клиентские ветви `OnOrgasysMessage` | `0x0048AE0C–0x0048AED2`, `0x0048AED7–0x0048AF9D` | MATCH (по шапке владельца) | проверяют нулевое состояние перед поиском complete/disband-script; запуск сценария принадлежит Game |

## Предметы и контейнеры

Реализация: `zone::items::*`, `zone::content::{goodsfactory, honorranks}`;
контракт — `docs/gameplay/items.md`.

| функция | RVA | статус | суть факта |
|---|---|---|---|
| `CEquipmentContainer` колонки/lookup/подсчёты/weight/`Clear`/mode-`Release` | `0x000ED180..0x000EDB30`, `0x000EEBF0` | VERIFIED_DISASSEMBLY | 17 колонок `0..=16` и их enum-order; число непустых колонок сравнивается именно с 17; вопреки позднему архивному донору `Clear`/`Release` не обнуляют `m_nExpantPkgNum`; `Release` очищает external listeners через `CGoodsContainer::Release`, а внутренний self-listener представлен прямым derived callback-ом (не хранится самоссылкой) |
| `CBattleFairyContainer` positional фильтры | — | MATCH по точной паре | 17 фиксированных ячеек боевой феи; gear-слоты публикуют ранний `BFPropertyAdd(+1)` effect до base Add (отказ storage его не отменяет); combine: check `0xbf92c` и execution `0x8fc27` с машинными notification `(56/57/58, затем 56/57/60)`, молчаливый execution no-match (tail `0x503F81`), порядок remove `body → stone → material`; `fistp` — truncation к нулю; восемь constructor-owned incompatible pairs `ResetSkill` подтверждены EXE immediate-ами; ошибочный повторный native pointer guard в optional-gem tail заменён независимой обработкой ячеек `13..=16` |
| автоматический overload `CBattleFairyContainer` | — | UB → typed block | читает неинициализированный `m_eBFEquipPlace` у catalog owner-а; Rust выражает typed block, не выбирая ячейку из позднего C++-донора |
| `CGoods::CanStacked` | — | null-deref ветвь | потерянный registry key давал legacy null-deref; выражен typed block-ом, не тихим `false` |
| `CWallet` decrease | — | VERIFIED_DISASSEMBLY | unsigned wrapping subtraction при запросе больше balance — наблюдаемый legacy-контракт; wallet владеет одним `CGoods` catalog `MONEY`, а не числовым balance; generic core обслуживает также `CYuanBao` и `CJiFen` |
| `CDepot` восстановление | — | VERIFIED_DISASSEMBLY | база 96 ячеек, шаг extension-группы 13, конец расширения `96 + 13·5 = 161`; restore очищает исходные 96 ячеек, при `bToAdd` расширяет их на 65; lock возвращается при любом результате |
| `CGoodsFactory::EquipmentWaste` | `0x00063E50` | VERIFIED_DISASSEMBLY | отдельные base/modifier: вычитает fray из итоговой прочности, но записывает результат в `base_value` |
| `ReCreateBattleFairyAttributes` | — | VERIFIED_DISASSEMBLY | странный повтор полного набора бросков RNG по числу дополнений экземпляра; однопроходная оптимизация донора не переносится, потому что меняла итоговое состояние игрового RNG |
| `CHonorRanks::DecordFromByteArray` | `0x0000D390` | VERIFIED_DISASSEMBLY | 4 rank types × 4 country lists; для country `-1` списки очищаются и декодируются по порядку; record: player ID, level byte, NUL-name, occupation byte, appellation ID, eliminate count; non-positive count — пустой list |
| `CHonorRanks::getInstance` | `0x0000CFA0` | VERIFIED_DISASSEMBLY | singleton заменён owned-полем `CGame::honor_ranks`; запись текущего дня — отдельная static `m_nSortDate`, достигнутые honour-маршруты её не читают |
| `CHonorRanks` dtor `$E4`/`$E2` | `0x0000D0D0` | VERIFIED_DISASSEMBLY | static list initializer и sized-delete покрыты `Default`, `Vec` и `Drop`; доменной семантики нет |
| `CHonorRanks::GetPlayerPosition`/`AddToByteArray` | — | IMPLEMENTED | 1-based snapshot order; exact count/record payload для client `0xBFF35` |
| `CContainer` ctor/dtor, `AddListener`/`RemoveListener`, `Find/Remove`, `tagPreviousContainer` | ctor/dtor `0x000DF570/0x000DF2A0`, `0x000DF5B0`, `0x000DF250`, `0x000DF1A0..0x000DF200`, `0x000DF1D0` | MATCH (по шапке владельца) | ctor/dtor владеют только ordered vector listener-ов; `AddListener` отклоняет null и duplicate, `RemoveListener` удаляет первое совпадение с сохранением порядка; `Find/Remove` — только virtual forwarding thunks (overload с type игнорирует type, overload с object извлекает `m_guExID`, null → null); `tagPreviousContainer` сохранён буквально (`zone::items::ccontainer`) |

Остальные ещё не подключённые player-integrated методы контейнеров и constructor/
release time-поля `CGoods` требуют реконструкции: достигнутые ядра не выдаются за
весь legacy object.

## Сетевой край GameServer

Реализация: `zone::app::{game_message, game_server_client, game_client, game_server}`;
исходники `nets/netserver/*.cpp` той же точной пары (PDB-owner
`e:\svn\fengyun_russia_dev\nets\netserver\*.cpp`); общие типы — Shared
(`CBaseMessage`, `CServerClient`, `CClient`).

### CMessage (направление GameServer)

| функция | RVA | статус | суть факта |
|---|---|---|---|
| ctor `CMessage(long)` | `0x000136D0` (VA `0x4136D0`) | VERIFIED_DISASSEMBLY | базовый `0x413520`, vtable `0x64CA48`, `MsgType` в header `+4`, нули пяти runtime dword `+0x18/+0x1C/+0x20/+0x24/+0x28` (region, player, numeric map, socket, IPv4); объект `0x2C` выделяется во всех synthetic-путях |
| `CreateMessage`/`CreateMessageWithoutRLE` | `0x00013700`/`0x00013850` | IMPLEMENTED, VERIFIED_DISASSEMBLY | RLE-путь: capacity `0x800000` для входа до `0x100000` включительно, затем exact `compressed_len * 8`; decode/create/free общего scratch под critical section до возврата |
| send-family | `SendToSocket/SendToPlayer/SendAll 0x00013910/0x000139C0/0x00013A70`, `Send/SendToBS 0x00013B40/0x00013BE0`, region/area/country `0x00014100/0x000142B0/0x00014760`, around `0x00014420/0x00014970`, `Run 0x000149D0` | IMPLEMENTED, VERIFIED_DISASSEMBLY | wire/domain-владелец процесса — сторона доменного расширения типа (`src/nets/netserver/message.rs` старого пакета) |
| synthetic close-сообщения | — | VERIFIED_DISASSEMBLY | `OnClose` принятого client (`0x41C670`) — `CMessage(0x6FA01)` через ctor `0x4136D0`; outgoing close (`0x41A590`) — `0x6F901` (World) / `0x6F903` (Billing), пустая cstring `0x64BB41` |

Известные классы повреждённого wire: ненулевой wire короче header и переполнение
старой RLE capacity — `HeaderTooShortReactionUnknown` /
`RleCapacityOverflowReactionUnknown` без фиктивной реакции.

### CMyServerClient (принятое client-соединение)

| функция | VA | статус | суть факта |
|---|---|---|---|
| ctor | `0x41C5C0` | VERIFIED_DISASSEMBLY | base `0x41B850`, vtable `0x64D6EC`, receive capacity ровно `0x5000`, два компаньона `0xC800` в `+0x70/+0x74` (исходный send accumulator) |
| `OnReceive` | `0x41C7F0` | VERIFIED_DISASSEMBLY | gate owner `+0xA8`; цикл пока накоплено `>= 0xC`; флаг `[owner+0x10C]`: предел полной длины (`declared > [owner+0x118]`) и CRC длины через `DataCrc32 0x47B0A0`, `declared > size` — останов без потери хвоста; флаг `[owner+0x10D]`: CRC по сжатым байтам `[+0xC, declared-0xC]`; create только RLE `0x413700`; opcode допустим в `(0x8F700, 0x9F600)` exclusive — точное `0x8F701..=0x9F5FF`; контекст `[+0x34]→[+0x24]` socket, `[+0x88]→[+0x20]` map, `[+0x2C]→[+0x28]` IPv4 (CD-key у Game-получателя не присваивается); publish через push helper `0x4126D0`; consume `sub size, declared`; shrink к `0x100000`; reject очищает accumulator без отката опубликованного |
| ban-пути | внутри `0x41C7F0` | VERIFIED_DISASSEMBLY | доказаны ровно четыре пути с `AddForbidIP 0x417960` + `QUIT 0x415080`: предел длины (с virtual `OnTotalMessageSizeOver [+0x40]`, `0x41C9E3`), length CRC (`0x41CA19`), content CRC (`0x41CA49`), opcode вне диапазона (`0x41CA93` с deleting dtor); create-null — без ban (`0x41CABE`: очистка size и sprintf-log) |
| `OnClose` | `0x41C670` | VERIFIED_DISASSEMBLY | при нулевом map identity — ни публикации, ни общего close; при ненулевом — `CMessage(0x6FA01)`, publish, общий close `0x41AD10(0)` |

Malformed length/RLE границы — локальные `BLOCKED_MISSING_FACT`.

### CMyNetClient (исходящий край к World/Billing)

| функция | VA | статус | суть факта |
|---|---|---|---|
| ctor / dtor / `GetSocketCommand` | `0x41A4B0` / `0x41A530` / `0x41A520` | VERIFIED_DISASSEMBLY | task type zero (исходный `ST_UNKNOWNSERVER == 0`) |
| `OnReceive` | `0x41A6A0` | IMPLEMENTED, VERIFIED_DISASSEMBLY | один вызов читает не более `0x2800`; `WSAEWOULDBLOCK 0x2733` и `recv == 0` — только журнал; цикл frames пока `>= 0xC`: CRC длины через `DataCrc32 0x47B0A0`, `declared > size` — останов без потери хвоста, create только несжатым `0x413850` (upstream без RLE), повторный CRC, publish в FIFO `+0x100` через `0x4126D0`; shrink к `0x100000` |
| `HandleClose` | `0x41A590` (virtual слот 19) | VERIFIED_DISASSEMBLY | по server type `[client+0x1F0]` публикует в ту же FIFO `+0x100` `CMessage(0x6F901)` (World) либо `0x6F903` (Billing); ветка unknown type ставила stack-local value без доказанного смысла — typed `UnknownServerTypeCloseReaction`, Rust-значения не получает |
| завершение send-loop | `CClient::Close 0x004191b0`, loop `0x0041a171..0x0041a19d` | VERIFIED_DISASSEMBLY | дописывание исходящей очереди до закрытия сокета: ExitSocketThread ждёт проверкой очереди и живого соединения |
| `SetSendRevBuf` | RVA `0x0001A650` | BLOCKED_MISSING_FACT | передавал Windows `SO_SNDBUF=0`; Linux backpressure-эквивалент не доказан — без фиктивного вызова |

### CMyNetServer (listener игровых клиентов)

| функция | VA | статус | суть факта |
|---|---|---|---|
| ctor | `0x418E70` | VERIFIED_DISASSEMBLY | базовый `CServer` ctor `0x418A80`, vtable `0x64D024`, поле `+0x120 = 0`, limits `+0x14C = 3` и `+0x150 = 0x400000` |
| `CreateServerClient` / sender-виртуалы / `OnMapIDError` | `0x418EB0`, slots `+0x38`/`+0x3C`, `0x418F10` | IMPLEMENTED | — |
| `ServerCommandHandle` | — | VERIFIED_DISASSEMBLY | во всех трёх случаях синхронно копирует payload в owned-команду до возврата — старый общий RLE scratch-buffer не требует Rust lifetime-lock |
| oversized `SendAll` diagnostic | — | локальная logging-граница | печатал inherited `CMySocket::m_lIndexID +0x34`; ctor это поле не инициализирует, а перенесённый `CGame::InitNetServer` writer-а не содержит — Rust хранит только явно наблюдённое позднее значение как `Option` |
