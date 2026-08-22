//! DB-владелец `CRsPlayer` исторического WorldServer из `rsplayer.cpp`.
//!
//! Статус `CreatePlayerBase` RVA `0x00102350`, `SavePlayerBase` RVA
//! `0x00102700`, `SavePlayerAbilities` RVA `0x00109B00`, `SaveHotKeyField` RVA
//! `0x00103B50`, `SaveScriptFlag` RVA `0x00104240`, `SaveSkillField` RVA
//! `0x00104BE0`, `SaveFriendField` RVA `0x00104E20` и `SaveStateField` RVA
//! `0x00105C10`, `SaveThingField` RVA `0x00105700` и `SaveCiQingField` RVA
//! `0x00106A60`, обратные `LoadHotKeyField/LoadStateField/LoadScriptFlag`
//! RVA `0x00103D40/0x00103FC0/0x00104470`, `LoadQuestData` RVA `0x00104790`,
//! `LoadCiQingField/LoadSkillField/LoadThingField/LoadFriendField` RVA
//! `0x0010EFF0/0x0010F8E0/0x00110260/0x00111050`, `SaveQuestData` RVA
//! `0x00106270`, полный query-owner `LoadQuestData` RVA `0x00104790`,
//! `CreatePlayerAbilities` RVA `0x00106CE0`, внешний
//! `CreatePlayer` RVA `0x0010ED40`, внешний `SavePlayer` RVA `0x0010EE70`,
//! `GetPlayerID` RVA `0x00102080`, `GetCDKey` RVA `0x0010EA20`,
//! `GetPlayerCountInDBbyCdkey` RVA `0x00100C40`, `GetPlayerDeletionDate` RVA
//! `0x00100EA0`, `OpenPlayerBaseInDB` RVA `0x0010D2D0`,
//! `GetPlayerCountryByID` RVA `0x00101E30`, `GetPlayerNameByID` RVA
//! `0x00105980`, `ValidatePlayerIDInCdkey` RVA `0x00101350`,
//! `GetPlayerData` RVA `0x00114EA0`,
//! caller-connection путь `LoadPlayer` RVA `0x001117F0`,
//! `OpenPlayerBaseInMem` RVA `0x0010F280`, внешний `OpenPlayerBase` RVA
//! `0x0010F750`,
//! `RestorePlayer` RVA `0x00101260` и
//! `DeletePlayer` RVA `0x00101A60`, а также `LoadHonorRanksByType` RVA
//! `0x0010FB90`, внешний `LoadHonorRanks` RVA `0x001113B0`, `InsertHonorRanks`
//! RVA `0x00101680`, `SaveHonorRanksByType` RVA `0x00105080` и внешний
//! `SaveHonorRanks` RVA `0x00105E20`, `StatRanks` RVA `0x00109570`,
//! `DbLetTingUpdate` RVA `0x00110520` и `ResetAllLeitingInDB` RVA
//! `0x00110F70` — `IMPLEMENTED`; `LoadPlayer` имеет статус
//! `IMPLEMENTED_PARTIAL/VERIFIED_DISASSEMBLY`; constructor, destructor
//! и остальные функции ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp`.
//!
//! PDB задаёт `CPlayer` размером `0x9B8`, `m_BaseProperty` по `+0x598` и
//! `m_btCountry: unsigned char` по `+0x844`; `tagBaseProperty` имеет размер
//! `0x1CC`, а `lLevel/lHeadPic/lOccupation/lSex` являются `unsigned char`.
//! Signed player ID, byte-exact name и signed region принадлежат base-owner-ам
//! уже достигнутого `CPlayer`. `CGame::GetPlayerEquipID` отдаёт одиннадцать
//! `unsigned long` ID и одиннадцать `unsigned char` уровней в SQL-порядке
//! `HELM, BODY, GLOV, BOOT, WEAPON, BACK, HEADGEAR, FROCK, WING, MANTEAU,
//! FAIRY`. Rust получает один caller-owned snapshot вместо чтения частично
//! материализованного `CPlayer` и singleton `CGame` внутри DB-owner-а.
//! `OpenPlayerBase` отдельно сохраняет два исходных SQL-прохода: byte-count
//! через `SELECT ID`, затем ordered `SELECT *`; при необходимости для каждой
//! DB-строки выполняется прежний `SELECT DelDate`. Parameter binding устраняет
//! только injection/stack-buffer дефект. Row wire и live/save/creation merge
//! принадлежат достигнутому caller-у `OnLogMessage`, поэтому DB-owner отдаёт
//! typed scalar rows без второго протокольного builder-а.
//! Exact `GetPlayerDeletionDate` `0x0050116B..0x00501238` нормализует
//! `_mktime == -1` в `0`, но catch возвращает signed `-1`; null `DelDate`, EOF
//! и нулевой player ID возвращают `0`. Эти различающиеся значения сохранены,
//! потому что caller считает `-1` ненулевым deletion timestamp.
//! `GetPlayerCountryByID` и `GetPlayerNameByID` выполняют отдельные exact
//! `SELECT Country/Name ... WHERE id=%d`; EOF/exception оставляют заранее
//! обнулённый output. Rust использует параметризованный TDS-запрос на
//! переданном синхронном connection и возвращает соответственно `0`/пустой
//! ANSI-вектор. Это заменяет отдельный ADO connection и небезопасные
//! `_sprintf/lstrcpyA`, сохраняя DB-порядок caller-а и значения отказа.
//!
//! Exact `0x005023BE..0x005026A8` исправляет потерянный raw vararg-хвост:
//! после одиннадцатого equipment level последним `%d` передаётся signed
//! `Region`; все 32 аргумента соответствуют literal SQL по `0x0054DA60`.
//! Успешный `ExecuteCn` выставляет `AL=1` по `0x005026A8`; null player/null
//! connection и catch `0x005026AC` сходятся к `AL=0` по `0x005026D6`.
//! Rust-ссылки исключают null, DB-ошибка возвращает `Failed`, число затронутых
//! строк исходно не проверяется.
//!
//! SQL сохраняется byte-exact, включая `N'%s'` только для имени, обычный
//! `'%s'` для account, табуляцию и отсутствие escaping. `unsigned long`
//! equipment ID форматируется как исходный `%d`, то есть через тот же signed
//! 32-битный шаблон. `_sprintf` писал в `char[1024]`: до 1023 output-байт batch
//! воспроизводится, а больший вывод остаётся локальным `BLOCKED_MISSING_FACT`,
//! потому что поведение старого buffer overflow не доказано. Windows-1251,
//! `tiberius`, stream consumption и Rust Drop заменяют только ANSI/ADO/COM и
//! compiler cleanup.
//!
//! `CreatePlayerAbilities` сначала создаёт updatable row, заполняет 84 scalar
//! поля, затем до единственного `Recordset::Update` последовательно вызывает
//! семь binary helper-ов. `SaveHotKeyField` точечно замыкается обратным
//! `LoadHotKeyField`: PDB-массив `dwHotKey` занимает `0x60` байт, save пишет 24
//! `unsigned long`, а load принимает field-size только `0x60` и читает 24
//! слова. Следующий `SaveSkillField` сохраняет list-order `m_listNewSkillID`:
//! PDB задаёт `CPlayer::tagSkill` размером четыре байта, `wID: unsigned short`
//! по `+0` и `wLevel: unsigned short` по `+2`; `LoadSkillField` делит byte-size
//! `ListSkill` на четыре и добавляет элементы в исходном порядке.
//! `SaveScriptFlag` пишет в `VariableList` signed `m_lVariableNum: long`, затем
//! ровно `m_lVariableDataLength: long` непрозрачных байт из
//! `m_pVariableData`; PDB располагает эти поля по `+0x838`, `+0x840` и
//! `+0x83C`. Обратный `LoadScriptFlag` принимает только blob длиннее трёх байт,
//! читает первое слово как signed `long`, а остаток сохраняет без разбора.
//! `SaveStateField` аналогично не интерпретирует состояние: PDB задаёт
//! `CMoveShape::m_vExStates` как `vector<unsigned char>` по `+0x6C`, его
//! `_Myfirst/_Mylast` лежат по `+0x70/+0x74`, а `LoadStateField` передаёт весь
//! `ListState` владельцу `CMoveShape::SetExStates`.
//! Для `SaveFriendField` PDB задаёт `m_listFriend` по `CPlayer +0x82C` и
//! `tagFriend` размером `0x20`: `strName` по `+0`, `bOnline` по `+0x1C`.
//! `ListFriendName` содержит только list-order имена с NUL после каждого;
//! обратный load выставляет `bOnline=false`, поэтому runtime-флаг не является
//! частью DB blob.
//! PDB задаёт `m_setCiQingList` по `CPlayer +0x84C` как
//! `std::set<unsigned long>`; save обходит его в unsigned ascending-порядке и
//! пишет в lowercase-колонку `ciqing` по четыре little-endian байта на ID, без
//! count. Load делит длину blob на четыре и вставляет каждое слово обратно в
//! set; соседние `AddByteCiQing/DeByteCiQing` подтверждают тот же порядок ID,
//! но в общем player-stream отдельно добавляют count.
//! PDB задаёт `m_listThing` по `CPlayer +0x9A4` и `tagThing` размером восемь
//! байт: `wTID/wCnt/wMaxCnt/wPoint: unsigned short` по `+0/+2/+4/+6`.
//! `ListThing` содержит элементы в логическом deque-order без count; load делит
//! размер на восемь. Его отдельный empty-field путь с `GetDailyThingList`
//! зависит от load-параметра и не является частью save-кодека.
//! Все восемь load-кодеков теперь также `IMPLEMENTED`: integer records
//! читаются little-endian, а неполный хвост skill/ciqing/thing/quest намеренно
//! игнорируется, как исходное целочисленное деление длины. `HotKey` по-прежнему
//! требует ровно `0x60` байт, пустые state/script сохраняют прежнее состояние,
//! friends добавляются с `online=false`, а Thing empty/fyEnergy ветвь остаётся
//! у `CPlayer`, где доступен `CThingSetup`. Единственный unchecked `strlen`
//! без NUL до конца friend-blob заменён typed malformed-границей; чтение за
//! SAFEARRAY и use-after-unaccess не воспроизводятся.
//! Scalar ability-row теперь также достигнута как typed Tiberius-проекция:
//! exact `SELECT * ... WHERE id=@P1 ORDER BY id`, ADO-compatible integer/bit/
//! real/text conversions, nullable-only `dwLT60Stamp=0`, все реально читаемые
//! поля и семь load-only silence/honor значений. `ID`, `Account` и
//! `BaseMaxRp` намеренно не читаются: exact `LoadPlayer` сохраняет уже
//! назначенные identity/account, а `CPlayer::LoadData` сразу вычисляет RP.
//! DB/EOF остаются доказанным `false`, malformed range/blob — отдельной typed
//! границей. Неявное создание connection пока принадлежит полному wrapper-у.
//!
//! Текущий raw задаёт byte-exact имена и порядок всех 84 обращений
//! `Fields::Item`; PDB полностью закрывает source-типы `tagBaseProperty` и
//! прямых `CPlayer::m_strDepotPassword/m_btCountry/m_lContribute/
//! m_dwMurdererTime`. Четыре строки проходят старый ANSI C-string путь в
//! `BSTR`, координаты — `VT_R4`, а integer-поля сохраняют исходное различие
//! `VT_I4/VT_UI4/VT_UI2/VT_UI1/VT_INT`. Семь логических полей записывались как
//! `VT_BOOL`; `bIsCharged` отдельно расширялся до `VT_I4`, а `bQuest` — до
//! `VT_INT`. `dwExploit` и `dwKudos` являются PDB `DWORD`, но ADO получал их
//! как signed `VT_I4`; Rust сохраняет тот же 32-битный шаблон через `as i32`.
//! `PlayerAbilityScalarSnapshot` представляет только caller-owned read-view,
//! а фиксированный массив assignments задаёт все значения нового DB row.
//!
//! Исходные `CreateRs + OpenRs("CSL_PLAYER_ABILITY") + AddNew`, ordered
//! `PutCollect/AppendChunk` и единственный `Recordset::Update` заменены одним
//! параметризованным `INSERT` из тех же 84 scalar и семи binary колонок. ADO
//! до Update только накапливал значения в новой строке; TDS также не создаёт
//! частичную строку до единственного execute. `Query` передаёт BSTR-значения
//! как декодированный Windows-1251 Unicode, `VT_UI4` как неотрицательный
//! `i64`, `VT_UI2` как `i32`, `VT_UI1` как `u8`, `VT_BOOL` как `bit`, signed
//! значения и floats без изменения. SQL Server выполняет окончательное
//! приведение к типам доказанной исходной таблицы; число затронутых строк, как
//! и у старого Update, не проверяется.
//!
//! `DbLetTingUpdate` проверен по exact `0x00510520`: worker сначала снимает
//! текущий `GetDailyThingList`, кодирует каждый `tagThing` как восемь байт
//! `u16` в deque-order, после чего открывает отдельное соединение и проходит
//! все строки updatable recordset без транзакции. Kind `1` очищает только
//! младшие четыре бита `baseblfyenergy`; kind `2` дополнительно обнуляет
//! `LTUp60Cnt` и всё `baseblfyenergy`. В обоих случаях `ListThing`, signed
//! `dwLT60Stamp`, `basefyEnergy=0` и `wRemainJLDanCnt` обновляются до
//! единственного `Recordset::Update` текущей строки. TDS-замена добавляет
//! `ID` лишь как ключ текущей ADO-строки и сохраняет отдельный statement на
//! строку, исходный natural order, partial commit и отсутствие retry/rollback.
//! `ResetAllLeitingInDB` передаёт два DWORD в ровно один `_beginthreadex`;
//! Rust worker владеет payload и immutable setup через `Arc` и освобождает системный
//! thread resource по `JoinHandle::drop`, устраняя внутреннюю утечку HANDLE
//! без изменения fire-and-forget DB-семантики.
//!
//! После успешного INSERT исходный owner вызывал `CRsJJcSys::SaveJJcData`.
//! Точная сигнатура действительно передавала текущий connection по значению,
//! но JJc-owner немедленно освобождал локальную копию через `CreateCn` и
//! открывал новое соединение. Поэтому procedure не входит в caller-транзакцию:
//! её успех может сохраниться, даже если внешний `CreatePlayer` позднее
//! откатит base/ability/goods. Rust вызывает отдельный `RsJjcSysOwner` после
//! INSERT и при его `false` ставит второй outer notice
//! `Create Charactor Property Info ERROR.`. Ссылки исключают старые null player
//! и null connection; raw функции, catch и COM/compiler cleanup удалены.
//!
//! Старый helper создавал странный `SAFEARRAY(VT_UI4, cElements=0x60)`, но
//! помещал его в `VARIANT` с tag `VT_ARRAY|VT_UI1`; `AppendChunk` поэтому
//! сохранял первые 96 байт, где находятся ровно 24 little-endian `u32`, а не
//! 96 четырёхбайтных элементов. Rust кодирует наблюдаемый blob явно и передаёт
//! его синхронному typed sink. `SaveSkillField` создавал обычный
//! `SAFEARRAY(VT_UI1)` длиной `list.size() * 4`; exact
//! `0x00504C84..0x00504DCA` подтверждает, что проверка временного указателя
//! относится только к `operator delete`, после чего и пустой, и непустой blob
//! всегда передаётся в `ListSkill`. Sink обязан скопировать bytes до возврата;
//! `SaveScriptFlag` использует тот же контракт для `VariableList`. Exact
//! `0x00504414..0x0050445E` подтверждает `true` после `AppendChunk`, общий
//! cleanup временного буфера/SAFEARRAY и `false` из catch. Rust-срез выражает
//! только доказанный valid-state: неотрицательную длину и читаемый payload;
//! реакция старого кода на отрицательный `long` или невалидный указатель не
//! назначена и остаётся `BLOCKED_MISSING_FACT` у будущей границы материализации
//! `CPlayer`. `Result` заменяет COM exception/catch, а null player и recordset
//! исключены ссылками. Exact `0x00505C3C..0x00505DC9` подтверждает, что
//! `SaveStateField` синхронно копирует `[first,last)` и записывает также пустой
//! blob. `SaveFriendField` считает allocation по `string::size()`, но копирует
//! имя по C-string до первого NUL. Rust-тип допускает только доказанный обычный
//! путь без embedded NUL; содержимое оставшегося SAFEARRAY-хвоста для странного
//! имени остаётся `BLOCKED_MISSING_FACT`. Исходный catch ошибочно логировал
//! `save goods ERROR`; это наблюдаемое имя сохраняется для будущего logger-а.
//! Exact `0x00506B4A..0x00506C8D` исправляет потерянную raw-ветку: временный
//! массив удаляется при non-null, затем `ciqing` всегда получает chunk, включая
//! пустой set. `BTreeSet<u32>` сохраняет unique unsigned ordering без ручной
//! реализации дерева. Exact `0x005057E3..0x00505929` аналогично подтверждает
//! conditional delete временного Thing-буфера и обязательный `ListThing`
//! chunk, включая пустой deque. Все семь binary fields и общий row-owner
//! закрыты.
//!
//! `SavePlayerAbilities(CPlayer*, connection)` обновляет существующую строку
//! `CSL_PLAYER_ABILITY`. До единственного `Recordset::Update` он присваивает
//! 92 scalar-поля и вызывает те же семь binary helper-ов. Базовые 84 значения
//! переиспользуют доказанный create-snapshot, но save-порядок отдельно вставляет
//! `SaveTime`, signed `silence`, четыре honor-eliminate DWORD и два honor ID;
//! кроме того, `BattleFairyEnabled` стоит перед `Mode`, а не на create-позиции.
//! Точный PDB подтверждает `m_lSilienceTime: long` по `CPlayer+0x858` и шесть
//! `unsigned long` honor-полей по `tagBaseProperty+0x140..+0x154`.
//!
//! Exact `0x00509B9B..0x00509BE1` имеет статус `VERIFIED_DISASSEMBLY` для
//! потерянных аргументов времени: один `GetLocalTime` форматируется как
//! `year-month-day hour:minute:second` без ведущих нулей. Хвост
//! `0x0050D1C5..0x0050D2BE` подтверждает отдельный `SaveJJcData` только после
//! успешного row-update, `AL=1` после его успеха и `AL=0` из catch
//! `0x0050D223`; после этих ответов reverse прекращён.
//!
//! Параметризованный `IF EXISTS + UPDATE TOP (1) + @@ROWCOUNT` заменяет только
//! updateable ADO-recordset и сохраняет `false` при отсутствии текущей строки.
//! Scalar `VARIANT`-формы, Windows-1251 C-string и семь byte-exact blob
//! остаются теми же, что у create-owner-а. Один local wall-clock снимается до
//! DB-update. JJc-owner затем по собственному доказанному контракту открывает
//! отдельное соединение, поэтому его эффект не входит в caller-транзакцию.
//! Null player тихо возвращает `false`, null connection и общий catch дают
//! typed save-notice; raw owner, catch и compiler cleanup удалены.
//!
//! `SaveQuestData(CPlayer*, connection)` сериализует mapped values из
//! упорядоченной `m_PlayerQuests` в `QuestData`: для каждого элемента в
//! unsigned key-order идут little-endian `tagPlayerQuest::wQuestID` и один
//! byte `byComplete`, без count. Сам map-key в blob не копируется. Точный PDB
//! задаёт layout mapped value как
//! `unsigned short` по `+0` и `unsigned char` по `+2`, а map располагает по
//! `CPlayer+0x8F0`; обратный `LoadQuestData` делит длину поля на три и передаёт
//! каждую пару в `CPlayer::AddQuestFromDB`.
//!
//! Exact `0x005062C5..0x005063CC` имеет статус `VERIFIED_DISASSEMBLY`: размер
//! равен `map.size() * 3`, обход читает mapped value, а освобождение
//! временного heap-buffer при non-null не является ранним return. Диапазон
//! `0x005063CC..0x00506962` подтверждает signed inherited player ID по `+0x8`,
//! update только `QuestData` существующей первой строки и создание новой пары
//! `PlayerID/QuestData` при EOF. Normal tail `0x005069AD` возвращает `true`,
//! catch `0x005069B4..0x00506A35` и null player — `false`; после этих ответов
//! reverse прекращён.
//!
//! Параметризованный `IF EXISTS + UPDATE TOP (1) ELSE INSERT` заменяет только
//! ADO updateable recordset, SAFEARRAY и `AppendChunk`; binary bytes, первая
//! строка и caller-транзакция сохраняются. `BTreeMap<u16, _>` заменяет MSVC
//! tree и ограничивает число уникальных key значениями исходного
//! `unsigned short`, поэтому старое `size * 3` не переполняется. Null player
//! остаётся тихим `false`, DB/null-connection ошибка — typed notice. Исходный catch перед
//! основным error-log писал длинную строку через `__snprintf(..., 4, ...)` и
//! мог оставить четыре байта без NUL; точные прочитанные logger-ом bytes
//! остаются локальным `BLOCKED_MISSING_FACT`, но доказанный `false` и DB-эффект
//! не меняются. Raw owner, catch и compiler cleanup удалены.
//!
//! PDB задаёт `SavePlayer` как public `bool`-метод с параметрами
//! `CPlayer *pPlayer` и connection `cn`, RVA `0x0010EE70`, длина `0x17C`.
//! Согласованный raw из `rsplayer.cpp:1627` полностью задаёт wrapper: null
//! player проверяется раньше null connection, затем на одном underlying
//! caller-connection строго выполняются `SavePlayerBase`,
//! `SavePlayerAbilities`, `SaveQuestData` и `CDBGoods::SaveGoodsFiled`.
//! Первый доказанный `false` немедленно завершает цепочку; только успех goods-
//! стадии возвращает `true`. Неоднозначности для машинного кода не осталось,
//! поэтому дополнительный reverse не выполнялся.
//!
//! Wrapper не начинает, не фиксирует и не откатывает транзакцию, не имеет
//! общего catch и не пишет собственного лога. Уже выполненные base/ability/
//! quest DB-эффекты остаются в caller-транзакции до решения внешнего owner-а;
//! отдельный JJc-вызов второй стадии может сохраниться и после более позднего
//! отказа. Передача COM connection по значению заменена последовательными
//! mutable reborrow-ами. Полный snapshot сохраняет обязанность получить все
//! четыре проекции из одного `CPlayer`, не добавляя отсутствующих в оригинале
//! сравнений ID. Вложенный goods `BLOCKED_MISSING_FACT` передаётся наружу, а не
//! превращается в выбранный Rust-результат. Raw wrapper и COM/compiler cleanup
//! удалены.
//!
//! PDB задаёт для `CreatePlayer`, `CreatePlayerBase` и
//! `CreatePlayerAbilities` один member-function type `0x2F0B`: bool и два
//! одинаковых параметра `CPlayer*`/connection smart pointer. Согласованный raw
//! `CreatePlayer` сначала проверяет только connection, затем на одном и том же
//! underlying caller-connection строго вызывает base, abilities и
//! `CDBGoods::SaveGoodsFiled`; первый `false` немедленно завершает цепочку.
//! AddRef/Release каждой переданной по значению COM-копии заменены
//! последовательными mutable reborrow-ами, а не отдельными соединениями.
//!
//! Null player отдельно не проверяется внешним owner-ом: он доходит до
//! `CreatePlayerBase`, где возвращает `false` до DB-эффекта. Rust `Option`
//! сохраняет этот API-путь без raw pointer. Null connection возвращает `false`
//! раньше player и получает единственный внешний `MissingConnection` notice.
//! Никакого общего catch или дополнительного лога после bool-отказа трёх
//! стадий у `CreatePlayer` нет; каждый вложенный owner сохраняет собственные
//! notices.
//!
//! JJc procedure остаётся побочным эффектом второй стадии на отдельном
//! соединении и может пережить последующий goods-отказ. Ещё важнее,
//! `SaveGoodsFiled` доказанно возвращает `true` после traversal, даже если
//! отдельный listener/`SaveGoods` вернул `false`; внешний `CreatePlayer` также
//! считает такую goods-стадию успешной. Rust не «исправляет» эту ветку.
//! Rust не переносит переполнение base SQL stack-buffer; вложенная безопасная
//! граница goods listener-а передаётся наружу отдельно. Полный snapshot сохраняет обязанность
//! получить base/ability/goods ID из одного `CPlayer`, не объявляя его Rust-
//! layout завершённым.
//!
//! `SavePlayerBase(CPlayer*, connection)` обновляет уже существующую строку
//! `CSL_PLAYER_BASE`. После null-проверок он открывает updateable recordset по
//! signed player ID и до единственного `Recordset::Update` присваивает ровно
//! 29 полей: `Name`, пять byte-полей base-состояния, одиннадцать equipment ID,
//! одиннадцать signed `GAP_WEAPON_LEVEL` и `Region`. Equipment идёт буквально
//! в порядке `HELM, BODY, GLOV, BOOT, WEAPON, BACK, HEADGEAR, FROCK, WING,
//! MANTEAU, FAIRY`; отсутствующий goods даёт ноль и для ID, и для уровня.
//!
//! Exact `0x00503A92..0x00503B45` имеет статус `VERIFIED_DISASSEMBLY`: после
//! успешных Update/Close/Release нормальный эпилог выставляет `AL=1`, а catch
//! `0x00503AA9` после исходных `PutLogInfo`/`PrintErr` сходится к `AL=0`.
//! Null player возвращает тихий `false`; null connection печатал отдельную
//! ошибку и тоже возвращал `false`. После этих ответов reverse прекращён.
//!
//! Параметризованный `UPDATE TOP (1)` заменяет только updateable ADO-recordset.
//! Предварительный `IF EXISTS` и возвращаемый `@@ROWCOUNT` сохраняют ошибку
//! исходного PutCollect/Update при отсутствии текущей строки вместо ложного
//! успеха обычного SQL UPDATE с нулём затронутых строк. ANSI C-string имени
//! декодируется как Windows-1251; `VT_UI4` equipment ID передаются как
//! неотрицательные `i64`, byte/signed поля — без изменения, а SQL Server
//! выполняет окончательное приведение к исходной схеме. Caller-транзакция не
//! начинается и не завершается внутри owner-а. Rust-срез, `Option`, Tiberius и
//! Drop заменяют только C++ string layout, null pointer, ADO/COM и cleanup;
//! сырой псевдокод и compiler continuation удалены.
//!
//! PDB задаёт `RestorePlayer(unsigned int, connection) -> bool`. Raw показывает
//! один batch `UPDATE csl_player_base SET DelDate = NULL WHERE ID=%d` в
//! `char[128]`. Exact `0x00501260..0x0050134A` возвращает потерянный vararg и
//! `AL`: в `_sprintf` передаётся тот же `u32` ID, который `%d` интерпретирует
//! как signed 32-битный шаблон; успешный `ExecuteCn` даёт `AL=1`, null
//! connection и catch — `AL=0`. Число затронутых строк не проверяется.
//!
//! Даже крайнее signed представление ID оставляет batch вместе с NUL короче
//! 128 байт, поэтому отдельной overflow-границы здесь нет. Параметризованный
//! TDS `UPDATE` передаёт `player_id as i32`, сохраняя тот же bit-pattern и
//! меняя только `_sprintf`/ADO transport. Метод использует уже активную caller-
//! транзакцию, не начинает и не завершает её; DB-ошибка ставит typed
//! `Restore` notice вместо исходного `Restore Charactor ERROR.`.
//!
//! PDB задаёт `DeletePlayer(unsigned long, long, connection) -> bool`, а raw —
//! один batch `UPDATE csl_player_base SET DelDate = '%d-%d-%d' WHERE ID=%d`.
//! Exact `0x00501A60..0x00501B6B` имеет статус `VERIFIED_DISASSEMBLY`: функция
//! сначала отвергает null connection, затем передаёт signed `long` в
//! `_localtime`, форматирует только local year/month/day и исходный `u32` как
//! signed `%d`. Успешный `ExecuteCn` явно ставит `AL=1`; его `false` бросает
//! `E_FAIL`, а catch и null connection сходятся к `AL=0`. Affected rows не
//! проверяются.
//!
//! Встроенная CRT `_localtime` по `0x0051CA5F` возвращает null для
//! отрицательного timestamp, но owner по `0x00501AB8` сразу разыменовывает
//! результат. Достижимость такого `tDelDate` не доказана; безопасный Rust не
//! назначает старому access violation результат и возвращает локальный
//! `BLOCKED_MISSING_FACT` с конкретным timestamp. Для неотрицательного пути
//! `chrono::Local` заменяет CRT и обязан видеть ту же server timezone, иначе
//! дата около полуночи наблюдаемо изменится. Параметризованный TDS сохраняет
//! исходную строку `year-month-day` и bit-pattern ID; даже максимальная дата и
//! signed ID вместе с NUL помещаются в старый `char[128]`.
//!
//! `InsertHonorRanks(connection)` читает caller-снимок
//! `CHonorRanks::m_stDBData.tCopyTime`, обеспечивает строку
//! `CSL_HonorRanks` для этой календарной даты, затем вызывает
//! `tagTime::AddDay(1)` и обеспечивает строку следующей даты. PDB задаёт
//! `tagTime` размером `0x10` как восемь последовательных `unsigned short` и
//! `CHonorRanks::tagDBData` размером `0x190`, где `tCopyTime` лежит по `+0`;
//! `InsertHonorRanks` наблюдает только `wYear`, `wMonth` и `wDay`, поэтому
//! из полной caller-копии использует лишь валидную календарную проекцию этих
//! трёх полей. Остальные пять полей сохраняет generator-owner, а полный layout
//! honor-list принадлежит следующему реализованному `SaveHonorRanksByType`.
//!
//! Exact `0x00501680..0x005019F4` имеет статус `VERIFIED_DISASSEMBLY`: все
//! четыре `_snprintf` получают `wYear, wMonth, wDay` и сохраняют исходное
//! форматирование без ведущих нулей; каждый `SELECT *` при EOF вызывает один
//! соответствующий `INSERT`, а существующая строка не изменяется. После
//! успешной второй проверки выставляется `AL=1`; null connection и catch
//! `Create HonorRanks abort` сходятся к `AL=0`. После этих ответов reverse
//! прекращён.
//!
//! Tiberius `simple_query` сохраняет исходные literal SQL и серверное
//! преобразование строки `year-month-day` в тип `SortDate`; значения происходят
//! только из трёх `u16`, поэтому SQL injection невозможен, а оба старых
//! `char[512]` гарантированно не переполняются. `SELECT` полностью потребляется
//! до возможного `INSERT`, Rust `Drop` заменяет Recordset/COM cleanup. Метод
//! работает внутри caller-транзакции и сам не делает begin/commit/rollback;
//! null connection остаётся тихим `false`, DB-ошибка создаёт один typed
//! `HonorRanksInsert` notice.
//!
//! `SaveHonorRanksByType(history, type, recordset)` выбирает один из четырёх
//! rank-type `0..=3`, последовательно сериализует его четыре country-list и
//! очищает каждую выбранную list до попытки записать field. Blob начинается
//! четырьмя отдельными `DWORD count`; после каждого count непосредственно идут
//! элементы соответствующей страны. PDB задаёт `tagHorRank` размером `0x24`:
//! `long nPlayerID`, `unsigned char bLevel`, `char name[20]`, `unsigned char
//! wOccupationID`, два layout-padding byte, `unsigned long dwAppellationID` и
//! `unsigned long dwElimilateNum`. Exact-код копирует каждый элемент девятью
//! `movsd`, поэтому имя вместе с полным хвостом и padding входят в наблюдаемый
//! DB blob и представлены Rust-типом явно, а не нормализуются.
//!
//! Поля выбираются буквально: `DayHonnorRank`, `WeekHonnorRank`,
//! `MonthHonnorRank`, `TotalHonnorRank`; исходная опечатка `Honnor` сохранена.
//! Exact `0x00505080..0x005056E5` имеет статус `VERIFIED_DISASSEMBLY` и
//! исправляет два ложных raw-return: очистка непустого list продолжает все
//! четыре country, а освобождение временного byte-buffer продолжает запись
//! field. Успех выставляет `AL=1`; null recordset, type вне `0..=3` и catch
//! после уничтожения SAFEARRAY сходятся к `AL=0`. После этих ответов reverse
//! прекращён.
//!
//! Rust enum и живая sink-ссылка исключают два ранних invalid-input пути у
//! единственных project-call-site внутри `SaveHonorRanks`; typed outcome
//! сохраняет field-error и отдельно локализует 32-битное переполнение старого
//! `count * 0x24 + 0x10`. Четыре `Vec` заменяют только list storage. Они
//! очищаются до вызова sink, поэтому ошибка будущего TDS update не восстанавливает
//! уже дренированный snapshot. SAFEARRAY/VARIANT, ручной byte-buffer и
//! compiler cleanup удалены как заменённый технический механизм.
//!
//! `SaveHonorRanks(connection)` сначала открывает строку даты
//! `m_stDBData.tCopyTime`, дренирует в неё четыре `History` type и выполняет
//! один `Recordset::Update`. Только после этого `tagTime::AddDay(1)` выбирает
//! строку следующего дня, куда тем же порядком дренируются четыре `Current`
//! type и выполняется второй Update. Отсутствие любой строки возвращает
//! `false`: создавать их обязан предшествующий `InsertHonorRanks` той же
//! внешней транзакции.
//!
//! Exact `0x00505E20..0x0050624A` имеет статус `VERIFIED_DISASSEMBLY` только
//! для утраченных raw-границ: первый call передаёт `history=1`, второй — `0`;
//! обе EOF-ветви сходятся к `AL=0`, обычный успех выставляет `AL=1` лишь после
//! второго Update, а catch `Save HonorRanks Error` также заканчивается
//! `AL=0`. Оба `_snprintf` получают `wYear, wMonth, wDay`; после этих ответов
//! reverse прекращён.
//!
//! Tiberius `SELECT *` сохраняет literal date lookup, а один параметризованный
//! `UPDATE TOP (1)` на дату заменяет четыре field-assignment и единый
//! Recordset Update. In-memory sink только удерживает уже сформированные blob
//! до этого вызова; при DB-ошибке все четыре списка данного периода уже
//! очищены, как в исходнике. Следующий период не начинается после обычного
//! отказа либо локального `BLOCKED_MISSING_FACT` размера.
//!
//! `LoadHonorRanks` снимает local SYSTEMTIME, выбирает history-строку текущей
//! даты и только после подтверждённого non-EOF очищает все history-list. Затем
//! четыре поля загружаются по порядку day/week/month/total. После успешного
//! history прохода `tagTime::AddDay(1)` выбирает current-строку следующего дня;
//! её отсутствие возвращает `false`, сохраняя уже заменённый history и прежний
//! current. Current очищается только после найденной второй строки. Старый
//! Linux C++ staging/swap делал загрузку атомарной и тем самым менял этот
//! порядок; Rust его не переносит.
//! Исходная null-ветвь создавала ADO connection внутри функции. Tiberius-owner
//! не владеет DB settings/runtime, поэтому внешний init-адаптер передаёт ему
//! уже открытый connection; его отсутствие является достигнутым техническим
//! `false`, а не поводом встраивать второй неявный connection-owner.
//!
//! Каждый непустой field состоит из четырёх последовательных секций
//! `[u32 count][count * tagHorRank(0x24)]`; пустой/NULL field означает четыре
//! пустых списка. Exact decoder не сверял `ActualSize` с count и отпускал
//! SAFEARRAY до чтения сохранённого pointer-а. Эти внутренние lifetime/OOB
//! дефекты заменены bounds-checked slice decoder-ом: корректные bytes и
//! insertion order остаются прежними, malformed blob возвращает typed block.
//! Лишний хвост после четвёртой секции намеренно игнорируется, как exact owner.
//!
//! `StatRanks` форматирует exact `SELECT TOP m_nMaxNum`, читает строки в DB-
//! порядке и сразу вызывает live `CPlayerRanks::AddRank`. Поэтому поздняя
//! ошибка сохраняет уже добавленный prefix; staging/swap Linux-донора не
//! переносится. ADO/BSTR заменены Tiberius и Windows-1251. Успех `AL=1` и
//! catch/null `AL=0` подтверждены machine-кодом `0x00509A6E/0x00509AD6`.
//! Неинициализированный constructor-ом `m_nMaxNum` и null faction внутри
//! `IsFreePlayer` остаются локальными typed-границами вместо чтения мусора или
//! raw null-dereference.
//!
//! `GetCDKey` сначала выполняет точный логический `GetPlayerID(name)`, затем
//! читает `Account` из `CSL_Player_base` по найденному signed ID. Отсутствующая
//! строка, пустой account и DB-ошибка дают пустую строку; ошибки двух стадий
//! сохраняются раздельными notice-ами, как исходные `get palyer id ERROR` и
//! `get cdkey ERROR`. Параметризованный `tiberius::Query` заменяет только
//! `_sprintf`/ADO и исключает старый SQL-injection/buffer-overflow дефект;
//! Windows-1251 сохраняет ANSI C-string границу имени и результата.
//!
//! Полный caller-connection `LoadPlayer` сохраняет исходную короткую цепочку:
//! одна ability-row со всеми scalar/binary post-load правилами, отдельный
//! `LoadQuestData`, затем `CDBGoods::LoadGoods` и `CRsJJcSys::LoadJJcData`.
//! Каждый следующий owner вызывается только после `true` предыдущего. Exact
//! `0x00514D0E/0x00514D39` подтверждают порядок Goods/JJC, `0x00514E56` —
//! normal `AL=1`, общий failure-tail `0x00514D9C` — `AL=0`. Таймер и запись
//! `TemptLoadDataLog` являются технической диагностикой и не входят в игровой
//! контракт. `TiberiusPlayerLoadData` связывает этот owner с готовым
//! `CPlayer::LoadData`, а `WorldPlayerLoadDataAdapter` передаёт его bool-итог
//! точному `LoadPlayerDataFromDB` worker-у, не пряча registry/config в mutable
//! singleton.
//! При null connection exact owner создаёт одно отдельное World DB connection
//! до ability-query и освобождает его после JJC либо любого раннего failure;
//! `WorldDatabaseSettings::connect` и Rust `Drop` заменяют только ADO plumbing.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::mem::{offset_of, size_of};

use chrono::{Datelike, Days, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike};
use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::{Query, Row};

use crate::dbaccess::worlddb::dbgoods::{
    DbGoodsOwner, GoodsFiledSaveOutcome, GoodsLoadBlock, GoodsLoadFailure, GoodsLoadOutcome,
    PlayerGoodsFiledSnapshot,
};
use crate::dbaccess::worlddb::goodslistener::GoodsTraversalBlock;
use crate::dbaccess::worlddb::rsjjcsys::{
    PlayerJjcDataSnapshot, PlayerJjcLoadFailure, PlayerJjcLoadOutcome, RsJjcSysOwner,
};
use crate::dbaccess::worlddb::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};
use crate::public::date::{TagTime, TagTimeArithmeticBlock};
use crate::setup::leitingsetup::{CThingSetup, LeiTingDailyThing};
use crate::worldserver::appworld::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use crate::worldserver::appworld::player::{CPlayer, PlayerLoadDataOwner};
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::worldserver::playerranks::{CPlayerRanks, PlayerRankAddBlock};

const CREATE_PLAYER_BASE_PREFIX: &[u8] = b"INSERT INTO CSL_PLAYER_BASE (id,name,Account,levels,occupation,sex,Country,HEAD,\t\t\t\t\t HELM,BODY,GLOV,BOOT,WEAPON,BACK,\t\t\t\t\t HEADGEAR,FROCK,WING,MANTEAU,FAIRY,\t\t\t\t\t HelmLevel,BodyLevel,GlovLevel,BootLevel,WeaponLevel,BackLevel,\t\t\t\t\t HEADGEARLevel,FROCKLevel,WINGLevel,MANTEAULevel,FAIRYLevel,\t\t\t\t\t Region) \t\t\t\t VALUES (";
const SAVE_PLAYER_BASE_SQL: &str = "IF EXISTS (SELECT TOP 1 id FROM CSL_PLAYER_BASE WHERE id = @P30) BEGIN UPDATE TOP (1) CSL_PLAYER_BASE SET [Name] = @P1, [Levels] = @P2, [Occupation] = @P3, [Sex] = @P4, [Country] = @P5, [HEAD] = @P6, [HELM] = @P7, [BODY] = @P8, [GLOV] = @P9, [BOOT] = @P10, [WEAPON] = @P11, [BACK] = @P12, [HEADGEAR] = @P13, [FROCK] = @P14, [WING] = @P15, [MANTEAU] = @P16, [FAIRY] = @P17, [HelmLevel] = @P18, [BodyLevel] = @P19, [GlovLevel] = @P20, [BootLevel] = @P21, [WeaponLevel] = @P22, [BackLevel] = @P23, [HEADGEARLevel] = @P24, [FROCKLevel] = @P25, [WINGLevel] = @P26, [MANTEAULevel] = @P27, [FAIRYLevel] = @P28, [Region] = @P29 WHERE id = @P30; SELECT CAST(@@ROWCOUNT AS int) AS UpdatedRows END ELSE SELECT CAST(0 AS int) AS UpdatedRows";
const VALUE_GROUP_BREAK: &[u8] = b",\t\t\t\t\t ";
const LEI_TING_RESET_SELECT_SQL: &str = "SELECT ID, LTUp60Cnt, basefyEnergy, baseblfyenergy, dwLT60Stamp, wRemainJLDanCnt, ListThing FROM csl_player_ability";
const LEI_TING_DAILY_RESET_SQL: &str = "UPDATE CSL_PLAYER_ABILITY SET ListThing=@P1,dwLT60Stamp=@P2,basefyEnergy=@P3,wRemainJLDanCnt=@P4,baseblfyenergy=@P5 WHERE ID=@P6";
const LEI_TING_MONTHLY_RESET_SQL: &str = "UPDATE CSL_PLAYER_ABILITY SET ListThing=@P1,dwLT60Stamp=@P2,basefyEnergy=@P3,wRemainJLDanCnt=@P4,LTUp60Cnt=@P5,baseblfyenergy=@P6 WHERE ID=@P7";

const HONOR_RANKS_SELECT_PREFIX: &str = "select * from CSL_HonorRanks where SortDate = '";
const HONOR_RANKS_INSERT_PREFIX: &str = "insert into CSL_HonorRanks(SortDate) values('";
const HONOR_RANKS_UPDATE_PREFIX: &str = "UPDATE TOP (1) CSL_HonorRanks SET ";
const HONOR_RANK_CATEGORY_COUNT: usize = 4;
const HONOR_RANK_TYPE_COUNT: usize = 4;
const HONOR_RANK_ENTRY_SIZE: usize = 0x24;
const HONOR_RANK_BLOB_HEADER_SIZE: usize = HONOR_RANK_CATEGORY_COUNT * size_of::<u32>();

/// Полный byte-наблюдаемый layout одного исходного `tagHorRank`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct HonorRankDbEntry {
    pub(crate) player_id: i32,
    pub(crate) level: u8,
    pub(crate) name: [u8; 20],
    pub(crate) occupation_id: u8,
    /// Два байта между `wOccupationID` и первым DWORD также попадали в blob.
    pub(crate) legacy_padding: [u8; 2],
    pub(crate) appellation_id: u32,
    pub(crate) eliminate_num: u32,
}

const _: [(); HONOR_RANK_ENTRY_SIZE] = [(); size_of::<HonorRankDbEntry>()];
const _: [(); 0x00] = [(); offset_of!(HonorRankDbEntry, player_id)];
const _: [(); 0x04] = [(); offset_of!(HonorRankDbEntry, level)];
const _: [(); 0x05] = [(); offset_of!(HonorRankDbEntry, name)];
const _: [(); 0x19] = [(); offset_of!(HonorRankDbEntry, occupation_id)];
const _: [(); 0x1a] = [(); offset_of!(HonorRankDbEntry, legacy_padding)];
const _: [(); 0x1c] = [(); offset_of!(HonorRankDbEntry, appellation_id)];
const _: [(); 0x20] = [(); offset_of!(HonorRankDbEntry, eliminate_num)];

impl HonorRankDbEntry {
    fn legacy_bytes(self) -> [u8; HONOR_RANK_ENTRY_SIZE] {
        let mut bytes = [0; HONOR_RANK_ENTRY_SIZE];
        bytes[0x00..0x04].copy_from_slice(&self.player_id.to_le_bytes());
        bytes[0x04] = self.level;
        bytes[0x05..0x19].copy_from_slice(&self.name);
        bytes[0x19] = self.occupation_id;
        bytes[0x1a..0x1c].copy_from_slice(&self.legacy_padding);
        bytes[0x1c..0x20].copy_from_slice(&self.appellation_id.to_le_bytes());
        bytes[0x20..0x24].copy_from_slice(&self.eliminate_num.to_le_bytes());
        bytes
    }

    fn from_legacy_bytes(bytes: &[u8; HONOR_RANK_ENTRY_SIZE]) -> Self {
        Self {
            player_id: i32::from_le_bytes(bytes[0x00..0x04].try_into().expect("fixed entry")),
            level: bytes[0x04],
            name: bytes[0x05..0x19].try_into().expect("fixed entry"),
            occupation_id: bytes[0x19],
            legacy_padding: bytes[0x1a..0x1c].try_into().expect("fixed entry"),
            appellation_id: u32::from_le_bytes(
                bytes[0x1c..0x20].try_into().expect("fixed entry"),
            ),
            eliminate_num: u32::from_le_bytes(
                bytes[0x20..0x24].try_into().expect("fixed entry"),
            ),
        }
    }
}

pub(crate) type HonorRankDbLists =
    [[Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT]; HONOR_RANK_TYPE_COUNT];

/// Полная caller-owned копия `CHonorRanks::tagDBData::tCopyTime`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HonorRanksCopyTimeSnapshot {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

/// Caller-owned копия `CHonorRanks::m_stDBData`, готовая к DB-дренированию.
pub(crate) struct HonorRanksDbDataSnapshot {
    copy_time: HonorRanksCopyTimeSnapshot,
    history: HonorRankDbLists,
    current: HonorRankDbLists,
}

impl HonorRanksDbDataSnapshot {
    /// Принимает уже скопированные `GenerateSaveData` списки без перестановки.
    pub(crate) fn from_legacy_copy(
        copy_time: HonorRanksCopyTimeSnapshot,
        history: HonorRankDbLists,
        current: HonorRankDbLists,
    ) -> Self {
        Self {
            copy_time,
            history,
            current,
        }
    }

    pub(crate) const fn copy_time(&self) -> HonorRanksCopyTimeSnapshot {
        self.copy_time
    }

    fn lists_mut(
        &mut self,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
    ) -> &mut [Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT] {
        let type_index = rank_type as usize;
        match period {
            HonorRanksSavePeriod::History => &mut self.history[type_index],
            HonorRanksSavePeriod::Current => &mut self.current[type_index],
        }
    }
}

/// Исходный bool-параметр выбора массива `m_stDBData`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HonorRanksSavePeriod {
    Current,
    History,
}

/// Owner-граница последовательной публикации загруженных DB-полей.
pub(crate) trait HonorRanksLoadSink {
    fn clear_honor_ranks_period(&mut self, period: HonorRanksSavePeriod);
    fn replace_honor_ranks_type(
        &mut self,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
        lists: [Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT],
    );
}

/// Malformed-граница старого unchecked blob traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HonorRanksBlobDecodeBlock {
    pub(crate) rank_type: HonorRanksType,
    pub(crate) country: u8,
    pub(crate) offset: usize,
    pub(crate) required_bytes: usize,
    pub(crate) available_bytes: usize,
}

/// Стадия, на которой внешний loader вернул исходный `false`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HonorRanksLoadFailure {
    MissingConnection,
    Database { period: HonorRanksSavePeriod },
    MissingRow { period: HonorRanksSavePeriod },
}

/// Уже достигнутая malformed-граница после прежних последовательных эффектов.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HonorRanksLoadBlock {
    pub(crate) period: HonorRanksSavePeriod,
    pub(crate) source: HonorRanksBlobDecodeBlock,
}

/// Доказанный итог полного `CRsPlayer::LoadHonorRanks`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HonorRanksLoadOutcome {
    ReturnedTrue,
    ReturnedFalse(HonorRanksLoadFailure),
    BlockedMissingFact(HonorRanksLoadBlock),
}

/// Допустимые значения исходного `int type` и связанные DB-поля.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(crate) enum HonorRanksType {
    Day = 0,
    Week = 1,
    Month = 2,
    Total = 3,
}

impl HonorRanksType {
    const ALL: [Self; HONOR_RANK_TYPE_COUNT] = [Self::Day, Self::Week, Self::Month, Self::Total];

    pub(crate) const fn column_name(self) -> &'static str {
        match self {
            Self::Day => "DayHonnorRank",
            Self::Week => "WeekHonnorRank",
            Self::Month => "MonthHonnorRank",
            Self::Total => "TotalHonnorRank",
        }
    }
}

/// Декодирует один exact DB-field без старого SAFEARRAY lifetime/OOB дефекта.
pub(crate) fn decode_honor_ranks_blob(
    rank_type: HonorRanksType,
    blob: Option<&[u8]>,
) -> Result<[Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT], HonorRanksBlobDecodeBlock> {
    let mut lists: [Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT] =
        std::array::from_fn(|_| Vec::new());
    let Some(blob) = blob.filter(|blob| !blob.is_empty()) else {
        return Ok(lists);
    };

    let mut cursor = 0usize;
    for (country, list) in lists.iter_mut().enumerate() {
        let available_bytes = blob.len().saturating_sub(cursor);
        let Some(count_bytes) = blob.get(cursor..cursor + size_of::<u32>()) else {
            return Err(HonorRanksBlobDecodeBlock {
                rank_type,
                country: country as u8,
                offset: cursor,
                required_bytes: size_of::<u32>(),
                available_bytes,
            });
        };
        let count = u32::from_le_bytes(count_bytes.try_into().expect("проверены четыре байта"));
        cursor += size_of::<u32>();

        let required_bytes = usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(HONOR_RANK_ENTRY_SIZE))
            .unwrap_or(usize::MAX);
        let available_bytes = blob.len().saturating_sub(cursor);
        let Some(entries) = cursor
            .checked_add(required_bytes)
            .and_then(|end| blob.get(cursor..end))
        else {
            return Err(HonorRanksBlobDecodeBlock {
                rank_type,
                country: country as u8,
                offset: cursor,
                required_bytes,
                available_bytes,
            });
        };
        list.reserve_exact(count as usize);
        for entry in entries.chunks_exact(HONOR_RANK_ENTRY_SIZE) {
            let entry: &[u8; HONOR_RANK_ENTRY_SIZE] =
                entry.try_into().expect("chunk имеет exact tagHorRank size");
            list.push(HonorRankDbEntry::from_legacy_bytes(entry));
        }
        cursor += required_bytes;
    }

    Ok(lists)
}

impl fmt::Display for HonorRanksBlobDecodeBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "honor blob {:?}, страна {}, offset {}: требуется {} байт, доступно {}",
            self.rank_type,
            self.country,
            self.offset,
            self.required_bytes,
            self.available_bytes
        )
    }
}

impl Error for HonorRanksBlobDecodeBlock {}

/// Typed-замена записи SAFEARRAY в одно поле updateable recordset.
pub(crate) trait HonorRanksFieldSink {
    type Error;

    fn put_honor_ranks_field(
        &mut self,
        rank_type: HonorRanksType,
        blob: Vec<u8>,
    ) -> Result<(), Self::Error>;
}

/// Неразрешённая 32-битная граница старого размера SAFEARRAY.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HonorRanksBlobBlock {
    pub(crate) total_entries: usize,
}

/// Доказанный итог `SaveHonorRanksByType` без invalid enum/null sink.
#[derive(Debug)]
pub(crate) enum HonorRanksByTypeSaveOutcome<E> {
    Saved,
    FieldFailed(E),
    BlockedMissingFact(HonorRanksBlobBlock),
}

/// Точная позиция неразрешённого размера внутри внешнего save-прохода.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HonorRanksSaveBlock {
    pub(crate) period: HonorRanksSavePeriod,
    pub(crate) rank_type: HonorRanksType,
    pub(crate) blob: HonorRanksBlobBlock,
}

/// Доказанный bool внешнего `SaveHonorRanks` либо локальная size-граница.
#[derive(Debug)]
pub(crate) enum HonorRanksSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(HonorRanksSaveBlock),
}

#[derive(Default)]
struct CollectedHonorRanksFields {
    fields: Vec<(HonorRanksType, Vec<u8>)>,
}

impl HonorRanksFieldSink for CollectedHonorRanksFields {
    type Error = Infallible;

    fn put_honor_ranks_field(
        &mut self,
        rank_type: HonorRanksType,
        blob: Vec<u8>,
    ) -> Result<(), Self::Error> {
        self.fields.push((rank_type, blob));
        Ok(())
    }
}

impl HonorRanksCopyTimeSnapshot {
    /// Принимает полный `SYSTEMTIME`, проверяя только календарную проекцию.
    pub(crate) fn from_legacy_fields(fields: [u16; 8]) -> Option<Self> {
        let [
            year,
            month,
            day_of_week,
            day,
            hour,
            minute,
            second,
            milliseconds,
        ] = fields;
        NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))?;
        Some(Self {
            year,
            month,
            day_of_week,
            day,
            hour,
            minute,
            second,
            milliseconds,
        })
    }

    fn next_day(self) -> Self {
        if self.year == u16::MAX && self.month == 12 && self.day == 31 {
            return Self {
                year: 0,
                month: 1,
                day_of_week: self.day_of_week.wrapping_add(1) % 7,
                day: 1,
                ..self
            };
        }

        let date = NaiveDate::from_ymd_opt(
            i32::from(self.year),
            u32::from(self.month),
            u32::from(self.day),
        )
        .expect("HonorRanksCopyTimeSnapshot сохраняет валидную дату");
        let next = date
            .checked_add_days(Days::new(1))
            .expect("диапазон u16 года помещается в chrono::NaiveDate");
        Self {
            year: u16::try_from(next.year()).expect("переполнение u16 обработано отдельной веткой"),
            month: u16::try_from(next.month()).expect("месяц помещается в u16"),
            day_of_week: self.day_of_week.wrapping_add(1) % 7,
            day: u16::try_from(next.day()).expect("день помещается в u16"),
            ..self
        }
    }

    fn legacy_sql_date(self) -> String {
        format!("{}-{}-{}", self.year, self.month, self.day)
    }
}

/// Immutable DB-view одного элемента `liDBCreationPlayer`.
pub(crate) struct PlayerCreationBaseSnapshot {
    pub(crate) id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) account: Vec<u8>,
    pub(crate) level: u8,
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) country: u8,
    pub(crate) head: u8,
    /// SQL-порядок: HELM..FAIRY, как перечислено в owner-документации.
    pub(crate) equipment_ids: [u32; 11],
    /// Тот же SQL-порядок для `*Level`.
    pub(crate) equipment_levels: [u8; 11],
    pub(crate) region_id: i32,
}

/// Immutable DB-view значений, которые `SavePlayerBase` читает из `CPlayer`.
pub(crate) struct PlayerBaseSaveSnapshot<'a> {
    pub(crate) id: i32,
    pub(crate) name: &'a [u8],
    pub(crate) level: u8,
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) country: u8,
    pub(crate) head: u8,
    /// SQL-порядок: HELM..FAIRY; отсутствующий runtime goods даёт ноль.
    pub(crate) equipment_ids: [u32; 11],
    /// Тот же SQL-порядок для signed результата `GAP_WEAPON_LEVEL`.
    pub(crate) equipment_levels: [i32; 11],
    pub(crate) region_id: i32,
}

/// Одна DB-строка exact `OpenPlayerBaseInDB` до подмены live/save-копией.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerBaseDatabaseRow {
    pub(crate) id: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) level: u8,
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) country: u8,
    pub(crate) head: u8,
    pub(crate) equipment_ids: [u32; 11],
    pub(crate) equipment_levels: [u8; 11],
    pub(crate) region_id: i32,
}

/// Safe-граница ADO row-conversion внутри `OpenPlayerBaseInDB`.
#[derive(Debug)]
pub(crate) enum PlayerBaseLoadFailure {
    MissingConnection,
    Database,
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
    NumericOutsideLegacyRange {
        row_index: usize,
        column: &'static str,
        value: i64,
    },
}

/// Доказанный bool `CreatePlayerBase`.
#[derive(Debug)]
pub(crate) enum PlayerBaseCreateOutcome {
    Created,
    Failed,
}

/// Полный caller-owned view трёх стадий `CRsPlayer::CreatePlayer`.
///
/// Все вложенные ID обязаны происходить из одного исходного `CPlayer`:
/// `base.id == abilities.scalar.id == goods.player_id`.
pub(crate) struct PlayerCreationSnapshot<'player, 'goods_snapshot> {
    pub(crate) base: PlayerCreationBaseSnapshot,
    pub(crate) abilities: PlayerAbilityCreationSnapshot<'player>,
    pub(crate) goods: PlayerGoodsFiledSnapshot<'goods_snapshot>,
}

/// Неразрешённая граница одной из трёх create-стадий.
#[derive(Debug)]
pub(crate) enum PlayerCreateBlock {
    Goods(GoodsTraversalBlock),
}

/// Доказанный bool внешнего `CreatePlayer` либо вложенная неизвестность.
#[derive(Debug)]
pub(crate) enum PlayerCreateOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(PlayerCreateBlock),
}

/// Неразрешённая отрицательная `time_t`-граница `DeletePlayer`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerDeleteTimeBlock {
    /// Timestamp, для которого exact `_localtime` возвращает null.
    pub(crate) deletion_time: i32,
}

/// Доказанный bool `DeletePlayer` либо локальная UB-граница времени.
#[derive(Debug)]
pub(crate) enum PlayerDeleteOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(PlayerDeleteTimeBlock),
}

/// Структурированная замена достигнутых `CRsPlayer` DB/log-ошибок.
#[derive(Debug)]
pub(crate) struct RsPlayerNotice {
    pub(crate) operation: RsPlayerOperation,
    pub(crate) error: RsPlayerSaveError,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RsPlayerOperation {
    OpenPlayerBaseCount,
    OpenPlayerBase,
    GetPlayerDeletionDate,
    GetPlayerCountryById,
    GetPlayerNameById,
    ValidatePlayerIdInCdkey,
    IsNameExist,
    GetPlayerId,
    GetCdKey,
    StatRanks,
    Outer,
    BaseRow,
    SaveBaseRow,
    AbilityRow,
    SaveAbilityRow,
    SaveQuestData,
    Restore,
    Delete,
    HonorRanksLoad,
    HonorRanksInsert,
    HonorRanksSave,
}

/// Причина достигнутого log-эквивалента без SQL и runtime player values.
#[derive(Debug)]
pub(crate) enum RsPlayerSaveError {
    Database(RsPlayerDatabaseError),
    MissingConnection,
    PlayerRanksStatFailed,
    MissingBaseRow,
    MissingAbilityRow,
    MissingHonorRanksRow { period: HonorRanksSavePeriod },
    MalformedPlayerBaseRow,
    JjcSaveFailed,
}

impl fmt::Display for RsPlayerSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => write!(formatter, "не передано соединение World player DB"),
            Self::PlayerRanksStatFailed => {
                write!(formatter, "пересчёт рейтинга игроков завершился ошибкой")
            }
            Self::MissingBaseRow => write!(formatter, "не найдена строка CSL_PLAYER_BASE"),
            Self::MissingAbilityRow => write!(formatter, "не найдена строка CSL_PLAYER_ABILITY"),
            Self::MissingHonorRanksRow { period } => write!(
                formatter,
                "не найдена строка CSL_HonorRanks для {period:?}"
            ),
            Self::MalformedPlayerBaseRow => {
                write!(formatter, "некорректная строка CSL_PLAYER_BASE")
            }
            Self::JjcSaveFailed => {
                write!(formatter, "отдельное сохранение JJc завершилось ошибкой")
            }
        }
    }
}

impl Error for RsPlayerSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::MissingConnection
            | Self::PlayerRanksStatFailed
            | Self::MissingBaseRow
            | Self::MissingAbilityRow
            | Self::MissingHonorRanksRow { .. }
            | Self::MalformedPlayerBaseRow
            | Self::JjcSaveFailed => None,
        }
    }
}

/// Ошибка достигнутой ADO/TDS-границы без SQL и runtime player values.
#[derive(Debug)]
pub(crate) struct RsPlayerDatabaseError(tiberius::error::Error);

impl fmt::Display for RsPlayerDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World player DB: {}", self.0)
    }
}

impl Error for RsPlayerDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsPlayerDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

#[derive(Debug)]
pub(crate) enum PlayerRanksStatFailure {
    MissingConnection,
    Database {
        row_index: Option<usize>,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
    NumericOutsideLegacyRange {
        row_index: usize,
        column: &'static str,
        value: i64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRanksStatBlock {
    MaximumCountUnknown,
    AddRank {
        row_index: usize,
        source: PlayerRankAddBlock,
    },
}

#[derive(Debug)]
pub(crate) enum PlayerRanksStatOutcome {
    ReturnedTrue { row_count: usize },
    ReturnedFalse(PlayerRanksStatFailure),
    BlockedMissingFact(PlayerRanksStatBlock),
}

fn read_ado_integer(
    row: &Row,
    column: &'static str,
) -> Result<Option<i64>, tiberius::error::Error> {
    let first_error = match row.try_get::<i32, _>(column) {
        Ok(value) => return Ok(value.map(i64::from)),
        Err(error) => error,
    };
    if let Ok(value) = row.try_get::<u8, _>(column) {
        return Ok(value.map(i64::from));
    }
    if let Ok(value) = row.try_get::<i16, _>(column) {
        return Ok(value.map(i64::from));
    }
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return Ok(value);
    }
    Err(first_error)
}

/// Узкая объектная граница достигнутой стадии исходного `CRsPlayer`.
pub(crate) trait RsPlayerOwner {
    /// Повторяет отдельный `SELECT ID`/ADO RecordCount и его byte/`0xFF` контракт.
    async fn get_player_count_in_db_by_cdkey(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Option<u8>;

    /// Повторяет World-only `GetPlayerCountInCdkey`: DB sentinel сохраняется,
    /// successful byte складывается с live creation-count с x86 wrapping.
    async fn get_player_count_in_cdkey(
        &mut self,
        account: &[u8],
        creation_count: u8,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Option<u8> {
        self.get_player_count_in_db_by_cdkey(account, active_transaction)
            .await
            .and_then(|database_count| {
                let count = database_count.wrapping_add(creation_count);
                (count != u8::MAX).then_some(count)
            })
    }

    /// Читает ordered DB-часть списка; live/save merge остаётся у `CGame`.
    async fn open_player_base_in_db(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Result<Vec<PlayerBaseDatabaseRow>, PlayerBaseLoadFailure>;

    /// Возвращает local-midnight `DelDate`: null/missing даёт `0`, catch — `-1`.
    async fn get_player_deletion_date(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> i32;

    /// Возвращает byte country либо исходный ноль при EOF/DB-отказе.
    async fn get_player_country_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> u8;

    /// Возвращает ANSI player-name либо исходную пустую строку при отказе.
    async fn get_player_name_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Vec<u8>;

    /// Обходит все DB ID указанного account и сравнивает exact u32 bit-pattern.
    async fn validate_player_id_in_cdkey(
        &mut self,
        account: &[u8],
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Проверяет case-insensitive player-name через parameterized TDS query.
    async fn is_name_exist(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Возвращает account по имени либо исходную пустую строку при любом отказе.
    async fn get_cd_key(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Vec<u8>;

    /// Потоково добавляет exact TOP-рейтинг в уже очищенный live owner.
    async fn stat_ranks(
        &mut self,
        ranks: &mut CPlayerRanks,
        organizing: &COrganizingCtrl,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> PlayerRanksStatOutcome;

    /// Выполняет полную цепочку `LoadPlayer` до первого false/block.
    async fn load_player<J, G, WeekDay>(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
        thing_setup: &CThingSetup,
        get_week_day: WeekDay,
        jjc_owner: &mut J,
        goods_owner: &mut G,
        goods_registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> PlayerLoadOutcome
    where
        J: RsJjcSysOwner,
        G: DbGoodsOwner,
        WeekDay: FnMut() -> u16;

    /// Выполняет три create-стадии, останавливаясь после первого исходного false.
    async fn create_player<J: RsJjcSysOwner, G: DbGoodsOwner>(
        &mut self,
        snapshot: Option<&PlayerCreationSnapshot<'_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> PlayerCreateOutcome;

    /// Выполняет четыре save-стадии до первого доказанного отказа.
    async fn save_player<J: RsJjcSysOwner, G: DbGoodsOwner>(
        &mut self,
        snapshot: Option<&PlayerSaveSnapshot<'_, '_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> PlayerSaveOutcome;

    /// Выполняет первый INSERT внутри уже начатой caller-транзакции.
    async fn create_player_base(
        &mut self,
        snapshot: &PlayerCreationBaseSnapshot,
        active_transaction: &mut WorldTdsClient,
    ) -> PlayerBaseCreateOutcome;

    /// Обновляет существующую base-row, сохраняя исходный порядок 29 полей.
    async fn save_player_base(
        &mut self,
        snapshot: Option<&PlayerBaseSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Создаёт ability-row в caller-транзакции, затем сохраняет JJc отдельно.
    async fn create_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: &PlayerAbilityCreationSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
        jjc_owner: &mut J,
    ) -> bool;

    /// Обновляет существующую ability-row и только после неё сохраняет JJc.
    async fn save_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: Option<&PlayerAbilitySaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
    ) -> bool;

    /// Обновляет либо создаёт единственный quest-blob текущего игрока.
    async fn save_quest_data(
        &mut self,
        snapshot: Option<&PlayerQuestSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Снимает `DelDate` у одного unsigned player ID в caller-транзакции.
    async fn restore_player(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Ставит local calendar-date удаления внутри caller-транзакции.
    async fn delete_player(
        &mut self,
        player_id: u32,
        deletion_time: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> PlayerDeleteOutcome;

    /// Загружает history текущей даты и current следующей, сохраняя side effects.
    async fn load_honor_ranks<S: HonorRanksLoadSink>(
        &mut self,
        sink: &mut S,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> HonorRanksLoadOutcome;

    /// Обеспечивает строки honor-ranks для дня копии и следующего дня.
    async fn insert_honor_ranks(
        &mut self,
        snapshot: &HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Дренирует четыре country-list выбранных period/type в один field blob.
    fn save_honor_ranks_by_type<S: HonorRanksFieldSink>(
        &mut self,
        snapshot: &mut HonorRanksDbDataSnapshot,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
        sink: &mut S,
    ) -> HonorRanksByTypeSaveOutcome<S::Error>;

    /// Записывает History в дату копии, затем Current в следующий день.
    async fn save_honor_ranks(
        &mut self,
        snapshot: &mut HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> HonorRanksSaveOutcome;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<RsPlayerNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CRsPlayer`.
pub(crate) struct TiberiusRsPlayer {
    settings: WorldDatabaseSettings,
    notices: VecDeque<RsPlayerNotice>,
}

/// Исходный двух-DWORD payload `ResetAllLeitingInDB`.
///
/// В EXE worker принимает его как `void *`: первое слово — kind, второе —
/// signed `mktime` stamp. Rust не переносит heap-allocation без владельца,
/// но сохраняет порядок и ширину обоих полей в явном значении.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingDatabaseResetRequest {
    pub(crate) update_kind: u32,
    pub(crate) stamp: i32,
}

/// Наблюдаемый bool-итог `CRsPlayer::DbLetTingUpdate`.
#[derive(Debug)]
pub(crate) enum LeiTingDatabaseResetOutcome {
    ReturnedTrue { updated_rows: usize },
    ReturnedFalse(LeiTingDatabaseResetFailure),
}

/// Причина exact false-ветви, сохранённая без исходного catch-all/SEH.
#[derive(Debug)]
pub(crate) enum LeiTingDatabaseResetFailure {
    UnsupportedUpdateKind(u32),
    Connection(WorldDatabaseConnectionError),
    Database(tiberius::error::Error),
    MissingRequiredValue { column: &'static str },
}

/// Caller-owned связка полного `CRsPlayer::LoadPlayer` для `CPlayer::LoadData`.
pub(crate) struct TiberiusPlayerLoadData<'owner, J, G, WeekDay> {
    pub(crate) player_owner: &'owner mut TiberiusRsPlayer,
    pub(crate) active_transaction: Option<&'owner mut WorldTdsClient>,
    pub(crate) thing_setup: &'owner CThingSetup,
    pub(crate) get_week_day: &'owner mut WeekDay,
    pub(crate) jjc_owner: &'owner mut J,
    pub(crate) goods_owner: &'owner mut G,
    pub(crate) goods_registry: &'owner GoodsBasePropertiesRegistry,
    pub(crate) changed_goods_indices: &'owner BTreeMap<u32, u32>,
    pub(crate) dakong_addon_types: &'owner BTreeSet<i32>,
}

#[derive(Debug)]
pub(crate) enum TiberiusPlayerLoadDataBlock {
    Reconstruction(PlayerLoadBlock),
}

impl<J, G, WeekDay> PlayerLoadDataOwner for TiberiusPlayerLoadData<'_, J, G, WeekDay>
where
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
    WeekDay: FnMut() -> u16,
{
    type Block = TiberiusPlayerLoadDataBlock;

    async fn load_player(&mut self, player: &mut CPlayer) -> Result<bool, Self::Block> {
        match self
            .player_owner
            .load_player(
                player,
                self.active_transaction.as_deref_mut(),
                self.thing_setup,
                &mut *self.get_week_day,
                self.jjc_owner,
                self.goods_owner,
                self.goods_registry,
                self.changed_goods_indices,
                self.dakong_addon_types,
            )
            .await
        {
            PlayerLoadOutcome::ReturnedTrue => Ok(true),
            PlayerLoadOutcome::ReturnedFalse(_) => Ok(false),
            PlayerLoadOutcome::BlockedMissingFact(source) => {
                Err(TiberiusPlayerLoadDataBlock::Reconstruction(source))
            }
        }
    }
}

/// Scalar-колонка создаваемой строки `CSL_PLAYER_ABILITY`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerAbilityScalarField {
    Id,
    SaveTime,
    Name,
    RegionId,
    PosX,
    PosY,
    Dir,
    Account,
    Title,
    Levels,
    Exp,
    HeadPic,
    FacePic,
    Occupation,
    Sex,
    SpouseId,
    UnionId,
    MurdererTime,
    PkCount,
    KillCount,
    HitTopLog,
    HotHit,
    LoanMax,
    Loan,
    LoanTime,
    RemainPoint,
    PkNormal,
    PkTeam,
    PkUnion,
    PkBadman,
    PkCountry,
    Yp,
    Hp,
    Mp,
    Rp,
    BaseMaxHp,
    BaseMaxMp,
    BaseMaxYp,
    BaseMaxRp,
    BaseStr,
    BaseDex,
    BaseCon,
    BaseInt,
    BaseMinAtk,
    BaseMaxAtk,
    BaseHit,
    BaseBurden,
    BaseCch,
    BaseDef,
    BaseDodge,
    BaseAtcSpeed,
    BaseElementResistant,
    BaseHpRecoverSpeed,
    BaseMpRecoverSpeed,
    BaseVigour,
    BaseMaxVigour,
    BaseEnergy,
    BaseMaxEnergy,
    BaseCredit,
    DisplayHeadPiece,
    Silence,
    Country,
    Contribute,
    IsCharged,
    QuestTimeBegin,
    QuestTimeLimit,
    Quest,
    DepotPassword,
    Exploit,
    Kudos,
    Mode,
    FairyEnabled,
    FosterNum,
    HatcherNum,
    BattleFairyEnabled,
    FetchPower,
    MaxFetchPower,
    AuctionSpace,
    DaysHonorEliminateNum,
    WeeksHonorEliminateNum,
    MonthsHonorEliminateNum,
    TotalHonorEliminateNum,
    RankOfNobilityId,
    AppellationId,
    Exalt,
    Szl,
    GodsBattleFaction,
    BaseFyEnergy,
    BaseBlFyEnergy,
    LtUp60Cnt,
    RemainJlDanCnt,
    Lt60Stamp,
}

impl PlayerAbilityScalarField {
    /// Возвращает byte-exact имя, переданное исходному `Fields::Item`.
    pub(crate) const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "ID",
            Self::SaveTime => "SaveTime",
            Self::Name => "Name",
            Self::RegionId => "RegionID",
            Self::PosX => "PosX",
            Self::PosY => "PosY",
            Self::Dir => "Dir",
            Self::Account => "Account",
            Self::Title => "Title",
            Self::Levels => "Levels",
            Self::Exp => "Exp",
            Self::HeadPic => "HeadPic",
            Self::FacePic => "FacePic",
            Self::Occupation => "Occupation",
            Self::Sex => "Sex",
            Self::SpouseId => "SpouseId",
            Self::UnionId => "UnionID",
            Self::MurdererTime => "MurdererTime",
            Self::PkCount => "PkCount",
            Self::KillCount => "KillCount",
            Self::HitTopLog => "HitTopLog",
            Self::HotHit => "HotHit",
            Self::LoanMax => "LoanMax",
            Self::Loan => "Loan",
            Self::LoanTime => "LoanTime",
            Self::RemainPoint => "RemainPoint",
            Self::PkNormal => "Pk_Normal",
            Self::PkTeam => "Pk_Team",
            Self::PkUnion => "Pk_Union",
            Self::PkBadman => "Pk_Badman",
            Self::PkCountry => "Pk_Country",
            Self::Yp => "Yp",
            Self::Hp => "Hp",
            Self::Mp => "Mp",
            Self::Rp => "Rp",
            Self::BaseMaxHp => "BaseMaxHp",
            Self::BaseMaxMp => "BaseMaxMp",
            Self::BaseMaxYp => "BaseMaxYp",
            Self::BaseMaxRp => "BaseMaxRp",
            Self::BaseStr => "BaseStr",
            Self::BaseDex => "BaseDex",
            Self::BaseCon => "BaseCon",
            Self::BaseInt => "BaseInt",
            Self::BaseMinAtk => "BaseMinAtk",
            Self::BaseMaxAtk => "BaseMaxAtk",
            Self::BaseHit => "BaseHit",
            Self::BaseBurden => "BaseBurden",
            Self::BaseCch => "BaseCCH",
            Self::BaseDef => "BaseDef",
            Self::BaseDodge => "BaseDodge",
            Self::BaseAtcSpeed => "BaseAtcSpeed",
            Self::BaseElementResistant => "BaseElementResistant",
            Self::BaseHpRecoverSpeed => "BaseHpRecoverSpeed",
            Self::BaseMpRecoverSpeed => "BaseMpRecoverSpeed",
            Self::BaseVigour => "BaseVigour",
            Self::BaseMaxVigour => "BaseMaxVigour",
            Self::BaseEnergy => "BaseEnergy",
            Self::BaseMaxEnergy => "BaseMaxEnergy",
            Self::BaseCredit => "BaseCredit",
            Self::DisplayHeadPiece => "DisplayHeadPiece",
            Self::Silence => "silence",
            Self::Country => "country",
            Self::Contribute => "contribute",
            Self::IsCharged => "IsCharged",
            Self::QuestTimeBegin => "QuestTimeBegin",
            Self::QuestTimeLimit => "QuestTimeLimit",
            Self::Quest => "Quest",
            Self::DepotPassword => "DepotPassword",
            Self::Exploit => "Exploit",
            Self::Kudos => "Kudos",
            Self::Mode => "Mode",
            Self::FairyEnabled => "FairyEnabled",
            Self::FosterNum => "FosterNum",
            Self::HatcherNum => "HatcherNum",
            Self::BattleFairyEnabled => "BattleFairyEnabled",
            Self::FetchPower => "FetchPower",
            Self::MaxFetchPower => "MaxFetchPower",
            Self::AuctionSpace => "dwAuctionSpace",
            Self::DaysHonorEliminateNum => "DaysHonorElimilateNum",
            Self::WeeksHonorEliminateNum => "WeeksHonorElimilateNum",
            Self::MonthsHonorEliminateNum => "MonthsHonorElimilateNum",
            Self::TotalHonorEliminateNum => "TotalHonorElimilateNum",
            Self::RankOfNobilityId => "RankOfNobilityID",
            Self::AppellationId => "AppellationID",
            Self::Exalt => "dwExalt",
            Self::Szl => "SZL",
            Self::GodsBattleFaction => "GodsBattleFaction",
            Self::BaseFyEnergy => "basefyEnergy",
            Self::BaseBlFyEnergy => "baseblfyenergy",
            Self::LtUp60Cnt => "LTUp60Cnt",
            Self::RemainJlDanCnt => "wRemainJLDanCnt",
            Self::Lt60Stamp => "dwLT60Stamp",
        }
    }
}

/// Значение с точным исходным ADO `VARIANT`-типом.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PlayerAbilityScalarValue<'a> {
    /// `VT_I4` (`long`).
    I4(i32),
    /// `VT_UI4` (`unsigned long`).
    Ui4(u32),
    /// `VT_UI2` (`unsigned short`).
    Ui2(u16),
    /// `VT_UI1` (`unsigned char`).
    Ui1(u8),
    /// `VT_BOOL`: `true` материализуется как `VARIANT_TRUE` (`-1`).
    VariantBool(bool),
    /// `VT_R4` (`float`).
    R4(f32),
    /// `VT_INT`, отдельно от равного по ширине `VT_I4`.
    Int(i32),
    /// `_bstr_t(char const*)`: ANSI C-string до первого NUL.
    BStr(&'a [u8]),
}

/// Одно ordered-присваивание `Recordset::Fields::Item(...)->Value`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlayerAbilityScalarAssignment<'a> {
    pub(crate) field: PlayerAbilityScalarField,
    pub(crate) value: PlayerAbilityScalarValue<'a>,
}

/// Immutable DB-view значений, прочитанных `CreatePlayerAbilities` из `CPlayer`.
pub(crate) struct PlayerAbilityScalarSnapshot<'a> {
    pub(crate) id: i32,
    pub(crate) name: &'a [u8],
    pub(crate) region_id: i32,
    pub(crate) pos_x: f32,
    pub(crate) pos_y: f32,
    pub(crate) dir: i32,
    pub(crate) account: &'a [u8],
    pub(crate) title: &'a [u8],
    pub(crate) level: u8,
    pub(crate) exp: u32,
    pub(crate) head_pic: u8,
    pub(crate) face_pic: u8,
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) spouse_id: u32,
    pub(crate) union_id: u32,
    pub(crate) murderer_time: u32,
    pub(crate) pk_count: u16,
    pub(crate) kill_count: u32,
    pub(crate) hit_top_log: u16,
    pub(crate) hot_hit: u32,
    pub(crate) loan_max: u32,
    pub(crate) loan: u32,
    pub(crate) loan_time: i32,
    pub(crate) remain_point: u16,
    pub(crate) pk_normal: bool,
    pub(crate) pk_team: bool,
    pub(crate) pk_union: bool,
    pub(crate) pk_badman: bool,
    pub(crate) pk_country: bool,
    pub(crate) yp: u16,
    pub(crate) hp: u32,
    pub(crate) mp: u32,
    pub(crate) rp: u16,
    pub(crate) base_max_hp: u32,
    pub(crate) base_max_mp: u32,
    pub(crate) base_max_yp: u16,
    pub(crate) base_max_rp: u16,
    pub(crate) base_str: u32,
    pub(crate) base_dex: u32,
    pub(crate) base_con: u32,
    pub(crate) base_int: u32,
    pub(crate) base_min_atk: u32,
    pub(crate) base_max_atk: u32,
    pub(crate) base_hit: u16,
    pub(crate) base_burden: u16,
    pub(crate) base_cch: u16,
    pub(crate) base_def: u32,
    pub(crate) base_dodge: u16,
    pub(crate) base_atc_speed: u16,
    pub(crate) base_element_resistant: u32,
    pub(crate) base_hp_recover_speed: u16,
    pub(crate) base_mp_recover_speed: u16,
    pub(crate) base_vigour: u32,
    pub(crate) base_max_vigour: u32,
    pub(crate) base_energy: u32,
    pub(crate) base_max_energy: u32,
    pub(crate) base_credit: u32,
    pub(crate) display_head_piece: u8,
    pub(crate) country: u8,
    pub(crate) contribute: i32,
    pub(crate) is_charged: bool,
    pub(crate) quest_time_begin: i32,
    pub(crate) quest_time_limit: i32,
    pub(crate) quest: bool,
    pub(crate) depot_password: &'a [u8],
    pub(crate) exploit: u32,
    pub(crate) kudos: u32,
    pub(crate) mode: u32,
    pub(crate) fairy_enabled: bool,
    pub(crate) foster_num: u32,
    pub(crate) hatcher_num: u32,
    pub(crate) battle_fairy_enabled: bool,
    pub(crate) fetch_power: u32,
    pub(crate) max_fetch_power: u32,
    pub(crate) auction_space: u32,
    pub(crate) exalt: u32,
    pub(crate) szl: u32,
    pub(crate) gods_battle_faction: i32,
    pub(crate) base_fy_energy: u32,
    pub(crate) base_bl_fy_energy: u32,
    pub(crate) lt_up_60_count: u16,
    pub(crate) remain_jl_dan_count: u16,
    pub(crate) lt_60_stamp: u32,
}

/// Полный scalar-набор, который `CRsPlayer::LoadPlayer` публикует в `CPlayer`.
pub(crate) struct PlayerAbilityLoadScalarSnapshot<'a> {
    pub(crate) ability: PlayerAbilityScalarSnapshot<'a>,
    pub(crate) silence_time: i32,
    pub(crate) days_honor_eliminate_num: u32,
    pub(crate) weeks_honor_eliminate_num: u32,
    pub(crate) months_honor_eliminate_num: u32,
    pub(crate) total_honor_eliminate_num: u32,
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) appellation_id: u32,
}

/// Восстанавливает точный порядок и `VARIANT`-форму 84 scalar-присваиваний.
pub(crate) fn player_ability_scalar_assignments<'a>(
    snapshot: &PlayerAbilityScalarSnapshot<'a>,
) -> [PlayerAbilityScalarAssignment<'a>; 84] {
    use PlayerAbilityScalarField as Field;
    use PlayerAbilityScalarValue as Value;

    let assignment = |field, value| PlayerAbilityScalarAssignment { field, value };
    [
        assignment(Field::Id, Value::I4(snapshot.id)),
        assignment(Field::Name, Value::BStr(visible_c_string(snapshot.name))),
        assignment(Field::RegionId, Value::I4(snapshot.region_id)),
        assignment(Field::PosX, Value::R4(snapshot.pos_x)),
        assignment(Field::PosY, Value::R4(snapshot.pos_y)),
        assignment(Field::Dir, Value::I4(snapshot.dir)),
        assignment(
            Field::Account,
            Value::BStr(visible_c_string(snapshot.account)),
        ),
        assignment(Field::Title, Value::BStr(visible_c_string(snapshot.title))),
        assignment(Field::Levels, Value::Ui1(snapshot.level)),
        assignment(Field::Exp, Value::Ui4(snapshot.exp)),
        assignment(Field::HeadPic, Value::Ui1(snapshot.head_pic)),
        assignment(Field::FacePic, Value::Ui1(snapshot.face_pic)),
        assignment(Field::Occupation, Value::Ui1(snapshot.occupation)),
        assignment(Field::Sex, Value::Ui1(snapshot.sex)),
        assignment(Field::SpouseId, Value::Ui4(snapshot.spouse_id)),
        assignment(Field::UnionId, Value::Ui4(snapshot.union_id)),
        assignment(Field::MurdererTime, Value::Ui4(snapshot.murderer_time)),
        assignment(Field::PkCount, Value::Ui2(snapshot.pk_count)),
        assignment(Field::KillCount, Value::Ui4(snapshot.kill_count)),
        assignment(Field::HitTopLog, Value::Ui2(snapshot.hit_top_log)),
        assignment(Field::HotHit, Value::Ui4(snapshot.hot_hit)),
        assignment(Field::LoanMax, Value::Ui4(snapshot.loan_max)),
        assignment(Field::Loan, Value::Ui4(snapshot.loan)),
        assignment(Field::LoanTime, Value::I4(snapshot.loan_time)),
        assignment(Field::RemainPoint, Value::Ui2(snapshot.remain_point)),
        assignment(Field::PkNormal, Value::VariantBool(snapshot.pk_normal)),
        assignment(Field::PkTeam, Value::VariantBool(snapshot.pk_team)),
        assignment(Field::PkUnion, Value::VariantBool(snapshot.pk_union)),
        assignment(Field::PkBadman, Value::VariantBool(snapshot.pk_badman)),
        assignment(Field::PkCountry, Value::VariantBool(snapshot.pk_country)),
        assignment(Field::Yp, Value::Ui2(snapshot.yp)),
        assignment(Field::Hp, Value::Ui4(snapshot.hp)),
        assignment(Field::Mp, Value::Ui4(snapshot.mp)),
        assignment(Field::Rp, Value::Ui2(snapshot.rp)),
        assignment(Field::BaseMaxHp, Value::Ui4(snapshot.base_max_hp)),
        assignment(Field::BaseMaxMp, Value::Ui4(snapshot.base_max_mp)),
        assignment(Field::BaseMaxYp, Value::Ui2(snapshot.base_max_yp)),
        assignment(Field::BaseMaxRp, Value::Ui2(snapshot.base_max_rp)),
        assignment(Field::BaseStr, Value::Ui4(snapshot.base_str)),
        assignment(Field::BaseDex, Value::Ui4(snapshot.base_dex)),
        assignment(Field::BaseCon, Value::Ui4(snapshot.base_con)),
        assignment(Field::BaseInt, Value::Ui4(snapshot.base_int)),
        assignment(Field::BaseMinAtk, Value::Ui4(snapshot.base_min_atk)),
        assignment(Field::BaseMaxAtk, Value::Ui4(snapshot.base_max_atk)),
        assignment(Field::BaseHit, Value::Ui2(snapshot.base_hit)),
        assignment(Field::BaseBurden, Value::Ui2(snapshot.base_burden)),
        assignment(Field::BaseCch, Value::Ui2(snapshot.base_cch)),
        assignment(Field::BaseDef, Value::Ui4(snapshot.base_def)),
        assignment(Field::BaseDodge, Value::Ui2(snapshot.base_dodge)),
        assignment(Field::BaseAtcSpeed, Value::Ui2(snapshot.base_atc_speed)),
        assignment(
            Field::BaseElementResistant,
            Value::Ui4(snapshot.base_element_resistant),
        ),
        assignment(
            Field::BaseHpRecoverSpeed,
            Value::Ui2(snapshot.base_hp_recover_speed),
        ),
        assignment(
            Field::BaseMpRecoverSpeed,
            Value::Ui2(snapshot.base_mp_recover_speed),
        ),
        assignment(Field::BaseVigour, Value::Ui4(snapshot.base_vigour)),
        assignment(Field::BaseMaxVigour, Value::Ui4(snapshot.base_max_vigour)),
        assignment(Field::BaseEnergy, Value::Ui4(snapshot.base_energy)),
        assignment(Field::BaseMaxEnergy, Value::Ui4(snapshot.base_max_energy)),
        assignment(Field::BaseCredit, Value::Ui4(snapshot.base_credit)),
        assignment(
            Field::DisplayHeadPiece,
            Value::Ui1(snapshot.display_head_piece),
        ),
        assignment(Field::Country, Value::Ui1(snapshot.country)),
        assignment(Field::Contribute, Value::I4(snapshot.contribute)),
        assignment(Field::IsCharged, Value::I4(i32::from(snapshot.is_charged))),
        assignment(Field::QuestTimeBegin, Value::I4(snapshot.quest_time_begin)),
        assignment(Field::QuestTimeLimit, Value::I4(snapshot.quest_time_limit)),
        assignment(Field::Quest, Value::Int(i32::from(snapshot.quest))),
        assignment(
            Field::DepotPassword,
            Value::BStr(visible_c_string(snapshot.depot_password)),
        ),
        // Исходные DWORD здесь намеренно попадали в signed VT_I4.
        assignment(Field::Exploit, Value::I4(snapshot.exploit as i32)),
        assignment(Field::Kudos, Value::I4(snapshot.kudos as i32)),
        assignment(Field::Mode, Value::Ui4(snapshot.mode)),
        assignment(
            Field::FairyEnabled,
            Value::VariantBool(snapshot.fairy_enabled),
        ),
        assignment(Field::FosterNum, Value::Ui4(snapshot.foster_num)),
        assignment(Field::HatcherNum, Value::Ui4(snapshot.hatcher_num)),
        assignment(
            Field::BattleFairyEnabled,
            Value::VariantBool(snapshot.battle_fairy_enabled),
        ),
        assignment(Field::FetchPower, Value::Ui4(snapshot.fetch_power)),
        assignment(Field::MaxFetchPower, Value::Ui4(snapshot.max_fetch_power)),
        assignment(Field::AuctionSpace, Value::Ui4(snapshot.auction_space)),
        assignment(Field::Exalt, Value::Ui4(snapshot.exalt)),
        assignment(Field::Szl, Value::Ui4(snapshot.szl)),
        assignment(
            Field::GodsBattleFaction,
            Value::I4(snapshot.gods_battle_faction),
        ),
        assignment(Field::BaseFyEnergy, Value::Ui4(snapshot.base_fy_energy)),
        assignment(
            Field::BaseBlFyEnergy,
            Value::Ui4(snapshot.base_bl_fy_energy),
        ),
        assignment(Field::LtUp60Cnt, Value::Ui2(snapshot.lt_up_60_count)),
        assignment(
            Field::RemainJlDanCnt,
            Value::Ui2(snapshot.remain_jl_dan_count),
        ),
        assignment(Field::Lt60Stamp, Value::Ui4(snapshot.lt_60_stamp)),
    ]
}

/// Binary-колонка создаваемой строки `CSL_PLAYER_ABILITY`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerAbilityBinaryField {
    HotKey,
    Skill,
    ScriptFlag,
    State,
    Friend,
    CiQing,
    Thing,
}

impl PlayerAbilityBinaryField {
    /// Возвращает byte-exact имя, использованное исходным `AppendChunk`.
    pub(crate) const fn column_name(self) -> &'static str {
        match self {
            Self::HotKey => "HotKey",
            Self::Skill => "ListSkill",
            Self::ScriptFlag => "VariableList",
            Self::State => "ListState",
            Self::Friend => "ListFriendName",
            Self::CiQing => "ciqing",
            Self::Thing => "ListThing",
        }
    }
}

/// PDB-layout одного элемента `CPlayer::m_listNewSkillID`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAbilitySkill {
    pub(crate) id: u16,
    pub(crate) level: u16,
}

/// PDB-layout одного элемента `CPlayer::m_listThing`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerThing {
    pub(crate) tid: u16,
    pub(crate) count: u16,
    pub(crate) max_count: u16,
    pub(crate) point: u16,
}

/// Доказанный valid-state view полей `m_lVariable*` исходного `CPlayer`.
pub(crate) struct PlayerScriptFlagSnapshot<'a> {
    pub(crate) variable_num: i32,
    /// Срез одновременно доказывает неотрицательную длину и читаемый payload.
    pub(crate) variable_data: &'a [u8],
}

/// Имя friend-элемента на доказанном C-string-compatible пути.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerFriendName<'a>(&'a [u8]);

/// Локальная граница старого несоответствия `string::size()` и `strlen`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EmbeddedFriendNameNul {
    pub(crate) offset: usize,
}

impl<'a> PlayerFriendName<'a> {
    /// Принимает полный byte-content `std::string`, если старые size/strlen равны.
    pub(crate) fn from_legacy_bytes(bytes: &'a [u8]) -> Result<Self, EmbeddedFriendNameNul> {
        match bytes.iter().position(|byte| *byte == 0) {
            Some(offset) => Err(EmbeddedFriendNameNul { offset }),
            None => Ok(Self(bytes)),
        }
    }
}

/// Полный caller-owned view одной новой строки `CSL_PLAYER_ABILITY`.
///
/// Все части обязаны происходить из одного `CPlayer` snapshot; в частности,
/// `scalar.id == jjc.id`, как два чтения одного исходного объекта.
pub(crate) struct PlayerAbilityCreationSnapshot<'a> {
    pub(crate) scalar: PlayerAbilityScalarSnapshot<'a>,
    pub(crate) hot_keys: &'a [u32; 24],
    pub(crate) skills: &'a [PlayerAbilitySkill],
    pub(crate) script_flag: PlayerScriptFlagSnapshot<'a>,
    pub(crate) ex_states: &'a [u8],
    pub(crate) friend_names: &'a [PlayerFriendName<'a>],
    pub(crate) ci_qing_ids: &'a BTreeSet<u32>,
    pub(crate) things: &'a [PlayerThing],
    pub(crate) jjc: PlayerJjcDataSnapshot,
}

/// Полный caller-owned view обновления существующей `CSL_PLAYER_ABILITY`.
pub(crate) struct PlayerAbilitySaveSnapshot<'a> {
    pub(crate) ability: PlayerAbilityCreationSnapshot<'a>,
    pub(crate) silence_time: i32,
    pub(crate) days_honor_eliminate_num: u32,
    pub(crate) weeks_honor_eliminate_num: u32,
    pub(crate) months_honor_eliminate_num: u32,
    pub(crate) total_honor_eliminate_num: u32,
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) appellation_id: u32,
}

/// PDB-поля mapped value `CPlayer::tagPlayerQuest`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerQuestSaveEntry {
    pub(crate) quest_id: u16,
    pub(crate) complete: u8,
}

/// Caller-owned view `CPlayer::m_PlayerQuests` для одного DB update.
pub(crate) struct PlayerQuestSaveSnapshot<'a> {
    pub(crate) player_id: i32,
    pub(crate) quests: &'a BTreeMap<u16, PlayerQuestSaveEntry>,
}

/// Полный caller-owned view четырёх стадий `CRsPlayer::SavePlayer`.
///
/// Все вложенные ID обязаны происходить из одного исходного `CPlayer`:
/// `base.id == abilities.base.scalar.id == quest.player_id == goods.player_id`.
pub(crate) struct PlayerSaveSnapshot<'player, 'quest, 'goods_snapshot> {
    pub(crate) base: PlayerBaseSaveSnapshot<'player>,
    pub(crate) abilities: PlayerAbilitySaveSnapshot<'player>,
    pub(crate) quest: PlayerQuestSaveSnapshot<'quest>,
    pub(crate) goods: PlayerGoodsFiledSnapshot<'goods_snapshot>,
}

/// Неразрешённая граница одной из четырёх save-стадий.
#[derive(Debug)]
pub(crate) enum PlayerSaveBlock {
    Goods(GoodsTraversalBlock),
}

/// Доказанный bool внешнего `SavePlayer` либо вложенная неизвестность.
#[derive(Debug)]
pub(crate) enum PlayerSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(PlayerSaveBlock),
}

/// Кодирует mapped values в unsigned key-order исходного `std::map`.
pub(crate) fn encode_player_quest_data(snapshot: &PlayerQuestSaveSnapshot<'_>) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(snapshot.quests.len() * 3);
    for quest in snapshot.quests.values() {
        encoded.extend_from_slice(&quest.quest_id.to_le_bytes());
        encoded.push(quest.complete);
    }
    encoded
}

/// Собирает точный save-порядок 92 scalar-присваиваний.
pub(crate) fn player_ability_save_scalar_assignments<'a>(
    snapshot: &'a PlayerAbilitySaveSnapshot<'a>,
    save_time: &'a [u8],
) -> Vec<PlayerAbilityScalarAssignment<'a>> {
    use PlayerAbilityScalarField as Field;
    use PlayerAbilityScalarValue as Value;

    let create = player_ability_scalar_assignments(&snapshot.ability.scalar);
    let assignment = |field, value| PlayerAbilityScalarAssignment { field, value };
    let mut save = Vec::with_capacity(92);
    save.push(create[0]);
    save.push(assignment(Field::SaveTime, Value::BStr(save_time)));
    // Save сохраняет Name..DisplayHeadPiece в том же порядке, что create.
    save.extend_from_slice(&create[1..59]);
    save.push(assignment(Field::Silence, Value::I4(snapshot.silence_time)));
    // Затем идут country..Kudos, но BattleFairy переставлен раньше Mode.
    save.extend_from_slice(&create[59..68]);
    save.push(create[72]);
    save.extend_from_slice(&create[68..72]);
    // FetchPower..dwAuctionSpace предшествуют шести save-only honor-полям.
    save.extend_from_slice(&create[73..76]);
    save.push(assignment(
        Field::DaysHonorEliminateNum,
        Value::Ui4(snapshot.days_honor_eliminate_num),
    ));
    save.push(assignment(
        Field::WeeksHonorEliminateNum,
        Value::Ui4(snapshot.weeks_honor_eliminate_num),
    ));
    save.push(assignment(
        Field::MonthsHonorEliminateNum,
        Value::Ui4(snapshot.months_honor_eliminate_num),
    ));
    save.push(assignment(
        Field::TotalHonorEliminateNum,
        Value::Ui4(snapshot.total_honor_eliminate_num),
    ));
    save.push(assignment(
        Field::RankOfNobilityId,
        Value::Ui4(snapshot.rank_of_nobility_id),
    ));
    save.push(assignment(
        Field::AppellationId,
        Value::Ui4(snapshot.appellation_id),
    ));
    // Хвост dwExalt..dwLT60Stamp снова совпадает с create.
    save.extend_from_slice(&create[76..]);
    debug_assert_eq!(save.len(), 92);
    save
}

#[derive(Default)]
struct CollectedPlayerAbilityBinaryFields {
    fields: Vec<(PlayerAbilityBinaryField, Vec<u8>)>,
}

impl PlayerAbilityFieldSink for CollectedPlayerAbilityBinaryFields {
    type Error = Infallible;

    fn append_binary_field(
        &mut self,
        field: PlayerAbilityBinaryField,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        self.fields.push((field, bytes.to_vec()));
        Ok(())
    }
}

/// Синхронная замена достигнутого `Field20::AppendChunk`.
pub(crate) trait PlayerAbilityFieldSink {
    type Error;

    /// Обязан скопировать `bytes` до возврата, как исходный ADO-вызов.
    fn append_binary_field(
        &mut self,
        field: PlayerAbilityBinaryField,
        bytes: &[u8],
    ) -> Result<(), Self::Error>;
}

/// Сохраняет byte-exact `HotKey` blob создаваемой ability-строки.
pub(crate) fn save_hot_key_field<S: PlayerAbilityFieldSink>(
    hot_keys: &[u32; 24],
    sink: &mut S,
) -> Result<(), S::Error> {
    let mut bytes = [0_u8; 0x60];
    for (chunk, hot_key) in bytes.chunks_exact_mut(4).zip(hot_keys) {
        chunk.copy_from_slice(&hot_key.to_le_bytes());
    }
    sink.append_binary_field(PlayerAbilityBinaryField::HotKey, &bytes)
}

/// Сохраняет `ListSkill` как последовательность little-endian `tagSkill`.
pub(crate) fn save_skill_field<S: PlayerAbilityFieldSink>(
    skills: &[PlayerAbilitySkill],
    sink: &mut S,
) -> Result<(), S::Error> {
    let mut bytes = Vec::with_capacity(skills.len() * 4);
    for skill in skills {
        bytes.extend_from_slice(&skill.id.to_le_bytes());
        bytes.extend_from_slice(&skill.level.to_le_bytes());
    }
    sink.append_binary_field(PlayerAbilityBinaryField::Skill, &bytes)
}

/// Сохраняет `VariableList`: signed count и следующий за ним opaque payload.
pub(crate) fn save_script_flag<S: PlayerAbilityFieldSink>(
    snapshot: &PlayerScriptFlagSnapshot<'_>,
    sink: &mut S,
) -> Result<(), S::Error> {
    let mut bytes = Vec::with_capacity(4);
    bytes.extend_from_slice(&snapshot.variable_num.to_le_bytes());
    bytes.extend_from_slice(snapshot.variable_data);
    sink.append_binary_field(PlayerAbilityBinaryField::ScriptFlag, &bytes)
}

/// Сохраняет opaque `m_vExStates` byte-exact в `ListState`.
pub(crate) fn save_state_field<S: PlayerAbilityFieldSink>(
    ex_states: &[u8],
    sink: &mut S,
) -> Result<(), S::Error> {
    sink.append_binary_field(PlayerAbilityBinaryField::State, ex_states)
}

/// Сохраняет `ListFriendName` как list-order последовательность C-строк.
pub(crate) fn save_friend_field<S: PlayerAbilityFieldSink>(
    friend_names: &[PlayerFriendName<'_>],
    sink: &mut S,
) -> Result<(), S::Error> {
    let byte_count = friend_names.iter().map(|name| name.0.len() + 1).sum();
    let mut bytes = Vec::with_capacity(byte_count);
    for name in friend_names {
        bytes.extend_from_slice(name.0);
        bytes.push(0);
    }
    sink.append_binary_field(PlayerAbilityBinaryField::Friend, &bytes)
}

/// Сохраняет sorted unique `m_setCiQingList` в lowercase-поле `ciqing`.
pub(crate) fn save_ci_qing_field<S: PlayerAbilityFieldSink>(
    ci_qing_ids: &BTreeSet<u32>,
    sink: &mut S,
) -> Result<(), S::Error> {
    let mut bytes = Vec::with_capacity(ci_qing_ids.len() * 4);
    for id in ci_qing_ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    sink.append_binary_field(PlayerAbilityBinaryField::CiQing, &bytes)
}

/// Сохраняет `ListThing` как deque-order последовательность `tagThing`.
pub(crate) fn save_thing_field<S: PlayerAbilityFieldSink>(
    things: &[PlayerThing],
    sink: &mut S,
) -> Result<(), S::Error> {
    let mut bytes = Vec::with_capacity(things.len() * 8);
    for thing in things {
        bytes.extend_from_slice(&thing.tid.to_le_bytes());
        bytes.extend_from_slice(&thing.count.to_le_bytes());
        bytes.extend_from_slice(&thing.max_count.to_le_bytes());
        bytes.extend_from_slice(&thing.point.to_le_bytes());
    }
    sink.append_binary_field(PlayerAbilityBinaryField::Thing, &bytes)
}

/// Кодирует worker-local `GetDailyThingList` в exact `ListThing` blob.
fn encode_lei_ting_daily_things(things: &VecDeque<LeiTingDailyThing>) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(things.len() * 8);
    for thing in things {
        bytes.extend_from_slice(&thing.thing_id.to_le_bytes());
        bytes.extend_from_slice(&thing.count.to_le_bytes());
        bytes.extend_from_slice(&thing.max_count.to_le_bytes());
        bytes.extend_from_slice(&thing.point.to_le_bytes());
    }
    bytes
}

/// Safe-границы исходных unchecked binary-field loader-ов.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerAbilityBlobDecodeBlock {
    HotKeySize {
        actual_bytes: usize,
    },
    FriendNameWithoutTerminator {
        offset: usize,
        available_bytes: usize,
    },
    ScriptPayloadTooLarge {
        actual_bytes: usize,
    },
}

impl fmt::Display for PlayerAbilityBlobDecodeBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HotKeySize { actual_bytes } => write!(
                formatter,
                "HotKey содержит {actual_bytes} байт вместо обязательных 96"
            ),
            Self::FriendNameWithoutTerminator {
                offset,
                available_bytes,
            } => write!(
                formatter,
                "ListFriendName с offset {offset} не содержит NUL в оставшихся {available_bytes} байтах"
            ),
            Self::ScriptPayloadTooLarge { actual_bytes } => write!(
                formatter,
                "VariableList payload содержит {actual_bytes} байт и не помещается в signed long"
            ),
        }
    }
}

impl Error for PlayerAbilityBlobDecodeBlock {}

/// Owned-результат непустого `VariableList`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoadedPlayerScriptFlag {
    pub(crate) variable_num: i32,
    pub(crate) variable_data: Vec<u8>,
}

/// Декодирует exact `LoadHotKeyField`: только blob длиной `0x60` допустим.
pub(crate) fn load_hot_key_field(
    blob: &[u8],
) -> Result<[u32; 24], PlayerAbilityBlobDecodeBlock> {
    if blob.len() != 0x60 {
        return Err(PlayerAbilityBlobDecodeBlock::HotKeySize {
            actual_bytes: blob.len(),
        });
    }

    Ok(std::array::from_fn(|index| {
        let offset = index * size_of::<u32>();
        u32::from_le_bytes(
            blob[offset..offset + size_of::<u32>()]
                .try_into()
                .expect("проверены все 96 байт HotKey"),
        )
    }))
}

/// Декодирует `ListSkill`; неполный хвост игнорируется как `size / 4` в EXE.
pub(crate) fn load_skill_field(blob: &[u8]) -> Vec<PlayerAbilitySkill> {
    blob.chunks_exact(4)
        .map(|entry| PlayerAbilitySkill {
            id: u16::from_le_bytes(entry[0..2].try_into().expect("полный tagSkill")),
            level: u16::from_le_bytes(entry[2..4].try_into().expect("полный tagSkill")),
        })
        .collect()
}

/// Декодирует `VariableList`; `0..=3` байта оставляют player-state прежним.
pub(crate) fn load_script_flag(
    blob: &[u8],
) -> Result<Option<LoadedPlayerScriptFlag>, PlayerAbilityBlobDecodeBlock> {
    let Some(header) = blob.get(..4) else {
        return Ok(None);
    };
    let payload = &blob[4..];
    if i32::try_from(payload.len()).is_err() {
        return Err(PlayerAbilityBlobDecodeBlock::ScriptPayloadTooLarge {
            actual_bytes: payload.len(),
        });
    }
    let variable_num = i32::from_le_bytes(header.try_into().expect("проверены 4 байта"));
    Ok(Some(LoadedPlayerScriptFlag {
        variable_num,
        variable_data: payload.to_vec(),
    }))
}

/// Декодирует opaque `ListState`; пустое поле не вызывает `SetExStates`.
pub(crate) fn load_state_field(blob: &[u8]) -> Option<Vec<u8>> {
    (!blob.is_empty()).then(|| blob.to_vec())
}

/// Декодирует последовательность C-строк `ListFriendName` в list-order.
pub(crate) fn load_friend_field(
    blob: &[u8],
) -> Result<Vec<Vec<u8>>, PlayerAbilityBlobDecodeBlock> {
    let mut names = Vec::new();
    let mut offset = 0usize;
    while offset < blob.len() {
        let remaining = &blob[offset..];
        let Some(name_length) = remaining.iter().position(|byte| *byte == 0) else {
            return Err(PlayerAbilityBlobDecodeBlock::FriendNameWithoutTerminator {
                offset,
                available_bytes: remaining.len(),
            });
        };
        names.push(remaining[..name_length].to_vec());
        offset += name_length + 1;
    }
    Ok(names)
}

/// Декодирует `ciqing`; неполный хвост игнорируется, set убирает дубликаты.
pub(crate) fn load_ci_qing_field(blob: &[u8]) -> BTreeSet<u32> {
    blob.chunks_exact(4)
        .map(|entry| u32::from_le_bytes(entry.try_into().expect("полный tattoo ID")))
        .collect()
}

/// Декодирует непустой `ListThing`; empty/default-ветвь принадлежит caller-у.
pub(crate) fn load_thing_field(blob: &[u8]) -> Vec<PlayerThing> {
    blob.chunks_exact(8)
        .map(|entry| PlayerThing {
            tid: u16::from_le_bytes(entry[0..2].try_into().expect("полный tagThing")),
            count: u16::from_le_bytes(entry[2..4].try_into().expect("полный tagThing")),
            max_count: u16::from_le_bytes(
                entry[4..6].try_into().expect("полный tagThing"),
            ),
            point: u16::from_le_bytes(entry[6..8].try_into().expect("полный tagThing")),
        })
        .collect()
}

/// Декодирует `QuestData`; неполный хвост игнорируется как `size / 3` в EXE.
pub(crate) fn load_quest_data(blob: &[u8]) -> Vec<PlayerQuestSaveEntry> {
    blob.chunks_exact(3)
        .map(|entry| PlayerQuestSaveEntry {
            quest_id: u16::from_le_bytes(entry[0..2].try_into().expect("полный quest value")),
            complete: entry[2],
        })
        .collect()
}

#[derive(Debug)]
pub(crate) enum PlayerAbilityRowLoadFailure {
    Database {
        column: &'static str,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        column: &'static str,
    },
    NumericOutsideLegacyRange {
        column: &'static str,
        value: i64,
        target: &'static str,
    },
    Blob {
        field: PlayerAbilityBinaryField,
        source: PlayerAbilityBlobDecodeBlock,
    },
    HonorTime(TagTimeArithmeticBlock),
}

#[derive(Debug)]
pub(crate) enum PlayerAbilityQueryLoadFailure {
    ZeroPlayerId,
    MissingConnection,
    Database(tiberius::error::Error),
    MissingRow,
}

#[derive(Debug)]
pub(crate) enum PlayerAbilityQueryLoadOutcome {
    ReturnedTrue,
    ReturnedFalse(PlayerAbilityQueryLoadFailure),
    BlockedMalformed(PlayerAbilityRowLoadFailure),
}

#[derive(Debug)]
pub(crate) enum PlayerQuestQueryLoadFailure {
    ZeroPlayerId,
    MissingConnection,
    Database(tiberius::error::Error),
}

#[derive(Debug)]
pub(crate) enum PlayerQuestQueryLoadOutcome {
    ReturnedTrue { quest_count: usize },
    ReturnedFalse(PlayerQuestQueryLoadFailure),
}

#[derive(Debug)]
pub(crate) enum PlayerLoadFailure {
    Connection(WorldDatabaseConnectionError),
    Ability(PlayerAbilityQueryLoadFailure),
    Quest(PlayerQuestQueryLoadFailure),
    Goods(GoodsLoadFailure),
    Jjc(PlayerJjcLoadFailure),
}

#[derive(Debug)]
pub(crate) enum PlayerLoadBlock {
    Ability(PlayerAbilityRowLoadFailure),
    Goods(GoodsLoadBlock),
}

#[derive(Debug)]
pub(crate) enum PlayerLoadOutcome {
    ReturnedTrue,
    ReturnedFalse(PlayerLoadFailure),
    BlockedMissingFact(PlayerLoadBlock),
}

fn required_ability_integer(
    row: &Row,
    column: &'static str,
) -> Result<i64, PlayerAbilityRowLoadFailure> {
    match read_ado_integer(row, column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(PlayerAbilityRowLoadFailure::MissingRequiredValue { column }),
        Err(source) => Err(PlayerAbilityRowLoadFailure::Database { column, source }),
    }
}

fn required_ability_bool(
    row: &Row,
    column: &'static str,
) -> Result<bool, PlayerAbilityRowLoadFailure> {
    let first_error = match row.try_get::<bool, _>(column) {
        Ok(Some(value)) => return Ok(value),
        Ok(None) => return Err(PlayerAbilityRowLoadFailure::MissingRequiredValue { column }),
        Err(source) => source,
    };
    match read_ado_integer(row, column) {
        Ok(Some(value)) => Ok(value != 0),
        Ok(None) => Err(PlayerAbilityRowLoadFailure::MissingRequiredValue { column }),
        Err(_) => Err(PlayerAbilityRowLoadFailure::Database {
            column,
            source: first_error,
        }),
    }
}

fn required_ability_float(
    row: &Row,
    column: &'static str,
) -> Result<f32, PlayerAbilityRowLoadFailure> {
    match row.try_get::<f32, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(PlayerAbilityRowLoadFailure::MissingRequiredValue { column }),
        Err(source) => Err(PlayerAbilityRowLoadFailure::Database { column, source }),
    }
}

fn required_ability_ansi(
    row: &Row,
    column: &'static str,
) -> Result<Vec<u8>, PlayerAbilityRowLoadFailure> {
    match row.try_get::<&str, _>(column) {
        Ok(Some(value)) => {
            let (encoded, _, _) = WINDOWS_1251.encode(value);
            Ok(encoded.into_owned())
        }
        Ok(None) => Err(PlayerAbilityRowLoadFailure::MissingRequiredValue { column }),
        Err(source) => Err(PlayerAbilityRowLoadFailure::Database { column, source }),
    }
}

fn ability_blob(
    row: &Row,
    field: PlayerAbilityBinaryField,
) -> Result<Vec<u8>, PlayerAbilityRowLoadFailure> {
    let column = field.column_name();
    match row.try_get::<&[u8], _>(column) {
        Ok(Some(value)) => Ok(value.to_vec()),
        Ok(None) => Ok(Vec::new()),
        Err(source) => Err(PlayerAbilityRowLoadFailure::Database { column, source }),
    }
}

/// Материализует доказанную scalar-часть одной ability-строки без ADO casts.
pub(crate) fn materialize_player_ability_scalar_row(
    row: &Row,
    player: &mut CPlayer,
) -> Result<(), PlayerAbilityRowLoadFailure> {
    macro_rules! integer {
        ($column:literal, $target:ty) => {{
            let value = required_ability_integer(row, $column)?;
            <$target>::try_from(value).map_err(|_| {
                PlayerAbilityRowLoadFailure::NumericOutsideLegacyRange {
                    column: $column,
                    value,
                    target: stringify!($target),
                }
            })?
        }};
    }
    macro_rules! bit_pattern_u32 {
        ($column:literal) => {{
            let value = required_ability_integer(row, $column)?;
            if let Ok(value) = u32::try_from(value) {
                value
            } else if let Ok(value) = i32::try_from(value) {
                value as u32
            } else {
                return Err(PlayerAbilityRowLoadFailure::NumericOutsideLegacyRange {
                    column: $column,
                    value,
                    target: "u32/i32 bit-pattern",
                });
            }
        }};
    }

    let name = required_ability_ansi(row, "Name")?;
    let title = required_ability_ansi(row, "Title")?;
    let depot_password = required_ability_ansi(row, "DepotPassword")?;
    let account = player.get_account().to_vec();
    let scalar = PlayerAbilityScalarSnapshot {
        id: player.get_id(),
        name: &name,
        region_id: integer!("RegionID", i32),
        pos_x: required_ability_float(row, "PosX")?,
        pos_y: required_ability_float(row, "PosY")?,
        dir: integer!("Dir", i32),
        account: &account,
        title: &title,
        level: integer!("Levels", u8),
        exp: bit_pattern_u32!("Exp"),
        head_pic: integer!("HeadPic", u8),
        face_pic: integer!("FacePic", u8),
        occupation: integer!("Occupation", u8),
        sex: integer!("Sex", u8),
        spouse_id: bit_pattern_u32!("SpouseID"),
        union_id: bit_pattern_u32!("UnionID"),
        murderer_time: bit_pattern_u32!("MurdererTime"),
        pk_count: integer!("PkCount", u16),
        kill_count: bit_pattern_u32!("KillCount"),
        hit_top_log: integer!("HitTopLog", u16),
        hot_hit: bit_pattern_u32!("HotHit"),
        loan_max: bit_pattern_u32!("LoanMax"),
        loan: bit_pattern_u32!("Loan"),
        loan_time: integer!("LoanTime", i32),
        remain_point: integer!("RemainPoint", u16),
        pk_normal: required_ability_bool(row, "Pk_Normal")?,
        pk_team: required_ability_bool(row, "Pk_Team")?,
        pk_union: required_ability_bool(row, "Pk_Union")?,
        pk_badman: required_ability_bool(row, "Pk_Badman")?,
        pk_country: required_ability_bool(row, "Pk_Country")?,
        yp: integer!("Yp", u16),
        hp: bit_pattern_u32!("Hp"),
        mp: bit_pattern_u32!("Mp"),
        rp: integer!("Rp", u16),
        base_max_hp: bit_pattern_u32!("BaseMaxHp"),
        base_max_mp: bit_pattern_u32!("BaseMaxMp"),
        base_max_yp: integer!("BaseMaxYp", u16),
        base_max_rp: 0,
        base_str: bit_pattern_u32!("BaseStr"),
        base_dex: bit_pattern_u32!("BaseDex"),
        base_con: bit_pattern_u32!("BaseCon"),
        base_int: bit_pattern_u32!("BaseInt"),
        base_min_atk: bit_pattern_u32!("BaseMinAtk"),
        base_max_atk: bit_pattern_u32!("BaseMaxAtk"),
        base_hit: integer!("BaseHit", u16),
        base_burden: integer!("BaseBurden", u16),
        base_cch: integer!("BaseCCH", u16),
        base_def: bit_pattern_u32!("BaseDef"),
        base_dodge: integer!("BaseDodge", u16),
        base_atc_speed: integer!("BaseAtcSpeed", u16),
        base_element_resistant: bit_pattern_u32!("BaseElementResistant"),
        base_hp_recover_speed: integer!("BaseHpRecoverSpeed", u16),
        base_mp_recover_speed: integer!("BaseMpRecoverSpeed", u16),
        base_vigour: bit_pattern_u32!("BaseVigour"),
        base_max_vigour: bit_pattern_u32!("BaseMaxVigour"),
        base_energy: bit_pattern_u32!("BaseEnergy"),
        base_max_energy: bit_pattern_u32!("BaseMaxEnergy"),
        base_credit: bit_pattern_u32!("BaseCredit"),
        display_head_piece: integer!("DisplayHeadPiece", u8),
        country: integer!("country", u8),
        contribute: integer!("contribute", i32),
        is_charged: required_ability_bool(row, "IsCharged")?,
        quest_time_begin: integer!("QuestTimeBegin", i32),
        quest_time_limit: integer!("QuestTimeLimit", i32),
        quest: required_ability_bool(row, "Quest")?,
        depot_password: &depot_password,
        exploit: bit_pattern_u32!("Exploit"),
        kudos: bit_pattern_u32!("Kudos"),
        mode: bit_pattern_u32!("Mode"),
        fairy_enabled: required_ability_bool(row, "FairyEnabled")?,
        foster_num: bit_pattern_u32!("FosterNum"),
        hatcher_num: bit_pattern_u32!("HatcherNum"),
        battle_fairy_enabled: required_ability_bool(row, "BattleFairyEnabled")?,
        fetch_power: bit_pattern_u32!("FetchPower"),
        max_fetch_power: bit_pattern_u32!("MaxFetchPower"),
        auction_space: bit_pattern_u32!("dwAuctionSpace"),
        exalt: bit_pattern_u32!("dwExalt"),
        szl: bit_pattern_u32!("SZL"),
        gods_battle_faction: integer!("GodsBattleFaction", i32),
        base_fy_energy: bit_pattern_u32!("basefyEnergy"),
        base_bl_fy_energy: bit_pattern_u32!("baseblfyenergy"),
        lt_up_60_count: integer!("LTUp60Cnt", u16),
        remain_jl_dan_count: integer!("wRemainJLDanCnt", u16),
        lt_60_stamp: match read_ado_integer(row, "dwLT60Stamp") {
            Ok(Some(value)) => {
                if let Ok(value) = u32::try_from(value) {
                    value
                } else if let Ok(value) = i32::try_from(value) {
                    value as u32
                } else {
                    return Err(PlayerAbilityRowLoadFailure::NumericOutsideLegacyRange {
                        column: "dwLT60Stamp",
                        value,
                        target: "u32/i32 bit-pattern",
                    });
                }
            }
            Ok(None) => 0,
            Err(source) => {
                return Err(PlayerAbilityRowLoadFailure::Database {
                    column: "dwLT60Stamp",
                    source,
                });
            }
        },
    };
    let loaded = PlayerAbilityLoadScalarSnapshot {
        ability: scalar,
        silence_time: integer!("silence", i32),
        days_honor_eliminate_num: bit_pattern_u32!("DaysHonorElimilateNum"),
        weeks_honor_eliminate_num: bit_pattern_u32!("WeeksHonorElimilateNum"),
        months_honor_eliminate_num: bit_pattern_u32!("MonthsHonorElimilateNum"),
        total_honor_eliminate_num: bit_pattern_u32!("TotalHonorElimilateNum"),
        rank_of_nobility_id: bit_pattern_u32!("RankOfNobilityID"),
        appellation_id: bit_pattern_u32!("AppellationID"),
    };
    player.apply_loaded_ability_scalars(&loaded);
    Ok(())
}

/// Материализует семь binary fields одной ability-строки в exact helper-order.
pub(crate) fn materialize_player_ability_binary_row(
    row: &Row,
    player: &mut CPlayer,
    thing_setup: &CThingSetup,
    get_week_day: &mut impl FnMut() -> u16,
) -> Result<(), PlayerAbilityRowLoadFailure> {
    let hot_keys_blob = ability_blob(row, PlayerAbilityBinaryField::HotKey)?;
    let hot_keys = load_hot_key_field(&hot_keys_blob).map_err(|source| {
        PlayerAbilityRowLoadFailure::Blob {
            field: PlayerAbilityBinaryField::HotKey,
            source,
        }
    })?;
    player.apply_loaded_hot_keys(&hot_keys);

    let skills = load_skill_field(&ability_blob(row, PlayerAbilityBinaryField::Skill)?);
    player.append_loaded_skills(&skills);

    if let Some(states) = load_state_field(&ability_blob(row, PlayerAbilityBinaryField::State)?) {
        player.apply_loaded_ex_states(states);
    }

    let friends_blob = ability_blob(row, PlayerAbilityBinaryField::Friend)?;
    let friends = load_friend_field(&friends_blob).map_err(|source| {
        PlayerAbilityRowLoadFailure::Blob {
            field: PlayerAbilityBinaryField::Friend,
            source,
        }
    })?;
    player.append_loaded_friends(friends);

    let script_blob = ability_blob(row, PlayerAbilityBinaryField::ScriptFlag)?;
    if let Some(script) = load_script_flag(&script_blob).map_err(|source| {
        PlayerAbilityRowLoadFailure::Blob {
            field: PlayerAbilityBinaryField::ScriptFlag,
            source,
        }
    })? {
        player.apply_loaded_script_flag(script);
    }

    let ci_qing = load_ci_qing_field(&ability_blob(row, PlayerAbilityBinaryField::CiQing)?);
    player.extend_loaded_ci_qing(ci_qing);

    let thing_blob = ability_blob(row, PlayerAbilityBinaryField::Thing)?;
    let things = load_thing_field(&thing_blob);
    let fy_energy_value = required_ability_integer(row, "basefyEnergy")?;
    let fy_energy = if let Ok(value) = u32::try_from(fy_energy_value) {
        value
    } else if let Ok(value) = i32::try_from(fy_energy_value) {
        value as u32
    } else {
        return Err(PlayerAbilityRowLoadFailure::NumericOutsideLegacyRange {
            column: "basefyEnergy",
            value: fy_energy_value,
            target: "u32/i32 bit-pattern",
        });
    };
    player.apply_loaded_things(
        thing_blob.is_empty(),
        things,
        fy_energy,
        thing_setup,
        get_week_day,
    );
    Ok(())
}

impl TiberiusRsPlayer {
    /// Копирует DB setup для исходных методов с автономным connection.
    pub(crate) fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            settings: settings.clone(),
            notices: VecDeque::new(),
        }
    }

    /// Выполняет один фоновый проход `DbLetTingUpdate` на отдельном соединении.
    ///
    /// Exact `0x00510520` строит `ListThing` до `CreateCn/OpenCn`, затем без
    /// transaction проходит updatable recordset `csl_player_ability`; успешные
    /// ранние строки сохраняются даже если последующая строка даёт ошибку.
    /// TDS не предоставляет этот ADO recordset API, поэтому `ID` добавлен
    /// только как технический ключ текущей строки, а каждый `UPDATE` остаётся
    /// отдельным statement в том же исходном порядке без `ORDER BY` и rollback.
    /// Снятие list происходит внутри worker-вызова, а не при его постановке.
    pub(crate) async fn db_lei_ting_update(
        &self,
        request: LeiTingDatabaseResetRequest,
        thing_setup: &CThingSetup,
        total_jing_li_dan_count: u16,
    ) -> LeiTingDatabaseResetOutcome {
        if request.update_kind != 1 && request.update_kind != 2 {
            return LeiTingDatabaseResetOutcome::ReturnedFalse(
                LeiTingDatabaseResetFailure::UnsupportedUpdateKind(request.update_kind),
            );
        }

        let mut daily_things = VecDeque::new();
        thing_setup.get_daily_thing_list(
            || Local::now().weekday().num_days_from_sunday() as u16,
            &mut daily_things,
        );
        let list_thing = encode_lei_ting_daily_things(&daily_things);

        let mut connection = match self.settings.connect().await {
            Ok(connection) => connection,
            Err(error) => {
                return LeiTingDatabaseResetOutcome::ReturnedFalse(
                    LeiTingDatabaseResetFailure::Connection(error),
                );
            }
        };
        let rows = match connection.simple_query(LEI_TING_RESET_SELECT_SQL).await {
            Ok(stream) => match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => {
                    return LeiTingDatabaseResetOutcome::ReturnedFalse(
                        LeiTingDatabaseResetFailure::Database(error),
                    );
                }
            },
            Err(error) => {
                return LeiTingDatabaseResetOutcome::ReturnedFalse(
                    LeiTingDatabaseResetFailure::Database(error),
                );
            }
        };

        let mut updated_rows = 0;
        for row in rows {
            let player_id = match row.try_get::<i32, _>("ID") {
                Ok(Some(value)) => value,
                Ok(None) => {
                    return LeiTingDatabaseResetOutcome::ReturnedFalse(
                        LeiTingDatabaseResetFailure::MissingRequiredValue { column: "ID" },
                    );
                }
                Err(error) => {
                    return LeiTingDatabaseResetOutcome::ReturnedFalse(
                        LeiTingDatabaseResetFailure::Database(error),
                    );
                }
            };
            let base_bl_fy_energy = if request.update_kind == 1 {
                match row.try_get::<i32, _>("baseblfyenergy") {
                    Ok(Some(value)) => value as u32,
                    Ok(None) => {
                        return LeiTingDatabaseResetOutcome::ReturnedFalse(
                            LeiTingDatabaseResetFailure::MissingRequiredValue {
                                column: "baseblfyenergy",
                            },
                        );
                    }
                    Err(error) => {
                        return LeiTingDatabaseResetOutcome::ReturnedFalse(
                            LeiTingDatabaseResetFailure::Database(error),
                        );
                    }
                }
            } else {
                0
            };

            let query = if request.update_kind == 1 {
                let mut query = Query::new(LEI_TING_DAILY_RESET_SQL);
                query.bind(list_thing.as_slice());
                query.bind(request.stamp);
                query.bind(0_i64);
                query.bind(i32::from(total_jing_li_dan_count));
                query.bind(i64::from(base_bl_fy_energy & !0x0f));
                query.bind(player_id);
                query
            } else {
                let mut query = Query::new(LEI_TING_MONTHLY_RESET_SQL);
                query.bind(list_thing.as_slice());
                query.bind(request.stamp);
                query.bind(0_i64);
                query.bind(i32::from(total_jing_li_dan_count));
                query.bind(0_i64);
                query.bind(0_i64);
                query.bind(player_id);
                query
            };
            if let Err(error) = query.execute(&mut connection).await {
                return LeiTingDatabaseResetOutcome::ReturnedFalse(
                    LeiTingDatabaseResetFailure::Database(error),
                );
            }
            updated_rows += 1;
        }

        LeiTingDatabaseResetOutcome::ReturnedTrue { updated_rows }
    }

    /// Загружает и публикует одну ordered ability-строку по signed player ID.
    pub(crate) async fn load_player_ability_row(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
        thing_setup: &CThingSetup,
        mut get_week_day: impl FnMut() -> u16,
    ) -> PlayerAbilityQueryLoadOutcome {
        let player_id = player.get_id();
        if player_id == 0 {
            return PlayerAbilityQueryLoadOutcome::ReturnedFalse(
                PlayerAbilityQueryLoadFailure::ZeroPlayerId,
            );
        }
        let Some(active_transaction) = active_transaction else {
            return PlayerAbilityQueryLoadOutcome::ReturnedFalse(
                PlayerAbilityQueryLoadFailure::MissingConnection,
            );
        };

        let mut query = Query::new(
            "SELECT * FROM CSL_PLAYER_ABILITY WHERE id=@P1 ORDER BY id",
        );
        query.bind(player_id);
        let row = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(Some(row)) => row,
                Ok(None) => {
                    return PlayerAbilityQueryLoadOutcome::ReturnedFalse(
                        PlayerAbilityQueryLoadFailure::MissingRow,
                    );
                }
                Err(source) => {
                    return PlayerAbilityQueryLoadOutcome::ReturnedFalse(
                        PlayerAbilityQueryLoadFailure::Database(source),
                    );
                }
            },
            Err(source) => {
                return PlayerAbilityQueryLoadOutcome::ReturnedFalse(
                    PlayerAbilityQueryLoadFailure::Database(source),
                );
            }
        };

        if let Err(source) = materialize_player_ability_scalar_row(&row, player) {
            return PlayerAbilityQueryLoadOutcome::BlockedMalformed(source);
        }
        if let Err(source) =
            materialize_player_ability_binary_row(&row, player, thing_setup, &mut get_week_day)
        {
            return PlayerAbilityQueryLoadOutcome::BlockedMalformed(source);
        }

        let lei_ting_now = Local::now();
        let _ = player.reset_loaded_lei_ting_if_needed(
            lei_ting_now.day() as u16,
            || Local::now().timestamp() as u32,
            thing_setup,
            &mut get_week_day,
        );

        let has_save_time = row
            .columns()
            .iter()
            .any(|column| column.name().eq_ignore_ascii_case("SaveTime"));
        if !has_save_time {
            return PlayerAbilityQueryLoadOutcome::BlockedMalformed(
                PlayerAbilityRowLoadFailure::MissingRequiredValue { column: "SaveTime" },
            );
        }
        let save_time = match row.try_get::<NaiveDateTime, _>("SaveTime") {
            Ok(Some(value)) => TagTime::from_fields([
                value.year() as u16,
                value.month() as u16,
                value.weekday().num_days_from_sunday() as u16,
                value.day() as u16,
                value.hour() as u16,
                value.minute() as u16,
                value.second() as u16,
                value.and_utc().timestamp_subsec_millis() as u16,
            ]),
            Ok(None) | Err(_) => TagTime::local_now(),
        };
        if let Err(source) = player.reset_honor_eliminate_num(save_time, TagTime::local_now()) {
            return PlayerAbilityQueryLoadOutcome::BlockedMalformed(
                PlayerAbilityRowLoadFailure::HonorTime(source),
            );
        }
        PlayerAbilityQueryLoadOutcome::ReturnedTrue
    }

    /// Повторяет отдельный `LoadQuestData`; EOF означает пустой успешный owner.
    pub(crate) async fn load_player_quest_data(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> PlayerQuestQueryLoadOutcome {
        let player_id = player.get_id();
        if player_id == 0 {
            return PlayerQuestQueryLoadOutcome::ReturnedFalse(
                PlayerQuestQueryLoadFailure::ZeroPlayerId,
            );
        }
        let Some(active_transaction) = active_transaction else {
            return PlayerQuestQueryLoadOutcome::ReturnedFalse(
                PlayerQuestQueryLoadFailure::MissingConnection,
            );
        };

        let mut query =
            Query::new("SELECT * FROM CSL_PLAYER_QUEST_EX WHERE PlayerID=@P1");
        query.bind(player_id);
        let row = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row,
                Err(source) => {
                    return PlayerQuestQueryLoadOutcome::ReturnedFalse(
                        PlayerQuestQueryLoadFailure::Database(source),
                    );
                }
            },
            Err(source) => {
                return PlayerQuestQueryLoadOutcome::ReturnedFalse(
                    PlayerQuestQueryLoadFailure::Database(source),
                );
            }
        };
        let Some(row) = row else {
            return PlayerQuestQueryLoadOutcome::ReturnedTrue { quest_count: 0 };
        };
        let blob = match row.try_get::<&[u8], _>("QuestData") {
            Ok(Some(blob)) => blob,
            Ok(None) => &[],
            Err(source) => {
                return PlayerQuestQueryLoadOutcome::ReturnedFalse(
                    PlayerQuestQueryLoadFailure::Database(source),
                );
            }
        };
        let quests = load_quest_data(blob);
        for quest in &quests {
            player.add_quest_from_db(quest.quest_id, quest.complete);
        }
        PlayerQuestQueryLoadOutcome::ReturnedTrue {
            quest_count: quests.len(),
        }
    }
}

impl RsPlayerOwner for TiberiusRsPlayer {
    async fn load_player<J, G, WeekDay>(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
        thing_setup: &CThingSetup,
        get_week_day: WeekDay,
        jjc_owner: &mut J,
        goods_owner: &mut G,
        goods_registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> PlayerLoadOutcome
    where
        J: RsJjcSysOwner,
        G: DbGoodsOwner,
        WeekDay: FnMut() -> u16,
    {
        let mut standalone_connection;
        let active_transaction = match active_transaction {
            Some(active_transaction) => active_transaction,
            None => {
                standalone_connection = match self.settings.connect().await {
                    Ok(connection) => connection,
                    Err(source) => {
                        return PlayerLoadOutcome::ReturnedFalse(
                            PlayerLoadFailure::Connection(source),
                        );
                    }
                };
                &mut standalone_connection
            }
        };

        match self
            .load_player_ability_row(
                player,
                Some(&mut *active_transaction),
                thing_setup,
                get_week_day,
            )
            .await
        {
            PlayerAbilityQueryLoadOutcome::ReturnedTrue => {}
            PlayerAbilityQueryLoadOutcome::ReturnedFalse(source) => {
                return PlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Ability(source));
            }
            PlayerAbilityQueryLoadOutcome::BlockedMalformed(source) => {
                return PlayerLoadOutcome::BlockedMissingFact(PlayerLoadBlock::Ability(source));
            }
        }

        match self
            .load_player_quest_data(player, Some(&mut *active_transaction))
            .await
        {
            PlayerQuestQueryLoadOutcome::ReturnedTrue { .. } => {}
            PlayerQuestQueryLoadOutcome::ReturnedFalse(source) => {
                return PlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Quest(source));
            }
        }

        match goods_owner
            .load_goods(
                player,
                Some(&mut *active_transaction),
                goods_registry,
                changed_goods_indices,
                dakong_addon_types,
            )
            .await
        {
            GoodsLoadOutcome::ReturnedTrue { .. } => {}
            GoodsLoadOutcome::ReturnedFalse(source) => {
                return PlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Goods(source));
            }
            GoodsLoadOutcome::BlockedMissingFact(source) => {
                return PlayerLoadOutcome::BlockedMissingFact(PlayerLoadBlock::Goods(source));
            }
        }

        match jjc_owner
            .load_jjc_data(player, Some(&mut *active_transaction))
            .await
        {
            PlayerJjcLoadOutcome::ReturnedTrue { .. } => PlayerLoadOutcome::ReturnedTrue,
            PlayerJjcLoadOutcome::ReturnedFalse(source) => {
                PlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Jjc(source))
            }
        }
    }

    async fn get_player_count_in_db_by_cdkey(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Option<u8> {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::OpenPlayerBaseCount,
                error: RsPlayerSaveError::MissingConnection,
            });
            return None;
        };

        let (account, _, _) = WINDOWS_1251.decode(visible_c_string(account));
        let mut query = Query::new("SELECT ID FROM csl_player_base WHERE Account=@P1");
        query.bind(account.into_owned());
        let rows = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::OpenPlayerBaseCount,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return None;
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::OpenPlayerBaseCount,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return None;
            }
        };

        // ADO GetRecordCount возвращался через `unsigned char`; `0xFF`
        // одновременно был sentinel-ом ошибки внешнего owner-а.
        let count = rows.len() as u8;
        (count != u8::MAX).then_some(count)
    }

    async fn open_player_base_in_db(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Result<Vec<PlayerBaseDatabaseRow>, PlayerBaseLoadFailure> {
        const EQUIPMENT_ID_FIELDS: [&str; 11] = [
            "HELM", "BODY", "GLOV", "BOOT", "WEAPON", "BACK", "HEADGEAR", "FROCK",
            "WING", "MANTEAU", "FAIRY",
        ];
        const EQUIPMENT_LEVEL_FIELDS: [&str; 11] = [
            "HelmLevel",
            "BodyLevel",
            "GlovLevel",
            "BootLevel",
            "WeaponLevel",
            "BackLevel",
            "HEADGEARLevel",
            "FROCKLevel",
            "WINGLevel",
            "MANTEAULevel",
            "FAIRYLevel",
        ];

        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::OpenPlayerBase,
                error: RsPlayerSaveError::MissingConnection,
            });
            return Err(PlayerBaseLoadFailure::MissingConnection);
        };

        let (account, _, _) = WINDOWS_1251.decode(visible_c_string(account));
        let mut query =
            Query::new("SELECT * FROM csl_player_base WHERE Account=@P1 ORDER BY id");
        query.bind(account.into_owned());
        let rows = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::OpenPlayerBase,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return Err(PlayerBaseLoadFailure::Database);
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::OpenPlayerBase,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return Err(PlayerBaseLoadFailure::Database);
            }
        };

        let mut result = Vec::with_capacity(rows.len());
        for (row_index, row) in rows.into_iter().enumerate() {
            macro_rules! required {
                ($value:expr) => {
                    match $value {
                        Ok(value) => value,
                        Err(failure) => {
                            self.notices.push_back(RsPlayerNotice {
                                operation: RsPlayerOperation::OpenPlayerBase,
                                error: RsPlayerSaveError::MalformedPlayerBaseRow,
                            });
                            return Err(failure);
                        }
                    }
                };
            }
            let integer = |column: &'static str| {
                read_ado_integer(&row, column)
                    .map_err(|_| PlayerBaseLoadFailure::Database)?
                    .ok_or(PlayerBaseLoadFailure::MissingRequiredValue {
                        row_index,
                        column,
                    })
            };
            let narrow_u8 = |column: &'static str| {
                let value = integer(column)?;
                u8::try_from(value).map_err(|_| {
                    PlayerBaseLoadFailure::NumericOutsideLegacyRange {
                        row_index,
                        column,
                        value,
                    }
                })
            };
            let narrow_u32 = |column: &'static str| {
                let value = integer(column)?;
                u32::try_from(value).map_err(|_| {
                    PlayerBaseLoadFailure::NumericOutsideLegacyRange {
                        row_index,
                        column,
                        value,
                    }
                })
            };

            let id = required!(narrow_u32("ID"));
            let name = match row.try_get::<&str, _>("Name") {
                Ok(Some(name)) => {
                    let (name, _, _) = WINDOWS_1251.encode(name);
                    visible_c_string(name.as_ref()).to_vec()
                }
                Ok(None) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::OpenPlayerBase,
                        error: RsPlayerSaveError::MalformedPlayerBaseRow,
                    });
                    return Err(PlayerBaseLoadFailure::MissingRequiredValue {
                        row_index,
                        column: "Name",
                    });
                }
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::OpenPlayerBase,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return Err(PlayerBaseLoadFailure::Database);
                }
            };
            let level = required!(narrow_u8("Levels"));
            let occupation = required!(narrow_u8("Occupation"));
            let sex = required!(narrow_u8("Sex"));
            let country = required!(narrow_u8("Country"));
            let head = required!(narrow_u8("HEAD"));

            let mut equipment_ids = [0; 11];
            for (destination, column) in equipment_ids.iter_mut().zip(EQUIPMENT_ID_FIELDS) {
                *destination = required!(narrow_u32(column));
            }
            let mut equipment_levels = [0; 11];
            for (destination, column) in equipment_levels
                .iter_mut()
                .zip(EQUIPMENT_LEVEL_FIELDS)
            {
                *destination = required!(narrow_u8(column));
            }
            let region_value = required!(integer("Region"));
            let region_id = required!(i32::try_from(region_value).map_err(|_| {
                PlayerBaseLoadFailure::NumericOutsideLegacyRange {
                    row_index,
                    column: "Region",
                    value: region_value,
                }
            }));
            result.push(PlayerBaseDatabaseRow {
                id,
                name,
                level,
                occupation,
                sex,
                country,
                head,
                equipment_ids,
                equipment_levels,
                region_id,
            });
        }
        Ok(result)
    }

    async fn get_player_deletion_date(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> i32 {
        if player_id == 0 {
            return 0;
        }
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::GetPlayerDeletionDate,
                error: RsPlayerSaveError::MissingConnection,
            });
            return -1;
        };

        let mut query = Query::new("SELECT DelDate FROM csl_player_base WHERE ID=@P1");
        query.bind(player_id as i32);
        let row = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::GetPlayerDeletionDate,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return -1;
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerDeletionDate,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return -1;
            }
        };
        let Some(row) = row else {
            return 0;
        };
        let deletion_date = match row.try_get::<NaiveDateTime, _>("DelDate") {
            Ok(value) => value,
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerDeletionDate,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return -1;
            }
        };
        let Some(deletion_date) = deletion_date else {
            return 0;
        };
        let Some(midnight) = deletion_date.date().and_hms_opt(0, 0, 0) else {
            return 0;
        };
        let Some(local_midnight) = Local.from_local_datetime(&midnight).earliest() else {
            return 0;
        };
        // Exact `_mktime == -1` нормализовался в ноль до возврата.
        i32::try_from(local_midnight.timestamp()).unwrap_or(0)
    }

    async fn get_player_country_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> u8 {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::GetPlayerCountryById,
                error: RsPlayerSaveError::MissingConnection,
            });
            return 0;
        };
        let mut query = Query::new("SELECT Country FROM CSL_PLAYER_BASE WHERE id=@P1");
        query.bind(player_id as i32);
        let row = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::GetPlayerCountryById,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return 0;
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerCountryById,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return 0;
            }
        };
        let Some(row) = row else {
            return 0;
        };
        match read_ado_integer(&row, "Country") {
            Ok(Some(country)) => country as u8,
            Ok(None) => 0,
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerCountryById,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                0
            }
        }
    }

    async fn get_player_name_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Vec<u8> {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::GetPlayerNameById,
                error: RsPlayerSaveError::MissingConnection,
            });
            return Vec::new();
        };
        let mut query = Query::new("SELECT Name FROM CSL_PLAYER_BASE WHERE id=@P1");
        query.bind(player_id as i32);
        let row = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::GetPlayerNameById,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return Vec::new();
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerNameById,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return Vec::new();
            }
        };
        let Some(row) = row else {
            return Vec::new();
        };
        match row.try_get::<&str, _>("Name") {
            Ok(Some(name)) => {
                let (name, _, _) = WINDOWS_1251.encode(name);
                name.into_owned()
            }
            Ok(None) => Vec::new(),
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerNameById,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                Vec::new()
            }
        }
    }

    async fn validate_player_id_in_cdkey(
        &mut self,
        account: &[u8],
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        if player_id == 0 {
            return false;
        }
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::ValidatePlayerIdInCdkey,
                error: RsPlayerSaveError::MissingConnection,
            });
            return false;
        };

        let (account, _, _) = WINDOWS_1251.decode(visible_c_string(account));
        let mut query = Query::new("SELECT id FROM csl_player_base WHERE account=@P1");
        query.bind(account.into_owned());
        let rows = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::ValidatePlayerIdInCdkey,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return false;
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::ValidatePlayerIdInCdkey,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return false;
            }
        };

        for row in rows {
            let database_id = match read_ado_integer(&row, "ID") {
                Ok(Some(value)) if i32::try_from(value).is_ok() => value as i32 as u32,
                Ok(Some(_)) | Ok(None) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::ValidatePlayerIdInCdkey,
                        error: RsPlayerSaveError::MalformedPlayerBaseRow,
                    });
                    return false;
                }
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::ValidatePlayerIdInCdkey,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return false;
                }
            };
            if database_id == player_id {
                return true;
            }
        }
        false
    }

    async fn is_name_exist(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let player_name = visible_c_string(player_name);
        if player_name.contains(&b'\'') {
            return false;
        }
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::IsNameExist,
                error: RsPlayerSaveError::MissingConnection,
            });
            return false;
        };

        let (player_name, _, _) = WINDOWS_1251.decode(player_name);
        let mut query = Query::new(
            "SELECT TOP (1) 1 AS Present FROM CSL_PLAYER_BASE WHERE LOWER(Name)=LOWER(@P1)",
        );
        query.bind(player_name.into_owned());
        match query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row.is_some(),
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::IsNameExist,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    false
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::IsNameExist,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    async fn get_cd_key(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Vec<u8> {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::GetPlayerId,
                error: RsPlayerSaveError::MissingConnection,
            });
            return Vec::new();
        };

        let (player_name, _, _) = WINDOWS_1251.decode(visible_c_string(player_name));
        let mut player_query = Query::new("SELECT * FROM CSL_PLAYER_BASE WHERE name=@P1");
        player_query.bind(player_name.into_owned());
        let player_row = match player_query.query(&mut *active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::GetPlayerId,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return Vec::new();
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerId,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return Vec::new();
            }
        };
        let Some(player_row) = player_row else {
            return Vec::new();
        };
        let player_id = match read_ado_integer(&player_row, "ID") {
            Ok(Some(value)) => match i32::try_from(value) {
                Ok(value) => value,
                Err(_) => return Vec::new(),
            },
            Ok(None) => return Vec::new(),
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetPlayerId,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return Vec::new();
            }
        };
        if player_id == 0 {
            return Vec::new();
        }

        let mut account_query =
            Query::new("SELECT Account FROM CSL_Player_base WHERE ID=@P1");
        account_query.bind(player_id);
        let account_row = match account_query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::GetCdKey,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return Vec::new();
                }
            },
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetCdKey,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return Vec::new();
            }
        };
        let Some(account_row) = account_row else {
            return Vec::new();
        };
        match account_row.try_get::<&str, _>("Account") {
            Ok(Some(account)) => {
                let (account, _, _) = WINDOWS_1251.encode(account);
                visible_c_string(account.as_ref()).to_vec()
            }
            Ok(None) => Vec::new(),
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::GetCdKey,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                Vec::new()
            }
        }
    }

    async fn stat_ranks(
        &mut self,
        ranks: &mut CPlayerRanks,
        organizing: &COrganizingCtrl,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> PlayerRanksStatOutcome {
        macro_rules! stat_failed {
            ($failure:expr) => {{
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::StatRanks,
                    error: RsPlayerSaveError::PlayerRanksStatFailed,
                });
                PlayerRanksStatOutcome::ReturnedFalse($failure)
            }};
        }

        let Some(maximum_count) = ranks.maximum_count() else {
            return PlayerRanksStatOutcome::BlockedMissingFact(
                PlayerRanksStatBlock::MaximumCountUnknown,
            );
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::StatRanks,
                error: RsPlayerSaveError::MissingConnection,
            });
            return PlayerRanksStatOutcome::ReturnedFalse(
                PlayerRanksStatFailure::MissingConnection,
            );
        };
        let sql = format!(
            "SELECT TOP {} ID, Name, Levels, Occupation\t\t\t\t\tFROM CSL_PLAYER_ABILITY \t\t\t\t\tORDER BY Levels DESC, Exp DESC",
            maximum_count,
        );
        let mut rows = match active_transaction.simple_query(sql).await {
            Ok(rows) => rows,
            Err(source) => {
                return stat_failed!(
                    PlayerRanksStatFailure::Database {
                        row_index: None,
                        source,
                    }
                );
            }
        };
        let mut row_index = 0usize;
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return stat_failed!(
                        PlayerRanksStatFailure::Database {
                            row_index: Some(row_index),
                            source,
                        }
                    );
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };

            macro_rules! required {
                ($type:ty, $column:literal) => {
                    match row.try_get::<$type, _>($column) {
                        Ok(Some(value)) => value,
                        Ok(None) => {
                            return stat_failed!(
                                PlayerRanksStatFailure::MissingRequiredValue {
                                    row_index,
                                    column: $column,
                                }
                            );
                        }
                        Err(source) => {
                            return stat_failed!(
                                PlayerRanksStatFailure::Database {
                                    row_index: Some(row_index),
                                    source,
                                }
                            );
                        }
                    }
                };
            }

            macro_rules! required_integer {
                ($column:literal) => {
                    match read_ado_integer(&row, $column) {
                        Ok(Some(value)) => value,
                        Ok(None) => {
                            return stat_failed!(
                                PlayerRanksStatFailure::MissingRequiredValue {
                                    row_index,
                                    column: $column,
                                }
                            );
                        }
                        Err(source) => {
                            return stat_failed!(
                                PlayerRanksStatFailure::Database {
                                    row_index: Some(row_index),
                                    source,
                                }
                            );
                        }
                    }
                };
            }

            let player_id_value = required_integer!("ID");
            let player_id = match i32::try_from(player_id_value) {
                Ok(player_id) => player_id,
                Err(_) => {
                    return stat_failed!(
                        PlayerRanksStatFailure::NumericOutsideLegacyRange {
                            row_index,
                            column: "ID",
                            value: player_id_value,
                        }
                    );
                }
            };
            let name = required!(&str, "Name");
            let (name, _, _) = WINDOWS_1251.encode(name);
            let name = name
                .split(|byte| *byte == 0)
                .next()
                .unwrap_or_default()
                .to_vec();
            let level_value = required_integer!("Levels");
            let level = match u16::try_from(level_value) {
                Ok(level) => level,
                Err(_) => {
                    return stat_failed!(
                        PlayerRanksStatFailure::NumericOutsideLegacyRange {
                            row_index,
                            column: "Levels",
                            value: level_value,
                        }
                    );
                }
            };
            let occupation_value = required_integer!("Occupation");
            let occupation = match u16::try_from(occupation_value) {
                Ok(occupation) => occupation,
                Err(_) => {
                    return stat_failed!(
                        PlayerRanksStatFailure::NumericOutsideLegacyRange {
                            row_index,
                            column: "Occupation",
                            value: occupation_value,
                        }
                    );
                }
            };
            if let Err(source) =
                ranks.add_rank(organizing, player_id, name, occupation, level)
            {
                return PlayerRanksStatOutcome::BlockedMissingFact(
                    PlayerRanksStatBlock::AddRank { row_index, source },
                );
            }
            row_index += 1;
        }

        PlayerRanksStatOutcome::ReturnedTrue {
            row_count: row_index,
        }
    }

    async fn create_player<J: RsJjcSysOwner, G: DbGoodsOwner>(
        &mut self,
        snapshot: Option<&PlayerCreationSnapshot<'_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> PlayerCreateOutcome {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::Outer,
                error: RsPlayerSaveError::MissingConnection,
            });
            return PlayerCreateOutcome::ReturnedFalse;
        };
        let Some(snapshot) = snapshot else {
            return PlayerCreateOutcome::ReturnedFalse;
        };

        match self
            .create_player_base(&snapshot.base, &mut *active_transaction)
            .await
        {
            PlayerBaseCreateOutcome::Created => {}
            PlayerBaseCreateOutcome::Failed => return PlayerCreateOutcome::ReturnedFalse,
        }
        if !self
            .create_player_abilities(&snapshot.abilities, &mut *active_transaction, jjc_owner)
            .await
        {
            return PlayerCreateOutcome::ReturnedFalse;
        }
        match goods_owner
            .save_goods_filed(&snapshot.goods, Some(active_transaction))
            .await
        {
            GoodsFiledSaveOutcome::ReturnedTrue => PlayerCreateOutcome::ReturnedTrue,
            GoodsFiledSaveOutcome::ReturnedFalse => PlayerCreateOutcome::ReturnedFalse,
            GoodsFiledSaveOutcome::BlockedMissingFact(block) => {
                PlayerCreateOutcome::BlockedMissingFact(PlayerCreateBlock::Goods(block))
            }
        }
    }

    async fn save_player<J: RsJjcSysOwner, G: DbGoodsOwner>(
        &mut self,
        snapshot: Option<&PlayerSaveSnapshot<'_, '_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> PlayerSaveOutcome {
        let Some(snapshot) = snapshot else {
            return PlayerSaveOutcome::ReturnedFalse;
        };
        let Some(active_transaction) = active_transaction else {
            return PlayerSaveOutcome::ReturnedFalse;
        };

        if !self
            .save_player_base(Some(&snapshot.base), Some(&mut *active_transaction))
            .await
        {
            return PlayerSaveOutcome::ReturnedFalse;
        }
        if !self
            .save_player_abilities(
                Some(&snapshot.abilities),
                Some(&mut *active_transaction),
                jjc_owner,
            )
            .await
        {
            return PlayerSaveOutcome::ReturnedFalse;
        }
        if !self
            .save_quest_data(Some(&snapshot.quest), Some(&mut *active_transaction))
            .await
        {
            return PlayerSaveOutcome::ReturnedFalse;
        }

        match goods_owner
            .save_goods_filed(&snapshot.goods, Some(active_transaction))
            .await
        {
            GoodsFiledSaveOutcome::ReturnedTrue => PlayerSaveOutcome::ReturnedTrue,
            GoodsFiledSaveOutcome::ReturnedFalse => PlayerSaveOutcome::ReturnedFalse,
            GoodsFiledSaveOutcome::BlockedMissingFact(block) => {
                PlayerSaveOutcome::BlockedMissingFact(PlayerSaveBlock::Goods(block))
            }
        }
    }

    async fn create_player_base(
        &mut self,
        snapshot: &PlayerCreationBaseSnapshot,
        active_transaction: &mut WorldTdsClient,
    ) -> PlayerBaseCreateOutcome {
        let sql = build_create_player_base_sql(snapshot);

        match execute_batch(active_transaction, sql).await {
            Ok(()) => PlayerBaseCreateOutcome::Created,
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::BaseRow,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                PlayerBaseCreateOutcome::Failed
            }
        }
    }

    async fn save_player_base(
        &mut self,
        snapshot: Option<&PlayerBaseSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(snapshot) = snapshot else {
            return false;
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::SaveBaseRow,
                error: RsPlayerSaveError::MissingConnection,
            });
            return false;
        };

        let (name, _, _) = WINDOWS_1251.decode(visible_c_string(snapshot.name));
        let mut query = Query::new(SAVE_PLAYER_BASE_SQL);
        query.bind(name.into_owned());
        query.bind(snapshot.level);
        query.bind(snapshot.occupation);
        query.bind(snapshot.sex);
        query.bind(snapshot.country);
        query.bind(snapshot.head);
        for equipment_id in snapshot.equipment_ids {
            query.bind(i64::from(equipment_id));
        }
        for equipment_level in snapshot.equipment_levels {
            query.bind(equipment_level);
        }
        query.bind(snapshot.region_id);
        query.bind(snapshot.id);

        let update_result = match query.query(active_transaction).await {
            Ok(stream) => stream.into_row().await,
            Err(error) => Err(error),
        };
        match update_result {
            Ok(Some(row)) if row.get::<i32, _>(0).is_some_and(|updated| updated != 0) => true,
            Ok(_) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::SaveBaseRow,
                    error: RsPlayerSaveError::MissingBaseRow,
                });
                false
            }
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::SaveBaseRow,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    async fn create_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: &PlayerAbilityCreationSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
        jjc_owner: &mut J,
    ) -> bool {
        if let Err(error) = insert_player_ability(snapshot, active_transaction).await {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::AbilityRow,
                error: RsPlayerSaveError::Database(error.into()),
            });
            return false;
        }

        if !jjc_owner.save_jjc_data(&snapshot.jjc).await {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::AbilityRow,
                error: RsPlayerSaveError::JjcSaveFailed,
            });
            return false;
        }

        true
    }

    async fn save_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: Option<&PlayerAbilitySaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
    ) -> bool {
        let Some(snapshot) = snapshot else {
            return false;
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::SaveAbilityRow,
                error: RsPlayerSaveError::MissingConnection,
            });
            return false;
        };

        let now = Local::now();
        let save_time = format!(
            "{}-{}-{} {}:{}:{}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        match update_player_ability(snapshot, save_time.as_bytes(), active_transaction).await {
            Ok(true) => {}
            Ok(false) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::SaveAbilityRow,
                    error: RsPlayerSaveError::MissingAbilityRow,
                });
                return false;
            }
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::SaveAbilityRow,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return false;
            }
        }

        if !jjc_owner.save_jjc_data(&snapshot.ability.jjc).await {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::SaveAbilityRow,
                error: RsPlayerSaveError::JjcSaveFailed,
            });
            return false;
        }

        true
    }

    async fn save_quest_data(
        &mut self,
        snapshot: Option<&PlayerQuestSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(snapshot) = snapshot else {
            return false;
        };
        let quest_data = encode_player_quest_data(snapshot);
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::SaveQuestData,
                error: RsPlayerSaveError::MissingConnection,
            });
            return false;
        };

        if let Err(error) =
            upsert_player_quest_data(snapshot.player_id, quest_data, active_transaction).await
        {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::SaveQuestData,
                error: RsPlayerSaveError::Database(error.into()),
            });
            return false;
        }

        true
    }

    async fn restore_player(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::Restore,
                error: RsPlayerSaveError::MissingConnection,
            });
            return false;
        };

        let mut query = Query::new("UPDATE csl_player_base SET DelDate = NULL WHERE ID=@P1");
        query.bind(player_id as i32);
        match query.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::Restore,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    async fn delete_player(
        &mut self,
        player_id: u32,
        deletion_time: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> PlayerDeleteOutcome {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::Delete,
                error: RsPlayerSaveError::MissingConnection,
            });
            return PlayerDeleteOutcome::ReturnedFalse;
        };
        if deletion_time < 0 {
            return PlayerDeleteOutcome::BlockedMissingFact(PlayerDeleteTimeBlock {
                deletion_time,
            });
        }
        let Some(local_time) = Local.timestamp_opt(i64::from(deletion_time), 0).single() else {
            return PlayerDeleteOutcome::BlockedMissingFact(PlayerDeleteTimeBlock {
                deletion_time,
            });
        };
        let deletion_date = format!(
            "{}-{}-{}",
            local_time.year(),
            local_time.month(),
            local_time.day()
        );
        let mut query = Query::new("UPDATE csl_player_base SET DelDate = @P1 WHERE ID=@P2");
        query.bind(deletion_date);
        query.bind(player_id as i32);
        match query.execute(active_transaction).await {
            Ok(_) => PlayerDeleteOutcome::ReturnedTrue,
            Err(error) => {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::Delete,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                PlayerDeleteOutcome::ReturnedFalse
            }
        }
    }

    async fn load_honor_ranks<S: HonorRanksLoadSink>(
        &mut self,
        sink: &mut S,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> HonorRanksLoadOutcome {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsPlayerNotice {
                operation: RsPlayerOperation::HonorRanksLoad,
                error: RsPlayerSaveError::MissingConnection,
            });
            return HonorRanksLoadOutcome::ReturnedFalse(
                HonorRanksLoadFailure::MissingConnection,
            );
        };

        let now = Local::now();
        let history_date = HonorRanksCopyTimeSnapshot::from_legacy_fields([
            u16::try_from(now.year()).expect("год SYSTEMTIME помещается в u16"),
            u16::try_from(now.month()).expect("месяц помещается в u16"),
            u16::try_from(now.weekday().num_days_from_sunday())
                .expect("день недели помещается в u16"),
            u16::try_from(now.day()).expect("день помещается в u16"),
            u16::try_from(now.hour()).expect("час помещается в u16"),
            u16::try_from(now.minute()).expect("минута помещается в u16"),
            u16::try_from(now.second()).expect("секунда помещается в u16"),
            u16::try_from(now.timestamp_subsec_millis()).expect("миллисекунды помещаются в u16"),
        ])
        .expect("chrono::Local возвращает календарно валидный SYSTEMTIME");
        let current_date = history_date.next_day();

        for (period, date) in [
            (HonorRanksSavePeriod::History, history_date),
            (HonorRanksSavePeriod::Current, current_date),
        ] {
            let row = match query_honor_ranks_row(active_transaction, date).await {
                Ok(Some(row)) => row,
                Ok(None) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::HonorRanksLoad,
                        error: RsPlayerSaveError::MissingHonorRanksRow { period },
                    });
                    return HonorRanksLoadOutcome::ReturnedFalse(
                        HonorRanksLoadFailure::MissingRow { period },
                    );
                }
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::HonorRanksLoad,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return HonorRanksLoadOutcome::ReturnedFalse(
                        HonorRanksLoadFailure::Database { period },
                    );
                }
            };

            sink.clear_honor_ranks_period(period);
            for rank_type in HonorRanksType::ALL {
                let blob = match row.try_get::<&[u8], _>(rank_type.column_name()) {
                    Ok(blob) => blob,
                    Err(error) => {
                        self.notices.push_back(RsPlayerNotice {
                            operation: RsPlayerOperation::HonorRanksLoad,
                            error: RsPlayerSaveError::Database(error.into()),
                        });
                        return HonorRanksLoadOutcome::ReturnedFalse(
                            HonorRanksLoadFailure::Database { period },
                        );
                    }
                };
                let lists = match decode_honor_ranks_blob(rank_type, blob) {
                    Ok(lists) => lists,
                    Err(source) => {
                        return HonorRanksLoadOutcome::BlockedMissingFact(HonorRanksLoadBlock {
                            period,
                            source,
                        });
                    }
                };
                sink.replace_honor_ranks_type(period, rank_type, lists);
            }
        }

        HonorRanksLoadOutcome::ReturnedTrue
    }

    async fn insert_honor_ranks(
        &mut self,
        snapshot: &HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(active_transaction) = active_transaction else {
            return false;
        };

        let copy_time = snapshot.copy_time();
        for date in [copy_time, copy_time.next_day()] {
            if let Err(error) = ensure_honor_ranks_row(active_transaction, date).await {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::HonorRanksInsert,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return false;
            }
        }

        true
    }

    fn save_honor_ranks_by_type<S: HonorRanksFieldSink>(
        &mut self,
        snapshot: &mut HonorRanksDbDataSnapshot,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
        sink: &mut S,
    ) -> HonorRanksByTypeSaveOutcome<S::Error> {
        let lists = snapshot.lists_mut(period, rank_type);
        let total_entries = match lists
            .iter()
            .try_fold(0usize, |total, list| total.checked_add(list.len()))
        {
            Some(total) => total,
            None => {
                return HonorRanksByTypeSaveOutcome::BlockedMissingFact(HonorRanksBlobBlock {
                    total_entries: usize::MAX,
                });
            }
        };
        let Some(blob_size) = u32::try_from(total_entries)
            .ok()
            .and_then(|count| count.checked_mul(HONOR_RANK_ENTRY_SIZE as u32))
            .and_then(|size| size.checked_add(HONOR_RANK_BLOB_HEADER_SIZE as u32))
        else {
            return HonorRanksByTypeSaveOutcome::BlockedMissingFact(HonorRanksBlobBlock {
                total_entries,
            });
        };

        let mut blob = Vec::with_capacity(blob_size as usize);
        for list in lists {
            let count = u32::try_from(list.len())
                .expect("общий проверенный размер включает каждый country-list");
            blob.extend_from_slice(&count.to_le_bytes());
            for entry in list.iter().copied() {
                blob.extend_from_slice(&entry.legacy_bytes());
            }
            list.clear();
        }

        match sink.put_honor_ranks_field(rank_type, blob) {
            Ok(()) => HonorRanksByTypeSaveOutcome::Saved,
            Err(error) => HonorRanksByTypeSaveOutcome::FieldFailed(error),
        }
    }

    async fn save_honor_ranks(
        &mut self,
        snapshot: &mut HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> HonorRanksSaveOutcome {
        let Some(active_transaction) = active_transaction else {
            return HonorRanksSaveOutcome::ReturnedFalse;
        };
        let copy_time = snapshot.copy_time();

        for (period, date) in [
            (HonorRanksSavePeriod::History, copy_time),
            (HonorRanksSavePeriod::Current, copy_time.next_day()),
        ] {
            let row_exists = honor_ranks_row_exists(active_transaction, date).await;
            match row_exists {
                Ok(true) => {}
                Ok(false) => return HonorRanksSaveOutcome::ReturnedFalse,
                Err(error) => {
                    self.notices.push_back(RsPlayerNotice {
                        operation: RsPlayerOperation::HonorRanksSave,
                        error: RsPlayerSaveError::Database(error.into()),
                    });
                    return HonorRanksSaveOutcome::ReturnedFalse;
                }
            }

            let mut fields = CollectedHonorRanksFields::default();
            for rank_type in HonorRanksType::ALL {
                match self.save_honor_ranks_by_type(snapshot, period, rank_type, &mut fields) {
                    HonorRanksByTypeSaveOutcome::Saved => {}
                    HonorRanksByTypeSaveOutcome::FieldFailed(never) => infallible(never),
                    HonorRanksByTypeSaveOutcome::BlockedMissingFact(blob) => {
                        return HonorRanksSaveOutcome::BlockedMissingFact(HonorRanksSaveBlock {
                            period,
                            rank_type,
                            blob,
                        });
                    }
                }
            }

            if let Err(error) =
                update_honor_ranks_row(active_transaction, date, fields.fields).await
            {
                self.notices.push_back(RsPlayerNotice {
                    operation: RsPlayerOperation::HonorRanksSave,
                    error: RsPlayerSaveError::Database(error.into()),
                });
                return HonorRanksSaveOutcome::ReturnedFalse;
            }
        }

        HonorRanksSaveOutcome::ReturnedTrue
    }

    fn pop_notice(&mut self) -> Option<RsPlayerNotice> {
        self.notices.pop_front()
    }
}

async fn query_honor_ranks_row(
    active_transaction: &mut WorldTdsClient,
    date: HonorRanksCopyTimeSnapshot,
) -> Result<Option<tiberius::Row>, tiberius::error::Error> {
    let date = date.legacy_sql_date();
    let mut select_sql = String::with_capacity(HONOR_RANKS_SELECT_PREFIX.len() + date.len() + 1);
    select_sql.push_str(HONOR_RANKS_SELECT_PREFIX);
    select_sql.push_str(&date);
    select_sql.push('\'');
    active_transaction
        .simple_query(select_sql)
        .await?
        .into_row()
        .await
}

async fn ensure_honor_ranks_row(
    active_transaction: &mut WorldTdsClient,
    date: HonorRanksCopyTimeSnapshot,
) -> Result<(), tiberius::error::Error> {
    if honor_ranks_row_exists(active_transaction, date).await? {
        return Ok(());
    }

    let date = date.legacy_sql_date();
    let mut insert_sql = String::with_capacity(HONOR_RANKS_INSERT_PREFIX.len() + date.len() + 2);
    insert_sql.push_str(HONOR_RANKS_INSERT_PREFIX);
    insert_sql.push_str(&date);
    insert_sql.push_str("')");
    active_transaction
        .simple_query(insert_sql)
        .await?
        .into_results()
        .await?;
    Ok(())
}

async fn honor_ranks_row_exists(
    active_transaction: &mut WorldTdsClient,
    date: HonorRanksCopyTimeSnapshot,
) -> Result<bool, tiberius::error::Error> {
    let date = date.legacy_sql_date();
    let mut select_sql = String::with_capacity(HONOR_RANKS_SELECT_PREFIX.len() + date.len() + 1);
    select_sql.push_str(HONOR_RANKS_SELECT_PREFIX);
    select_sql.push_str(&date);
    select_sql.push('\'');
    Ok(active_transaction
        .simple_query(select_sql)
        .await?
        .into_row()
        .await?
        .is_some())
}

async fn update_honor_ranks_row(
    active_transaction: &mut WorldTdsClient,
    date: HonorRanksCopyTimeSnapshot,
    fields: Vec<(HonorRanksType, Vec<u8>)>,
) -> Result<(), tiberius::error::Error> {
    debug_assert_eq!(fields.len(), HONOR_RANK_TYPE_COUNT);
    let date = date.legacy_sql_date();
    let mut sql = String::from(HONOR_RANKS_UPDATE_PREFIX);
    for (index, (rank_type, _)) in fields.iter().enumerate() {
        if index != 0 {
            sql.push(',');
        }
        sql.push('[');
        sql.push_str(rank_type.column_name());
        sql.push_str("] = @P");
        sql.push_str(&(index + 1).to_string());
    }
    sql.push_str(" WHERE SortDate = '");
    sql.push_str(&date);
    sql.push('\'');

    let mut query = Query::new(sql);
    for (_, blob) in fields {
        query.bind(blob);
    }
    query.execute(active_transaction).await?;
    Ok(())
}

async fn insert_player_ability(
    snapshot: &PlayerAbilityCreationSnapshot<'_>,
    active_transaction: &mut WorldTdsClient,
) -> Result<(), tiberius::error::Error> {
    let scalar = player_ability_scalar_assignments(&snapshot.scalar);
    let binary = collect_player_ability_binary_fields(snapshot);
    let mut sql = String::from("INSERT INTO CSL_PLAYER_ABILITY (");

    for (index, field) in scalar.iter().map(|item| item.field).enumerate() {
        if index != 0 {
            sql.push(',');
        }
        sql.push('[');
        sql.push_str(field.column_name());
        sql.push(']');
    }
    for (field, _) in &binary {
        sql.push(',');
        sql.push('[');
        sql.push_str(field.column_name());
        sql.push(']');
    }
    sql.push_str(") VALUES (");
    for index in 1..=(scalar.len() + binary.len()) {
        if index != 1 {
            sql.push(',');
        }
        sql.push_str("@P");
        sql.push_str(&index.to_string());
    }
    sql.push(')');

    let mut query = Query::new(sql);
    for assignment in scalar {
        bind_player_ability_scalar(&mut query, assignment.value);
    }
    for (_, bytes) in binary {
        query.bind(bytes);
    }
    query.execute(active_transaction).await?;
    Ok(())
}

async fn update_player_ability(
    snapshot: &PlayerAbilitySaveSnapshot<'_>,
    save_time: &[u8],
    active_transaction: &mut WorldTdsClient,
) -> Result<bool, tiberius::error::Error> {
    let scalar = player_ability_save_scalar_assignments(snapshot, save_time);
    let binary = collect_player_ability_binary_fields(&snapshot.ability);
    let assignment_count = scalar.len() + binary.len();
    let id_parameter = assignment_count + 1;
    let mut sql = String::from("IF EXISTS (SELECT TOP 1 ID FROM CSL_PLAYER_ABILITY WHERE ID = @P");
    sql.push_str(&id_parameter.to_string());
    sql.push_str(") BEGIN UPDATE TOP (1) CSL_PLAYER_ABILITY SET ");

    for (index, assignment) in scalar.iter().enumerate() {
        if index != 0 {
            sql.push(',');
        }
        sql.push('[');
        sql.push_str(assignment.field.column_name());
        sql.push_str("] = @P");
        sql.push_str(&(index + 1).to_string());
    }
    for (index, (field, _)) in binary.iter().enumerate() {
        sql.push(',');
        sql.push('[');
        sql.push_str(field.column_name());
        sql.push_str("] = @P");
        sql.push_str(&(scalar.len() + index + 1).to_string());
    }
    sql.push_str(" WHERE ID = @P");
    sql.push_str(&id_parameter.to_string());
    sql.push_str("; SELECT CAST(@@ROWCOUNT AS int) AS UpdatedRows END ELSE SELECT CAST(0 AS int) AS UpdatedRows");

    let mut query = Query::new(sql);
    for assignment in scalar {
        bind_player_ability_scalar(&mut query, assignment.value);
    }
    for (_, bytes) in binary {
        query.bind(bytes);
    }
    query.bind(snapshot.ability.scalar.id);

    Ok(query
        .query(active_transaction)
        .await?
        .into_row()
        .await?
        .and_then(|row| row.get::<i32, _>(0))
        .is_some_and(|updated| updated != 0))
}

async fn upsert_player_quest_data(
    player_id: i32,
    quest_data: Vec<u8>,
    active_transaction: &mut WorldTdsClient,
) -> Result<(), tiberius::error::Error> {
    let mut query = Query::new(
        "IF EXISTS (SELECT TOP 1 PlayerID FROM CSL_PLAYER_QUEST_EX WHERE PlayerID = @P1) \
         BEGIN UPDATE TOP (1) CSL_PLAYER_QUEST_EX SET QuestData = @P2 WHERE PlayerID = @P1 END \
         ELSE BEGIN INSERT INTO CSL_PLAYER_QUEST_EX (PlayerID, QuestData) VALUES (@P1, @P2) END",
    );
    query.bind(player_id);
    query.bind(quest_data);
    query.execute(active_transaction).await?;
    Ok(())
}

fn bind_player_ability_scalar(query: &mut Query<'static>, value: PlayerAbilityScalarValue<'_>) {
    match value {
        PlayerAbilityScalarValue::I4(value) | PlayerAbilityScalarValue::Int(value) => {
            query.bind(value);
        }
        PlayerAbilityScalarValue::Ui4(value) => query.bind(i64::from(value)),
        PlayerAbilityScalarValue::Ui2(value) => query.bind(i32::from(value)),
        PlayerAbilityScalarValue::Ui1(value) => query.bind(value),
        PlayerAbilityScalarValue::VariantBool(value) => query.bind(value),
        PlayerAbilityScalarValue::R4(value) => query.bind(value),
        PlayerAbilityScalarValue::BStr(bytes) => {
            let (decoded, _, _) = WINDOWS_1251.decode(bytes);
            query.bind(decoded.into_owned());
        }
    }
}

fn collect_player_ability_binary_fields(
    snapshot: &PlayerAbilityCreationSnapshot<'_>,
) -> Vec<(PlayerAbilityBinaryField, Vec<u8>)> {
    let mut sink = CollectedPlayerAbilityBinaryFields::default();
    save_hot_key_field(snapshot.hot_keys, &mut sink).unwrap_or_else(infallible);
    save_skill_field(snapshot.skills, &mut sink).unwrap_or_else(infallible);
    save_script_flag(&snapshot.script_flag, &mut sink).unwrap_or_else(infallible);
    save_state_field(snapshot.ex_states, &mut sink).unwrap_or_else(infallible);
    save_friend_field(snapshot.friend_names, &mut sink).unwrap_or_else(infallible);
    save_ci_qing_field(snapshot.ci_qing_ids, &mut sink).unwrap_or_else(infallible);
    save_thing_field(snapshot.things, &mut sink).unwrap_or_else(infallible);
    sink.fields
}

fn infallible(never: Infallible) {
    match never {}
}

fn build_create_player_base_sql(snapshot: &PlayerCreationBaseSnapshot) -> String {
    let name = visible_c_string(&snapshot.name);
    let account = visible_c_string(&snapshot.account);
    let mut sql = Vec::with_capacity(CREATE_PLAYER_BASE_PREFIX.len() + name.len() + account.len());
    sql.extend_from_slice(CREATE_PLAYER_BASE_PREFIX);
    append_i32(&mut sql, snapshot.id);
    sql.extend_from_slice(b",N'");
    sql.extend_from_slice(name);
    sql.extend_from_slice(b"','");
    sql.extend_from_slice(account);
    sql.extend_from_slice(b"',");
    append_u8(&mut sql, snapshot.level);
    sql.push(b',');
    append_u8(&mut sql, snapshot.occupation);
    sql.push(b',');
    append_u8(&mut sql, snapshot.sex);
    sql.push(b',');
    append_u8(&mut sql, snapshot.country);
    sql.push(b',');
    append_u8(&mut sql, snapshot.head);
    sql.extend_from_slice(VALUE_GROUP_BREAK);
    append_u32_as_i32_group(&mut sql, &snapshot.equipment_ids[..6]);
    sql.extend_from_slice(VALUE_GROUP_BREAK);
    append_u32_as_i32_group(&mut sql, &snapshot.equipment_ids[6..]);
    sql.extend_from_slice(VALUE_GROUP_BREAK);
    append_u8_group(&mut sql, &snapshot.equipment_levels[..6]);
    sql.extend_from_slice(VALUE_GROUP_BREAK);
    append_u8_group(&mut sql, &snapshot.equipment_levels[6..]);
    sql.extend_from_slice(VALUE_GROUP_BREAK);
    append_i32(&mut sql, snapshot.region_id);
    sql.push(b')');

    let (decoded, _, _) = WINDOWS_1251.decode(&sql);
    decoded.into_owned()
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn append_i32(sql: &mut Vec<u8>, value: i32) {
    sql.extend_from_slice(value.to_string().as_bytes());
}

fn append_u8(sql: &mut Vec<u8>, value: u8) {
    sql.extend_from_slice(value.to_string().as_bytes());
}

fn append_u32_as_i32_group(sql: &mut Vec<u8>, values: &[u32]) {
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            sql.push(b',');
        }
        append_i32(sql, *value as i32);
    }
}

fn append_u8_group(sql: &mut Vec<u8>, values: &[u8]) {
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            sql.push(b',');
        }
        append_u8(sql, *value);
    }
}

async fn execute_batch(
    connection: &mut WorldTdsClient,
    sql: String,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp



// ============================================================================
// FUNCTION: CRsPlayer::GetPlayerCountInDBbyCdkey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2416
// RVA: 0x00100C40
// ADDRESS: 00500c40
// PROTOTYPE: uchar __thiscall GetPlayerCountInDBbyCdkey(char * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00500e06
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2464
// RVA: 0x00100E06
// ADDRESS: 00500e06
// PROTOTYPE: undefined Catch@00500e06()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00500e4a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2475
// RVA: 0x00100E4A
// ADDRESS: 00500e4a
// PROTOTYPE: undefined FUN_00500e4a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::GetPlayerDeletionDate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2479
// RVA: 0x00100EA0
// ADDRESS: 00500ea0
// PROTOTYPE: long __thiscall GetPlayerDeletionDate(uint param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005011d4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2545
// RVA: 0x001011D4
// ADDRESS: 005011d4
// PROTOTYPE: undefined Catch@005011d4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00501238
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2556
// RVA: 0x00101238
// ADDRESS: 00501238
// PROTOTYPE: undefined FUN_00501238()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::ValidatePlayerIDInCdkey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2668
// RVA: 0x00101350
// ADDRESS: 00501350
// PROTOTYPE: bool __thiscall ValidatePlayerIDInCdkey(char * param_1, uint param_2, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_3)
//
// Реализация находится в `TiberiusRsPlayer::validate_player_id_in_cdkey`
// выше: ordered row scan, ID bit-pattern и false/catch semantics сохранены;
// parameter binding заменяет только `_sprintf` и ADO/COM plumbing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005015fb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2726
// RVA: 0x001015FB
// ADDRESS: 005015fb
// PROTOTYPE: undefined Catch@005015fb()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00501648
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2737
// RVA: 0x00101648
// ADDRESS: 00501648
// PROTOTYPE: undefined FUN_00501648()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::IsNameExist
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:641
// RVA: 0x00101B70
// ADDRESS: 00501b70
// PROTOTYPE: bool __thiscall IsNameExist(char * param_1)
//
// IMPLEMENTED_OWNER: `RsPlayerOwner::is_name_exist` использует parameterized
// TDS query, сохраняя case-insensitive lookup и false при quote/DB failure без
// исходных SQL injection, stack buffers и COM plumbing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00501d5a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:722
// RVA: 0x00101D5A
// ADDRESS: 00501d5a
// PROTOTYPE: undefined Catch@00501d5a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00501ddd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:736
// RVA: 0x00101DDD
// ADDRESS: 00501ddd
// PROTOTYPE: undefined FUN_00501ddd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::GetPlayerCountryByID
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:768
// RVA: 0x00101E30
// ADDRESS: 00501e30
// PROTOTYPE: void __thiscall GetPlayerCountryByID(ulong param_1, uchar * param_2)
//
// IMPLEMENTED_OWNER: `RsPlayerOwner::get_player_country_by_id` находится выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00501ffb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:789
// RVA: 0x00101FFB
// ADDRESS: 00501ffb
// PROTOTYPE: undefined Catch@00501ffb()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00502017
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:792
// RVA: 0x00102017
// ADDRESS: 00502017
// PROTOTYPE: undefined FUN_00502017()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::GetPlayerID
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:797
// RVA: 0x00102080
// ADDRESS: 00502080
// PROTOTYPE: long __thiscall GetPlayerID(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005022ce
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:829
// RVA: 0x001022CE
// ADDRESS: 005022ce
// PROTOTYPE: undefined Catch@005022ce()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0050231c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:834
// RVA: 0x0010231C
// ADDRESS: 0050231c
// PROTOTYPE: undefined FUN_0050231c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// FUNCTION: CRsPlayer::LoadHotKeyField
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2010
// RVA: 0x00103D40
// ADDRESS: 00503d40
// PROTOTYPE: bool __thiscall LoadHotKeyField(CPlayer * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// IMPLEMENTED_OWNER: `load_hot_key_field` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00503f92
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2033
// RVA: 0x00103F92
// ADDRESS: 00503f92
// PROTOTYPE: undefined Catch@00503f92()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadStateField
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2167
// RVA: 0x00103FC0
// ADDRESS: 00503fc0
// PROTOTYPE: bool __thiscall LoadStateField(CPlayer * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// IMPLEMENTED_OWNER: `load_state_field` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00504204
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2187
// RVA: 0x00104204
// ADDRESS: 00504204
// PROTOTYPE: undefined Catch@00504204()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadScriptFlag
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2331
// RVA: 0x00104470
// ADDRESS: 00504470
// PROTOTYPE: bool __thiscall LoadScriptFlag(CPlayer * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// IMPLEMENTED_OWNER: `load_script_flag` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0050470f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2355
// RVA: 0x0010470F
// ADDRESS: 0050470f
// PROTOTYPE: undefined Catch@0050470f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::GetPlayerCountInCdkey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2399
// RVA: 0x00104750
// ADDRESS: 00504750
// PROTOTYPE: uchar __thiscall GetPlayerCountInCdkey(char * param_1)
//
// IMPLEMENTED_OWNER: default `RsPlayerOwner::get_player_count_in_cdkey` выше
// сохраняет DB `0xFF` sentinel и wrapping addition, получая прежний singleton-
// count явным аргументом.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadQuestData
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2741
// RVA: 0x00104790
// ADDRESS: 00504790
// PROTOTYPE: bool __thiscall LoadQuestData(CPlayer * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// IMPLEMENTED_OWNER: `TiberiusRsPlayer::load_player_quest_data` и binary
// `load_quest_data` выше; Tiberius заменяет ADO, а трёхбайтовые записи,
// игнорирование неполного хвоста и успешный EOF сохранены.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00504b6a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2774
// RVA: 0x00104B6A
// ADDRESS: 00504b6a
// PROTOTYPE: undefined Catch@00504b6a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00504bba
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2780
// RVA: 0x00104BBA
// ADDRESS: 00504bba
// PROTOTYPE: undefined FUN_00504bba()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::GetPlayerNameByID
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:739
// RVA: 0x00105980
// ADDRESS: 00505980
// PROTOTYPE: void __thiscall GetPlayerNameByID(ulong param_1, char * param_2)
//
// IMPLEMENTED_OWNER: `RsPlayerOwner::get_player_name_by_id` находится выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00505b83
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:760
// RVA: 0x00105B83
// ADDRESS: 00505b83
// PROTOTYPE: undefined Catch@00505b83()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00505b9f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:763
// RVA: 0x00105B9F
// ADDRESS: 00505b9f
// PROTOTYPE: undefined FUN_00505b9f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::StatRanks
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:1561
// RVA: 0x00109570
// ADDRESS: 00509570
// PROTOTYPE: bool __thiscall StatRanks(CPlayerRanks * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Реализовано выше потоковым Tiberius-чтением с exact live-prefix publication.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00509a72
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:1611
// RVA: 0x00109A72
// ADDRESS: 00509a72
// PROTOTYPE: undefined Catch@00509a72()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00509aaf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:1622
// RVA: 0x00109AAF
// ADDRESS: 00509aaf
// PROTOTYPE: undefined FUN_00509aaf()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::OpenPlayerBaseInDB
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:160
// RVA: 0x0010D2D0
// ADDRESS: 0050d2d0
// PROTOTYPE: bool __thiscall OpenPlayerBaseInDB(char * param_1, CMessage * param_2, long * param_3, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0050e98a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:375
// RVA: 0x0010E98A
// ADDRESS: 0050e98a
// PROTOTYPE: undefined Catch@0050e98a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0050e9f5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:386
// RVA: 0x0010E9F5
// ADDRESS: 0050e9f5
// PROTOTYPE: undefined FUN_0050e9f5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::GetCDKey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:837
// RVA: 0x0010EA20
// ADDRESS: 0050ea20
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __thiscall GetCDKey(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0050ec8a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:863
// RVA: 0x0010EC8A
// ADDRESS: 0050ec8a
// PROTOTYPE: undefined Catch@0050ec8a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0050ecab
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:866
// RVA: 0x0010ECAB
// ADDRESS: 0050ecab
// PROTOTYPE: undefined FUN_0050ecab()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadCiQingField
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3254
// RVA: 0x0010EFF0
// ADDRESS: 0050eff0
// PROTOTYPE: bool __thiscall LoadCiQingField(CPlayer * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// IMPLEMENTED_OWNER: `load_ci_qing_field` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0050f24d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3278
// RVA: 0x0010F24D
// ADDRESS: 0050f24d
// PROTOTYPE: undefined Catch@0050f24d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::OpenPlayerBaseInMem
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:44
// RVA: 0x0010F280
// ADDRESS: 0050f280
// PROTOTYPE: bool __thiscall OpenPlayerBaseInMem(char * param_1, CMessage * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::OpenPlayerBase
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:389
// RVA: 0x0010F750
// ADDRESS: 0050f750
// PROTOTYPE: bool __thiscall OpenPlayerBase(char * param_1, CMessage * param_2, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadSkillField
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2041
// RVA: 0x0010F8E0
// ADDRESS: 0050f8e0
// PROTOTYPE: bool __thiscall LoadSkillField(CPlayer * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// IMPLEMENTED_OWNER: `load_skill_field` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0050fb61
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2076
// RVA: 0x0010FB61
// ADDRESS: 0050fb61
// PROTOTYPE: undefined Catch@0050fb61()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadHonorRanksByType
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2963
// RVA: 0x0010FB90
// ADDRESS: 0050fb90
// PROTOTYPE: bool __thiscall LoadHonorRanksByType(bool param_1, int param_2, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_3)
//
// Реализовано выше как bounds-checked decoder одного typed DB-field.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0051022b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3023
// RVA: 0x0011022B
// ADDRESS: 0051022b
// PROTOTYPE: undefined Catch@0051022b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadThingField
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3332
// RVA: 0x00110260
// ADDRESS: 00510260
// PROTOTYPE: bool __thiscall LoadThingField(CPlayer * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2, ulong param_3)
//
// IMPLEMENTED_OWNER: binary `load_thing_field` выше и reached daily-ветвь
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005104eb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3361
// RVA: 0x001104EB
// ADDRESS: 005104eb
// PROTOTYPE: undefined Catch@005104eb()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::DbLetTingUpdate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3369
// RVA: 0x00110520
// ADDRESS: 00510520
// PROTOTYPE: uint __stdcall DbLetTingUpdate(void * param_1)
//
// IMPLEMENTED_OWNER: `TiberiusRsPlayer::db_lei_ting_update` выше сохраняет
// отдельное connection, worker-time daily list, per-row порядок, partial
// commit и kind-specific bit mask; `ID` заменяет только hidden ADO row key.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00510f00
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3450
// RVA: 0x00110F00
// ADDRESS: 00510f00
// PROTOTYPE: undefined Catch@00510f00()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00510f4e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3464
// RVA: 0x00110F4E
// ADDRESS: 00510f4e
// PROTOTYPE: undefined FUN_00510f4e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::ResetAllLeitingInDB
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:3468
// RVA: 0x00110F70
// ADDRESS: 00510f70
// PROTOTYPE: bool __thiscall ResetAllLeitingInDB(ulong param_1, long param_2)
//
// IMPLEMENTED_OWNER: `WorldLeiTingResetWorker::dispatch` создаёт один
// detached Rust thread с owned payload/config и не переносит Win32 HANDLE leak.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadFriendField
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2240
// RVA: 0x00111050
// ADDRESS: 00511050
// PROTOTYPE: bool __thiscall LoadFriendField(CPlayer * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// IMPLEMENTED_OWNER: `load_friend_field` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00511373
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2283
// RVA: 0x00111373
// ADDRESS: 00511373
// PROTOTYPE: undefined Catch@00511373()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0051138c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2287
// RVA: 0x0011138C
// ADDRESS: 0051138c
// PROTOTYPE: undefined FUN_0051138c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadHonorRanks
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2870
// RVA: 0x001113B0
// ADDRESS: 005113b0
// PROTOTYPE: bool __thiscall LoadHonorRanks(_com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_1)
//
// Реализовано выше с exact today/tomorrow и последовательной публикацией.
// Exact `0x005116FA` возвращает true только после второго полного прохода;
// error/catch хвост `0x005117D0` возвращает false.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0051177f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2955
// RVA: 0x0011177F
// ADDRESS: 0051177f
// PROTOTYPE: undefined Catch@0051177f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_005117a9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:2960
// RVA: 0x001117A9
// ADDRESS: 005117a9
// PROTOTYPE: undefined FUN_005117a9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::LoadPlayer
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:1135
// RVA: 0x001117F0
// ADDRESS: 005117f0
// PROTOTYPE: bool __thiscall LoadPlayer(CPlayer * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// IMPLEMENTED_OWNER: `RsPlayerOwner::load_player` и
// `TiberiusPlayerLoadData` выше; caller-connection переиспользуется, а null
// connection открывается один раз через `WorldDatabaseSettings::connect`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00514e5d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:1545
// RVA: 0x00114E5D
// ADDRESS: 00514e5d
// PROTOTYPE: undefined Catch@00514e5d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsPlayer::GetPlayerData
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsplayer.cpp:432
// RVA: 0x00114EA0
// ADDRESS: 00514ea0
// PROTOTYPE: bool __thiscall GetPlayerData(char * param_1, uint param_2, ulong param_3, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_4)
//
// Реализация распределена между `player_select`, clone/queue owners и
// `CGame::route_loaded_player`; observable order прямого пути сохранён.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
