//! DB-save orchestration исторического WorldServer из `savedb.cpp`.
//!
//! Статус setup-ID, VarData, New Character, Restore Character, Delete Character,
//! Delete Union, Delete Faction, Save Faction, Save Union, Save Region, Save
//! HonorRanks, двух GodsBattle, EnemyFactions, Update Country Data и Save
//! Charactor LoadDetails Data, Save Charactor Data и финального отчёта
//! transaction-участков `DoSaveData` RVA `0x0001C610`, а также folded
//! copy/destructor `CPlayerRanks::tagRank` RVA `0x0001B920/0x0001B7F0` —
//! `IMPLEMENTED`; остальные фазы владельца ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\savedb.cpp`.
//!
//! После успешного `OpenCn` исходный участок по `0x0041C6DF..0x0041C9C0`
//! вызывает `BeginTran`, затем `CRsSetup::SavePlayerID` и, только если тот
//! вернул `true`, `CRsSetup::SaveLeaveWorldID` на одном connection. Два успеха
//! ведут к `CommitTran` и безусловному `ShowSaveInfo("++Save PlayerID
//! SUCCESS!")`; первый `false` ведёт к `RollbackTran` и безусловному
//! `AddLogText("--Save PlayerID FAILED!")`. Поэтому `None` у результата второго
//! UPDATE означает именно пропущенный вызов, а не DB-ошибку.
//!
//! Exact wrapper-ы `BeginTran` `0x004EBFE0`, `CommitTran` `0x004EC090` и
//! `RollbackTran` `0x004EC140` поглощают `_com_error`, печатают собственный
//! `PrintErr` и возвращают `0/false`. `DoSaveData` не проверяет эти результаты:
//! ошибка begin не запрещает последующие UPDATE, а ошибка commit не отменяет
//! исходный success-log. Это наблюдаемая странность сохранена отчётом, а не
//! исправлена fail-closed поведением. Вложенный catch по `0x0041C9C5` проверял
//! ненулевой ADO transaction level только для неожиданного `_com_error`; все
//! ожидаемые TDS-ошибки Rust проходят явными значениями и не требуют SEH.
//!
//! Следующий участок `0x0041C7A5..0x0041C860` независимо вызывает новый
//! `BeginTran`, затем `CVariableList::SaveVarData` на том же connection.
//! `true` ведёт к `CommitTran` и безусловному
//! `ShowSaveInfo("++Save VarDate SUCCESS!")`; `false` — к `RollbackTran` и
//! `AddLogText("--Save VarDate FAILED!")`. Результаты begin/commit/rollback
//! снова игнорируются. Catch `0x0041CA1F` откатывал только при ненулевом ADO
//! transaction level и писал `**Save VarDate ABNORMAL!`; достигнутые DB-ошибки
//! Rust уже выражены `GenVarSaveOutcome::Failed`, поэтому не требуют SEH.
//!
//! Единственный `BlockedMissingFact` принадлежит неизвестному переполнению
//! исходного `char[1024]`. Для него EXE не задаёт commit, rollback или log;
//! orchestration поэтому не придумывает ни одну ветку и возвращает блокировку
//! отдельно. Connection после неё нельзя передавать следующим save-фазам до
//! решения владельца: это локальная граница реконструкции, а не runtime
//! fallback.
//!
//! Закреплённый `tiberius 0.12.3` выполняет транзакции поддерживаемым им
//! `simple_query("BEGIN TRAN"/"COMMIT"/"ROLLBACK")`; `into_results` полностью
//! потребляет ответ и сохраняет серверные TDS transaction descriptor/env-change
//! перед следующей командой. Параметризованные UPDATE остаются у настоящего
//! DB-owner `CRsSetup`. Mutable borrow одного `WorldTdsClient` сохраняет одно
//! исходное соединение, а caller-owned snapshot заменяет защищённый
//! `CGame::m_stDBData` без глобального scratch-state.
//!
//! Начальная граница `0x0041C63C..0x0041C6BB` также восстановлена отдельно.
//! Один `timeGetTime` сначала попадает и в local `saved_start_tick`, и в
//! `g_dwThisSaveStartTime`; четыре счётчика и ранний player-map count после
//! этого инициализируются нулями. Затем исходник создаёт ADO connection и
//! вызывает `OpenCn`. Rust использует закреплённый Tiberius/Tokio connector с
//! теми же byte-exact World DB setup-полями и без Windows COM.
//!
//! Только успешный `OpenCn` ставит `g_bIsSavingData = true`, после чего пишет
//! `Save Variables Start...` и переходит к setup-ID. Ошибка пишет
//! `Connect To DB FAILED!`, не меняет прежний save-флаг и входит в уже
//! восстановленный finalizer как `ConnectionOpenFailed`; его четыре счётчика
//! остаются нулями. Typed `begin_do_save_data` возвращает открытую connection
//! либо error вместе с точным final snapshot и не публикует credential values.
//!
//! Frozen handoff из `CGame::tagDBData` теперь реализован для setup ID, всех
//! пяти player-фаз, Delete/Save Union/Faction, Region, EnemyFactions и Country,
//! а из отдельного `CHonorRanks::m_stDBData` — для HonorRanks. Эксклюзивная
//! save-сессия выражает внешний `g_CriticalSectionSaveThread`, не удерживая
//! player-list lock во время DB I/O; success cleanup каждой связанной фазы
//! применяется в исходной точке. DB-последовательность собрана, но общий
//! lifecycle ещё не замкнут; `SaveThreadFunc` не запускается, сервис и MSSQL не
//! поднимаются.
//! Все DB/container-фазы от setup-ID до конца Save Character теперь связаны
//! одной последовательностью поверх двух локальных участков. Все phase logs до
//! конца Save Character публикуются синхронно. `do_save_data_lifecycle` также
//! связывает connection begin/cleanup и общий final tail, но не создаёт runtime
//! entry и не запускает `SaveThreadFunc`.
//!
//! New Character начинается с list-order обхода `liDBCreationPlayer`. Null
//! `CPlayer*` только пишет `**Create New Charactor NULL Pointer!!!!!!!!!!!!!!!!`,
//! остаётся в list и не вызывает begin. Для ненулевого элемента owner сохраняет
//! результат `BeginTran`, вызывает готовый `CRsPlayer::CreatePlayer`, а его
//! `false` превращает в `E_FAIL` и catch. Если begin вернул ненулевой
//! transaction level, catch пытается rollback; при `0` rollback пропускается.
//! В обоих случаях пишется `--Create New Charactor FAILED.`, node остаётся и
//! обход продолжается.
//!
//! Exact `0x0041C860..0x0041CB3D` имеет статус `VERIFIED_DISASSEMBLY` для
//! повреждённого EH-разбиением raw. `CreatePlayer == true` безусловно вызывает
//! commit, не проверяет его результат, пишет
//! `Create New Charactor SUCCESS : %s.%s.%d`, затем под
//! `g_CriticalSectionSavePlayerList` deleting-destructor-ом освобождает player,
//! обнуляет pointer, unlink/delete-ит только текущий list-node, уменьшает size,
//! снимает lock и продолжает со следующим node. Begin-ошибка не запрещает ни
//! `CreatePlayer`, ни последующий commit/success/remove путь.
//!
//! `save_new_characters_from_world_snapshot` теперь применяет эту судьбу к
//! реальному `VecDeque<Box<CPlayer>>`: `Commit` немедленно уничтожает owner и
//! удаляет текущий node, `Failure` сохраняет node и продвигает cursor.
//! Safe-state не содержит null pointer. Если вложенный owner останавливается
//! на неизвестном исходе старого UB, orchestration не придумывает
//! commit/rollback/log и запрещает продолжать lifecycle на этом connection.
//!
//! Exact `0x0041C874..0x0041C890` отдельно подтверждает: итоговый `Charactor
//! CREATED` получает ранний `_Mysize` creation-list до traversal, а не число
//! successful commit/remove. На Rust-хосте непредставимый 32-битный count
//! блокируется до первого entry-вызова.
//!
//! Restore Charactor обходит `lDBRestorePlayer` в list-order. Exact
//! `0x0041CB3D..0x0041CCEA` имеет статус `VERIFIED_DISASSEMBLY`: каждый `u32`
//! сначала получает отдельный begin, затем готовый `RestorePlayer`. Его `false`
//! бросает `E_FAIL`; catch пытается rollback только при ненулевом сохранённом
//! begin-level, безусловно пишет `--Restore Charactor FAILED!`, сохраняет
//! текущий node и продолжает со следующим ID.
//!
//! Успешный restore безусловно вызывает commit, игнорирует его результат и
//! пишет `++Restore Charactor SUCCESS : %d`. Только затем critical section
//! охватывает чтение next, unlink/delete текущего list-node и уменьшение size;
//! после unlock обход продолжается с сохранённого next. В отличие от New
//! Character, node содержит только ID и owned player не уничтожается.
//! Begin-ошибка опять не препятствует restore и последующему success/commit.
//!
//! `save_restore_characters_from_world_snapshot` применяет отчёт к реальному
//! `VecDeque<u32>`: `Commit` удаляет только текущий ID-node, `Failure` сохраняет
//! его и продвигает cursor. Одинаковая доказанная catch-механика использует
//! общий `FailedTransactionFinish`, но create/restore outcomes и сообщения
//! остаются раздельными.
//! Ранний `_Mysize` по `0x0041CB4A..0x0041CB58` сохраняется для итогового
//! `CANCEL DELETE`; это размер restore-list, а не success-счётчик.
//!
//! Delete Charactor обходит 8-байтные значения `liDBDeletionPlayer` в
//! list-order. Exact `0x0041CCEA..0x0041CE9E` имеет статус
//! `VERIFIED_DISASSEMBLY`: `nPlayerID: u32` лежит в node по `+8`, а
//! `tDelDate: long` по `+0xC`. Каждый элемент получает отдельный begin и затем
//! готовый `DeletePlayer` на том же connection. Его доказанный `false` бросает
//! `E_FAIL`; catch делает rollback только при ненулевом begin-level, пишет
//! `--Delete Charactor FAILED!`, сохраняет node и переходит к следующему.
//!
//! `ReturnedTrue` безусловно вызывает commit, игнорирует его результат и пишет
//! `++Delete Charactor SUCCESS : %d` с тем же player ID. Затем critical section
//! охватывает чтение next, unlink/delete текущего node и уменьшение size;
//! обход продолжается после unlock. Begin-ошибка не препятствует delete и
//! success/commit/remove. `save_delete_characters_from_world_snapshot`
//! немедленно удаляет только successful record, сохраняет failure для retry и
//! останавливает всю фазу на отрицательной time-границе.
//! Ранний `_Mysize` deletion-list по `0x0041CCF7..0x0041CD05` становится
//! итоговым `SIGN DELETE`. Таким образом, две итоговые delete-цифры принадлежат
//! разным входным очередям и не выводятся из результата одного DB-owner-а.
//!
//! Вложенный `PlayerDeleteOutcome::BlockedMissingFact` означает отрицательный
//! timestamp, где оригинал не возвращался из-за null-разыменования. Для него
//! EXE не задаёт commit, rollback, log или судьбу node, поэтому orchestration
//! сохраняет block и запрещает продолжать lifecycle на connection. Следующая
//! raw-фаза начинается с Delete Union по `0x0041CE9E`.
//!
//! Delete Union сохраняет начальный `_Mysize`, затем обходит signed `long` ID
//! из `listDeleteUnions` в list-order. Exact `0x0041CE9E..0x0041D01B` имеет
//! статус `VERIFIED_DISASSEMBLY`: каждый ID получает отдельные begin,
//! `DelConfederation` и commit, но bool DB-owner-а намеренно игнорируется.
//! Ошибки begin/commit wrapper-ов также не меняют обход и финальный success-log.
//!
//! Отдельный catch `0x0041CF8A` относится только к неожиданному `_com_error`:
//! он откатывает при ненулевом begin-level, пишет
//! `**Delte Union ABNORMAL!` и всё равно продолжает со следующим node.
//! Достигнутые Tiberius-ошибки уже выражены значениями begin/delete/commit и не
//! являются Rust unwind, поэтому typed normal-path не выдаёт их за abnormal.
//!
//! После обхода исходник без critical section отделяет список, зануляет size и
//! удаляет каждый текущий node, затем пишет `++Delte %d Unions SUCCESS!` с
//! размером, захваченным до цикла. `DeleteUnionClear` передаёт будущему list-
//! owner-у эту обязанность отдельно от entry-отчётов; сохранённый count не
//! выводится из длины Rust-среза, потому что исходник читал live list без lock.
//! Следующая восстановленная фаза начинается с Delete Faction по
//! `0x0041D01B`.
//!
//! Delete Faction отдельно захватывает `_Mysize`, затем обходит signed `long`
//! ID из `listDeleteFactions` в list-order. Потерянный raw-аргумент точечно
//! подтверждён по `0x0041D01B..0x0041D105`: `0x0041D0C7` читает ID текущего
//! node по `+8` и передаёт его в готовый `CRsFaction::DelFaction`. Его bool
//! намеренно не проверяется; commit выполняется после каждого вызова даже при
//! `false`, а ошибки begin/commit wrapper-ов не меняют обход.
//!
//! Отдельный catch `0x0041D10A` относится только к неожиданному `_com_error`:
//! он откатывает при ненулевом begin-level, пишет
//! `**Delete Faction ABNORMAL!` и возвращается в продолжение traversal.
//! Достигнутые Tiberius-ошибки являются явными значениями entry-отчёта, а не
//! Rust unwind, поэтому normal-path не выдаёт их за abnormal.
//!
//! После traversal исходник без critical section отделяет список, зануляет
//! size, удаляет все текущие nodes и пишет `++ Delte %d Factions SUCCESS!` с
//! исходным count. `DeleteFactionClear` отдельно передаёт эту обязанность
//! будущему list-owner-у; элементы с false/error не сохраняются для retry.
//! Save Faction сохраняет начальный `_Mysize`, затем обходит
//! `list<CFaction*> listSaveFactions` в live list-order. Каждый node независимо
//! получает begin, готовый `CRsFaction::SaveFaction` и commit. Bool dispatcher-а
//! намеренно игнорируется; null snapshot и leaf DB-failure поэтому не отменяют
//! commit и cleanup текущего node.
//!
//! Exact `0x0041D1A8..0x0041D356` имеет статус `VERIFIED_DISASSEMBLY` для
//! искажённой EH-разбиением судьбы элемента. Normal path и catch
//! `0x0041D284` сходятся к `0x0041D2C9`: сначала unlink/delete текущего list-
//! node, затем при non-null virtual deleting destructor `CFaction*`, затем
//! переход к уже сохранённому next. Catch перед этим откатывает только при
//! ненулевом begin-level и пишет `**Save Faction Data ABNORMAL!`.
//!
//! Достигнутые Tiberius-ошибки уже являются явными begin/save/commit
//! результатами, а не Rust unwind, поэтому не выдаются за abnormal catch. После
//! traversal исходник пишет `++Save %d Faction Data SUCCESS!` с начальным
//! count и отдельно очищает все оставшиеся list-nodes без вызова destructor для
//! их pointer-values. `SaveFactionFinalClear` передаёт эту точную обязанность
//! будущему concurrent list-owner-у.
//!
//! Уже локализованный member/leave-word overread остаётся
//! `BlockedMissingFact`: оригинал не возвращался из неизвестного UB, поэтому для
//! текущего node не назначены commit, unlink, destructor либо продолжение на
//! том же connection.
//!
//! `save_factions_from_world_snapshot` материализует property-группу только
//! для dirty-bit `1`, а canonical Goods War count получает через точный
//! `COrganizingCtrl::GetpFactionById`-эквивалент. После любого обычного bool
//! текущий `VecDeque<Box<CFaction>>` node сразу удаляется вместе с owner-ом;
//! projection/owner block сохраняет текущий и все последующие nodes.
//!
//! Save Union отдельно сохраняет начальный `_Mysize`, затем обходит
//! `list<CUnion*> listSaveUnions` в live list-order. Exact
//! `0x0041D356..0x0041D516` имеет статус `VERIFIED_DISASSEMBLY`: текущая
//! save-копия берётся из list-node по `+8`, получает отдельный begin и
//! передаётся готовому `CRsUnion::SaveConfederation`; его bool не проверяется,
//! после обоих обычных значений безусловно вызывается commit.
//!
//! Catch `0x0041D444` откатывает только при ненулевом begin-level, пишет
//! `**Save Union Data ABNORMAL!` и по `0x0041D47D` входит в тот же cleanup, что
//! normal path. Cleanup по `0x0041D489..0x0041D4D2` сохраняет next, удаляет
//! текущий list-node, уменьшает size, затем virtual-уничтожает non-null
//! `CUnion*` и продолжает обход. После traversal исходник пишет
//! `++Save %d Union Data SUCCESS!` с начальным count и отдельно очищает только
//! оставшиеся list-nodes без destructor pointer-values.
//!
//! Достигнутые Tiberius-ошибки представлены begin/save/commit полями normal-
//! отчёта и не выдаются за `_com_error` catch. Известные overflow/overread/null
//! границы `SaveConfederation` остаются `BlockedMissingFact`: оригинал не
//! возвращался из такого UB, поэтому текущему union-node не назначаются commit,
//! cleanup или продолжение на том же connection.
//!
//! `save_unions_from_world_snapshot` заимствует поля прямо из frozen
//! `Box<CUnion>`, выполняет тот же begin/save/commit и после обычного bool
//! немедленно удаляет node вместе с save-копией. Block оставляет текущий front
//! и хвост очереди без придуманного cleanup.
//!
//! `do_save_data_through_unions` теперь вызывает эти владельцы строго в
//! подтверждённом порядке: setup-ID, VarData, New, Restore, Delete Character,
//! Delete Union, Delete Faction, Save Faction и Save Union. Каждая
//! Неизвестный исход старого UB останавливает передачу connection следующей фазе;
//! отчёты и уже применённые container cleanup остаются доступны caller-у.
//!
//! Save Region отдельно сохраняет начальный `_Mysize`, затем без lock обходит
//! `list<CWorldRegion*> listRegionParam` в live list-order. Exact
//! `0x0041D516..0x0041D6A3` имеет статус `VERIFIED_DISASSEMBLY`: pointer текущей
//! save-копии читается из node по `+8`, получает отдельный begin и передаётся
//! готовому `CRsRegion::Save`. Его bool не проверяется, поэтому commit следует
//! после обоих обычных значений и после явных Tiberius-ошибок.
//!
//! В отличие от faction/union, cleanup по `0x0041D64A` не unlink-ит текущий
//! node: он только вызывает virtual deleting destructor с флагом `1` для
//! non-null `CWorldRegion*`, затем берёт next из всё ещё живого node. Catch
//! `0x0041D5FF` перед тем же cleanup откатывает лишь при ненулевом begin-level
//! и пишет `**Save Region Data ABNORMAL.`. После traversal исходник пишет
//! `++Save %ld Region Data SUCCESS.` с начальным count, отделяет весь список,
//! зануляет size и удаляет nodes без повторного уничтожения pointer-values.
//!
//! `SaveRegionEntryCleanup` и `SaveRegionFinalClear` раздельно передают эти
//! обязанности будущему list-owner-у. Достигнутые Rust-ошибки остаются явными
//! полями normal-path и не выдаются за старый `_com_error` catch. Следующая
//! Save HonorRanks по `0x0041D6A3` отдельно начинает транзакцию, затем на
//! одном connection вызывает `CRsPlayer::InsertHonorRanks` и только после его
//! `true` — `CRsPlayer::SaveHonorRanks`. Два успеха ведут к commit и точной
//! строке `+Success to save HonorRanks!!!l`; первый `false` превращался в
//! `_com_error`, catch откатывал только при ненулевом сохранённом begin-level и
//! писал `Save HonorRanks start ABNORMAL`, после чего переходил к Save
//! GodsBattle. Ошибка begin не запрещает оба owner-вызова, а ошибка commit не
//! отменяет success-log.
//!
//! Согласованные raw owner-а и catch вместе с уже доказанными ADO-wrapper-ами
//! полностью задают это ветвление, поэтому повторный reverse не выполнялся.
//! `HonorRanksSaveDisposition::BlockedMissingFact` принадлежит неизвестному
//! переполнению старого blob-size: для него оригинал не возвращался из owner-а,
//! следовательно orchestration не назначает commit, rollback, log либо переход
//! к первой GodsBattle-фазе. Обычные исходы продолжаются с неё по
//! `0x0041D856`.
//!
//! Две GodsBattle-фазы используют один уже открытый World DB connection, но
//! каждая начинает и завершает собственную транзакцию. Первая по
//! `0x0041D856..0x0041D943` вызывает caller-connection
//! `CRSGodsBattle::SaveFacitonXYD`; вторая по
//! `0x0041D943..0x0041DA23` независимо вызывает caller-connection
//! `CRSGodsBattle::SaveNpcFaction`. No-argument overload второго owner-а и
//! следующая Save EnemyFactions phase сюда не входят.
//!
//! Оба `false` превращались в `_com_error`. Catch-и `0x0041DC69` и
//! `0x0041DC97` откатывали только при ненулевом сохранённом begin-level, писали
//! разные GodsBattle abnormal-сообщения и продолжали соответственно со второй
//! фазой либо с EnemyFactions. `true` безусловно вызывал commit и success-log;
//! ошибка begin не запрещала owner-вызов, а ошибка commit не отменяла success.
//! Tiberius возвращает begin/commit/rollback ошибки отдельными полями, тогда
//! как bool owner-а буквально выбирает исходную success/failure ветку.
//!
//! Caller-owned `GodsBattleFactionXydSnapshot` и ordered NPC slice заменяют
//! только чтение уже созданных `CShape/CGodsBattleConf` из process singleton.
//! Оба owner-а получают последовательные mutable reborrow одного connection;
//! никаких дополнительных DB-соединений, общей транзакции либо cleanup между
//! двумя фазами Rust не вводит. Raw обеих фаз, их catch-и и дублированные
//! compiler-continuation удалены; следующая восстановленная EnemyFactions
//! фаза начинается по `0x0041DA23`.
//!
//! EnemyFactions отдельно пытается begin, копирует pointer-list
//! `CGame::m_stDBData.listEnemyFactions` по значению и вызывает готовый
//! `CRsEnemyFactions::SaveAllEnemyFactions` на том же connection. Его bool
//! исходно не проверяется: и `true`, и обычный `false` безусловно ведут к
//! commit, после чего одинаково уничтожаются исходные значения и nodes.
//! Ошибка begin не пропускает owner, а ошибка commit не отменяет cleanup и
//! точную строку `++Save EnemyFactions SUCCESS.`.
//!
//! Raw ошибочно показывал ранний return после первого `operator_delete`.
//! Exact cleanup `0x0041DAE2..0x0041DB48` имеет статус
//! `VERIFIED_DISASSEMBLY`: первый полный проход удаляет каждый non-null
//! `tagEnemyFaction*` в list-order и зануляет `_Myval`; второй отделяет список,
//! зануляет size и удаляет все прежние list-node-ы. Только после этого по
//! `0x0041DB48` пишется success-log и выполняется переход к Update Country Data
//! по `0x0041DB5C`. После ответа reverse прекращён.
//!
//! Catch `0x0041DCC5` относился только к неожиданному `_com_error`: он
//! безусловно пытался rollback, писал `**Save EnemyFactions ABNORMAL.` и
//! переходил к Country-фазе без normal cleanup. Достигнутые Tiberius-ошибки
//! уже являются явными begin/save/commit результатами и не выдаются за Rust
//! unwind. Локальный null-entry `BlockedMissingFact` владельца означает
//! неизвестное разыменование после DELETE/предыдущих INSERT: ему не назначены
//! commit, cleanup, log либо продолжение на connection.
//!
//! `EnemyFactionsFinalCleanup` возвращает будущему list-owner-у точную normal-
//! обязанность destroy-values-then-clear-nodes. Raw фазы удалён из трёх
//! continuation-копий вместе с её catch; следующий доказанный участок
//! начинается с Update Country Data по `0x0041DB5C`.
//!
//! Update Country Data сохраняет начальный `_Mysize`, затем обходит
//! `list<CCountry*> ltDBCountrys` в live list-order. Каждый nullable snapshot
//! получает отдельные begin, готовый `CDBCountry::Save` и безусловный commit;
//! возвращённый bool и ошибки wrapper-ов не меняют normal cleanup.
//!
//! `VERIFIED_DISASSEMBLY`: normal-путь по `0x0041DC36` и catch
//! `0x0041DCF2..0x0041DD2A` сходятся к cleanup `0x0041DD2B..0x0041DD80`.
//! Сначала сохраняется next, unlink/delete-ится текущий list-node и уменьшается
//! size, затем virtual deleting destructor уничтожает non-null `CCountry*`,
//! после чего traversal продолжается. Catch перед этим откатывает только при
//! ненулевом begin-level и пишет `UpDate Country ABNORMAL....!`.
//!
//! После полного traversal исходник пишет `++Update %ld Country Data
//! SUCCESS.` с начальным count, отделяет список, зануляет size и удаляет только
//! оставшиеся nodes без повторного destructor pointer-values по
//! `0x0041DD85..0x0041DDC2`. Переход к следующей Save Charactor LoadDetails
//! Data phase начинается по `0x0041DDC4`. Typed normal-path не выдаёт явные
//! Tiberius-ошибки за старый `_com_error` unwind. Три дублирующие compiler-
//! continuation и заменённый country-префикс `FUN_0041dd37` удалены; его
//! незаменённый Character-хвост сохранён ниже.

//! Save Charactor LoadDetails Data по `0x0041DDC4..0x0041DFB7` берёт исходный
//! `CGame::tagDBData::mDBPlayer` типа
//! `map<unsigned int, CPlayer*>` и обходит его в порядке unsigned keys. Null
//! player пишет `** Save LoadDetails NULL Pointer!!!!!!`, не открывает
//! транзакцию и не меняет node. Ни normal path, ни catch не стирают и не
//! удаляют entries: та же map затем повторно обходится Save Charactor Data
//! phase.
//!
//! PDB задаёт `bUseOldSaveLargessWay` как `bool` по `tagSetup+0x2C8`,
//! `CBaseObject::m_lID` как signed `long` по `CPlayer+8`, а обе перегрузки
//! `CLargess::SaveLoadDetails` принимают `std::string&` CD-key и `long`
//! player ID. Exact calls `0x0041DE39..0x0041DE4B` и
//! `0x0041DEB8..0x0041DEF9` подтверждают `CPlayer+0x72C`, ID по `+8` и
//! игнорирование обоих bool-результатов.
//!
//! При `bUseOldSaveLargessWay == true` вызывается старая self-opening
//! перегрузка без transaction wrapper. Новая ветка для каждого non-null player
//! вызывает begin, caller-connection перегрузку и commit; ошибки wrapper-ов и
//! bool owner-а не меняют порядок. Catch `0x0041DF05` откатывал только при
//! ненулевом begin-level, писал `-- Save LoadDetails FAILED!` и продолжал map
//! traversal. Достигнутые Tiberius-ошибки уже выражены значениями
//! wrapper/owner и не выдаются за `_com_error`. Catch `0x0041E0CE` относится
//! уже к следующей Save Charactor Data phase.
//!
//! Единственный `BlockedMissingFact` по-прежнему принадлежит `_sprintf`
//! overflow внутри готового `CLargess`: для новой ветки begin уже мог
//! выполниться, но EXE не задаёт commit, rollback, log, переход к next либо
//! судьбу map. Typed phase поэтому останавливает lifecycle точно на этой
//! границе.

//! Save Charactor Data по `0x0041DFB7..0x0041E1CF` повторно обходит тот же
//! `CGame::tagDBData::mDBPlayer` в unsigned key-order. PDB подтверждает
//! `map<unsigned int, CPlayer*>` по `tagDBData+0x30`; его `_Mysize` снимается
//! один раз по `0x0041DDD6` ещё перед LoadDetails и позже выводится как число
//! `Charactor SAVED`, даже если часть values null либо завершилась отказом.
//! `BTreeMap<u32, _>` сохраняет порядок, а отдельный `logged_count` — именно
//! этот ранний 32-битный snapshot.
//!
//! Каждый non-null player получает `BeginTran`, `CPlayer::SaveData` и только
//! при `true` — `CommitTran`; `false` искусственно входит в catch. Catch
//! `0x0041E0CE..0x0041E107` откатывает только при ненулевом begin-level, пишет
//! `-- Save Charactor FAILED!` и возвращается к общему continuation. Null value
//! не открывает транзакцию, пишет `** Save Charactor NULL Pointer!!!!!!` и
//! остаётся в map. Ошибки самих transaction-wrapper-ов поглощались внутри
//! `CMyAdoBase` и не меняли дальнейшее ветвление.
//!
//! Exact `0x0041E048..0x0041E15D` имеет статус `VERIFIED_DISASSEMBLY`:
//! результат `SaveData` сохраняется до commit/catch, success-log получает
//! account по `CPlayer+0x72C`, inherited name и signed ID, затем под
//! `g_CriticalSectionSavePlayerList` выполняются virtual delete player,
//! зануление pointer-value и `map::erase(current)`, возвращающий next. Поэтому
//! cleanup выполняется при сохранённом `true` даже после ошибки commit;
//! сохранённый `false` и null оставляют node/value и переходят к обычному next.
//! Конец map достигает следующего close/report tail по `0x0041E1CF`.
//!
//! Rust-фаза вызывает готовый `CPlayer::save_data`, сохраняет begin/commit/
//! rollback ошибки typed-значениями и применяет cleanup к реальному map-owner-
//! у: после `Saved` уничтожает player и стирает entry, failure сохраняет его.
//! Предшествующий LoadDetails wrapper обходит тот же map без мутации и передаёт
//! ранний `logged_count` в Save Character. Projection либо `PlayerSaveBlock`
//! останавливают lifecycle без придуманного rollback, cleanup или next.
//! Заменённые Character-хвост `FUN_0041DD37`, catch `0x0041E0CE` и continuation
//! `FUN_0041E113` удалены.
//!
//! Все достигнутые фазовые `AddLogText`/`ShowSaveInfo` теперь материализуются
//! как ordered `SaveDataLogEvent` в соответствующем phase-report. Статические
//! start/final/failure payload-ы сохраняют исходные опечатки, пробелы и
//! punctuation; подробные player success payload-ы строятся до уничтожения
//! owner-а. Exact call-sites `0x0041C96E..0x0041CA78` и
//! `0x0041E08C..0x0041E0C4` имеют статус `VERIFIED_DISASSEMBLY`: оба передают
//! `account`, затем унаследованное `name`, затем signed player ID.
//!
//! Четыре нераскрытые raw-символом GodsBattle строки точечно прочитаны из exact
//! EXE по `0x0053F8BC/0x0053F88C/0x0053F7F8/0x0053F7D8` и хранятся byte-exact,
//! включая CP936 bytes `A1 F0/A1 A3/A1 F9`. Найденный runtime-журнал того же
//! WorldServer независимо подтверждает полный normal-порядок статических строк
//! от `Save Variables Start...` до `Save Data end ...`; персональные runtime-
//! значения из него не перенесены. После ответов reverse прекращён.
//!
//! `Debug` фазовых отчётов показывает только target и длину event payload,
//! поэтому account/name не размножаются диагностикой. Полная phase-цепь от
//! общего `Save Variables Start...` до последнего Save Character entry передаёт
//! один mutable publisher синхронно по точным позициям. Logger block
//! возвращается до следующего DB/container cleanup, а batch replay отсутствует.
//! Владеющий состоянием `SaveDataLogPublisher` уже связывает один event с точным target:
//! он хранит общий `WorldLogTextOwner`, gate, timeout и три callback-а, полностью
//! завершает вызов до возврата и сводит оба logger-result к редактированному
//! typed outcome без строки с account/name. Batch-публикации у него нет.
//!
//! `do_save_data_after_unions` связывает Region, HonorRanks, обе GodsBattle,
//! EnemyFactions, Country, LoadDetails и Save Character в этом точном порядке.
//! Обычный `false` продолжает цепь ровно у тех owner-ов, чьи catch/normal paths
//! это доказывают; typed block сохраняет текущий report и не достигает next.
//! `do_save_data_phases` последовательно соединяет этот suffix с уже готовым
//! префиксом и только после полного normal traversal возвращает четыре ранних
//! counters. `do_save_data_lifecycle` поверх него выполняет begin, точный
//! connection-log порядок, обе ступени finalizer-а и monitoring send.
//!
//! Финальный хвост `0x0041E1CF..0x0041E324` теперь также восстановлен. После
//! успешного traversal исходник выполняет `CloseCn`, пишет `Save Data end ...`
//! и вызывает `ReleaseCn`, который внутри повторно вызывает `CloseCn`, а затем
//! освобождает COM pointer. Ветка неуспешного `OpenCn` пишет
//! `Connect To DB FAILED!` и сразу вызывает `ReleaseCn`; обе ветви сходятся на
//! общем `timeGetTime`. Tiberius `Client::close`/`Drop` заменяет только
//! ADO/COM lifetime, а `SaveDataConnectionFinish` сохраняет различимый порядок
//! connection/log-эффектов внутри единого typed orchestration.
//!
//! Итоговый summary получает четыре исходных счётчика и ранний `_Mysize`; на
//! open-failure все пять значений равны нулю. Время вычисляется как wrapping
//! разность двух `DWORD`, затем один `GetLocalTime` обновляет
//! `g_stLastSaveTime`, `g_dwThisSaveStartTime` обнуляется и
//! `g_dwLastSaveTick` получает конечный tick. `rustix::CLOCK_BOOTTIME` и
//! `chrono::Local` заменяют только Windows clock API; caller снимает tick после
//! connection cleanup, а local time — после итогового summary-log, как в
//! исходнике.
//!
//! Мониторинговая C-string строится byte-exact с Windows-1251 server name,
//! шестью полями `SYSTEMTIME`, signed `%d`-проекциями elapsed/log count и
//! отправляется как `0x1FE08` с типом `-2`, server ID и битами `dwNumber`.
//! Rust строит monitoring C-string в owned buffer и не переносит переполнение
//! старого `_sprintf(char[128])`. Send-result исходно игнорируется, после чего
//! флаг обязательно сбрасывается.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;

use chrono::{Datelike, Local, Timelike};
use rustix::time::{ClockId, clock_gettime};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::dbaccess::worlddb::dbcountry::{CountrySaveSnapshot, DbCountryOwner};
use crate::dbaccess::worlddb::dbgoods::DbGoodsOwner;
use crate::dbaccess::worlddb::largess::{LargessOwner, SaveLoadDetailsOutcome};
use crate::dbaccess::worlddb::rsenemyfactions::{
    EnemyFactionNullEntryBlock, EnemyFactionSaveSnapshot, EnemyFactionsSaveOutcome,
    RsEnemyFactionsOwner,
};
use crate::dbaccess::worlddb::rsfaction::{
    FactionSaveBlock, FactionSaveOutcome, FactionSaveProjectionBlock, FactionSaveSnapshot,
    RsFactionOwner,
};
use crate::dbaccess::worlddb::rsgenvar::{GenVarSaveOutcome, RsGenVarOwner};
use crate::dbaccess::worlddb::rsgodsbattle::{
    GodsBattleFactionXydSnapshot, GodsBattleNpcFactionSnapshot, GodsBattleSaveOperation,
    RsGodsBattleOwner,
};
use crate::dbaccess::worlddb::rsjjcsys::RsJjcSysOwner;
use crate::dbaccess::worlddb::rsplayer::{
    HonorRanksDbDataSnapshot, HonorRanksSaveBlock, HonorRanksSaveOutcome, PlayerCreateBlock,
    PlayerCreateOutcome, PlayerCreationSnapshot, PlayerDeleteOutcome, PlayerDeleteTimeBlock,
    PlayerSaveBlock, PlayerSaveOutcome, PlayerSaveSnapshot, RsPlayerOwner,
};
use crate::dbaccess::worlddb::rsregion::{RegionSaveSnapshot, RsRegionOwner};
use crate::dbaccess::worlddb::rssetup::{RsSetupOwner, WorldDatabaseSettings, WorldTdsClient};
use crate::dbaccess::worlddb::rsunion::{
    RsUnionOwner, UnionSaveBlock, UnionSaveOutcome, UnionSaveSnapshot,
};
use crate::worldserver::appworld::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::appworld::player::{CPlayer, PlayerDbProjectionBlock};
use crate::worldserver::appworld::script::variablelist::{VariableListSaveSource, save_var_data};
use crate::worldserver::worldserver::game::{
    CGame, DeletionPlayerSnapshot, ShowSaveInfoDisposition,
    WorldDbDataSaveSession, show_save_info,
};
use crate::worldserver::worldserver::honorranks::CHonorRanks;
use crate::worldserver::worldserver::worldserver::{
    AddLogTextBlock, AddLogTextDisposition, SaveLogTextDisposition, WorldLogLocalTime,
    WorldLogTextOwner,
};

const BEGIN_TRANSACTION_SQL: &str = "BEGIN TRAN";
const COMMIT_TRANSACTION_SQL: &str = "COMMIT";
const ROLLBACK_TRANSACTION_SQL: &str = "ROLLBACK";

fn legacy_snapshot_count(phase: &'static str, count: usize) -> Result<u32, WorldSnapshotSaveBlock> {
    u32::try_from(count)
        .map_err(|_| WorldSnapshotSaveBlock::ContainerCountOverflow { phase, count })
}

/// Какой из двух исторических владельцев должен опубликовать payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SaveDataLogTarget {
    AddLogText,
    ShowSaveInfo,
}

/// Уже материализованный ANSI payload одного фазового log-вызова.
///
/// `Debug` намеренно скрывает байты: подробные player-сообщения содержат
/// account/name и не должны случайно попадать в диагностический вывод отчёта.
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct SaveDataLogEvent {
    pub(crate) target: SaveDataLogTarget,
    /// Результат первого call-site форматирования без конечного C NUL.
    pub(crate) payload: Vec<u8>,
}

impl fmt::Debug for SaveDataLogEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SaveDataLogEvent")
            .field("target", &self.target)
            .field("payload_len", &self.payload.len())
            .finish()
    }
}

/// Локальная неизвестность конкретного достигнутого logger-owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SaveDataLogPublishBlock {
    /// `AddLogText` уже выполнил rotation-check и снял собственное local time.
    AddLogText {
        target: SaveDataLogTarget,
        rotation: SaveLogTextDisposition,
        local_time: WorldLogLocalTime,
        block: AddLogTextBlock,
    },
}

/// Наблюдаемый итог немедленной публикации одного фазового события.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SaveDataLogPublishDisposition {
    /// `ShowSaveInfo` остановился на выключенном process-global gate.
    Suppressed,
    /// Строка синхронно передана file sink и добавлена в operator log.
    Written {
        target: SaveDataLogTarget,
        rotation: SaveLogTextDisposition,
    },
    /// Следующий DB/cleanup-эффект нельзя выполнять до разрешения границы.
    BlockedMissingFact(SaveDataLogPublishBlock),
}

impl SaveDataLogPublishDisposition {
    pub(crate) const fn is_blocked(self) -> bool {
        matches!(self, Self::BlockedMissingFact(_))
    }
}

/// Узкая синхронная граница между phase-owner-ами и process-global logger-ом.
pub(crate) trait SaveDataLogSink {
    fn publish(&mut self, event: &SaveDataLogEvent) -> SaveDataLogPublishDisposition;
}

/// Владеющий состоянием мост фаз `DoSaveData` к двум готовым log-owner-ам.
///
/// Clock и file callbacks хранятся одним owner-ом, чтобы последовательные
/// события наблюдали общий `WorldLogTextOwner` и не получали нового порядка.
pub(crate) struct SaveDataLogPublisher<'a, GetTick, GetLocalTime, PutLogInfo> {
    show_save_info_enabled: bool,
    save_info_time_ms: u32,
    log: &'a mut WorldLogTextOwner,
    get_tick: GetTick,
    get_local_time: GetLocalTime,
    put_log_info: PutLogInfo,
}

impl<'a, GetTick, GetLocalTime, PutLogInfo>
    SaveDataLogPublisher<'a, GetTick, GetLocalTime, PutLogInfo>
where
    GetTick: FnMut() -> u32,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    PutLogInfo: FnMut(&[u8]),
{
    pub(crate) fn new(
        show_save_info_enabled: bool,
        save_info_time_ms: u32,
        log: &'a mut WorldLogTextOwner,
        get_tick: GetTick,
        get_local_time: GetLocalTime,
        put_log_info: PutLogInfo,
    ) -> Self {
        Self {
            show_save_info_enabled,
            save_info_time_ms,
            log,
            get_tick,
            get_local_time,
            put_log_info,
        }
    }

    /// Публикует ровно один event и полностью завершает его до возврата.
    pub(crate) fn publish(&mut self, event: &SaveDataLogEvent) -> SaveDataLogPublishDisposition {
        match event.target {
            SaveDataLogTarget::AddLogText => {
                let disposition = self.log.add_log_text(
                    &event.payload,
                    self.save_info_time_ms,
                    &mut self.get_tick,
                    &mut self.get_local_time,
                    &mut self.put_log_info,
                );
                map_add_log_text_disposition(event.target, disposition)
            }
            SaveDataLogTarget::ShowSaveInfo => match show_save_info(
                self.show_save_info_enabled,
                &event.payload,
                self.save_info_time_ms,
                self.log,
                &mut self.get_tick,
                &mut self.get_local_time,
                &mut self.put_log_info,
            ) {
                ShowSaveInfoDisposition::Suppressed => SaveDataLogPublishDisposition::Suppressed,
                ShowSaveInfoDisposition::Logged(disposition) => {
                    map_add_log_text_disposition(event.target, disposition)
                }
            },
        }
    }
}

impl<GetTick, GetLocalTime, PutLogInfo> SaveDataLogSink
    for SaveDataLogPublisher<'_, GetTick, GetLocalTime, PutLogInfo>
where
    GetTick: FnMut() -> u32,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    PutLogInfo: FnMut(&[u8]),
{
    fn publish(&mut self, event: &SaveDataLogEvent) -> SaveDataLogPublishDisposition {
        SaveDataLogPublisher::publish(self, event)
    }
}

fn publish_save_data_log(
    sink: &mut impl SaveDataLogSink,
    event: &SaveDataLogEvent,
) -> Result<(), SaveDataLogPublishBlock> {
    match sink.publish(event) {
        SaveDataLogPublishDisposition::Suppressed
        | SaveDataLogPublishDisposition::Written { .. } => Ok(()),
        SaveDataLogPublishDisposition::BlockedMissingFact(block) => Err(block),
    }
}

fn map_add_log_text_disposition(
    target: SaveDataLogTarget,
    disposition: AddLogTextDisposition,
) -> SaveDataLogPublishDisposition {
    match disposition {
        AddLogTextDisposition::Written { rotation, .. } => {
            SaveDataLogPublishDisposition::Written { target, rotation }
        }
        AddLogTextDisposition::BlockedMissingFact {
            rotation,
            local_time,
            block,
        } => {
            SaveDataLogPublishDisposition::BlockedMissingFact(SaveDataLogPublishBlock::AddLogText {
                target,
                rotation,
                local_time,
                block,
            })
        }
    }
}

fn add_log_event(payload: impl Into<Vec<u8>>) -> SaveDataLogEvent {
    SaveDataLogEvent {
        target: SaveDataLogTarget::AddLogText,
        payload: payload.into(),
    }
}

fn show_save_info_event(payload: impl Into<Vec<u8>>) -> SaveDataLogEvent {
    SaveDataLogEvent {
        target: SaveDataLogTarget::ShowSaveInfo,
        payload: payload.into(),
    }
}

fn legacy_c_string(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn player_success_log(prefix: &[u8], player: &CPlayer) -> SaveDataLogEvent {
    let mut payload = Vec::with_capacity(
        prefix.len() + player.get_account().len() + player.get_name().len() + 24,
    );
    payload.extend_from_slice(prefix);
    payload.extend_from_slice(legacy_c_string(player.get_account()));
    payload.push(b'.');
    payload.extend_from_slice(legacy_c_string(player.get_name()));
    payload.push(b'.');
    payload.extend_from_slice(player.get_id().to_string().as_bytes());
    show_save_info_event(payload)
}

/// Владеющая копия двух setup-ID из одного `CGame::m_stDBData` snapshot.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SetupIdSnapshot {
    pub(crate) player_id: u32,
    pub(crate) leave_world_id: i32,
}

/// Локальная граница передачи frozen owner-данных в `DoSaveData`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldSnapshotSaveBlock {
    /// `CHonorRanks::GenerateSaveData` ещё не создал отдельную DB-копию.
    MissingHonorRanks,
    /// На 64-битном Rust-хосте создан контейнер, невозможный в 32-битном EXE.
    ContainerCountOverflow { phase: &'static str, count: usize },
}

/// Последняя транзакционная команда и её поглощённая исходником ошибка.
#[derive(Debug)]
pub(crate) enum SetupIdSaveFinish {
    /// Оба UPDATE вернули `true`; исходник сообщал успех даже при ошибке commit.
    Commit {
        error: Option<tiberius::error::Error>,
    },
    /// Первый либо второй UPDATE вернул `false`; rollback тоже мог отказать.
    Rollback {
        error: Option<tiberius::error::Error>,
    },
}

/// Полный доказанный результат setup-ID участка `DoSaveData`.
#[derive(Debug)]
pub(crate) struct SetupIdSaveReport {
    /// Ошибка begin не останавливала исходные UPDATE.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    pub(crate) player_saved: bool,
    /// `None`, когда `SavePlayerID == false` и второй вызов исходно пропущен.
    pub(crate) leave_world_saved: Option<bool>,
    /// Commit соответствует исходному success-log, rollback — failure-log.
    pub(crate) finish: SetupIdSaveFinish,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Финальная ветка второй транзакционной фазы `DoSaveData`.
#[derive(Debug)]
pub(crate) enum GeneralVariableSaveDisposition {
    /// `SaveVarData == true`; success-log исходно не зависел от ошибки commit.
    Commit {
        error: Option<tiberius::error::Error>,
    },
    /// `SaveVarData == false`; failure-log исходно не зависел от rollback.
    Rollback {
        error: Option<tiberius::error::Error>,
    },
}

/// Полный доказанный результат VarData-участка `DoSaveData`.
#[derive(Debug)]
pub(crate) struct GeneralVariableSaveReport {
    /// Ошибка begin не останавливала исходный `SaveVarData`.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Commit/rollback задают исходный final log; blocked останавливает lifecycle.
    pub(crate) disposition: GeneralVariableSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Завершение failure-ветки одной транзакционной записи игрока.
#[derive(Debug)]
pub(crate) enum FailedTransactionFinish {
    /// Begin создал транзакцию; rollback вызван, но его ошибка поглощена.
    Rollback {
        error: Option<tiberius::error::Error>,
    },
    /// `BeginTran` вернул `0`, поэтому catch не вызывал rollback.
    NoTransaction,
}

/// Финальная ветка одного элемента `liDBCreationPlayer`.
#[derive(Debug)]
pub(crate) enum NewCharacterSaveDisposition {
    /// Commit соответствует success-log и последующему удалению list-node.
    Commit {
        error: Option<tiberius::error::Error>,
    },
    /// Исходный catch писал failure-log, сохранял node и переходил к следующему.
    Failure(FailedTransactionFinish),
    /// Null `CPlayer*` сохранялся в очереди и не открывал транзакцию.
    NullPlayer,
    /// Не назначает неизвестной вложенной границе commit, rollback или log.
    BlockedMissingFact(PlayerCreateBlock),
}

/// Полный доказанный результат одного New Character элемента `DoSaveData`.
#[derive(Debug)]
pub(crate) struct NewCharacterSaveReport {
    /// Для `NullPlayer` begin не вызывался и это поле равно `None`.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Задаёт исходный log, судьбу list-node и возможность продолжить обход.
    pub(crate) disposition: NewCharacterSaveDisposition,
}

/// Завершение полного frozen creation-list traversal.
#[derive(Debug)]
pub(crate) enum NewCharactersSaveDisposition {
    Complete,
    /// DB-ветка текущего entry завершена, но его log и следующий cleanup — нет.
    BlockedLog {
        entry_index: usize,
        entry: NewCharacterSaveReport,
        block: SaveDataLogPublishBlock,
    },
    BlockedProjection {
        entry_index: usize,
        block: PlayerDbProjectionBlock,
    },
    BlockedCreate {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: PlayerCreateBlock,
    },
}

/// Результат New Character phase с уже применённым success-cleanup.
#[derive(Debug)]
pub(crate) struct NewCharactersSaveReport {
    /// `_Mysize`, снятый до traversal для итогового `Charactor CREATED`.
    pub(crate) logged_count: u32,
    pub(crate) entries: Vec<NewCharacterSaveReport>,
    pub(crate) disposition: NewCharactersSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Финальная ветка одного ID из `lDBRestorePlayer`.
#[derive(Debug)]
pub(crate) enum RestoreCharacterSaveDisposition {
    /// Commit соответствует success-log и удалению текущего ID-node под lock.
    Commit {
        error: Option<tiberius::error::Error>,
    },
    /// Failure-log сохраняет текущий ID-node и продолжает обход.
    Failure(FailedTransactionFinish),
}

/// Полный доказанный результат одного Restore Charactor элемента.
#[derive(Debug)]
pub(crate) struct RestoreCharacterSaveReport {
    /// Ошибка begin не останавливала вызов `RestorePlayer`.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Задаёт исходный log и судьбу текущего ID-node.
    pub(crate) disposition: RestoreCharacterSaveDisposition,
}

/// Полный Restore Character traversal с уже удалёнными success-node-ами.
#[derive(Debug)]
pub(crate) struct RestoreCharactersSaveReport {
    /// `_Mysize`, снятый до traversal для итогового `CANCEL DELETE`.
    pub(crate) logged_count: u32,
    pub(crate) entries: Vec<RestoreCharacterSaveReport>,
    pub(crate) disposition: RestoreCharactersSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Завершение Restore Character traversal либо остановка до судьбы node.
#[derive(Debug)]
pub(crate) enum RestoreCharactersSaveDisposition {
    Complete,
    BlockedLog {
        entry_index: usize,
        entry: Box<RestoreCharacterSaveReport>,
        block: SaveDataLogPublishBlock,
    },
}

/// Финальная ветка одного элемента `liDBDeletionPlayer`.
#[derive(Debug)]
pub(crate) enum DeleteCharacterSaveDisposition {
    /// Success-log требует удалить текущий node под исходным lock.
    Commit {
        error: Option<tiberius::error::Error>,
    },
    /// Failure-log сохраняет текущий node и продолжает обход.
    Failure(FailedTransactionFinish),
    /// Не назначает исходному access violation DB-команду, log или remove.
    BlockedMissingFact(PlayerDeleteTimeBlock),
}

/// Полный доказанный результат одного Delete Charactor элемента.
#[derive(Debug)]
pub(crate) struct DeleteCharacterSaveReport {
    /// Ошибка begin не останавливала вызов `DeletePlayer`.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Задаёт log, судьбу node либо обязательную остановку lifecycle.
    pub(crate) disposition: DeleteCharacterSaveDisposition,
}

/// Завершение полного frozen deletion-list traversal.
#[derive(Debug)]
pub(crate) enum DeleteCharactersSaveDisposition {
    Complete,
    /// DB-ветка завершена; event не опубликован и node/cursor ещё не изменены.
    BlockedLog {
        entry_index: usize,
        entry: DeleteCharacterSaveReport,
        block: SaveDataLogPublishBlock,
    },
    BlockedMissingFact {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: PlayerDeleteTimeBlock,
    },
}

/// Результат Delete Character phase с уже применённым success-cleanup.
#[derive(Debug)]
pub(crate) struct DeleteCharactersSaveReport {
    /// `_Mysize`, снятый до traversal для итогового `SIGN DELETE`.
    pub(crate) logged_count: u32,
    pub(crate) entries: Vec<DeleteCharacterSaveReport>,
    pub(crate) disposition: DeleteCharactersSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Caller-owned view исходного `listDeleteUnions` перед началом фазы.
pub(crate) struct DeleteUnionListSnapshot<'entries> {
    /// ID в доказанном list-order.
    pub(crate) union_ids: &'entries [i32],
    /// `_Mysize`, захваченный отдельно до traversal и используемый `%d`-логом.
    pub(crate) logged_count: u32,
}

/// Результат одного ID; ни одно поле не меняет продолжение исходного цикла.
#[derive(Debug)]
pub(crate) struct DeleteUnionEntrySaveReport {
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// `false` уже имеет собственный `Delete Union ERROR`, но не отменяет commit.
    pub(crate) delete_succeeded: bool,
    pub(crate) commit_error: Option<tiberius::error::Error>,
}

/// Обязанность list-owner-а после завершения всех entry-вызовов.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeleteUnionClear {
    /// После clear выводится как исходный signed `%d`-шаблон.
    pub(crate) logged_count: u32,
}

/// Полный доказанный normal-path Delete Union фазы.
#[derive(Debug)]
pub(crate) struct DeleteUnionSaveReport {
    pub(crate) entries: Vec<DeleteUnionEntrySaveReport>,
    /// Требует без исходного player-list lock удалить все текущие list-nodes.
    pub(crate) clear: DeleteUnionClear,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Caller-owned view исходного `listDeleteFactions` перед началом фазы.
pub(crate) struct DeleteFactionListSnapshot<'entries> {
    /// ID в доказанном list-order.
    pub(crate) faction_ids: &'entries [i32],
    /// `_Mysize`, захваченный до traversal для финального `%d`-лога.
    pub(crate) logged_count: u32,
}

/// Результат одного ID; исходный цикл продолжался при любом значении полей.
#[derive(Debug)]
pub(crate) struct DeleteFactionEntrySaveReport {
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// `false` уже создал `delete faction ERROR`, но не отменял commit.
    pub(crate) delete_succeeded: bool,
    pub(crate) commit_error: Option<tiberius::error::Error>,
}

/// Обязанность list-owner-а после завершения всех entry-вызовов.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeleteFactionClear {
    /// После clear выводится как исходный signed `%d`-шаблон.
    pub(crate) logged_count: u32,
}

/// Полный доказанный normal-path Delete Faction фазы.
#[derive(Debug)]
pub(crate) struct DeleteFactionSaveReport {
    pub(crate) entries: Vec<DeleteFactionEntrySaveReport>,
    /// Требует без исходного player-list lock удалить все текущие list-nodes.
    pub(crate) clear: DeleteFactionClear,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Caller-owned view исходного `listSaveFactions` перед началом фазы.
pub(crate) struct SaveFactionListSnapshot<'entries, 'faction> {
    /// Nullable save-копии в доказанном live list-order.
    pub(crate) factions: &'entries mut [Option<FactionSaveSnapshot<'faction>>],
    /// `_Mysize`, захваченный до traversal для финального `%d`-лога.
    pub(crate) logged_count: u32,
}

/// Доказанный cleanup одного нормально завершённого entry.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SaveFactionEntryCleanup {
    /// Null `CFaction*`: удалить только текущий list-node.
    RemoveNode,
    /// Сначала удалить node, затем virtual-уничтожить non-null save-копию.
    RemoveNodeThenDestroySnapshot,
}

/// Результат одного entry до перехода к следующему live node.
#[derive(Debug)]
pub(crate) struct SaveFactionEntrySaveReport {
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Bool dispatcher-а исходно игнорировался и не менял commit/cleanup.
    pub(crate) save_returned: bool,
    pub(crate) commit_error: Option<tiberius::error::Error>,
    pub(crate) cleanup: SaveFactionEntryCleanup,
}

/// Финальная очистка pointer-list после завершения traversal.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SaveFactionFinalClear {
    /// Сначала выводится как исходный signed `%d`-шаблон.
    pub(crate) logged_count: u32,
}

/// Завершение Save Faction Data phase либо неизвестная overread-граница.
#[derive(Debug)]
pub(crate) enum SaveFactionSaveDisposition {
    /// После entry-cleanup-ов очистить все оставшиеся list-nodes и перейти дальше.
    Complete(SaveFactionFinalClear),
    /// Все entry уже очищены, но final log остановился перед остаточным clear.
    BlockedLog {
        logged_count: u32,
        block: SaveDataLogPublishBlock,
    },
    /// Не назначает текущему node commit, cleanup или продолжение lifecycle.
    BlockedMissingFact {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: SaveFactionPhaseBlock,
    },
}

/// Локальная projection- либо DB-owner-граница faction-фазы.
#[derive(Debug)]
pub(crate) enum SaveFactionPhaseBlock {
    Projection(FactionSaveProjectionBlock),
    Owner(FactionSaveBlock),
}

/// Полный доказанный результат Save Faction Data phase.
#[derive(Debug)]
pub(crate) struct SaveFactionSaveReport {
    /// Только полностью завершённые entry; каждый требует указанного cleanup.
    pub(crate) entries: Vec<SaveFactionEntrySaveReport>,
    pub(crate) disposition: SaveFactionSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Caller-owned view исходного `listSaveUnions` перед началом фазы.
pub(crate) struct SaveUnionListSnapshot<'entries, 'union> {
    /// Nullable save-копии в доказанном live list-order.
    pub(crate) unions: &'entries mut [Option<UnionSaveSnapshot<'union>>],
    /// `_Mysize`, захваченный до traversal для финального `%d`-лога.
    pub(crate) logged_count: u32,
}

/// Доказанный cleanup одного нормально завершённого union-entry.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SaveUnionEntryCleanup {
    /// Null `CUnion*`: удалить только текущий list-node.
    RemoveNode,
    /// Сначала удалить node, затем virtual-уничтожить non-null save-копию.
    RemoveNodeThenDestroySnapshot,
}

/// Результат одного union-entry до перехода к следующему live node.
#[derive(Debug)]
pub(crate) struct SaveUnionEntrySaveReport {
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Bool `SaveConfederation` исходно игнорировался и не менял commit/cleanup.
    pub(crate) save_returned: bool,
    pub(crate) commit_error: Option<tiberius::error::Error>,
    pub(crate) cleanup: SaveUnionEntryCleanup,
}

/// Финальная очистка pointer-list после завершения union traversal.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SaveUnionFinalClear {
    /// Сначала выводится как исходный signed `%d`-шаблон.
    pub(crate) logged_count: u32,
}

/// Завершение Save Union Data phase либо неизвестная owner-граница.
#[derive(Debug)]
pub(crate) enum SaveUnionSaveDisposition {
    /// После entry-cleanup-ов очистить оставшиеся list-nodes и перейти к region.
    Complete(SaveUnionFinalClear),
    /// Все entry уже очищены, но final log остановился перед остаточным clear.
    BlockedLog {
        logged_count: u32,
        block: SaveDataLogPublishBlock,
    },
    /// Не назначает текущему node commit, cleanup или продолжение lifecycle.
    BlockedMissingFact {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: UnionSaveBlock,
    },
}

/// Полный доказанный результат Save Union Data phase.
#[derive(Debug)]
pub(crate) struct SaveUnionSaveReport {
    /// Только полностью завершённые entry; каждый требует указанного cleanup.
    pub(crate) entries: Vec<SaveUnionEntrySaveReport>,
    pub(crate) disposition: SaveUnionSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Caller-owned view исходного `listRegionParam` перед началом фазы.
pub(crate) struct SaveRegionListSnapshot<'entries> {
    /// Nullable save-копии в доказанном live list-order.
    pub(crate) regions: &'entries [Option<RegionSaveSnapshot>],
    /// `_Mysize`, захваченный до traversal для финального `%ld`-лога.
    pub(crate) logged_count: u32,
}

/// Доказанный cleanup одного завершённого region-entry до чтения next.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SaveRegionEntryCleanup {
    /// Null `CWorldRegion*`: сохранить текущий node и только перейти к next.
    RetainNode,
    /// Virtual-уничтожить save-копию, сохранив node для финальной очистки.
    DestroySnapshotAndRetainNode,
}

/// Результат одного region-entry до перехода к следующему live node.
#[derive(Debug)]
pub(crate) struct SaveRegionEntrySaveReport {
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Bool `CRsRegion::Save` исходно игнорировался и не менял commit/cleanup.
    pub(crate) save_returned: bool,
    pub(crate) commit_error: Option<tiberius::error::Error>,
    pub(crate) cleanup: SaveRegionEntryCleanup,
}

/// Финальная очистка list-nodes после уничтожения region save-копий.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SaveRegionFinalClear {
    /// Сначала выводится как исходный signed `%ld`-шаблон.
    pub(crate) logged_count: u32,
}

/// Полный доказанный результат Save Region Data phase.
#[derive(Debug)]
pub(crate) struct SaveRegionSaveReport {
    /// Каждый entry сохраняет node до общей финальной очистки.
    pub(crate) entries: Vec<SaveRegionEntrySaveReport>,
    /// Требует отделить список, занулить size и удалить все его nodes.
    pub(crate) clear: SaveRegionFinalClear,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Финальная ветка Save HonorRanks transaction phase.
#[derive(Debug)]
pub(crate) enum HonorRanksSaveDisposition {
    /// Оба owner-а вернули `true`; исходный success-log не зависел от commit.
    Commit {
        error: Option<tiberius::error::Error>,
    },
    /// Первый `false` приводил в общий catch и его conditional rollback.
    Failure(FailedTransactionFinish),
    /// Не назначает старому size-overflow завершение транзакции либо log.
    BlockedMissingFact(HonorRanksSaveBlock),
}

/// Полный доказанный результат Save HonorRanks участка `DoSaveData`.
#[derive(Debug)]
pub(crate) struct HonorRanksTransactionSaveReport {
    /// Ошибка begin не останавливала исходный `InsertHonorRanks`.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    pub(crate) insert_succeeded: bool,
    /// `None`, если insert вернул `false` либо save достиг size-overflow.
    pub(crate) save_returned: Option<bool>,
    /// Commit соответствует success-log, Failure — abnormal catch/log.
    pub(crate) disposition: HonorRanksSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Финальная ветка одной из двух независимых GodsBattle transaction phases.
#[derive(Debug)]
pub(crate) enum GodsBattleSaveDisposition {
    /// Owner вернул `true`; исходный success-log не зависел от ошибки commit.
    Commit {
        error: Option<tiberius::error::Error>,
    },
    /// Owner вернул `false`; abnormal-log следовал после conditional rollback.
    Failure(FailedTransactionFinish),
}

/// Полный доказанный результат одной GodsBattle transaction phase.
#[derive(Debug)]
pub(crate) struct GodsBattleTransactionSaveReport {
    /// Отличает Belief/XYD и NPC ветви без объединения их owner-вызовов.
    pub(crate) operation: GodsBattleSaveOperation,
    /// Ошибка begin не останавливала соответствующий owner.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Этот bool буквально выбирал commit/success либо catch/failure.
    pub(crate) save_returned: bool,
    pub(crate) disposition: GodsBattleSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Точная normal-path очистка исходного `listEnemyFactions` после commit.
#[derive(Clone, Copy, Debug)]
pub(crate) enum EnemyFactionsFinalCleanup {
    /// Уничтожить все non-null values в list-order, затем удалить все nodes.
    DestroyValuesThenClearNodes,
}

/// Завершение EnemyFactions transaction phase.
#[derive(Debug)]
pub(crate) enum EnemyFactionsTransactionSaveDisposition {
    /// Bool owner-а проигнорирован; commit-error не отменяет cleanup/success-log.
    Complete {
        commit_error: Option<tiberius::error::Error>,
        cleanup: EnemyFactionsFinalCleanup,
    },
    /// Не назначает неизвестному null-разыменованию commit, cleanup либо log.
    BlockedMissingFact(EnemyFactionNullEntryBlock),
}

/// Полный доказанный результат EnemyFactions участка `DoSaveData`.
#[derive(Debug)]
pub(crate) struct EnemyFactionsTransactionSaveReport {
    /// Ошибка begin не останавливала копирование списка и вызов DB-owner-а.
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// `None` означает, что owner достиг null-entry и не вернул исходный bool.
    pub(crate) save_returned: Option<bool>,
    pub(crate) disposition: EnemyFactionsTransactionSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Caller-owned view исходного `ltDBCountrys` перед началом фазы.
pub(crate) struct SaveCountryListSnapshot<'entries> {
    /// Nullable save-копии в доказанном live list-order.
    pub(crate) countries: &'entries [Option<CountrySaveSnapshot>],
    /// `_Mysize`, захваченный до traversal для финального `%ld`-лога.
    pub(crate) logged_count: u32,
}

/// Доказанный cleanup одного завершённого country-entry.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SaveCountryEntryCleanup {
    /// Null `CCountry*`: удалить только текущий list-node.
    RemoveNode,
    /// Сначала удалить node, затем virtual-уничтожить non-null save-копию.
    RemoveNodeThenDestroySnapshot,
}

/// Результат одного country-entry до перехода к следующему live node.
#[derive(Debug)]
pub(crate) struct SaveCountryEntrySaveReport {
    pub(crate) begin_error: Option<tiberius::error::Error>,
    /// Bool `CDBCountry::Save` исходно игнорировался и не менял commit/cleanup.
    pub(crate) save_returned: bool,
    pub(crate) commit_error: Option<tiberius::error::Error>,
    pub(crate) cleanup: SaveCountryEntryCleanup,
}

/// Финальная очистка pointer-list после завершения country traversal.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SaveCountryFinalClear {
    /// Сначала выводится как исходный signed `%ld`-шаблон.
    pub(crate) logged_count: u32,
}

/// Полный доказанный normal-path Update Country Data phase.
#[derive(Debug)]
pub(crate) struct SaveCountrySaveReport {
    /// Каждый завершённый entry требует указанного cleanup до следующего node.
    pub(crate) entries: Vec<SaveCountryEntrySaveReport>,
    /// Требует удалить оставшиеся list-nodes и перейти к LoadDetails phase.
    pub(crate) clear: SaveCountryFinalClear,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Неизменяемые поля non-null `CPlayer*`, достигнутые LoadDetails-фазой.
///
/// `Debug` сознательно не реализован, чтобы CD-key не попадал в обычный log.
#[derive(Clone, Copy)]
pub(crate) struct LoadDetailsPlayerSnapshot<'player> {
    /// Signed `CBaseObject::m_lID` по `CPlayer+8`.
    pub(crate) player_id: i32,
    /// Byte-exact C-string view исходного `std::string` по `CPlayer+0x72C`.
    pub(crate) cd_key: &'player [u8],
}

/// Caller-owned view исходного `map<unsigned int, CPlayer*>`.
pub(crate) struct LoadDetailsPlayerMapSnapshot<'entries, 'player> {
    /// `BTreeMap` сохраняет exact unsigned-key order и nullable pointer-values.
    pub(crate) players: &'entries BTreeMap<u32, Option<LoadDetailsPlayerSnapshot<'player>>>,
    /// Буквальный snapshot `CGame::tagSetup::bUseOldSaveLargessWay`.
    pub(crate) use_old_save_largess_way: bool,
}

/// Выбранная один раз до traversal ветка и её обязательный mode-log.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LoadDetailsSaveWay {
    /// `** use old save way` и self-opening перегрузка.
    Old,
    /// `** use new save way` и caller-connection transaction.
    New,
}

/// Один полностью завершённый map-entry до перехода к next.
#[derive(Debug)]
pub(crate) enum LoadDetailsEntrySaveReport {
    /// Null pointer не вызывал DB-owner и требует null-pointer log.
    NullPlayer { map_key: u32 },
    /// Старая перегрузка открывала своё Cost DB connection без transaction.
    OldWay { map_key: u32, save_returned: bool },
    /// Новая перегрузка безусловно получала begin и commit.
    NewWay {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        save_returned: bool,
        commit_error: Option<tiberius::error::Error>,
    },
}

/// Финальная судьба LoadDetails phase.
#[derive(Debug)]
pub(crate) enum LoadDetailsSaveDisposition {
    /// Все entries завершены; `mDBPlayer` остаётся byte-for-byte неизменной.
    CompleteRetainPlayerMap,
}

/// Полный доказанный LoadDetails normal-path либо локальная UB-граница.
#[derive(Debug)]
pub(crate) struct LoadDetailsSaveReport {
    /// Сохраняется даже для пустой map и задаёт исходный mode-log.
    pub(crate) way: LoadDetailsSaveWay,
    /// Только entries, для которых исходно задан переход к next.
    pub(crate) entries: Vec<LoadDetailsEntrySaveReport>,
    pub(crate) disposition: LoadDetailsSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// LoadDetails report вместе с ранним count для следующего map traversal.
#[derive(Debug)]
pub(crate) struct LoadDetailsWorldSaveReport {
    pub(crate) logged_count: u32,
    pub(crate) phase: LoadDetailsSaveReport,
}

/// Non-null value одного элемента `mDBPlayer` для Save Charactor Data.
pub(crate) struct SaveCharacterPlayerSnapshot<'player, 'snapshot> {
    /// Тот же живой owner, который исходный map хранил как `CPlayer*`.
    pub(crate) player: &'player CPlayer,
    /// Полная проекция этого же owner-а для достигнутого DB-save.
    pub(crate) save: PlayerSaveSnapshot<'snapshot, 'snapshot, 'snapshot>,
}

/// Caller-owned view повторного обхода `map<unsigned int, CPlayer*>`.
pub(crate) struct SaveCharacterPlayerMapSnapshot<'entries, 'player, 'snapshot> {
    /// `BTreeMap` сохраняет unsigned key-order и nullable pointer-values.
    pub(crate) players:
        &'entries BTreeMap<u32, Option<SaveCharacterPlayerSnapshot<'player, 'snapshot>>>,
    /// `_Mysize`, захваченный до предшествующей LoadDetails phase.
    pub(crate) logged_count: u32,
}

/// Точная success-очистка после `ShowSaveInfo` и до перехода к next.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SaveCharacterSuccessCleanup {
    /// Под player-list lock уничтожить player, занулить value и стереть entry.
    DestroyPlayerThenEraseEntryUnderLock,
}

/// Один полностью завершённый map-entry Save Charactor Data.
#[derive(Debug)]
pub(crate) enum SaveCharacterEntrySaveReport {
    /// Null value не открывает транзакцию, пишет null-log и сохраняет entry.
    NullPlayer { map_key: u32 },
    /// `SaveData == true`: commit вызывается, затем entry обязательно удаляется.
    Saved {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        commit_error: Option<tiberius::error::Error>,
        cleanup: SaveCharacterSuccessCleanup,
    },
    /// `SaveData == false`: conditional rollback, failure-log и сохранение entry.
    Failed {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        finish: FailedTransactionFinish,
    },
}

/// Финальная судьба Save Charactor Data traversal.
#[derive(Debug)]
pub(crate) enum SaveCharacterSaveDisposition {
    /// Все entries завершены; следующий owner закрывает connection и пишет итог.
    Complete { logged_count: u32 },
    /// DB-ветка завершена, но её log и следующий map-эффект ещё не завершены.
    BlockedLog {
        map_key: u32,
        entry: Box<SaveCharacterEntrySaveReport>,
        block: SaveDataLogPublishBlock,
    },
    /// Не назначает локальной goods-неизвестности transaction finish и cleanup.
    BlockedMissingFact {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        block: SaveCharacterPhaseBlock,
    },
}

/// Локальная граница до либо внутри `CPlayer::SaveData`.
#[derive(Debug)]
pub(crate) enum SaveCharacterPhaseBlock {
    Projection(PlayerDbProjectionBlock),
    Save(PlayerSaveBlock),
}

/// Полный доказанный результат Save Charactor Data phase.
#[derive(Debug)]
pub(crate) struct SaveCharacterSaveReport {
    /// Только entries, для которых исходно доказан переход к next.
    pub(crate) entries: Vec<SaveCharacterEntrySaveReport>,
    pub(crate) disposition: SaveCharacterSaveDisposition,
    pub(crate) log_events: Vec<SaveDataLogEvent>,
}

/// Доказанный порядок фаз от setup-ID до конца Save Union Data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DoSaveDataPhase {
    SetupIds,
    GeneralVariables,
    NewCharacters,
    RestoreCharacters,
    DeleteCharacters,
    DeleteUnions,
    DeleteFactions,
    SaveFactions,
    SaveUnions,
    SaveRegions,
    HonorRanks,
    GodsBattleFactionXyd,
    GodsBattleNpcFactions,
    EnemyFactions,
    Countries,
    LoadDetails,
    SaveCharacters,
}

/// Все отчёты уже достигнутых фаз; `None` означает, что фаза ещё не начиналась.
#[derive(Debug, Default)]
pub(crate) struct DoSaveDataThroughUnionsPhases {
    pub(crate) setup_ids: Option<SetupIdSaveReport>,
    pub(crate) general_variables: Option<GeneralVariableSaveReport>,
    pub(crate) new_characters: Option<NewCharactersSaveReport>,
    pub(crate) restore_characters: Option<RestoreCharactersSaveReport>,
    pub(crate) delete_characters: Option<DeleteCharactersSaveReport>,
    pub(crate) delete_unions: Option<DeleteUnionSaveReport>,
    pub(crate) delete_factions: Option<DeleteFactionSaveReport>,
    pub(crate) save_factions: Option<SaveFactionSaveReport>,
    pub(crate) save_unions: Option<SaveUnionSaveReport>,
}

/// Три ранних `_Mysize`, которые полный normal-path передаёт итоговому логу.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SaveDataEarlyCounters {
    pub(crate) created: u32,
    pub(crate) cancel_deleted: u32,
    pub(crate) sign_deleted: u32,
}

/// Точная достигнутая позиция event-а внутри последовательности фазы.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SaveDataLogCheckpoint {
    /// Первый общий event сразу после успешного открытия connection.
    SaveVariablesStart,
    /// Фазовый start-event до первого DB-вызова.
    PhaseStart,
    /// Индекс уже материализованного event-а в phase-report.
    PhaseEvent(usize),
}

/// Точка остановки последовательности либо переход к Save Region Data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DoSaveDataThroughUnionsDisposition {
    /// Safe snapshot-граница возникла до первого DB-вызова названной фазы.
    BlockedSnapshot {
        phase: DoSaveDataPhase,
        block: WorldSnapshotSaveBlock,
    },
    /// Текущий phase-report содержит конкретную нерешённую границу вложенного owner-а.
    BlockedPhase(DoSaveDataPhase),
    /// Logger остановил lifecycle до следующего DB/container эффекта.
    BlockedLog {
        phase: DoSaveDataPhase,
        checkpoint: SaveDataLogCheckpoint,
        block: SaveDataLogPublishBlock,
    },
    /// Все фазы закончены; следующий observable owner — Save Region Data.
    ContinueWithRegion(SaveDataEarlyCounters),
}

/// Полный результат уже связанного префикса `DoSaveData`.
#[derive(Debug)]
pub(crate) struct DoSaveDataThroughUnionsReport {
    pub(crate) phases: DoSaveDataThroughUnionsPhases,
    pub(crate) disposition: DoSaveDataThroughUnionsDisposition,
}

/// Все отчёты suffix-а после Save Union Data.
#[derive(Debug, Default)]
pub(crate) struct DoSaveDataAfterUnionsPhases {
    pub(crate) regions: Option<SaveRegionSaveReport>,
    pub(crate) honor_ranks: Option<HonorRanksTransactionSaveReport>,
    pub(crate) gods_battle_faction_xyd: Option<GodsBattleTransactionSaveReport>,
    pub(crate) gods_battle_npc_factions: Option<GodsBattleTransactionSaveReport>,
    pub(crate) enemy_factions: Option<EnemyFactionsTransactionSaveReport>,
    pub(crate) countries: Option<SaveCountrySaveReport>,
    pub(crate) load_details: Option<LoadDetailsWorldSaveReport>,
    pub(crate) save_characters: Option<SaveCharacterSaveReport>,
}

/// Точка остановки suffix-а либо четыре готовых final counters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DoSaveDataAfterUnionsDisposition {
    BlockedSnapshot {
        phase: DoSaveDataPhase,
        block: WorldSnapshotSaveBlock,
    },
    /// Вложенный phase-report содержит конкретную неизвестную границу.
    BlockedPhase(DoSaveDataPhase),
    /// Logger остановил suffix до следующего DB/container эффекта.
    BlockedLog {
        phase: DoSaveDataPhase,
        checkpoint: SaveDataLogCheckpoint,
        block: SaveDataLogPublishBlock,
    },
    /// Все DB/container фазы закончены; следующий owner закрывает connection.
    Complete(SaveDataCounters),
}

/// Полный результат последовательности после Save Union Data.
#[derive(Debug)]
pub(crate) struct DoSaveDataAfterUnionsReport {
    pub(crate) phases: DoSaveDataAfterUnionsPhases,
    pub(crate) disposition: DoSaveDataAfterUnionsDisposition,
}

/// Завершение всей последовательности DB/container фаз на открытом connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DoSaveDataPhasesDisposition {
    BlockedThroughUnions,
    BlockedAfterUnions,
    /// Следующий owner выполняет connection cleanup и общий final report.
    Complete(SaveDataCounters),
}

/// Два точных последовательных участка всех фаз `DoSaveData`.
#[derive(Debug)]
pub(crate) struct DoSaveDataPhasesReport {
    pub(crate) through_unions: DoSaveDataThroughUnionsReport,
    pub(crate) after_unions: Option<DoSaveDataAfterUnionsReport>,
    pub(crate) disposition: DoSaveDataPhasesDisposition,
}

/// Счётчики успешной ветки, переданные в итоговый `AddLogText` как `%d`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SaveDataCounters {
    pub(crate) created: u32,
    pub(crate) cancel_deleted: u32,
    pub(crate) sign_deleted: u32,
    /// Ранний `_Mysize`, а не число фактически удалённых success-entry.
    pub(crate) saved: u32,
}

/// Ветка, по которой `DoSaveData` достиг общего финального отчёта.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SaveDataFinalPath {
    /// Полный traversal: счётчики берутся из четырёх сохранённых locals.
    Completed(SaveDataCounters),
    /// `OpenCn == false`: все четыре locals остались исходными нулями.
    ConnectionOpenFailed,
}

/// Exact connection/log-порядок перед единичным замером конечного tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SaveDataConnectionFinish {
    /// `CloseCn -> "Save Data end ..." -> ReleaseCn` (ещё один `CloseCn`).
    CloseLogEndThenRelease,
    /// `"Connect To DB FAILED!" -> ReleaseCn` без success-end log.
    LogConnectFailureThenRelease,
}

/// Владеющая Linux-проекция Windows `SYSTEMTIME` из одного `GetLocalTime`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SaveDataLocalTime {
    pub(crate) year: u16,
    pub(crate) month: u16,
    pub(crate) day_of_week: u16,
    pub(crate) day: u16,
    pub(crate) hour: u16,
    pub(crate) minute: u16,
    pub(crate) second: u16,
    pub(crate) milliseconds: u16,
}

/// Заменяет один `timeGetTime` через Linux boot-time clock.
pub(crate) fn capture_save_data_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1_000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

/// Заменяет один `GetLocalTime` после уже опубликованного summary-log.
pub(crate) fn capture_save_data_local_time() -> SaveDataLocalTime {
    let local = Local::now();
    SaveDataLocalTime {
        year: local.year() as u16,
        month: local.month() as u16,
        day_of_week: local.weekday().num_days_from_sunday() as u16,
        day: local.day() as u16,
        hour: local.hour() as u16,
        minute: local.minute() as u16,
        second: local.second() as u16,
        milliseconds: local.nanosecond().div_euclid(1_000_000) as u16,
    }
}

/// Caller-owned замена четырёх process-global значений save lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SaveDataLifecycleState {
    pub(crate) this_save_start_tick_ms: u32,
    pub(crate) last_save_tick_ms: u32,
    pub(crate) last_save_time: SaveDataLocalTime,
    pub(crate) is_saving_data: bool,
}

/// Аргументы доказанного `SendErrLog(-2, server, world, text)`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SaveDataMonitoringReport {
    pub(crate) message_type: i8,
    pub(crate) server_id: i32,
    /// `SendErrLog` принимал `long`; это те же четыре бита setup `DWORD`.
    pub(crate) world_number_bits: u32,
    /// C-string без конечного NUL; component message-owner обязан добавить его.
    pub(crate) text: Vec<u8>,
}

/// Последняя наблюдаемая ветка после обновления трёх time-global значений.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum SaveDataFinalDisposition {
    /// `SendErrLog` вызван, его send-result проигнорирован и save-флаг сброшен.
    Complete(SaveDataMonitoringReport),
}

/// Полный доказанный результат общего close/report tail `DoSaveData`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SaveDataFinalReport {
    pub(crate) connection_finish: SaveDataConnectionFinish,
    /// Уже отформатированный observable payload итогового `AddLogText`.
    pub(crate) summary_log: Vec<u8>,
    pub(crate) elapsed_ms: u32,
    pub(crate) disposition: SaveDataFinalDisposition,
}

/// Первая половина хвоста после connection cleanup и конечного `timeGetTime`.
pub(crate) struct SaveDataFinalStart {
    pub(crate) connection_finish: SaveDataConnectionFinish,
    /// Caller обязан опубликовать этот log до следующего `GetLocalTime`.
    pub(crate) summary_log: Vec<u8>,
    pub(crate) elapsed_ms: u32,
    end_tick_ms: u32,
}

/// Неизменяемые входы до connection cleanup и конечного `timeGetTime`.
pub(crate) struct SaveDataFinalSnapshot {
    pub(crate) path: SaveDataFinalPath,
    /// Local `saved_start_tick`, снятый одновременно с global start tick.
    pub(crate) started_at_tick_ms: u32,
}

impl SaveDataFinalSnapshot {
    /// Даёт caller-у connection/log-обязанность до конечного `timeGetTime`.
    pub(crate) const fn connection_finish(&self) -> SaveDataConnectionFinish {
        match self.path {
            SaveDataFinalPath::Completed(_) => SaveDataConnectionFinish::CloseLogEndThenRelease,
            SaveDataFinalPath::ConnectionOpenFailed => {
                SaveDataConnectionFinish::LogConnectFailureThenRelease
            }
        }
    }
}

/// Значения, которые exact tail читает после обновления time-global полей.
///
/// `Debug` не реализован, чтобы byte-string имени сервера не размножался в log.
pub(crate) struct SaveDataMonitoringSnapshot {
    /// Byte-exact `std::string`; старый `%s` читает только prefix до NUL.
    pub(crate) server_name: Vec<u8>,
    pub(crate) write_log_count: u32,
    pub(crate) server_id: i32,
    pub(crate) world_number_bits: u32,
}

/// Ошибка достигнутой `CreateCn/OpenCn`-границы без connection string.
#[derive(Debug)]
pub(crate) enum SaveDataConnectionError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
}

impl fmt::Display for SaveDataConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => {
                write!(formatter, "не открыто соединение World save DB: {error}")
            }
            Self::Tds(error) => write!(formatter, "ошибка TDS World save DB: {error}"),
        }
    }
}

impl Error for SaveDataConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
        }
    }
}

impl From<tiberius::error::Error> for SaveDataConnectionError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

/// Результат начальной `DoSaveData`-границы перед первой save-фазой.
pub(crate) enum DoSaveDataStart {
    /// Флаг уже поставлен; caller следующим пишет `Save Variables Start...`.
    Opened {
        connection: WorldTdsClient,
        started_at_tick_ms: u32,
    },
    /// Флаг не менялся; caller выполняет failure connection-finish и finalizer.
    ConnectionOpenFailed {
        error: SaveDataConnectionError,
        final_snapshot: SaveDataFinalSnapshot,
    },
}

/// Какой connection-log остановил общий lifecycle до следующего эффекта.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SaveDataConnectionLogCheckpoint {
    SaveDataEnd,
    ConnectToDatabaseFailed,
}

/// Доказательства уже завершённого пути после выхода из connection-владельца.
#[derive(Debug)]
pub(crate) struct SaveDataLifecycleEvidence {
    /// `None` только у ветки, где World connection не был открыт.
    pub(crate) phases: Option<DoSaveDataPhasesReport>,
    pub(crate) open_error: Option<SaveDataConnectionError>,
    /// Ошибка Tiberius close наблюдаема в Rust, но не меняет исходный порядок.
    pub(crate) close_error: Option<tiberius::error::Error>,
}

/// Результат единого owner-а `DoSaveData` без создания runtime entry/thread.
pub(crate) enum DoSaveDataLifecycleReport {
    /// Phase block не разрешает выбирать transaction/connection cleanup.
    BlockedPhases {
        phases: DoSaveDataPhasesReport,
        connection: WorldTdsClient,
        started_at_tick_ms: u32,
    },
    /// Первый close либо failed-open уже достигнут, следующий эффект запрещён.
    BlockedConnectionLog {
        evidence: SaveDataLifecycleEvidence,
        final_snapshot: SaveDataFinalSnapshot,
        checkpoint: SaveDataConnectionLogCheckpoint,
        block: SaveDataLogPublishBlock,
    },
    /// Connection cleanup и конечный tick завершены, но summary-log — нет.
    BlockedSummaryLog {
        evidence: SaveDataLifecycleEvidence,
        final_start: SaveDataFinalStart,
        block: SaveDataLogPublishBlock,
    },
    /// `SaveDataFinalDisposition` отдельно различает monitoring success/block.
    Final {
        evidence: SaveDataLifecycleEvidence,
        report: SaveDataFinalReport,
    },
}

/// Снимает start tick и выполняет исходную `CreateCn/OpenCn`-границу.
///
/// При успехе save-флаг ставится до возврата, поэтому caller обязан следующим
/// observable-действием опубликовать `Save Variables Start...`. При ошибке
/// прежнее значение флага сохраняется; `final_snapshot` ведёт в общий хвост с
/// нулевыми счётчиками и тем же local start tick.
pub(crate) async fn begin_do_save_data(
    settings: &WorldDatabaseSettings,
    state: &mut SaveDataLifecycleState,
) -> DoSaveDataStart {
    let started_at_tick_ms = capture_save_data_tick_ms();
    state.this_save_start_tick_ms = started_at_tick_ms;

    match open_save_data_connection(settings).await {
        Ok(connection) => {
            state.is_saving_data = true;
            DoSaveDataStart::Opened {
                connection,
                started_at_tick_ms,
            }
        }
        Err(error) => DoSaveDataStart::ConnectionOpenFailed {
            error,
            final_snapshot: SaveDataFinalSnapshot {
                path: SaveDataFinalPath::ConnectionOpenFailed,
                started_at_tick_ms,
            },
        },
    }
}

async fn open_save_data_connection(
    settings: &WorldDatabaseSettings,
) -> Result<WorldTdsClient, SaveDataConnectionError> {
    let config = settings.tds_config();
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(SaveDataConnectionError::Connect)?;
    tcp.set_nodelay(true)
        .map_err(SaveDataConnectionError::Connect)?;
    tiberius::Client::connect(config, tcp.compat_write())
        .await
        .map_err(SaveDataConnectionError::Tds)
}

/// Выполняет setup-ID участок на уже открытом World DB соединении.
///
/// Ошибки транзакционных команд возвращаются в отчёте, но не меняют порядок и
/// ветвление UPDATE: именно так caller игнорировал результаты ADO wrapper-ов.
pub(crate) async fn save_setup_ids<O: RsSetupOwner>(
    setup: &mut O,
    connection: &mut WorldTdsClient,
    snapshot: SetupIdSnapshot,
) -> SetupIdSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let player_saved = setup.save_player_id(connection, snapshot.player_id).await;
    let leave_world_saved = if player_saved {
        Some(
            setup
                .save_leave_world_id(connection, snapshot.leave_world_id)
                .await,
        )
    } else {
        None
    };

    let finish = if leave_world_saved == Some(true) {
        SetupIdSaveFinish::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        SetupIdSaveFinish::Rollback {
            error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                .await
                .err(),
        }
    };
    let log_events = vec![match &finish {
        SetupIdSaveFinish::Commit { .. } => {
            show_save_info_event(b"++Save PlayerID SUCCESS!".to_vec())
        }
        SetupIdSaveFinish::Rollback { .. } => add_log_event(b"--Save PlayerID FAILED!".to_vec()),
    }];

    SetupIdSaveReport {
        begin_error,
        player_saved,
        leave_world_saved,
        finish,
        log_events,
    }
}

/// Передаёт scalar ID прямо из сериализованного `CGame::tagDBData`.
pub(crate) async fn save_setup_ids_from_world_snapshot<O: RsSetupOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    setup: &mut O,
    connection: &mut WorldTdsClient,
) -> Result<SetupIdSaveReport, WorldSnapshotSaveBlock> {
    let (player_id, leave_world_id) = world.setup_ids();
    Ok(save_setup_ids(
        setup,
        connection,
        SetupIdSnapshot {
            player_id,
            leave_world_id,
        },
    )
    .await)
}

/// Выполняет VarData-участок на том же уже открытом World DB соединении.
///
/// `Commit` означает исходный `++Save VarDate SUCCESS!`, даже если commit
/// отказал; `Rollback` — `--Save VarDate FAILED!`, даже если rollback отказал.
/// `BlockedMissingFact` не завершает неизвестную транзакцию по догадке, поэтому
/// caller обязан остановиться на отчёте и не переиспользовать connection.
pub(crate) async fn save_general_variables<S, O>(
    variables: &S,
    database: &mut O,
    connection: &mut WorldTdsClient,
) -> GeneralVariableSaveReport
where
    S: VariableListSaveSource,
    O: RsGenVarOwner,
{
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let mut log_events = vec![add_log_event(b"Save VarDate Start...".to_vec())];
    let disposition = match save_var_data(variables, database, connection).await {
        GenVarSaveOutcome::Saved => GeneralVariableSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        },
        GenVarSaveOutcome::Failed => GeneralVariableSaveDisposition::Rollback {
            error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                .await
                .err(),
        },
    };
    match &disposition {
        GeneralVariableSaveDisposition::Commit { .. } => {
            log_events.push(show_save_info_event(b"++Save VarDate SUCCESS!".to_vec()));
        }
        GeneralVariableSaveDisposition::Rollback { .. } => {
            log_events.push(add_log_event(b"--Save VarDate FAILED!".to_vec()));
        }
    }

    GeneralVariableSaveReport {
        begin_error,
        disposition,
        log_events,
    }
}

/// Выполняет транзакционную часть одного элемента `liDBCreationPlayer`.
///
/// `Commit` требует от будущего list-owner-а удалить player и текущий node под
/// `g_CriticalSectionSavePlayerList`. `Failure` и `NullPlayer` сохраняют node и
/// разрешают перейти к следующему. После `BlockedMissingFact` connection и
/// очередь нельзя использовать дальше до решения локальной границы.
pub(crate) async fn save_new_character_entry<P, J, G>(
    snapshot: Option<&PlayerCreationSnapshot<'_, '_>>,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
) -> NewCharacterSaveReport
where
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
{
    let Some(snapshot) = snapshot else {
        return NewCharacterSaveReport {
            begin_error: None,
            disposition: NewCharacterSaveDisposition::NullPlayer,
        };
    };

    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let disposition = match player_database
        .create_player(
            Some(snapshot),
            Some(&mut *connection),
            jjc_database,
            goods_database,
        )
        .await
    {
        PlayerCreateOutcome::ReturnedTrue => NewCharacterSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        },
        PlayerCreateOutcome::ReturnedFalse => {
            NewCharacterSaveDisposition::Failure(if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            })
        }
        PlayerCreateOutcome::BlockedMissingFact(block) => {
            NewCharacterSaveDisposition::BlockedMissingFact(block)
        }
    };

    NewCharacterSaveReport {
        begin_error,
        disposition,
    }
}

/// Выполняет транзакционную часть одного ID из `lDBRestorePlayer`.
///
/// `Commit` требует от будущего list-owner-а удалить текущий node под
/// `g_CriticalSectionSavePlayerList`; `Failure` сохраняет его. Обе доказанные
/// ветви продолжают обход со следующим ID.
pub(crate) async fn save_restore_character_entry<P: RsPlayerOwner>(
    player_id: u32,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> RestoreCharacterSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let disposition = if player_database
        .restore_player(player_id, Some(&mut *connection))
        .await
    {
        RestoreCharacterSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        RestoreCharacterSaveDisposition::Failure(if begin_succeeded {
            FailedTransactionFinish::Rollback {
                error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                    .await
                    .err(),
            }
        } else {
            FailedTransactionFinish::NoTransaction
        })
    };

    RestoreCharacterSaveReport {
        begin_error,
        disposition,
    }
}

/// Выполняет транзакционную часть одного `tagDeletionPlayer`.
///
/// `Commit` требует удалить текущий node под
/// `g_CriticalSectionSavePlayerList`; `Failure` сохраняет его и разрешает
/// перейти к следующему. После `BlockedMissingFact` connection и очередь
/// нельзя использовать дальше до решения локальной time-границы.
pub(crate) async fn save_delete_character_entry<P: RsPlayerOwner>(
    snapshot: DeletionPlayerSnapshot,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> DeleteCharacterSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let disposition = match player_database
        .delete_player(
            snapshot.player_id,
            snapshot.deletion_time,
            Some(&mut *connection),
        )
        .await
    {
        PlayerDeleteOutcome::ReturnedTrue => DeleteCharacterSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        },
        PlayerDeleteOutcome::ReturnedFalse => {
            DeleteCharacterSaveDisposition::Failure(if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            })
        }
        PlayerDeleteOutcome::BlockedMissingFact(block) => {
            DeleteCharacterSaveDisposition::BlockedMissingFact(block)
        }
    };

    DeleteCharacterSaveReport {
        begin_error,
        disposition,
    }
}

/// Обходит frozen `liDBCreationPlayer`, публикует entry-log и лишь затем cleanup.
pub(crate) async fn save_new_characters_from_world_snapshot<P, J, G>(
    world: &mut WorldDbDataSaveSession<'_>,
    registry: &GoodsBasePropertiesRegistry,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<NewCharactersSaveReport, WorldSnapshotSaveBlock>
where
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
{
    let logged_count = legacy_snapshot_count("New Character", world.creation_players_len())?;
    let mut entries = Vec::with_capacity(world.creation_players_len());
    let mut log_events = vec![add_log_event(b"Save New Charactor Start...".to_vec())];
    let mut cursor = 0usize;
    let mut entry_index = 0usize;

    while cursor < world.creation_players_len() {
        let Some(player) = world.creation_player(cursor) else {
            unreachable!("cursor проверен против frozen creation-list length");
        };
        let projection = match player.db_projection(registry) {
            Ok(projection) => projection,
            Err(block) => {
                return Ok(NewCharactersSaveReport {
                    logged_count,
                    entries,
                    log_events,
                    disposition: NewCharactersSaveDisposition::BlockedProjection {
                        entry_index,
                        block,
                    },
                });
            }
        };
        let snapshot = match projection.creation_snapshot() {
            Ok(snapshot) => snapshot,
            Err(block) => {
                return Ok(NewCharactersSaveReport {
                    logged_count,
                    entries,
                    log_events,
                    disposition: NewCharactersSaveDisposition::BlockedProjection {
                        entry_index,
                        block,
                    },
                });
            }
        };
        let report = save_new_character_entry(
            Some(&snapshot),
            player_database,
            jjc_database,
            goods_database,
            connection,
        )
        .await;
        let succeeded = matches!(
            &report.disposition,
            NewCharacterSaveDisposition::Commit { .. }
        );
        let event = match &report.disposition {
            NewCharacterSaveDisposition::Commit { .. } => Some(player_success_log(
                b"Create New Charactor SUCCESS : ",
                player,
            )),
            NewCharacterSaveDisposition::Failure(_) => {
                Some(add_log_event(b"--Create New Charactor FAILED.".to_vec()))
            }
            NewCharacterSaveDisposition::NullPlayer => Some(add_log_event(
                b"**Create New Charactor NULL Pointer!!!!!!!!!!!!!!!!".to_vec(),
            )),
            NewCharacterSaveDisposition::BlockedMissingFact(_) => None,
        };
        if matches!(
            &report.disposition,
            NewCharacterSaveDisposition::BlockedMissingFact(_)
        ) {
            let NewCharacterSaveReport {
                begin_error,
                disposition,
            } = report;
            let NewCharacterSaveDisposition::BlockedMissingFact(block) = disposition else {
                unreachable!("variant проверен выше");
            };
            return Ok(NewCharactersSaveReport {
                logged_count,
                entries,
                log_events,
                disposition: NewCharactersSaveDisposition::BlockedCreate {
                    entry_index,
                    begin_error,
                    block,
                },
            });
        }
        let event = event.expect("обычная New Character ветка всегда пишет log");
        log_events.push(event);
        if let Err(block) = publish_save_data_log(
            log_sink,
            log_events.last().expect("event только что добавлен"),
        ) {
            return Ok(NewCharactersSaveReport {
                logged_count,
                entries,
                log_events,
                disposition: NewCharactersSaveDisposition::BlockedLog {
                    entry_index,
                    entry: report,
                    block,
                },
            });
        }
        entries.push(report);
        if succeeded {
            world.remove_creation_player(cursor);
        } else {
            cursor += 1;
        }
        entry_index += 1;
    }

    Ok(NewCharactersSaveReport {
        logged_count,
        entries,
        disposition: NewCharactersSaveDisposition::Complete,
        log_events,
    })
}

/// Выполняет Save Faction над frozen owner-ами; final log предшествует clear.
pub(crate) async fn save_factions_from_world_snapshot<F: RsFactionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    organizing: &COrganizingCtrl,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<SaveFactionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Save Faction", world.save_factions_len())?;
    let mut entries = Vec::with_capacity(world.save_factions_len());
    let mut log_events = vec![add_log_event(b"Save Faction Data Start...".to_vec())];

    while let Some(faction_id) = world
        .first_saved_faction_mut()
        .map(|faction| faction.faction_id())
    {
        let entry_index = entries.len();
        let canonical_goods_war_count = CGame::get_faction_by_id(organizing, faction_id)
            .map(|faction| faction.goods_war_count());
        let projection = match world.first_saved_faction_mut() {
            Some(faction) => FactionSaveSnapshot::from_faction(faction, canonical_goods_war_count),
            None => unreachable!("faction front исчез между двумя эксклюзивными borrow"),
        };
        let projection = match projection {
            Ok(projection) => projection,
            Err(block) => {
                return Ok(SaveFactionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveFactionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error: None,
                        block: SaveFactionPhaseBlock::Projection(block),
                    },
                });
            }
        };

        let entry = {
            let mut projection = projection;
            let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
                .await
                .err();
            let save_outcome = faction_database
                .save_faction(Some(&mut projection), Some(&mut *connection))
                .await;
            let save_returned = match save_outcome {
                FactionSaveOutcome::ReturnedTrue => true,
                FactionSaveOutcome::ReturnedFalse => false,
                FactionSaveOutcome::BlockedMissingFact(block) => {
                    return Ok(SaveFactionSaveReport {
                        entries,
                        log_events,
                        disposition: SaveFactionSaveDisposition::BlockedMissingFact {
                            entry_index,
                            begin_error,
                            block: SaveFactionPhaseBlock::Owner(block),
                        },
                    });
                }
            };
            let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err();
            SaveFactionEntrySaveReport {
                begin_error,
                save_returned,
                commit_error,
                cleanup: SaveFactionEntryCleanup::RemoveNodeThenDestroySnapshot,
            }
        };
        entries.push(entry);
        world.remove_first_saved_faction();
    }

    log_events.push(add_log_event(
        format!("++Save {} Faction Data SUCCESS!", logged_count as i32).into_bytes(),
    ));
    if let Err(block) = publish_save_data_log(
        log_sink,
        log_events.last().expect("final event только что добавлен"),
    ) {
        return Ok(SaveFactionSaveReport {
            entries,
            disposition: SaveFactionSaveDisposition::BlockedLog {
                logged_count,
                block,
            },
            log_events,
        });
    }
    world.clear_saved_faction_nodes();
    Ok(SaveFactionSaveReport {
        entries,
        disposition: SaveFactionSaveDisposition::Complete(SaveFactionFinalClear { logged_count }),
        log_events,
    })
}

/// Обходит frozen `lDBRestorePlayer`; entry-log предшествует судьбе node.
pub(crate) async fn save_restore_characters_from_world_snapshot<P: RsPlayerOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<RestoreCharactersSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Restore Character", world.restore_players_len())?;
    let mut entries = Vec::with_capacity(world.restore_players_len());
    let mut log_events = vec![add_log_event(b"Save Restore Charactor Start...".to_vec())];
    let mut cursor = 0usize;

    while cursor < world.restore_players_len() {
        let player_id = world
            .restore_player_id(cursor)
            .expect("cursor проверен против frozen restore-list length");
        let report = save_restore_character_entry(player_id, player_database, connection).await;
        let succeeded = matches!(
            &report.disposition,
            RestoreCharacterSaveDisposition::Commit { .. }
        );
        let event = match &report.disposition {
            RestoreCharacterSaveDisposition::Commit { .. } => show_save_info_event(
                format!("++Restore Charactor SUCCESS : {}", player_id as i32).into_bytes(),
            ),
            RestoreCharacterSaveDisposition::Failure(_) => {
                add_log_event(b"--Restore Charactor FAILED!".to_vec())
            }
        };
        log_events.push(event);
        if let Err(block) = publish_save_data_log(
            log_sink,
            log_events.last().expect("event только что добавлен"),
        ) {
            let entry_index = entries.len();
            return Ok(RestoreCharactersSaveReport {
                logged_count,
                entries,
                disposition: RestoreCharactersSaveDisposition::BlockedLog {
                    entry_index,
                    entry: Box::new(report),
                    block,
                },
                log_events,
            });
        }
        entries.push(report);
        if succeeded {
            world.remove_restore_player(cursor);
        } else {
            cursor += 1;
        }
    }

    Ok(RestoreCharactersSaveReport {
        logged_count,
        entries,
        disposition: RestoreCharactersSaveDisposition::Complete,
        log_events,
    })
}

/// Обходит frozen deletion-list; entry-log предшествует судьбе текущего node.
pub(crate) async fn save_delete_characters_from_world_snapshot<P: RsPlayerOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<DeleteCharactersSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Delete Character", world.deletion_players_len())?;
    let mut entries = Vec::with_capacity(world.deletion_players_len());
    let mut log_events = vec![add_log_event(b"Save Delete Charactor ...".to_vec())];
    let mut cursor = 0usize;
    let mut entry_index = 0usize;

    while cursor < world.deletion_players_len() {
        let snapshot = world
            .deletion_player(cursor)
            .expect("cursor проверен против frozen deletion-list length");
        let report = save_delete_character_entry(snapshot, player_database, connection).await;
        let succeeded = matches!(
            &report.disposition,
            DeleteCharacterSaveDisposition::Commit { .. }
        );
        let event = match &report.disposition {
            DeleteCharacterSaveDisposition::Commit { .. } => Some(show_save_info_event(
                format!("++Delete Charactor SUCCESS : {}", snapshot.player_id as i32).into_bytes(),
            )),
            DeleteCharacterSaveDisposition::Failure(_) => {
                Some(add_log_event(b"--Delete Charactor FAILED!".to_vec()))
            }
            DeleteCharacterSaveDisposition::BlockedMissingFact(_) => None,
        };
        if matches!(
            &report.disposition,
            DeleteCharacterSaveDisposition::BlockedMissingFact(_)
        ) {
            let DeleteCharacterSaveReport {
                begin_error,
                disposition,
            } = report;
            let DeleteCharacterSaveDisposition::BlockedMissingFact(block) = disposition else {
                unreachable!("variant проверен выше");
            };
            return Ok(DeleteCharactersSaveReport {
                logged_count,
                entries,
                log_events,
                disposition: DeleteCharactersSaveDisposition::BlockedMissingFact {
                    entry_index,
                    begin_error,
                    block,
                },
            });
        }
        let event = event.expect("обычная Delete Character ветка всегда пишет log");
        log_events.push(event);
        if let Err(block) = publish_save_data_log(
            log_sink,
            log_events.last().expect("event только что добавлен"),
        ) {
            return Ok(DeleteCharactersSaveReport {
                logged_count,
                entries,
                disposition: DeleteCharactersSaveDisposition::BlockedLog {
                    entry_index,
                    entry: report,
                    block,
                },
                log_events,
            });
        }
        entries.push(report);
        if succeeded {
            world.remove_deletion_player(cursor);
        } else {
            cursor += 1;
        }
        entry_index += 1;
    }

    Ok(DeleteCharactersSaveReport {
        logged_count,
        entries,
        disposition: DeleteCharactersSaveDisposition::Complete,
        log_events,
    })
}

/// Выполняет Delete Union DB-вызовы в исходном list-order.
///
/// Bool `DelConfederation` и ошибки транзакционных wrapper-ов только попадают в
/// отчёт. После возврата caller обязан очистить весь live `listDeleteUnions` и
/// написать success-log с `report.clear.logged_count`; ни один entry не
/// сохраняется для retry.
pub(crate) async fn save_delete_unions<U: RsUnionOwner>(
    snapshot: DeleteUnionListSnapshot<'_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
) -> DeleteUnionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.union_ids.len());
    let log_events = vec![add_log_event(b"Save Delete Union ...".to_vec())];
    for &union_id in snapshot.union_ids {
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let delete_succeeded = union_database
            .del_confederation(union_id, Some(&mut *connection))
            .await;
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(DeleteUnionEntrySaveReport {
            begin_error,
            delete_succeeded,
            commit_error,
        });
    }

    DeleteUnionSaveReport {
        entries,
        clear: DeleteUnionClear {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

/// Выполняет Delete Union над frozen list и применяет доказанный общий clear.
pub(crate) async fn save_delete_unions_from_world_snapshot<U: RsUnionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
) -> Result<DeleteUnionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Delete Union", world.delete_union_ids().len())?;
    let mut report = save_delete_unions(
        DeleteUnionListSnapshot {
            union_ids: world.delete_union_ids(),
            logged_count,
        },
        union_database,
        connection,
    )
    .await;
    world.clear_delete_unions();
    report.log_events.push(add_log_event(
        format!(
            "++Delte {} Unions SUCCESS!",
            report.clear.logged_count as i32
        )
        .into_bytes(),
    ));
    Ok(report)
}

/// Выполняет Delete Faction DB-вызовы в исходном list-order.
///
/// Bool `DelFaction` и ошибки транзакционных wrapper-ов только попадают в
/// отчёт. После возврата caller обязан очистить весь live
/// `listDeleteFactions` и написать success-log с
/// `report.clear.logged_count`; ни один entry не сохраняется для retry.
pub(crate) async fn save_delete_factions<F: RsFactionOwner>(
    snapshot: DeleteFactionListSnapshot<'_>,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
) -> DeleteFactionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.faction_ids.len());
    let log_events = vec![add_log_event(b"Save Delete Faction Start...".to_vec())];
    for &faction_id in snapshot.faction_ids {
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let delete_succeeded = faction_database
            .del_faction(faction_id, Some(&mut *connection))
            .await;
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(DeleteFactionEntrySaveReport {
            begin_error,
            delete_succeeded,
            commit_error,
        });
    }

    DeleteFactionSaveReport {
        entries,
        clear: DeleteFactionClear {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

/// Выполняет Delete Faction над frozen list и применяет доказанный общий clear.
pub(crate) async fn save_delete_factions_from_world_snapshot<F: RsFactionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
) -> Result<DeleteFactionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Delete Faction", world.delete_faction_ids().len())?;
    let mut report = save_delete_factions(
        DeleteFactionListSnapshot {
            faction_ids: world.delete_faction_ids(),
            logged_count,
        },
        faction_database,
        connection,
    )
    .await;
    world.clear_delete_factions();
    report.log_events.push(add_log_event(
        format!(
            "++ Delte {} Factions SUCCESS!",
            report.clear.logged_count as i32
        )
        .into_bytes(),
    ));
    Ok(report)
}

/// Выполняет Save Faction Data DB-вызовы в исходном live list-order.
///
/// Для каждого обычного bool-результата commit вызывается безусловно, после чего
/// caller обязан применить `cleanup` до следующего node. `BlockedMissingFact`
/// запрещает продолжать работу с connection и оставляет текущую судьбу открытой.
pub(crate) async fn save_factions<F: RsFactionOwner>(
    snapshot: SaveFactionListSnapshot<'_, '_>,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
) -> SaveFactionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.factions.len());
    let mut log_events = vec![add_log_event(b"Save Faction Data Start...".to_vec())];

    for (entry_index, faction_snapshot) in snapshot.factions.iter_mut().enumerate() {
        let cleanup = if faction_snapshot.is_some() {
            SaveFactionEntryCleanup::RemoveNodeThenDestroySnapshot
        } else {
            SaveFactionEntryCleanup::RemoveNode
        };
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_outcome = faction_database
            .save_faction(faction_snapshot.as_mut(), Some(&mut *connection))
            .await;
        let save_returned = match save_outcome {
            FactionSaveOutcome::ReturnedTrue => true,
            FactionSaveOutcome::ReturnedFalse => false,
            FactionSaveOutcome::BlockedMissingFact(block) => {
                return SaveFactionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveFactionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error,
                        block: SaveFactionPhaseBlock::Owner(block),
                    },
                };
            }
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(SaveFactionEntrySaveReport {
            begin_error,
            save_returned,
            commit_error,
            cleanup,
        });
    }

    log_events.push(add_log_event(
        format!(
            "++Save {} Faction Data SUCCESS!",
            snapshot.logged_count as i32
        )
        .into_bytes(),
    ));
    SaveFactionSaveReport {
        entries,
        disposition: SaveFactionSaveDisposition::Complete(SaveFactionFinalClear {
            logged_count: snapshot.logged_count,
        }),
        log_events,
    }
}

/// Выполняет Save Union Data DB-вызовы в исходном live list-order.
///
/// Для каждого обычного bool-результата commit вызывается безусловно, после чего
/// caller обязан применить `cleanup` до следующего node. `BlockedMissingFact`
/// запрещает продолжать работу с connection и оставляет текущую судьбу открытой.
pub(crate) async fn save_unions<U: RsUnionOwner>(
    snapshot: SaveUnionListSnapshot<'_, '_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
) -> SaveUnionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.unions.len());
    let mut log_events = vec![add_log_event(b"Save Union Data Start...".to_vec())];

    for (entry_index, union_snapshot) in snapshot.unions.iter_mut().enumerate() {
        let cleanup = if union_snapshot.is_some() {
            SaveUnionEntryCleanup::RemoveNodeThenDestroySnapshot
        } else {
            SaveUnionEntryCleanup::RemoveNode
        };
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_outcome = union_database
            .save_confederation(union_snapshot.as_ref(), Some(&mut *connection))
            .await;
        let save_returned = match save_outcome {
            UnionSaveOutcome::ReturnedTrue => true,
            UnionSaveOutcome::ReturnedFalse => false,
            UnionSaveOutcome::BlockedMissingFact(block) => {
                return SaveUnionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveUnionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error,
                        block,
                    },
                };
            }
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(SaveUnionEntrySaveReport {
            begin_error,
            save_returned,
            commit_error,
            cleanup,
        });
    }

    log_events.push(add_log_event(
        format!(
            "++Save {} Union Data SUCCESS!",
            snapshot.logged_count as i32
        )
        .into_bytes(),
    ));
    SaveUnionSaveReport {
        entries,
        disposition: SaveUnionSaveDisposition::Complete(SaveUnionFinalClear {
            logged_count: snapshot.logged_count,
        }),
        log_events,
    }
}

/// Выполняет Save Union над frozen owner-ами; final log предшествует clear.
pub(crate) async fn save_unions_from_world_snapshot<U: RsUnionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<SaveUnionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Save Union", world.save_unions_len())?;
    let mut entries = Vec::with_capacity(world.save_unions_len());
    let mut log_events = vec![add_log_event(b"Save Union Data Start...".to_vec())];

    while world.first_saved_union().is_some() {
        let entry_index = entries.len();
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_outcome = {
            let union = world
                .first_saved_union()
                .expect("union front проверен в условии цикла");
            let projection = UnionSaveSnapshot::from_union(union);
            union_database
                .save_confederation(Some(&projection), Some(&mut *connection))
                .await
        };
        let save_returned = match save_outcome {
            UnionSaveOutcome::ReturnedTrue => true,
            UnionSaveOutcome::ReturnedFalse => false,
            UnionSaveOutcome::BlockedMissingFact(block) => {
                return Ok(SaveUnionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveUnionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error,
                        block,
                    },
                });
            }
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(SaveUnionEntrySaveReport {
            begin_error,
            save_returned,
            commit_error,
            cleanup: SaveUnionEntryCleanup::RemoveNodeThenDestroySnapshot,
        });
        world.remove_first_saved_union();
    }

    log_events.push(add_log_event(
        format!("++Save {} Union Data SUCCESS!", logged_count as i32).into_bytes(),
    ));
    if let Err(block) = publish_save_data_log(
        log_sink,
        log_events.last().expect("final event только что добавлен"),
    ) {
        return Ok(SaveUnionSaveReport {
            entries,
            disposition: SaveUnionSaveDisposition::BlockedLog {
                logged_count,
                block,
            },
            log_events,
        });
    }
    world.clear_saved_union_nodes();
    Ok(SaveUnionSaveReport {
        entries,
        disposition: SaveUnionSaveDisposition::Complete(SaveUnionFinalClear { logged_count }),
        log_events,
    })
}

/// Выполняет доказанный префикс `DoSaveData` от setup-ID до Save Union Data.
///
/// Аргументы остаются раздельными, потому что исходник использовал независимые
/// DB-owner-ы и snapshot-owner-ы. На первой safe-границе функция возвращается,
/// не передавая connection следующей фазе; уже применённые DB/container эффекты
/// и отчёты сохраняются буквально.
#[allow(
    clippy::too_many_arguments,
    reason = "точные границы DoSaveData требуют отдельных component DB-owner-ов"
)]
pub(crate) async fn do_save_data_through_unions<S, O, V, P, J, G, U, F>(
    world: &mut WorldDbDataSaveSession<'_>,
    variables: &S,
    registry: &GoodsBasePropertiesRegistry,
    organizing: &COrganizingCtrl,
    setup_database: &mut O,
    variable_database: &mut V,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    union_database: &mut U,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> DoSaveDataThroughUnionsReport
where
    S: VariableListSaveSource,
    O: RsSetupOwner,
    V: RsGenVarOwner,
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
    U: RsUnionOwner,
    F: RsFactionOwner,
{
    let mut phases = DoSaveDataThroughUnionsPhases::default();

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Variables Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SetupIds,
                checkpoint: SaveDataLogCheckpoint::SaveVariablesStart,
                block,
            },
        };
    }

    let setup_ids =
        match save_setup_ids_from_world_snapshot(world, setup_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::SetupIds,
                        block,
                    },
                };
            }
        };
    let setup_log_index = setup_ids.log_events.len() - 1;
    let setup_log_block =
        publish_save_data_log(log_sink, &setup_ids.log_events[setup_log_index]).err();
    phases.setup_ids = Some(setup_ids);
    if let Some(block) = setup_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SetupIds,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(setup_log_index),
                block,
            },
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save VarDate Start...".to_vec()))
    {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GeneralVariables,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let variables_report = save_general_variables(variables, variable_database, connection).await;
    let variables_log_block = variables_report
        .log_events
        .get(1)
        .and_then(|event| publish_save_data_log(log_sink, event).err());
    phases.general_variables = Some(variables_report);
    if let Some(block) = variables_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GeneralVariables,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }
    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save New Charactor Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::NewCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let new_characters = match save_new_characters_from_world_snapshot(
        world,
        registry,
        player_database,
        jjc_database,
        goods_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::NewCharacters,
                    block,
                },
            };
        }
    };
    let new_characters_log_block = match &new_characters.disposition {
        NewCharactersSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let new_characters_log_index = new_characters.log_events.len().saturating_sub(1);
    let new_characters_blocked = !matches!(
        &new_characters.disposition,
        NewCharactersSaveDisposition::Complete
    );
    phases.new_characters = Some(new_characters);
    if let Some(block) = new_characters_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::NewCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(new_characters_log_index),
                block,
            },
        };
    }
    if new_characters_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::NewCharacters,
            ),
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Restore Charactor Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::RestoreCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let restore_characters = match save_restore_characters_from_world_snapshot(
        world,
        player_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::RestoreCharacters,
                    block,
                },
            };
        }
    };
    let restore_log_block = match &restore_characters.disposition {
        RestoreCharactersSaveDisposition::BlockedLog { block, .. } => Some(*block),
        RestoreCharactersSaveDisposition::Complete => None,
    };
    let restore_log_index = restore_characters.log_events.len().saturating_sub(1);
    phases.restore_characters = Some(restore_characters);
    if let Some(block) = restore_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::RestoreCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(restore_log_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Delete Charactor ...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let delete_characters = match save_delete_characters_from_world_snapshot(
        world,
        player_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::DeleteCharacters,
                    block,
                },
            };
        }
    };
    let delete_characters_log_block = match &delete_characters.disposition {
        DeleteCharactersSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let delete_characters_log_index = delete_characters.log_events.len().saturating_sub(1);
    let delete_characters_blocked = !matches!(
        &delete_characters.disposition,
        DeleteCharactersSaveDisposition::Complete
    );
    phases.delete_characters = Some(delete_characters);
    if let Some(block) = delete_characters_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(delete_characters_log_index),
                block,
            },
        };
    }
    if delete_characters_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::DeleteCharacters,
            ),
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save Delete Union ...".to_vec()))
    {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let delete_unions =
        match save_delete_unions_from_world_snapshot(world, union_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::DeleteUnions,
                        block,
                    },
                };
            }
        };
    let delete_unions_log_index = delete_unions.log_events.len() - 1;
    let delete_unions_log_block =
        publish_save_data_log(log_sink, &delete_unions.log_events[delete_unions_log_index]).err();
    phases.delete_unions = Some(delete_unions);
    if let Some(block) = delete_unions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(delete_unions_log_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Delete Faction Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let delete_factions =
        match save_delete_factions_from_world_snapshot(world, faction_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::DeleteFactions,
                        block,
                    },
                };
            }
        };
    let delete_factions_log_index = delete_factions.log_events.len() - 1;
    let delete_factions_log_block = publish_save_data_log(
        log_sink,
        &delete_factions.log_events[delete_factions_log_index],
    )
    .err();
    phases.delete_factions = Some(delete_factions);
    if let Some(block) = delete_factions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(delete_factions_log_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Faction Data Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let save_factions = match save_factions_from_world_snapshot(
        world,
        organizing,
        faction_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::SaveFactions,
                    block,
                },
            };
        }
    };
    let save_factions_log_block = match &save_factions.disposition {
        SaveFactionSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let save_factions_log_index = save_factions.log_events.len().saturating_sub(1);
    let save_factions_blocked = matches!(
        &save_factions.disposition,
        SaveFactionSaveDisposition::BlockedMissingFact { .. }
    );
    phases.save_factions = Some(save_factions);
    if let Some(block) = save_factions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(save_factions_log_index),
                block,
            },
        };
    }
    if save_factions_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::SaveFactions,
            ),
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Union Data Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let save_unions =
        match save_unions_from_world_snapshot(world, union_database, connection, log_sink).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::SaveUnions,
                        block,
                    },
                };
            }
        };
    let save_unions_log_block = match &save_unions.disposition {
        SaveUnionSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let save_unions_log_index = save_unions.log_events.len().saturating_sub(1);
    let save_unions_blocked = matches!(
        &save_unions.disposition,
        SaveUnionSaveDisposition::BlockedMissingFact { .. }
    );
    phases.save_unions = Some(save_unions);
    if let Some(block) = save_unions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(save_unions_log_index),
                block,
            },
        };
    }
    if save_unions_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::SaveUnions,
            ),
        };
    }

    let counters = SaveDataEarlyCounters {
        created: phases
            .new_characters
            .as_ref()
            .expect("New Character phase завершена")
            .logged_count,
        cancel_deleted: phases
            .restore_characters
            .as_ref()
            .expect("Restore Character phase завершена")
            .logged_count,
        sign_deleted: phases
            .delete_characters
            .as_ref()
            .expect("Delete Character phase завершена")
            .logged_count,
    };
    DoSaveDataThroughUnionsReport {
        phases,
        disposition: DoSaveDataThroughUnionsDisposition::ContinueWithRegion(counters),
    }
}

/// Выполняет Save Region Data DB-вызовы в исходном live list-order.
///
/// Для каждого обычного bool-результата commit вызывается безусловно. Caller
/// обязан применить entry-cleanup без удаления node, а после всего traversal —
/// единый `clear`; этот порядок отличает region-list от faction/union-list.
pub(crate) async fn save_regions<R: RsRegionOwner>(
    snapshot: SaveRegionListSnapshot<'_>,
    region_database: &mut R,
    connection: &mut WorldTdsClient,
) -> SaveRegionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.regions.len());
    let mut log_events = vec![add_log_event(b"Save Region Data...".to_vec())];

    for region_snapshot in snapshot.regions {
        entries
            .push(save_region_entry(region_snapshot.as_ref(), region_database, connection).await);
    }

    log_events.push(add_log_event(
        format!(
            "++Save {} Region Data SUCCESS.",
            snapshot.logged_count as i32
        )
        .into_bytes(),
    ));
    SaveRegionSaveReport {
        entries,
        clear: SaveRegionFinalClear {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

/// Сохраняет frozen region-list и уничтожает values, оставляя final node-clear caller-у.
pub(crate) async fn save_regions_from_world_snapshot<R: RsRegionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    region_database: &mut R,
    connection: &mut WorldTdsClient,
) -> Result<SaveRegionSaveReport, WorldSnapshotSaveBlock> {
    let region_count = world.regions_len();
    let logged_count = legacy_snapshot_count("Save Region", region_count)?;
    let mut entries = Vec::with_capacity(region_count);
    let mut log_events = vec![add_log_event(b"Save Region Data...".to_vec())];
    for index in 0..region_count {
        let region = world.region(index).expect("frozen region node не исчезает");
        let entry = save_region_entry(region.as_ref(), region_database, connection).await;
        if matches!(
            entry.cleanup,
            SaveRegionEntryCleanup::DestroySnapshotAndRetainNode
        ) {
            world.destroy_saved_region(index);
        }
        entries.push(entry);
    }
    log_events.push(add_log_event(
        format!("++Save {} Region Data SUCCESS.", logged_count as i32).into_bytes(),
    ));
    Ok(SaveRegionSaveReport {
        entries,
        clear: SaveRegionFinalClear { logged_count },
        log_events,
    })
}

async fn save_region_entry<R: RsRegionOwner>(
    region_snapshot: Option<&RegionSaveSnapshot>,
    region_database: &mut R,
    connection: &mut WorldTdsClient,
) -> SaveRegionEntrySaveReport {
    let cleanup = if region_snapshot.is_some() {
        SaveRegionEntryCleanup::DestroySnapshotAndRetainNode
    } else {
        SaveRegionEntryCleanup::RetainNode
    };
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let save_returned = region_database
        .save(region_snapshot, Some(&mut *connection))
        .await;
    let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
        .await
        .err();
    SaveRegionEntrySaveReport {
        begin_error,
        save_returned,
        commit_error,
        cleanup,
    }
}

/// Выполняет Save HonorRanks на том же уже открытом World DB соединении.
///
/// `InsertHonorRanks == false` пропускает второй owner и входит в тот же
/// abnormal-catch, что и `SaveHonorRanks == false`. После
/// `BlockedMissingFact` caller обязан остановить lifecycle и не передавать
/// connection следующей Save GodsBattle phase.
pub(crate) async fn save_honor_ranks<P: RsPlayerOwner>(
    snapshot: &mut HonorRanksDbDataSnapshot,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> HonorRanksTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let insert_succeeded = player_database
        .insert_honor_ranks(snapshot, Some(&mut *connection))
        .await;

    let (save_returned, disposition) = if !insert_succeeded {
        (
            None,
            HonorRanksSaveDisposition::Failure(if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            }),
        )
    } else {
        match player_database
            .save_honor_ranks(snapshot, Some(&mut *connection))
            .await
        {
            HonorRanksSaveOutcome::ReturnedTrue => (
                Some(true),
                HonorRanksSaveDisposition::Commit {
                    error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                        .await
                        .err(),
                },
            ),
            HonorRanksSaveOutcome::ReturnedFalse => (
                Some(false),
                HonorRanksSaveDisposition::Failure(if begin_succeeded {
                    FailedTransactionFinish::Rollback {
                        error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                            .await
                            .err(),
                    }
                } else {
                    FailedTransactionFinish::NoTransaction
                }),
            ),
            HonorRanksSaveOutcome::BlockedMissingFact(block) => {
                (None, HonorRanksSaveDisposition::BlockedMissingFact(block))
            }
        }
    };

    let mut log_events = vec![add_log_event(b"Save HonorRanks start...".to_vec())];
    match &disposition {
        HonorRanksSaveDisposition::Commit { .. } => {
            log_events.push(add_log_event(b"+Success to save HonorRanks!!!l".to_vec()))
        }
        HonorRanksSaveDisposition::Failure(_) => {
            log_events.push(add_log_event(b"Save HonorRanks start ABNORMAL".to_vec()))
        }
        HonorRanksSaveDisposition::BlockedMissingFact(_) => {}
    }
    HonorRanksTransactionSaveReport {
        begin_error,
        insert_succeeded,
        save_returned,
        disposition,
        log_events,
    }
}

/// Передаёт отдельную DB-копию `CHonorRanks`, не затрагивая live rank-list.
pub(crate) async fn save_honor_ranks_from_world_snapshot<P: RsPlayerOwner>(
    honor_ranks: &mut CHonorRanks,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> Result<HonorRanksTransactionSaveReport, WorldSnapshotSaveBlock> {
    let snapshot = honor_ranks
        .db_data_mut()
        .ok_or(WorldSnapshotSaveBlock::MissingHonorRanks)?;
    Ok(save_honor_ranks(snapshot, player_database, connection).await)
}

/// Выполняет отдельную Save GodsBattle Belief/XYD transaction phase.
///
/// `false` условно откатывается и не запрещает caller-у перейти к следующей
/// NPC-фазе. `true` означает исходный success-log даже при ошибке commit.
pub(crate) async fn save_gods_battle_faction_xyd<G: RsGodsBattleOwner>(
    snapshot: GodsBattleFactionXydSnapshot,
    gods_battle_database: &mut G,
    connection: &mut WorldTdsClient,
) -> GodsBattleTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let save_returned = gods_battle_database
        .save_faction_xyd(snapshot, Some(&mut *connection))
        .await;
    let disposition = if save_returned {
        GodsBattleSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        GodsBattleSaveDisposition::Failure(if begin_succeeded {
            FailedTransactionFinish::Rollback {
                error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                    .await
                    .err(),
            }
        } else {
            FailedTransactionFinish::NoTransaction
        })
    };

    let result_log = if matches!(&disposition, GodsBattleSaveDisposition::Commit { .. }) {
        add_log_event(b"\xA1\xF0Success to save GodsBattle-Belief\xA1\xA3".to_vec())
    } else {
        add_log_event(b"\xA1\xF9Save GodsBattle-Belief ABNORMAL!".to_vec())
    };
    GodsBattleTransactionSaveReport {
        operation: GodsBattleSaveOperation::FactionXyd,
        begin_error,
        save_returned,
        disposition,
        log_events: vec![
            add_log_event(b"Save GodsBattle...start...".to_vec()),
            result_log,
        ],
    }
}

/// Выполняет следующую отдельную Save GodsBattle NPC transaction phase.
///
/// Функция принимает ordered snapshot, но не вызывает no-argument overload и
/// не начинает следующую EnemyFactions phase.
pub(crate) async fn save_gods_battle_npc_factions<G: RsGodsBattleOwner>(
    snapshot: &[GodsBattleNpcFactionSnapshot],
    gods_battle_database: &mut G,
    connection: &mut WorldTdsClient,
) -> GodsBattleTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let save_returned = gods_battle_database
        .save_npc_faction(snapshot, Some(&mut *connection))
        .await;
    let disposition = if save_returned {
        GodsBattleSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        GodsBattleSaveDisposition::Failure(if begin_succeeded {
            FailedTransactionFinish::Rollback {
                error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                    .await
                    .err(),
            }
        } else {
            FailedTransactionFinish::NoTransaction
        })
    };

    let result_log = if matches!(&disposition, GodsBattleSaveDisposition::Commit { .. }) {
        add_log_event(b"\xA1\xF0Success to save GodsBattle-NPC ...start...".to_vec())
    } else {
        add_log_event(b"\xA1\xF9Save GodsBattle-NPC ABNORMAL!".to_vec())
    };
    GodsBattleTransactionSaveReport {
        operation: GodsBattleSaveOperation::NpcFaction,
        begin_error,
        save_returned,
        disposition,
        log_events: vec![result_log],
    }
}

/// Выполняет следующую отдельную EnemyFactions transaction phase.
///
/// Оба обычных bool-результата owner-а безусловно получают commit и одну
/// normal cleanup-обязанность. После `BlockedMissingFact` caller обязан
/// остановить lifecycle и не передавать connection следующей Country-фазе.
pub(crate) async fn save_enemy_factions<E: RsEnemyFactionsOwner>(
    snapshot: &[Option<EnemyFactionSaveSnapshot>],
    enemy_factions_database: &mut E,
    connection: &mut WorldTdsClient,
) -> EnemyFactionsTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();

    let (save_returned, blocked) = match enemy_factions_database
        .save_all_enemy_factions(snapshot, Some(&mut *connection))
        .await
    {
        EnemyFactionsSaveOutcome::ReturnedTrue => (Some(true), None),
        EnemyFactionsSaveOutcome::ReturnedFalse => (Some(false), None),
        EnemyFactionsSaveOutcome::BlockedMissingFact(block) => (None, Some(block)),
    };
    let disposition = if let Some(block) = blocked {
        EnemyFactionsTransactionSaveDisposition::BlockedMissingFact(block)
    } else {
        EnemyFactionsTransactionSaveDisposition::Complete {
            commit_error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
            cleanup: EnemyFactionsFinalCleanup::DestroyValuesThenClearNodes,
        }
    };

    EnemyFactionsTransactionSaveReport {
        begin_error,
        save_returned,
        disposition,
        log_events: vec![add_log_event(b"Save EnemyFactions...".to_vec())],
    }
}

/// Сохраняет frozen enemy-list; cleanup выполняется только на normal-path.
pub(crate) async fn save_enemy_factions_from_world_snapshot<E: RsEnemyFactionsOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    enemy_factions_database: &mut E,
    connection: &mut WorldTdsClient,
) -> EnemyFactionsTransactionSaveReport {
    let mut report =
        save_enemy_factions(world.enemy_factions(), enemy_factions_database, connection).await;
    if matches!(
        &report.disposition,
        EnemyFactionsTransactionSaveDisposition::Complete { .. }
    ) {
        world.clear_saved_enemy_factions();
        report
            .log_events
            .push(add_log_event(b"++Save EnemyFactions SUCCESS.".to_vec()));
    }
    report
}

/// Выполняет Update Country Data DB-вызовы в исходном live list-order.
///
/// Для каждого обычного bool-результата commit вызывается безусловно. Caller
/// обязан удалить текущий node, затем уничтожить non-null snapshot; после
/// traversal он очищает оставшиеся nodes и пишет success-log с исходным count.
pub(crate) async fn save_countries<C: DbCountryOwner>(
    snapshot: SaveCountryListSnapshot<'_>,
    country_database: &mut C,
    connection: &mut WorldTdsClient,
) -> SaveCountrySaveReport {
    let mut entries = Vec::with_capacity(snapshot.countries.len());
    let mut log_events = vec![add_log_event(b"Update Country Data...".to_vec())];

    for country_snapshot in snapshot.countries {
        entries.push(
            save_country_entry(country_snapshot.as_ref(), country_database, connection).await,
        );
    }

    log_events.push(add_log_event(
        format!(
            "++Update {} Country Data SUCCESS.",
            snapshot.logged_count as i32
        )
        .into_bytes(),
    ));
    SaveCountrySaveReport {
        entries,
        clear: SaveCountryFinalClear {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

/// Сохраняет frozen country-list и удаляет каждый законченный save-owner.
pub(crate) async fn save_countries_from_world_snapshot<C: DbCountryOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    country_database: &mut C,
    connection: &mut WorldTdsClient,
) -> Result<SaveCountrySaveReport, WorldSnapshotSaveBlock> {
    let country_count = world.countries_len();
    let logged_count = legacy_snapshot_count("Update Country", country_count)?;
    let mut entries = Vec::with_capacity(country_count);
    let mut log_events = vec![add_log_event(b"Update Country Data...".to_vec())];
    while let Some(country) = world.first_country() {
        let entry = save_country_entry(country, country_database, connection).await;
        world.remove_first_saved_country();
        entries.push(entry);
    }
    log_events.push(add_log_event(
        format!("++Update {} Country Data SUCCESS.", logged_count as i32).into_bytes(),
    ));
    Ok(SaveCountrySaveReport {
        entries,
        clear: SaveCountryFinalClear { logged_count },
        log_events,
    })
}

async fn save_country_entry<C: DbCountryOwner>(
    country_snapshot: Option<&CountrySaveSnapshot>,
    country_database: &mut C,
    connection: &mut WorldTdsClient,
) -> SaveCountryEntrySaveReport {
    let cleanup = if country_snapshot.is_some() {
        SaveCountryEntryCleanup::RemoveNodeThenDestroySnapshot
    } else {
        SaveCountryEntryCleanup::RemoveNode
    };
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let save_returned = country_database
        .save(country_snapshot, Some(&mut *connection))
        .await;
    let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
        .await
        .err();
    SaveCountryEntrySaveReport {
        begin_error,
        save_returned,
        commit_error,
        cleanup,
    }
}

/// Выполняет Save Charactor LoadDetails Data в исходном unsigned map-order.
///
/// Null values только создают исходный log-результат. Old-way вызывает
/// self-opening owner без transaction; new-way всегда вызывает begin и commit
/// вокруг caller-connection owner, игнорируя их ошибки и bool owner-а. После
/// полного обхода caller обязан сохранить `mDBPlayer` неизменной для следующей
/// Save Charactor Data phase.
pub(crate) async fn save_load_details<L: LargessOwner>(
    snapshot: LoadDetailsPlayerMapSnapshot<'_, '_>,
    largess: &mut L,
    connection: &mut WorldTdsClient,
) -> LoadDetailsSaveReport {
    let mut entries = Vec::with_capacity(snapshot.players.len());
    let way = if snapshot.use_old_save_largess_way {
        LoadDetailsSaveWay::Old
    } else {
        LoadDetailsSaveWay::New
    };
    let mut log_events = vec![
        add_log_event(b"Save Charactor LoadDetails Data...".to_vec()),
        add_log_event(
            match way {
                LoadDetailsSaveWay::Old => b"** use old save way".as_slice(),
                LoadDetailsSaveWay::New => b"** use new save way".as_slice(),
            }
            .to_vec(),
        ),
    ];

    for (&map_key, player) in snapshot.players {
        let Some(player) = player else {
            log_events.push(add_log_event(
                b"** Save LoadDetails NULL Pointer!!!!!!".to_vec(),
            ));
            entries.push(LoadDetailsEntrySaveReport::NullPlayer { map_key });
            continue;
        };

        if snapshot.use_old_save_largess_way {
            match largess
                .save_load_details(player.cd_key, player.player_id)
                .await
            {
                SaveLoadDetailsOutcome::ReturnedTrue => {
                    entries.push(LoadDetailsEntrySaveReport::OldWay {
                        map_key,
                        save_returned: true,
                    });
                }
                SaveLoadDetailsOutcome::ReturnedFalse => {
                    entries.push(LoadDetailsEntrySaveReport::OldWay {
                        map_key,
                        save_returned: false,
                    });
                }
            }
            continue;
        }

        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_returned = match largess
            .save_load_details_with_connection(
                player.cd_key,
                player.player_id,
                Some(&mut *connection),
            )
            .await
        {
            SaveLoadDetailsOutcome::ReturnedTrue => true,
            SaveLoadDetailsOutcome::ReturnedFalse => false,
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(LoadDetailsEntrySaveReport::NewWay {
            map_key,
            begin_error,
            save_returned,
            commit_error,
        });
    }

    LoadDetailsSaveReport {
        way,
        entries,
        disposition: LoadDetailsSaveDisposition::CompleteRetainPlayerMap,
        log_events,
    }
}

/// Связывает LoadDetails с реальным frozen player-map без его изменения.
pub(crate) async fn save_load_details_from_world_snapshot<L: LargessOwner>(
    world: &WorldDbDataSaveSession<'_>,
    logged_count: u32,
    use_old_save_largess_way: bool,
    largess: &mut L,
    connection: &mut WorldTdsClient,
) -> LoadDetailsWorldSaveReport {
    let players = world
        .players()
        .iter()
        .map(|(&map_key, player)| {
            (
                map_key,
                Some(LoadDetailsPlayerSnapshot {
                    player_id: player.get_id(),
                    cd_key: player.get_account(),
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let phase = save_load_details(
        LoadDetailsPlayerMapSnapshot {
            players: &players,
            use_old_save_largess_way,
        },
        largess,
        connection,
    )
    .await;
    LoadDetailsWorldSaveReport {
        logged_count,
        phase,
    }
}

enum SaveCharacterEntryOutcome {
    Finished(SaveCharacterEntrySaveReport),
    Blocked {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        block: PlayerSaveBlock,
    },
}

async fn save_character_entry<P, J, G>(
    map_key: u32,
    player: &CPlayer,
    save: &PlayerSaveSnapshot<'_, '_, '_>,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
) -> SaveCharacterEntryOutcome
where
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
{
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    match player
        .save_data(
            save,
            Some(&mut *connection),
            player_database,
            jjc_database,
            goods_database,
        )
        .await
    {
        PlayerSaveOutcome::ReturnedTrue => {
            let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err();
            SaveCharacterEntryOutcome::Finished(SaveCharacterEntrySaveReport::Saved {
                map_key,
                begin_error,
                commit_error,
                cleanup: SaveCharacterSuccessCleanup::DestroyPlayerThenEraseEntryUnderLock,
            })
        }
        PlayerSaveOutcome::ReturnedFalse => {
            let finish = if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            };
            SaveCharacterEntryOutcome::Finished(SaveCharacterEntrySaveReport::Failed {
                map_key,
                begin_error,
                finish,
            })
        }
        PlayerSaveOutcome::BlockedMissingFact(block) => SaveCharacterEntryOutcome::Blocked {
            map_key,
            begin_error,
            block,
        },
    }
}

/// Выполняет Save Charactor Data в исходном unsigned map-order.
///
/// `Saved` требует сразу после success-log применить указанный cleanup под
/// player-list lock. `Failed` и `NullPlayer` сохраняют entry. После
/// `BlockedMissingFact` caller обязан остановить lifecycle и не использовать
/// connection либо map до разрешения локальной границы.
pub(crate) async fn save_characters<P, J, G>(
    snapshot: SaveCharacterPlayerMapSnapshot<'_, '_, '_>,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
) -> SaveCharacterSaveReport
where
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
{
    let mut entries = Vec::with_capacity(snapshot.players.len());
    let mut log_events = vec![add_log_event(b"Save Charactor Data...".to_vec())];

    for (&map_key, player) in snapshot.players {
        let Some(player) = player else {
            log_events.push(add_log_event(
                b"** Save Charactor NULL Pointer!!!!!!".to_vec(),
            ));
            entries.push(SaveCharacterEntrySaveReport::NullPlayer { map_key });
            continue;
        };

        match save_character_entry(
            map_key,
            player.player,
            &player.save,
            player_database,
            jjc_database,
            goods_database,
            connection,
        )
        .await
        {
            SaveCharacterEntryOutcome::Finished(report) => {
                match &report {
                    SaveCharacterEntrySaveReport::Saved { .. } => log_events.push(
                        player_success_log(b"++ Save Charactor SUCCESS : ", player.player),
                    ),
                    SaveCharacterEntrySaveReport::Failed { .. } => {
                        log_events.push(add_log_event(b"-- Save Charactor FAILED!".to_vec()))
                    }
                    SaveCharacterEntrySaveReport::NullPlayer { .. } => {
                        unreachable!("non-null snapshot не создаёт null-result")
                    }
                }
                entries.push(report);
            }
            SaveCharacterEntryOutcome::Blocked {
                map_key,
                begin_error,
                block,
            } => {
                return SaveCharacterSaveReport {
                    entries,
                    log_events,
                    disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                        map_key,
                        begin_error,
                        block: SaveCharacterPhaseBlock::Save(block),
                    },
                };
            }
        }
    }

    SaveCharacterSaveReport {
        entries,
        disposition: SaveCharacterSaveDisposition::Complete {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

/// Повторно обходит frozen map; entry-log завершается до success erase/next.
#[allow(
    clippy::too_many_arguments,
    reason = "Save Character сохраняет раздельные DB-owner-ы и log-owner"
)]
pub(crate) async fn save_characters_from_world_snapshot<P, J, G>(
    world: &mut WorldDbDataSaveSession<'_>,
    logged_count: u32,
    registry: &GoodsBasePropertiesRegistry,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> SaveCharacterSaveReport
where
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
{
    let keys = world.players().keys().copied().collect::<Vec<_>>();
    let mut entries = Vec::with_capacity(keys.len());
    let mut log_events = vec![add_log_event(b"Save Charactor Data...".to_vec())];

    for map_key in keys {
        let outcome = {
            let player = world
                .players()
                .get(&map_key)
                .expect("frozen player-map не меняется вне success cleanup");
            let projection = match player.db_projection(registry) {
                Ok(projection) => projection,
                Err(block) => {
                    return SaveCharacterSaveReport {
                        entries,
                        log_events,
                        disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                            map_key,
                            begin_error: None,
                            block: SaveCharacterPhaseBlock::Projection(block),
                        },
                    };
                }
            };
            let snapshot = match projection.save_snapshot() {
                Ok(snapshot) => snapshot,
                Err(block) => {
                    return SaveCharacterSaveReport {
                        entries,
                        log_events,
                        disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                            map_key,
                            begin_error: None,
                            block: SaveCharacterPhaseBlock::Projection(block),
                        },
                    };
                }
            };
            save_character_entry(
                map_key,
                player,
                &snapshot,
                player_database,
                jjc_database,
                goods_database,
                connection,
            )
            .await
        };

        match outcome {
            SaveCharacterEntryOutcome::Finished(report) => {
                let succeeded = matches!(&report, SaveCharacterEntrySaveReport::Saved { .. });
                if succeeded {
                    let player = world
                        .players()
                        .get(&map_key)
                        .expect("success cleanup ещё не удалил player");
                    log_events.push(player_success_log(b"++ Save Charactor SUCCESS : ", player));
                } else {
                    log_events.push(add_log_event(b"-- Save Charactor FAILED!".to_vec()));
                }
                if let Err(block) = publish_save_data_log(
                    log_sink,
                    log_events.last().expect("entry event только что добавлен"),
                ) {
                    return SaveCharacterSaveReport {
                        entries,
                        log_events,
                        disposition: SaveCharacterSaveDisposition::BlockedLog {
                            map_key,
                            entry: Box::new(report),
                            block,
                        },
                    };
                }
                entries.push(report);
                if succeeded {
                    world.remove_player(map_key);
                }
            }
            SaveCharacterEntryOutcome::Blocked {
                map_key,
                begin_error,
                block,
            } => {
                return SaveCharacterSaveReport {
                    entries,
                    log_events,
                    disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                        map_key,
                        begin_error,
                        block: SaveCharacterPhaseBlock::Save(block),
                    },
                };
            }
        }
    }

    SaveCharacterSaveReport {
        entries,
        disposition: SaveCharacterSaveDisposition::Complete { logged_count },
        log_events,
    }
}

/// Выполняет доказанный suffix `DoSaveData` после Save Union Data.
///
/// Region, обе GodsBattle-фазы и Country продолжаются после обычного `false`
/// ровно как исходник. Только typed snapshot/owner block прекращает передачу
/// connection следующему владельцу. Ранний player-map count из LoadDetails
/// становится четвёртым итоговым `SAVED` и повторно используется Save Character.
#[allow(
    clippy::too_many_arguments,
    reason = "точный suffix DoSaveData сохраняет независимые component owner-ы"
)]
pub(crate) async fn do_save_data_after_unions<P, J, G, R, B, E, C, L>(
    world: &mut WorldDbDataSaveSession<'_>,
    honor_ranks: &mut CHonorRanks,
    early_counters: SaveDataEarlyCounters,
    gods_battle_faction_xyd: GodsBattleFactionXydSnapshot,
    gods_battle_npc_factions: &[GodsBattleNpcFactionSnapshot],
    use_old_save_largess_way: bool,
    registry: &GoodsBasePropertiesRegistry,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    region_database: &mut R,
    gods_battle_database: &mut B,
    enemy_factions_database: &mut E,
    country_database: &mut C,
    largess: &mut L,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> DoSaveDataAfterUnionsReport
where
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
    R: RsRegionOwner,
    B: RsGodsBattleOwner,
    E: RsEnemyFactionsOwner,
    C: DbCountryOwner,
    L: LargessOwner,
{
    let mut phases = DoSaveDataAfterUnionsPhases::default();

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save Region Data...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveRegions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let regions = match save_regions_from_world_snapshot(world, region_database, connection).await {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataAfterUnionsReport {
                phases,
                disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::SaveRegions,
                    block,
                },
            };
        }
    };
    let regions_final_index = regions.log_events.len() - 1;
    let regions_log_block =
        publish_save_data_log(log_sink, &regions.log_events[regions_final_index]).err();
    phases.regions = Some(regions);
    if let Some(block) = regions_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveRegions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(regions_final_index),
                block,
            },
        };
    }
    world.clear_saved_region_nodes();

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save HonorRanks start...".to_vec()),
    ) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::HonorRanks,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let honor_report = match save_honor_ranks_from_world_snapshot(
        honor_ranks,
        player_database,
        connection,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataAfterUnionsReport {
                phases,
                disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::HonorRanks,
                    block,
                },
            };
        }
    };
    let honor_blocked = matches!(
        &honor_report.disposition,
        HonorRanksSaveDisposition::BlockedMissingFact(_)
    );
    let honor_log_block = honor_report
        .log_events
        .get(1)
        .and_then(|event| publish_save_data_log(log_sink, event).err());
    phases.honor_ranks = Some(honor_report);
    if let Some(block) = honor_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::HonorRanks,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }
    if honor_blocked {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::HonorRanks,
            ),
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save GodsBattle...start...".to_vec()),
    ) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GodsBattleFactionXyd,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let gods_battle_faction_xyd_report =
        save_gods_battle_faction_xyd(gods_battle_faction_xyd, gods_battle_database, connection)
            .await;
    let gods_battle_faction_xyd_log_block =
        publish_save_data_log(log_sink, &gods_battle_faction_xyd_report.log_events[1]).err();
    phases.gods_battle_faction_xyd = Some(gods_battle_faction_xyd_report);
    if let Some(block) = gods_battle_faction_xyd_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GodsBattleFactionXyd,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }

    let gods_battle_npc_report =
        save_gods_battle_npc_factions(gods_battle_npc_factions, gods_battle_database, connection)
            .await;
    let gods_battle_npc_log_block =
        publish_save_data_log(log_sink, &gods_battle_npc_report.log_events[0]).err();
    phases.gods_battle_npc_factions = Some(gods_battle_npc_report);
    if let Some(block) = gods_battle_npc_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GodsBattleNpcFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(0),
                block,
            },
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save EnemyFactions...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::EnemyFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let enemy_report =
        save_enemy_factions_from_world_snapshot(world, enemy_factions_database, connection).await;
    let enemy_blocked = matches!(
        &enemy_report.disposition,
        EnemyFactionsTransactionSaveDisposition::BlockedMissingFact(_)
    );
    let enemy_log_block = enemy_report
        .log_events
        .get(1)
        .and_then(|event| publish_save_data_log(log_sink, event).err());
    phases.enemy_factions = Some(enemy_report);
    if let Some(block) = enemy_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::EnemyFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }
    if enemy_blocked {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::EnemyFactions,
            ),
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Update Country Data...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::Countries,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let countries =
        match save_countries_from_world_snapshot(world, country_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataAfterUnionsReport {
                    phases,
                    disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::Countries,
                        block,
                    },
                };
            }
        };
    let countries_final_index = countries.log_events.len() - 1;
    let countries_log_block =
        publish_save_data_log(log_sink, &countries.log_events[countries_final_index]).err();
    phases.countries = Some(countries);
    if let Some(block) = countries_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::Countries,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(countries_final_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Charactor LoadDetails Data...".to_vec()),
    ) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::LoadDetails,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let saved = match legacy_snapshot_count("Save Character", world.players().len()) {
        Ok(count) => count,
        Err(block) => {
            return DoSaveDataAfterUnionsReport {
                phases,
                disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::LoadDetails,
                    block,
                },
            };
        }
    };
    let load_details_way_event = add_log_event(
        if use_old_save_largess_way {
            b"** use old save way".as_slice()
        } else {
            b"** use new save way".as_slice()
        }
        .to_vec(),
    );
    if let Err(block) = publish_save_data_log(log_sink, &load_details_way_event) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::LoadDetails,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }

    let load_details = save_load_details_from_world_snapshot(
        world,
        saved,
        use_old_save_largess_way,
        largess,
        connection,
    )
    .await;
    phases.load_details = Some(load_details);

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save Charactor Data...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let save_characters = save_characters_from_world_snapshot(
        world,
        saved,
        registry,
        player_database,
        jjc_database,
        goods_database,
        connection,
        log_sink,
    )
    .await;
    let save_characters_log_block = match &save_characters.disposition {
        SaveCharacterSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let save_characters_log_index = save_characters.log_events.len().saturating_sub(1);
    let save_characters_blocked = matches!(
        &save_characters.disposition,
        SaveCharacterSaveDisposition::BlockedMissingFact { .. }
    );
    phases.save_characters = Some(save_characters);
    if let Some(block) = save_characters_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(save_characters_log_index),
                block,
            },
        };
    }
    if save_characters_blocked {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::SaveCharacters,
            ),
        };
    }

    DoSaveDataAfterUnionsReport {
        phases,
        disposition: DoSaveDataAfterUnionsDisposition::Complete(SaveDataCounters {
            created: early_counters.created,
            cancel_deleted: early_counters.cancel_deleted,
            sign_deleted: early_counters.sign_deleted,
            saved,
        }),
    }
}

/// Выполняет все доказанные DB/container фазы на уже открытом World connection.
///
/// Функция не создаёт и не закрывает connection и не запускает save-thread.
/// Вся phase-цепь публикует logs через переданный owner в exact порядке. Первый
/// block либо четыре counters передаются существующему finalizer-у.
#[allow(
    clippy::too_many_arguments,
    reason = "полный DoSaveData сохраняет все независимые component owner-ы"
)]
pub(crate) async fn do_save_data_phases<S, O, V, P, J, G, U, F, R, B, E, C, L, Log>(
    world: &mut WorldDbDataSaveSession<'_>,
    variables: &S,
    registry: &GoodsBasePropertiesRegistry,
    organizing: &COrganizingCtrl,
    honor_ranks: &mut CHonorRanks,
    gods_battle_faction_xyd: GodsBattleFactionXydSnapshot,
    gods_battle_npc_factions: &[GodsBattleNpcFactionSnapshot],
    use_old_save_largess_way: bool,
    setup_database: &mut O,
    variable_database: &mut V,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    union_database: &mut U,
    faction_database: &mut F,
    region_database: &mut R,
    gods_battle_database: &mut B,
    enemy_factions_database: &mut E,
    country_database: &mut C,
    largess: &mut L,
    connection: &mut WorldTdsClient,
    log_sink: &mut Log,
) -> DoSaveDataPhasesReport
where
    S: VariableListSaveSource,
    O: RsSetupOwner,
    V: RsGenVarOwner,
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
    U: RsUnionOwner,
    F: RsFactionOwner,
    R: RsRegionOwner,
    B: RsGodsBattleOwner,
    E: RsEnemyFactionsOwner,
    C: DbCountryOwner,
    L: LargessOwner,
    Log: SaveDataLogSink,
{
    let through_unions = do_save_data_through_unions(
        world,
        variables,
        registry,
        organizing,
        setup_database,
        variable_database,
        player_database,
        jjc_database,
        goods_database,
        union_database,
        faction_database,
        connection,
        log_sink,
    )
    .await;
    let early_counters = match through_unions.disposition {
        DoSaveDataThroughUnionsDisposition::ContinueWithRegion(counters) => counters,
        DoSaveDataThroughUnionsDisposition::BlockedSnapshot { .. }
        | DoSaveDataThroughUnionsDisposition::BlockedPhase(_)
        | DoSaveDataThroughUnionsDisposition::BlockedLog { .. } => {
            return DoSaveDataPhasesReport {
                through_unions,
                after_unions: None,
                disposition: DoSaveDataPhasesDisposition::BlockedThroughUnions,
            };
        }
    };

    let after_unions = do_save_data_after_unions(
        world,
        honor_ranks,
        early_counters,
        gods_battle_faction_xyd,
        gods_battle_npc_factions,
        use_old_save_largess_way,
        registry,
        player_database,
        jjc_database,
        goods_database,
        region_database,
        gods_battle_database,
        enemy_factions_database,
        country_database,
        largess,
        connection,
        log_sink,
    )
    .await;
    let disposition = match after_unions.disposition {
        DoSaveDataAfterUnionsDisposition::Complete(counters) => {
            DoSaveDataPhasesDisposition::Complete(counters)
        }
        DoSaveDataAfterUnionsDisposition::BlockedSnapshot { .. }
        | DoSaveDataAfterUnionsDisposition::BlockedPhase(_)
        | DoSaveDataAfterUnionsDisposition::BlockedLog { .. } => {
            DoSaveDataPhasesDisposition::BlockedAfterUnions
        }
    };

    DoSaveDataPhasesReport {
        through_unions,
        after_unions: Some(after_unions),
        disposition,
    }
}

/// Начинает общий отчёт после уже выполненного component-specific cleanup.
///
/// Caller заранее получает cleanup через `SaveDataFinalSnapshot::connection_finish`,
/// выполняет его и только затем снимает `end_tick_ms`. Возвращённый summary-log
/// должен быть опубликован до `capture_save_data_local_time`.
pub(crate) fn begin_finish_save_data(
    snapshot: SaveDataFinalSnapshot,
    end_tick_ms: u32,
) -> SaveDataFinalStart {
    let connection_finish = snapshot.connection_finish();
    let counters = match snapshot.path {
        SaveDataFinalPath::Completed(counters) => counters,
        SaveDataFinalPath::ConnectionOpenFailed => SaveDataCounters {
            created: 0,
            cancel_deleted: 0,
            sign_deleted: 0,
            saved: 0,
        },
    };
    let elapsed_ms = end_tick_ms.wrapping_sub(snapshot.started_at_tick_ms);
    let summary_log = legacy_save_summary(counters, elapsed_ms);

    SaveDataFinalStart {
        connection_finish,
        summary_log,
        elapsed_ms,
        end_tick_ms,
    }
}

/// Завершает хвост после summary-log и следующего `GetLocalTime`.
///
/// `get_monitoring` читает queue-size, server name/ID и `dwNumber` после трёх
/// time-global мутаций, а `send_monitoring` вызывается после успешного
/// `_sprintf`. Его результат исходно отсутствовал, поэтому сразу после
/// возврата closure save-флаг сбрасывается.
pub(crate) fn finish_save_data<GetMonitoring, SendMonitoring>(
    start: SaveDataFinalStart,
    state: &mut SaveDataLifecycleState,
    local_time: SaveDataLocalTime,
    get_monitoring: GetMonitoring,
    send_monitoring: SendMonitoring,
) -> SaveDataFinalReport
where
    GetMonitoring: FnOnce() -> SaveDataMonitoringSnapshot,
    SendMonitoring: FnOnce(&SaveDataMonitoringReport),
{
    state.last_save_time = local_time;
    state.last_save_tick_ms = start.end_tick_ms;
    state.this_save_start_tick_ms = 0;

    let monitoring_snapshot = get_monitoring();

    let text = legacy_save_monitoring_text(
        &monitoring_snapshot.server_name,
        local_time,
        start.elapsed_ms,
        monitoring_snapshot.write_log_count,
    );
    let monitoring = SaveDataMonitoringReport {
        message_type: -2,
        server_id: monitoring_snapshot.server_id,
        world_number_bits: monitoring_snapshot.world_number_bits,
        text,
    };
    // Исходный SendErrLog возвращал void: попытка send всегда ведёт к
    // сбросу флага, независимо от результата внутреннего CMessage::Send.
    send_monitoring(&monitoring);
    state.is_saving_data = false;
    let disposition = SaveDataFinalDisposition::Complete(monitoring);

    SaveDataFinalReport {
        connection_finish: start.connection_finish,
        summary_log: start.summary_log,
        elapsed_ms: start.elapsed_ms,
        disposition,
    }
}

/// Выполняет один полный typed lifecycle `DoSaveData` без runtime thread/entry.
///
/// Phase block возвращает ещё открытый connection и не выбирает cleanup. На
/// normal/open-failure путях функция сама сохраняет доказанный порядок
/// connection-log, конечного tick, summary-log, local time и monitoring send.
#[allow(
    clippy::too_many_arguments,
    reason = "DoSaveData сохраняет независимые snapshot-, DB-, log- и monitoring-owner-ы"
)]
pub(crate) async fn do_save_data_lifecycle<
    S,
    O,
    V,
    P,
    J,
    G,
    U,
    F,
    R,
    B,
    E,
    C,
    L,
    Log,
    GetMonitoring,
    SendMonitoring,
>(
    settings: &WorldDatabaseSettings,
    state: &mut SaveDataLifecycleState,
    world: &mut WorldDbDataSaveSession<'_>,
    variables: &S,
    registry: &GoodsBasePropertiesRegistry,
    organizing: &COrganizingCtrl,
    honor_ranks: &mut CHonorRanks,
    gods_battle_faction_xyd: GodsBattleFactionXydSnapshot,
    gods_battle_npc_factions: &[GodsBattleNpcFactionSnapshot],
    use_old_save_largess_way: bool,
    setup_database: &mut O,
    variable_database: &mut V,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    union_database: &mut U,
    faction_database: &mut F,
    region_database: &mut R,
    gods_battle_database: &mut B,
    enemy_factions_database: &mut E,
    country_database: &mut C,
    largess: &mut L,
    log_sink: &mut Log,
    get_monitoring: GetMonitoring,
    send_monitoring: SendMonitoring,
) -> DoSaveDataLifecycleReport
where
    S: VariableListSaveSource,
    O: RsSetupOwner,
    V: RsGenVarOwner,
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
    U: RsUnionOwner,
    F: RsFactionOwner,
    R: RsRegionOwner,
    B: RsGodsBattleOwner,
    E: RsEnemyFactionsOwner,
    C: DbCountryOwner,
    L: LargessOwner,
    Log: SaveDataLogSink,
    GetMonitoring: FnOnce() -> SaveDataMonitoringSnapshot,
    SendMonitoring: FnOnce(&SaveDataMonitoringReport),
{
    let (evidence, final_snapshot) = match begin_do_save_data(settings, state).await {
        DoSaveDataStart::Opened {
            mut connection,
            started_at_tick_ms,
        } => {
            let phases = do_save_data_phases(
                world,
                variables,
                registry,
                organizing,
                honor_ranks,
                gods_battle_faction_xyd,
                gods_battle_npc_factions,
                use_old_save_largess_way,
                setup_database,
                variable_database,
                player_database,
                jjc_database,
                goods_database,
                union_database,
                faction_database,
                region_database,
                gods_battle_database,
                enemy_factions_database,
                country_database,
                largess,
                &mut connection,
                log_sink,
            )
            .await;
            let counters = match phases.disposition {
                DoSaveDataPhasesDisposition::Complete(counters) => counters,
                DoSaveDataPhasesDisposition::BlockedThroughUnions
                | DoSaveDataPhasesDisposition::BlockedAfterUnions => {
                    return DoSaveDataLifecycleReport::BlockedPhases {
                        phases,
                        connection,
                        started_at_tick_ms,
                    };
                }
            };
            let final_snapshot = SaveDataFinalSnapshot {
                path: SaveDataFinalPath::Completed(counters),
                started_at_tick_ms,
            };

            // `Client::close(self)` одновременно завершает TDS transport и
            // потребляет Rust-owner. Это совместимая замена первого CloseCn;
            // исходный повторный CloseCn внутри ReleaseCn был idempotent.
            let close_error = connection.close().await.err();
            let evidence = SaveDataLifecycleEvidence {
                phases: Some(phases),
                open_error: None,
                close_error,
            };
            if let Err(block) =
                publish_save_data_log(log_sink, &add_log_event(b"Save Data end ...".to_vec()))
            {
                return DoSaveDataLifecycleReport::BlockedConnectionLog {
                    evidence,
                    final_snapshot,
                    checkpoint: SaveDataConnectionLogCheckpoint::SaveDataEnd,
                    block,
                };
            }
            // Здесь находится исходная ReleaseCn-граница; Tiberius owner уже
            // потреблён первым close, поэтому второго observable вызова нет.
            (evidence, final_snapshot)
        }
        DoSaveDataStart::ConnectionOpenFailed {
            error,
            final_snapshot,
        } => {
            let evidence = SaveDataLifecycleEvidence {
                phases: None,
                open_error: Some(error),
                close_error: None,
            };
            if let Err(block) =
                publish_save_data_log(log_sink, &add_log_event(b"Connect To DB FAILED!".to_vec()))
            {
                return DoSaveDataLifecycleReport::BlockedConnectionLog {
                    evidence,
                    final_snapshot,
                    checkpoint: SaveDataConnectionLogCheckpoint::ConnectToDatabaseFailed,
                    block,
                };
            }
            // Неуспешный Tiberius connect уже освободил transport-owner; эта
            // точка сохраняет исходную логическую ReleaseCn-границу после log.
            (evidence, final_snapshot)
        }
    };

    let end_tick_ms = capture_save_data_tick_ms();
    let final_start = begin_finish_save_data(final_snapshot, end_tick_ms);
    let summary_event = add_log_event(final_start.summary_log.clone());
    if let Err(block) = publish_save_data_log(log_sink, &summary_event) {
        return DoSaveDataLifecycleReport::BlockedSummaryLog {
            evidence,
            final_start,
            block,
        };
    }

    let local_time = capture_save_data_local_time();
    let report = finish_save_data(
        final_start,
        state,
        local_time,
        get_monitoring,
        send_monitoring,
    );
    DoSaveDataLifecycleReport::Final { evidence, report }
}

fn legacy_save_summary(counters: SaveDataCounters, elapsed_ms: u32) -> Vec<u8> {
    format!(
        "\r\n{} Charactor CREATED,\r\n{} Charactor CANCEL DELETE,\r\n{} Charactor SIGN DELETE,\r\n{} Charactor SAVED\r\nUSED TIME:{}ms",
        counters.created as i32,
        counters.cancel_deleted as i32,
        counters.sign_deleted as i32,
        counters.saved as i32,
        elapsed_ms as i32,
    )
    .into_bytes()
}

fn legacy_save_monitoring_text(
    server_name: &[u8],
    local_time: SaveDataLocalTime,
    elapsed_ms: u32,
    write_log_count: u32,
) -> Vec<u8> {
    let server_name = server_name
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    let suffix = format!(
        "|SAVE TIME~{:4}-{:2}-{:2} {:2}:{:2}:{:2}|USED TIME~{}|LOG NUM~{}",
        local_time.year,
        local_time.month,
        local_time.day,
        local_time.hour,
        local_time.minute,
        local_time.second,
        elapsed_ms as i32,
        write_log_count as i32,
    );
    let mut text = Vec::with_capacity(4 + server_name.len() + suffix.len());
    text.extend_from_slice(b"|ws~");
    text.extend_from_slice(server_name);
    text.extend_from_slice(suffix.as_bytes());

    text
}

async fn run_transaction_command(
    connection: &mut WorldTdsClient,
    sql: &'static str,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}
