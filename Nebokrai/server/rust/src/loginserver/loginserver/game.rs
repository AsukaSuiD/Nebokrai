//! Фактический `CGame` LoginServer из `loginserver/game.cpp` и `.h`.
//!
//! Статус владельца: `IMPLEMENTED` для AuthServer lifecycle-функций
//! `LoadASList`, `SendLSInfoToAS`, `IsConnectAS`, `DisconnectAS`,
//! `ReconnectAS`, `ReassignAS`, `InitAuthClient`, обработки типизированного
//! reconnect-события, управляемых `StartReconnectThread/_ReconnectThread`,
//! `GetWorldIDByName`, `IsExitWorld`, `GetLoginWorldPlayerNumByWorldName`,
//! `GetWorldNameByID`, `AddWorld`, `DelWorld`,
//! `ClearCDKeyByWorldServerID`, `UpdateWorldInfoToAllClient`,
//! `AddCdkey`, `FindCdkey`, `ClearCDKey`, `ClearLoginCdkey`,
//! `UpdateOnlineUser2DB`, `GetCdkeyCount`, накопления
//! `m_vectorPingWorldServerInfo`, `AppendServerInfoLog`, `ServerInfoLog`,
//! `GetLoginCdkeyWorldServer`, `SendMsg2World`, `SendToClient`, `KickOut`, baseline-
//! ветвей `PrepareEnter/EnterGame`, `AddWorldInfoToMsg`,
//! `L2W_PlayerBase_Send`, `L2W_QuestDetail_Send`, `L2W_CreateRole_Send`,
//! `L2W_DeleteRole_Send`, `L2W_RestoreRole_Send`,
//! `SetLoginCdkeyWorldServer`, typed-границы
//! `AccountEnterLog`, `RoleEnterLog`, `LeaveLog`, `AccountLeaveLog`,
//! password-error части
//! `AuthHandler`, `tagSetup`,
//! позиционного `LoadSetup`, точного `tagSetupEx`, `LoadSetupEx`,
//! `ReLoadSetup`, `ReLoadSetupEx`, `load_listen_port`, `LoadWorldSetup`,
//! `ReLoadWorldSetup`, `LoadNoQueueCDKeyList`, `GetLoginWorldCdkeyNumbers`,
//! `SetListWorldInfoBySetup`, `WorldServerIsOpenState`,
//! `InitNetServer_Client`, `InitNetServer_World`,
//! `ClearOnlineUserDatabase`, полного `CGame::Init/Release` и запуска его
//! `CGasThread`/`AccLogThread`, `ExecuteProce`, owned `GameThreadFunc` и
//! управляемого World/Client/Auth network cadence,
//! `ProcessMessage`, достигнутого turn `MainLoop`, `ChangeAllWorldSate`,
//! достигнутых границ
//! `CRsCDKey`, создания его Linux/TDS-owner из точных DB setup-полей,
//! `CLoginQueue::OnInitial` и readiness-driven Linux read/send шага.
//! После закрытия последних серверных границ заменённый псевдокод,
//! constructor/destructor, STL/CRT и compiler cleanup удалены; оставшиеся
//! неизвестности локализованы у конкретных безопасных границ.
//!
//! Точная пара: `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`;
//! SHA-256 EXE
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`,
//! SHA-256 PDB
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`.
//! Исходные пути PDB:
//! `d:\complite_version\fengyun_russia\trunk\server\loginserver\loginserver\game.cpp`
//! и `.h`. Существенные RVA: `SendLSInfoToAS` `0x00003490`, `IsConnectAS`
//! `0x00003520`, `DisconnectAS` `0x00004320`, `ReconnectAS` `0x00006240`,
//! `ReassignAS` `0x00009BF0`, `InitAuthClient` `0x0000C9D0`, `LoadASList`
//! `0x000100F0`, `SendMsg2World` `0x00003260`, `GetWorldIDByName`
//! `0x00005DB0`, `AddWorldInfoToMsg` `0x000074E0`, `L2W_PlayerBase_Send`
//! `0x00007770`, `L2W_DeleteRole_Send` `0x00005E60`,
//! `L2W_RestoreRole_Send` `0x00005F30`, `L2W_CreateRole_Send` `0x000091D0`,
//! `EnterGame` `0x00007970`, `SetLoginCdkeyWorldServer`
//! `0x00011890`, `PrepareEnter` `0x00011940`, `KickOut` `0x0000C910`,
//! `IsExitWorld` `0x00005E40`, `L2W_QuestDetail_Send` `0x00005FD0`,
//! `GetLoginWorldPlayerNumByWorldName` `0x00007590`,
//! `GetLoginCdkeyWorldServer` `0x000099B0`,
//! `tagSetup` `0x0000C6E0`, `LoadSetup` `0x0000E690`, `LoadSetupEx`
//! `0x0000D9E0`, `ReLoadSetupEx` `0x0000DC60`, `ReLoadSetup` `0x0000F4E0`,
//! `load_listen_port` `0x0000DCB0`, `LoadWorldSetup` `0x00010F30`,
//! `ReLoadWorldSetup` `0x00011FF0`, `LoadNoQueueCDKeyList` `0x00002D90`,
//! `GetLoginWorldCdkeyNumbers` `0x00007740`,
//! `SetListWorldInfoBySetup` `0x00011170`, `WorldServerIsOpenState`
//! `0x000074A0`,
//! `InitNetServer_Client` `0x00002DA0`, `InitNetServer_World` `0x00002F90`,
//! `ProcessMessage` `0x00003180`, `ChangeAllWorldSate` `0x000075E0`,
//! `MainLoop` `0x00013EC0`, `GetCdkeyCount` `0x00007930`,
//! `AppendServerInfoLog` `0x00013A20`, `ServerInfoLog` `0x00013050`,
//! `GetWorldNameByID` `0x00007460`, `UpdateWorldInfoToAllClient` `0x00007860`,
//! `ClearCDKeyByWorldServerID` `0x0000DE70`, `AddWorld` `0x00012910`,
//! `DelWorld` `0x00012A60`,
//! `UpdateOnlineUser2DB` `0x000060A0`, `FindCdkey` `0x00007660`,
//! `AddCdkey` `0x0000F600`, `ClearLoginCdkey` `0x00010B00`,
//! `ClearCDKey` `0x00011720`,
//! `AccountEnterLog` `0x00003280`, `RoleEnterLog` `0x00003320`,
//! `LeaveLog` `0x000033B0`, `AccountLeaveLog` `0x00003420`,
//! `ClearOnlineUserDatabase` `0x00005220`, `Release` `0x00005C60`, полный
//! `Init` `0x000111F0`, `ExecuteProce` `0x00007AB0`, `_ReconnectThread`
//! `0x000063B0`, `StartReconnectThread` `0x00007A50` и `GameThreadFunc`
//! `0x00014680`.
//!
//! `LoadASList` сначала очищал список, затем читал whitespace-пары
//! `string + unsigned short` и возвращал успех даже для пустого либо частично
//! разобранного файла. Rust сохраняет этот partial-read контракт и bytes имени
//! без требования UTF-8. Числовой endpoint разбирается в формах `inet_addr`,
//! включая сокращённые, octal и hex; `INADDR_NONE` запускает исходный DNS
//! fallback с выбором первого IPv4.
//!
//! `InitAuthClient` всегда возвращал `true`, даже если список пуст или ни один
//! AuthServer не подключился. После попыток он безусловно включал
//! `m_bControlSend` и ставил `0xCF503 + area_id` в send-очередь. Эта странность
//! сохранена в `AuthInitializationReport::legacy_result`; ошибки отдельных
//! endpoint не превращаются в новый фатальный init.
//!
//! `ReconnectAS` создавал отдельный client, проходил список по порядку и при
//! успехе публиковал `0xCF302 + CMyNetClientAuth*` в FIFO старого клиента.
//! Rust-представление этого type-erased 32-bit pointer описано у
//! `nets/netlogin/mynetclient_auth.rs`: владеющий typed event сохраняет ту же
//! очередь и момент `ReassignAS` без `unsafe`. `ReassignAS` сначала уничтожает
//! старый client, затем включает send у нового, копирует resolved endpoint и
//! только после этого ставит LoginServer-info в очередь.
//! Управляемая reconnect-задача отменяет и дожидается прежней задачи, сразу
//! выполняет первый `ReconnectAS`-проход, затем повторяет его через пять секунд.
//! `JoinHandle::abort` заменяет служебное Win32-сообщение `0x464`; publisher
//! имеет доступ только к FIFO конкретного прежнего Auth-клиента.
//!
//! `m_listWorldInfo` представлен `BTreeMap`: как исходный `std::map`, он ищет
//! в порядке numeric world ID. Имя сравнивается byte-exact, а найденная запись
//! с нулевым state возвращает тот же sentinel `-1`. `ServerCommandHandle`
//! заменяет фактический `s_pNetServer_World`, не перенося состояние сети в
//! message-handler.
//! `AddWorld`/`DelWorld` сохраняют setup-запись и меняют только runtime state,
//! рассылают `0xAF509`, записывают неизменный размер карты в `m_nWordNum` и
//! затем заменяют/удаляют World-список CD-key. Неоднозначный decompiler iterator
//! `AddWorld` имеет статус `VERIFIED_DISASSEMBLY`: в точном EXE инструкции
//! `0x0041294B..0x00412958` передают в `std::map::find` адрес первого stack-
//! аргумента (`world_id`), а слот второго аргумента используют как iterator-
//! output после того, как byte-name уже сохранён в `EDI`. Возврат карты читается
//! из `this+0x60` в `0x004129EA`, как и в `DelWorld` `0x00412A95`.
//! Старый `UpdateDisplayWorldInfo` только очищал/заполнял MFC listbox строками
//! `Disconnected/Started/Closed`; Linux runtime не получает Windows GUI/FFI,
//! а фактическое состояние остаётся доступно у typed `CGame`.
//! `AddCdkey` принимает только уже созданный `s_listCdkey[world_id]`, отвергает
//! byte-exact duplicate и после добавления вычисляет state `1/2/3` по первым
//! двум рабочим порогам `m_StateLvl`. `FindCdkey` возвращает первый World ID в
//! numeric порядке, а `ClearLoginCdkey` удаляет C-string account только из
//! `m_LoginCdkeyWorld`. `ClearCDKey` удаляет только первое совпадение в порядке
//! World ID и пересчитывает тот же state; лишь полное отсутствие account во
//! всех World вызывает `ClearLoginCdkey`. Повреждённый
//! хвост имеет статус `VERIFIED_DISASSEMBLY`: `0x00411829` сохраняет ключ
//! найденного World-узла, `0x00411859..0x00411866` ищет его в runtime-карте и
//! `0x00411875` пишет state в `tagWorldInfo+0x2C`.
//! Rust-slice не представляет nullable `char*`; все достигнутые вызовы
//! `ClearLoginCdkey` передают ненулевой адрес буфера, а C-string граница
//! сохраняется усечением по первому NUL.
//!
//! `UpdateOnlineUser2DB` сохраняет отдельное соединение и нетранзакционный
//! `DELETE world -> INSERT account...` в порядке snapshot. Аргументы потерянных
//! `sprintf` имеют статус `VERIFIED_DISASSEMBLY`: `0x0040612D` передаёт
//! `context.world_id` в DELETE, `0x00406175..0x00406191` — byte-account и тот
//! же ID в INSERT. Игнорируемая ошибка каждого `ExecuteCn` не обрывала цикл;
//! Rust собирает её в typed report и продолжает. Detached `_beginthreadex` и
//! COM apartment заменены параллельными owned `std::thread::JoinHandle`, каждый
//! с отдельным current-thread Tokio/Tiberius runtime; `CGame::Drop` дожидается
//! оставшихся workers, не оставляя доступ к уничтоженным credentials/state.
//! `m_vectorPingWorldServerInfo` сохраняет append-only порядок ответов
//! `OnServerMessage(0x1FE04)` до исходной позиции очистки при следующем ping в
//! достигнутом `MainLoop`; `Vec` заменяет `std::vector`, а byte-exact строки не
//! требуют UTF-8.
//! `AppendServerInfoLog` использует неблокирующий `Mutex::try_lock`, глубоко
//! копирует весь текущий telemetry-вектор в существующий staging и при занятом
//! consumer lock отбрасывает только эту попытку копирования. `ServerInfoLog`
//! держит тот же lock на всём TDS-проходе, поэтому несколько detached workers
//! остаются сериализованными; owned `JoinHandle` и `CGame::Drop` заменяют
//! закрываемые Win32 handles без доступа потока к уничтоженному состоянию.
//! Ветка Login читает canonical IP World listener и атомарный снимок суммы
//! `s_listCdkey` в исходной позиции `GetCdkeyCount`; World использует свой port
//! одновременно как `server_num/world_id`, Game — свой port и World port.
//! Потерянные variadic-аргументы имеют статус `VERIFIED_DISASSEMBLY`: существенные
//! call sites `0x00413162..0x004132AB`, `0x0041335B..0x004134C9`,
//! `0x0041356D..0x00413717` и `0x004137F3..0x00413820` подтверждают IP, signed
//! `%d` bit-pattern, parent port и порядок IPv4 octets. ADO/COM заменены зрелым
//! `tiberius`; исходные непараметризованные SQL-строки, lookup-before-write,
//! отсутствие транзакции, pop-before-INSERT и очистка staging также при COM-
//! ошибке сохранены. Provider-строка была механизмом ADO и не переносится в
//! Linux/TDS; остальные пять server-info setup-полей сохраняют свои роли.
//! `IsExitWorld`, `GetLoginWorldPlayerNumByWorldName` и
//! `GetLoginCdkeyWorldServer` читают те же owned карты без копии чужой
//! семантики в LoginQueue. `L2W_QuestDetail_Send` сохраняет nullable World,
//! точный `0x4FB05` payload и signed bit-pattern IPv4. Интервал повторного
//! player-запроса хранит exact signed `long` из `tagSetupEx`; при использовании
//! как boot-tick интервала его bit-pattern переводится в `u32`.
//! Три role-route owner также сохраняют nullable World и byte-exact account.
//! Create-role меняет opcode входного сообщения на `0x4FB04`, дописывает
//! account к уже существующему payload и только затем отправляет тот же объект.
//! Delete/restore строят новые `0x4FB02/0x4FB03`; delete единственный проверяет
//! пустой C-string account и добавляет raw IPv4 после signed player ID, restore
//! добавляет только account и unsigned 32-битный player ID. Результат старого void-send
//! становится `Result<bool, GameRouteError>`: `false` означает исходный no-op
//! до отправки, а transport-ошибка не маскируется.
//!
//! `KickOut` сначала проверяет `m_LoginCdkeyWorld`: найденный account получает
//! client `QUIT` по строковой identity. Только при отсутствии этой записи
//! `FindCdkey` проходит `s_listCdkey` в порядке World ID и отправляет
//! `0x4FB07 + account` найденному WorldServer. Password-error `BTreeMap`
//! сохраняет тот же ordered map и пороговую проверку; сам записывающий вызов
//! остаётся за присоединяемым `CRsCDKey`, а не за `CGame` или `AuthHandler`.
//! Тот же DB-owner выполняет достигнутый `matrix_validate`; его отсутствие не
//! превращается во временный успешный ответ.
//! Embedded C++ `mAuthHandler` не воспроизводится самоссылкой: технический
//! `AuthListener for CGame` синхронно передаёт callbacks stateless-владельцу
//! `authhandler.rs`.
//!
//! `PrepareEnter` сохраняет порядок `KickOut -> 0xAF501/8` либо client
//! identity -> account-enter record -> `m_LoginCdkeyWorld`. Только после этих
//! side effects исходный matrix-путь возвращает управление фактическому
//! `CLoginQueue::matrix_register` и не запускает `EnterGame`. Обычная ветвь
//! `EnterGame` либо отправляет клиенту `0xAF501/2 + account + world list`, либо
//! публикует `0x4FB01 + account` выбранному открытому WorldServer. Закрытый мир
//! остаётся доступен только account из исходного no-queue списка.
//! `AccountEnterLog`, `RoleEnterLog`, `LeaveLog` и `AccountLeaveLog`
//! сохраняются вариантами одной typed-FIFO исходной `_acc_logs`, поэтому их
//! взаимный порядок не теряется. Account-enter хранит raw IPv4, role-enter —
//! byte-exact имя, unsigned level и переданный signed World number. `LeaveLog`
//! сохраняет одну метку для будущего одновременного обновления RoleLeaveTime и
//! AccountLeaveTime; все варианты сохраняют byte-exact account и local time,
//! снятые в исходной позиции. `AccLogQueue` сохраняет общую FIFO и semaphore-
//! семантику, а `AccLogThread` строит и выполняет доказанные legacy SQL в
//! отдельном Tiberius-соединении для каждой записи.
//!
//! `LoadSetup` сначала открывает обычный `setup.ini`, а только при неуспехе —
//! побайтово декодированный `setup.dat`. Обе формы читаются как пары
//! `label + value`; label не проверяется, строки остаются byte-exact, а
//! stream fail-state сохраняет уже выполненные мутации и прежние значения
//! хвоста. Найденный Login `setup.ini` содержит 54 полные пары: следующие
//! `m_lIsInsideUse/m_strVerificationAddr/m_lVerifiSignUpper` читаются уже на
//! EOF, поэтому первые два сохраняют constructor-default, а последний остаётся
//! явной uninitialized-границей. Четыре DB-поля создают единый Tiberius-owner
//! в достигнутой позиции `Init` после ADO-конфигурации и до network owners;
//! секреты не входят ни в отчёт, ни в ошибки.
//!
//! `LoadSetupEx` читает одиннадцать whitespace-пар из `setupex.ini`, игнорирует
//! label и после успешного open сохраняет partial mutation при stream fail.
//! Signed `long/int` и единственный unsigned `dwValidErrStayTime` представлены
//! буквально. Найденный Login-файл содержит все одиннадцать корректных пар.
//! Доказанные defaults `10/4000`, `5/4000`, `3000`, `60000`, `0`, `60000`,
//! `3/180000` происходят из `CGame::CGame`; area ID исходный constructor не
//! инициализировал, поэтому Rust-конструктор требует его явно до загрузки.
//! `ReLoadSetupEx` после успешного чтения меняет только четыре поздних поля
//! существующих Client/World network owners и не перезапускает listener.
//! Owned `Option` заменяет raw pointers; отсутствие owner вне доказанного
//! вызова после `Init` возвращается как Rust-ошибка предусловия. Остальные
//! setup-ex значения уже изменены самим load.
//!
//! `ReLoadSetup` игнорирует ошибку повторного открытия setup и в любом случае
//! применяет сохранённые значения строго Client -> World -> `AuthManager`, не
//! меняя listener port и не перезапуская network owner. `ReLoadWorldSetup`
//! также игнорирует результат load: уже очищенная/частично прочитанная setup-
//! карта рассылается текущим client account, а отдельный runtime world-map не
//! перестраивается. MFC display-update не имеет Linux-эффекта.
//! `LoadNoQueueCDKeyList` сохраняет безусловный внешний успех и передаёт
//! фактическое чтение соседнему `CLoginQueue`; его typed ошибка остаётся
//! operator-visible. `GetLoginWorldCdkeyNumbers` суммирует все World-списки в
//! numeric порядке с исходным 32-битным wrapping и обслуживает тот же atomic
//! snapshot, который читает server-info worker.
//!
//! `load_listen_port` отдельно читает две пары из `port.ini` и сохраняет
//! partial mutation при stream fail. Его связь с listeners имеет статус
//! `VERIFIED_DISASSEMBLY`: точный EXE пишет Client/World значения в
//! `CGame+0x32C/+0x328`, а `InitNetServer_Client/World` читают именно эти
//! offsets при вызове `Host`; одноимённые поля `tagSetup` не подменяют их.
//! Rust хранит ещё не извлечённые порты как `Option` и блокирует только Host-
//! границу вместо чтения исходной uninitialized памяти.
//! `LoadWorldSetup` очищает setup-карту до открытия `WorldInfoSetup.ini`, ищет
//! точный `#`, читает `long + byte-name + long` и заменяет duplicate numeric
//! ID. ASCII-регистр имени файла совместим с Windows lookup, содержимое не
//! нормализуется. `SetListWorldInfoBySetup` глубоко копирует setup-карту в
//! отдельную runtime-карту и обнуляет только runtime state; проверка открытия
//! мира продолжает читать ненулевой state из setup-карты.
//!
//! Префикс `Init` сохраняет исходный порядок обязательных и нефатальных
//! границ вплоть до создания двух worker threads. Первый `LoadSetup` не вызывает
//! `CLoginQueue::OnInitial`, потому что старый pointer создавался только после
//! single-instance проверки; constructor очереди в этой позиции сначала
//! загружает `NoQueueAccounts.conf`, а явный `OnInitial` остаётся после
//! публикации world-map. Нефатальный итог constructor-load входит в отчёт.
//! `ClearOnlineUserDatabase` открывает отдельное TDS-соединение и исполняет
//! точный `DELETE FROM online_user`; ошибка соединения/execute возвращается в
//! typed-отчёте, но, как исходный `false`, не останавливает `Init` или `Release`.
//! ADO/COM и MFC error-log заменены существующим Tiberius-owner и typed ошибкой.
//! Начальные `srand(time) + random(100)` обслуживали общий CRT RNG; достигнутые
//! Login-потребители случайности уже используют системный `getrandom`, поэтому
//! отдельное process-global seed/warm-up состояние в Rust не создаётся.
//!
//! Windows `FindWindowA/SetWindowTextA` проверяли ключ
//! `LoginServer[area_id][client_port]-FengYun` до очистки runtime-БД. Порядок и
//! точные аргументы подтверждены инструкциями `0x004112B2..0x004112D0` exact
//! EXE. На единственной Linux-платформе тот же ключ атомарно удерживается
//! abstract Unix listener до `Drop`; он не создаёт файл в read-only runtime и
//! не допускает вторую очистку БД раньше bind основных listeners. Стартовая
//! ServLog сохраняет source IPv4 World owner, `server_num = -1`, `world_id = 0`
//! и исходную опечатку `LoingServer`; порядок отдельных date/time буферов
//! подтверждён `0x00411638..0x00411660`.
//!
//! Оба `InitNetServer_*` сначала уничтожали прежний owner, сохраняли новый
//! pointer и только затем вызывали `Host(nullptr, 1, true)`. Ошибка bind/listen
//! оставляла этот новый pointer до `Release`; Rust так же сохраняет неслушающий
//! `Option<owner>` и его command handle. После успешного `Host` один результат
//! `gethostname/gethostbyname` задавал одновременно canonical dotted IPv4 и
//! x86 DWORD. `rustix::uname + ToSocketAddrs` заменяют эти системные вызовы и
//! выбирают первый IPv4; неуспех сохраняет constructor default. Затем Client
//! получает обе CRC-настройки, receive/ban/message и connection/send limits,
//! World — только наблюдаемые receive/ban и connection/send limits, после чего
//! обоим слишком поздно записываются setup-ex backlog/first-message timeout.
//! `bWorldCheckMsgCon` и `dwWorldMaxMsgLen` исходный код записывал, но World
//! parser этой сборки их не читал; притворное Rust-состояние для них не создано.
//!
//! `ProcessMessage` снимает размер каждой FIFO только при достижении её
//! позиции и обрабатывает snapshots строго World -> Client -> Auth. Поэтому
//! World-handler ещё может добавить сообщение в Client/Auth до их замера, а
//! сообщение, добавленное в уже обрабатываемую очередь, остаётся следующему
//! проходу. Переданный async runner является узкой заменой виртуального
//! `CMessage::Run`: он вызывается немедленно для каждого owned сообщения, а
//! `Drop` после возврата заменяет deleting destructor. Typed Auth reconnect
//! применяется в своей позиции FIFO самим `CGame` и не выдаётся как wire-
//! сообщение. Конкретная композиция находится в `applogin/message`; общий
//! protocol framework здесь не создаётся.
//!
//! Достигнутый `MainLoop` сохраняет два вызова boot clock при первой
//! инициализации, strict/wrapping refresh и ping cadence, порядок
//! `ProcessMessage -> CLoginQueue::Run`, очистку telemetry перед `0x4FC01`,
//! независимые complete/timeout отчёты и inclusive server-info cadence.
//! `RefeashInfoText` и `AddLogText` были MFC/operator side effects: Linux turn
//! возвращает их typed-признаками и snapshots без Windows GUI. `Sleep(1)`
//! заменён `tokio::time::sleep` после всех side effects. Достигнутый срок
//! server-info сначала выполняет неблокирующий append, затем при непустом
//! staging/FIFO запускает сериализованный worker; typed outcome и отдельный
//! DB-отчёт не выдают ошибку соединения либо SQL за успешную запись.
//! Старые `int 0/1` представлены вариантами `Exit/Continue`; безопасные
//! lifecycle-ошибки вынесены в `Result` и не маскируются под исходный успех.
//!
//! WinSock socket/net threads и `Sleep(1)` заменены owned Tokio accept/I/O
//! tasks и одним неблокирующим poll `CMyNetClientAuth::run_io_once` перед
//! доменным `MainLoop`. Новый общий runtime, доменные Auth handlers и обработка
//! неизвестного opcode `0x10F101` здесь не создаются. Для следующего
//! endpoint Tokio требует новый consumed `TcpSocket`; повторный bind сохраняет
//! исходные local IP/port, а socket identity не являлась wire-контрактом.
//! Полный `CGame::Init` восстановлен до исходного успешного `return 1`:
//! после `CGasThread` создаётся и запускается `AccLogThread`, а оба
//! проигнорированных результата старого `Thread::Start` публикуются в отчёте.
//! `Release` сохраняет порядок World/Client `ExitWorkerThread`, Auth close,
//! очистки account FIFO, остановки AccLog/GAS, удаления network/DB/queue
//! owners и нефатального `ClearOnlineUserDatabase`. Оба network shutdown
//! останавливают accept, ставят `QUITALL`, обрабатывают snapshots до пустой
//! client-map и затем присоединяют оставшиеся I/O tasks. Внешний
//! `GameThreadFunc` всегда вызывает этот проход после partial `Init`, shutdown
//! либо фатальной typed runtime-ошибки.
//! `CMySocket` cleanup, ADO/COM apartment и static message buffers не получают
//! пустых вызовов: их эффект уже выражен Tokio/Tiberius и локальным владением.
//! Rust дополнительно дожидается detached DB workers перед уничтожением их
//! shared state; они по-прежнему могут конкурировать с исходной DB-clear до
//! момента join. Повторный `Release` после `m_bExit` выполняет только две
//! operator-visible границы и возвращает исходный успех без повторной очистки.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::fs;
use std::future::{Future, poll_fn};
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr as UnixSocketAddr, UnixListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::task::Poll;
use std::thread::{self, JoinHandle as ThreadJoinHandle};
use std::time::Duration;

use chrono::{Local, NaiveDateTime};
use encoding_rs::WINDOWS_1251;
use parking_lot::Mutex;
use rustix::system::uname;
use rustix::time::{ClockId, clock_gettime};
use tokio::net::{TcpStream, lookup_host};
use tokio::task::{JoinError, JoinHandle, JoinSet};
use tokio::time::Instant;

use crate::dbaccess::logindb::rscdkey::{
    LoginDatabaseSettings, MatrixValidation, RsCdKeyDatabaseError, RsCdKeyInitializationError,
    RsCdKeyNotice, RsCdKeyOwner, TdsClient, TiberiusRsCdKey, connect_login_database,
};
use crate::loginserver::applogin::acclogthread::{AccLogThread, AccLogThreadNotice};
use crate::loginserver::applogin::gasthread::{
    CGasThread, GasProcessOutcome, GasVerificationConfig, apply_worker_event,
};
use crate::loginserver::applogin::message::{
    LoginComponentMessageError, LoginProcessMessageOutcome, process_login_messages,
};
use crate::loginserver::loginserver::acclogqueue::AccLogQueue;
use crate::loginserver::loginserver::authhandler::AuthHandler;
use crate::loginserver::loginserver::authmanager::{
    AuthListener, AuthManager, AuthQuest, AuthResult,
};
use crate::loginserver::loginserver::loginqueue::{
    CLoginQueue, HandlePwdCheckedReport, LoginQueueRunReport, LoginQueueTimeoutReport,
    NoQueueAccountsLoadError, NoQueueAccountsLoadReport, TagPwdChecked,
};
use crate::loginserver::loginserver::servlogqueue::{ServLog, ServLogQueue};
use crate::nets::clients::{ClientConnectError, ClientSendQueue};
use crate::nets::mysocket::{DEFAULT_SOCKET_TYPE, legacy_inet_addr, legacy_ipv4_word};
use crate::nets::netlogin::message::{CMessage, SendMessageError};
use crate::nets::netlogin::mynetclient_auth::{
    AuthClientEvent, AuthClientEventPublisher, AuthClientIoError, AuthClientIoStep,
    CMyNetClientAuth,
};
use crate::nets::netlogin::mynetserver_client::CMyNetServerClient;
use crate::nets::netlogin::mynetserver_world::CMyNetServerWorld;
use crate::nets::netlogin::mynetserverclient_client::ClientReceiveError;
use crate::nets::netlogin::mynetserverclient_world::WorldReceiveError;
use crate::nets::servers::{
    ACCEPT_AT_CAPACITY_DELAY, ACCEPT_THREAD_DELAY, AcceptStart, AdmissionOutcome,
    ServerCommandHandle, ServerHostError, ServerIoAction, ServerIoCompletion, ServerSnapshotError,
};
use crate::public::readwrite::read_to;
use crate::public::tools::ini_decode;
use crate::transport::bind_tcp_ipv4;

const LOGIN_SERVER_INFO_MESSAGE_TYPE: i32 = 0x000C_F503;
const KICK_PLAYER_MESSAGE_TYPE: i32 = 0x0004_FB07;
const WORLD_PLAYER_BASE_MESSAGE_TYPE: i32 = 0x0004_FB01;
const WORLD_DELETE_ROLE_MESSAGE_TYPE: i32 = 0x0004_FB02;
const WORLD_RESTORE_ROLE_MESSAGE_TYPE: i32 = 0x0004_FB03;
const WORLD_CREATE_ROLE_MESSAGE_TYPE: i32 = 0x0004_FB04;
const WORLD_PLAYER_DETAIL_MESSAGE_TYPE: i32 = 0x0004_FB05;
const WORLD_PING_REQUEST_MESSAGE_TYPE: i32 = 0x0004_FC01;
const LOGIN_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F501;
const WORLD_INFO_MESSAGE_TYPE: i32 = 0x000A_F509;
const AUTH_RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Одна строка исходного `aslist.ini`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthServerConfig {
    host: Vec<u8>,
    port: u16,
}

impl AuthServerConfig {
    /// Создаёт byte-exact адрес и исходный `unsigned short` port.
    pub(crate) fn new(host: Vec<u8>, port: u16) -> Self {
        Self { host, port }
    }

    /// Возвращает исходный whitespace-token адреса без перекодирования.
    pub(crate) fn host(&self) -> &[u8] {
        &self.host
    }

    /// Возвращает порт в типе исходного `ASConfig::_port`.
    pub(crate) const fn port(&self) -> u16 {
        self.port
    }

    fn from_resolved(endpoint: SocketAddrV4) -> Self {
        Self {
            host: endpoint.ip().to_string().into_bytes(),
            port: endpoint.port(),
        }
    }

    async fn resolve_ipv4(&self) -> Result<SocketAddrV4, AuthConnectFailure> {
        if let Some(address) = legacy_inet_addr(&self.host) {
            return Ok(SocketAddrV4::new(address, self.port));
        }
        let host = std::str::from_utf8(legacy_c_string_prefix(&self.host)).map_err(|_| {
            AuthConnectFailure::AddressEncodingUnsupported {
                host: self.host.clone(),
            }
        })?;
        lookup_host((host, self.port))
            .await
            .map_err(|error| AuthConnectFailure::AddressResolution {
                host: self.host.clone(),
                error,
            })?
            .find_map(|address| match address {
                SocketAddr::V4(address) => Some(address),
                SocketAddr::V6(_) => None,
            })
            .ok_or_else(|| AuthConnectFailure::AddressResolution {
                host: self.host.clone(),
                error: io::Error::new(io::ErrorKind::AddrNotAvailable, "IPv4-адрес отсутствует"),
            })
    }
}

/// Результат исходного partial-read `LoadASList` после успешного открытия.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoadAsListOutcome {
    /// Число полностью прочитанных пар `host + port`.
    pub(crate) loaded: usize,
    /// Последняя неполная/некорректная пара выставила failbit старого stream.
    pub(crate) stopped_on_invalid_pair: bool,
}

/// Ошибка одной попытки подключиться к Auth endpoint.
#[derive(Debug)]
pub(crate) enum AuthConnectFailure {
    /// Byte-oriented hostname нельзя передать системному Linux resolver.
    AddressEncodingUnsupported { host: Vec<u8> },
    /// Числовой адрес и исходный DNS fallback не дали IPv4.
    AddressResolution { host: Vec<u8>, error: io::Error },
    /// Bind IP из setup не принимается исходным `inet_addr`.
    BindAddressInvalid { host: Vec<u8> },
    /// Не удалось создать либо bind-нуть очередной Linux socket.
    Bind(io::Error),
    /// Общий `CClient` не завершил connect успешно.
    Connect(ClientConnectError),
}

impl fmt::Display for AuthConnectFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressEncodingUnsupported { host } => write!(
                formatter,
                "кодировка Auth-адреса {} не поддерживается Linux resolver",
                String::from_utf8_lossy(host)
            ),
            Self::AddressResolution { host, error } => write!(
                formatter,
                "Auth-адрес {} не разрешён в IPv4: {error}",
                String::from_utf8_lossy(host)
            ),
            Self::BindAddressInvalid { host } => write!(
                formatter,
                "Auth bind-адрес {} отклонён inet_addr",
                String::from_utf8_lossy(host)
            ),
            Self::Bind(error) => write!(formatter, "не удалось bind-нуть Auth socket: {error}"),
            Self::Connect(error) => error.fmt(formatter),
        }
    }
}

impl Error for AuthConnectFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AddressResolution { error, .. } | Self::Bind(error) => Some(error),
            Self::Connect(error) => Some(error),
            Self::AddressEncodingUnsupported { .. } | Self::BindAddressInvalid { .. } => None,
        }
    }
}

/// Одна выполненная в исходном порядке попытка `CClient::Connect`.
#[derive(Debug)]
pub(crate) struct AuthConnectAttempt {
    /// Конфигурационная пара, использованная для попытки.
    pub(crate) endpoint: AuthServerConfig,
    /// Причина неуспеха; `None` означает выбранное соединение.
    pub(crate) failure: Option<AuthConnectFailure>,
}

/// Наблюдаемый итог `InitAuthClient` без превращения неуспеха в новый rollback.
#[derive(Debug)]
pub(crate) struct AuthInitializationReport {
    /// Буквальный исходный return `true`.
    pub(crate) legacy_result: bool,
    /// Все попытки до первого успеха либо конца списка.
    pub(crate) attempts: Vec<AuthConnectAttempt>,
    /// Resolved endpoint выбранного AuthServer.
    pub(crate) connected: Option<AuthServerConfig>,
    /// Результат безусловной постановки `0xCF503 + area_id` в send-очередь.
    pub(crate) login_server_info: Result<i32, SendMessageError>,
}

/// Итог одного вызова `ReconnectAS`.
#[derive(Debug)]
pub(crate) struct AuthReconnectReport {
    /// Буквальный исходный bool: удалось ли выбрать endpoint.
    pub(crate) connected: bool,
    /// Все попытки до первого успеха либо конца списка.
    pub(crate) attempts: Vec<AuthConnectAttempt>,
}

/// Живая часть исходного `tagWorldInfo`, нужная GMA и входу в мир.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldRoute {
    world_id: i32,
    name: Vec<u8>,
    state_level: i32,
}

impl WorldRoute {
    /// Сохраняет ID, byte-exact имя и текущий runtime state-level.
    pub(crate) fn new(world_id: i32, name: Vec<u8>, state_level: i32) -> Self {
        Self {
            world_id,
            name,
            state_level,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldSetupEntry {
    name: Vec<u8>,
    configured_state: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct LoginListenPorts {
    client: Option<u32>,
    world: Option<u32>,
}

/// Одна GameServer-запись исходного `tagPingGameServerInfo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PingGameServerInfo {
    ip: Vec<u8>,
    port: u32,
    player_count: u32,
}

impl PingGameServerInfo {
    /// Сохраняет byte-exact IP и 32-битные поля без изменения bit pattern.
    pub(crate) fn new(ip: Vec<u8>, port: u32, player_count: u32) -> Self {
        Self {
            ip,
            port,
            player_count,
        }
    }
}

/// Один WorldServer-ответ исходного `tagPingWorldServerInfo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PingWorldServerInfo {
    ip: Vec<u8>,
    port: u32,
    player_count: u32,
    game_servers: Vec<PingGameServerInfo>,
}

impl PingWorldServerInfo {
    /// Сохраняет dotted IPv4, map-ID port и GameServer-вектор одним snapshot.
    pub(crate) fn new(
        ip: Vec<u8>,
        port: u32,
        player_count: u32,
        game_servers: Vec<PingGameServerInfo>,
    ) -> Self {
        Self {
            ip,
            port,
            player_count,
            game_servers,
        }
    }
}

/// Причина буквального отказа `CGame::AddWorld` активировать соединение.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldActivationFailure {
    /// Numeric World ID отсутствует в загруженном `WorldInfoSetup.ini`.
    UnknownWorld,
    /// ID существует, но присланное имя не совпадает byte-exact.
    NameMismatch,
}

/// Ошибка проигнорированной оригиналом рассылки `0xAF509` одному клиенту.
#[derive(Debug)]
pub(crate) struct WorldInfoBroadcastFailure {
    /// Byte-exact account, по которому выполнялся `SendToClient`.
    pub(crate) account: Vec<u8>,
    /// Фактическая ошибка Client transport-owner.
    pub(crate) error: GameRouteError,
}

/// Наблюдаемый итог `CGame::AddWorld`.
#[derive(Debug)]
pub(crate) enum WorldActivationOutcome {
    /// Мир активирован, а возвращённое число равно размеру setup-карты.
    Activated {
        world_count: i32,
        broadcast_failures: Vec<WorldInfoBroadcastFailure>,
    },
    /// Оригинал написал `INVALID World Server Connectting!` и вернул `-1`.
    Rejected(WorldActivationFailure),
}

/// Наблюдаемый итог `CGame::DelWorld`.
#[derive(Debug)]
pub(crate) struct WorldDeactivationOutcome {
    /// Размер setup-карты; отключение не удаляет её запись.
    pub(crate) world_count: i32,
    /// Ошибки проигнорированных оригиналом client-send операций.
    pub(crate) broadcast_failures: Vec<WorldInfoBroadcastFailure>,
}

/// Результат условной постановки connect-записи в `_serv_logs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ServerInfoLogDisposition {
    Disabled,
    Queued,
    /// `LoadSetup` ещё не доказал значение `dwServerInfoLogTime`.
    SetupValueMissing,
}

/// Точная SQL-стадия `ServerInfoLog`, на которой остановился один worker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ServerInfoDatabaseOperation {
    Connect,
    LoginLookup,
    LoginWrite,
    WorldLookup {
        world_index: usize,
    },
    WorldWrite {
        world_index: usize,
    },
    GameLookup {
        world_index: usize,
        game_index: usize,
    },
    GameWrite {
        world_index: usize,
        game_index: usize,
    },
    ServerLogWrite {
        log_index: usize,
    },
}

/// Неуспех одного сериализованного `ServerInfoLog` без credentials и payload.
#[derive(Debug)]
pub(crate) enum ServerInfoDatabaseFailure {
    Runtime(io::Error),
    Database {
        operation: ServerInfoDatabaseOperation,
        error: RsCdKeyDatabaseError,
    },
    WorkerPanicked,
}

/// Итог одного owned-аналога исходного `ServerInfoLog`.
#[derive(Debug)]
pub(crate) struct ServerInfoDatabaseReport {
    pub(crate) staged_world_records: usize,
    pub(crate) staged_game_records: usize,
    pub(crate) queued_log_records: i32,
    pub(crate) attempted_log_records: usize,
    pub(crate) failure: Option<ServerInfoDatabaseFailure>,
}

struct ServerInfoLogTask {
    staged_world_records: usize,
    staged_game_records: usize,
    queued_log_records: i32,
    handle: ThreadJoinHandle<ServerInfoDatabaseReport>,
}

struct ServerInfoLogShared {
    staging: Mutex<Vec<PingWorldServerInfo>>,
    staged_world_records: AtomicUsize,
    logs: ServLogQueue,
}

impl ServerInfoLogShared {
    fn new() -> Self {
        Self {
            staging: Mutex::new(Vec::new()),
            staged_world_records: AtomicUsize::new(0),
            logs: ServLogQueue::new(),
        }
    }

    fn append_without_waiting(&self, records: &[PingWorldServerInfo]) -> Option<usize> {
        let mut staging = self.staging.try_lock()?;
        staging.extend_from_slice(records);
        let staged_records = staging.len();
        self.staged_world_records
            .store(staged_records, Ordering::SeqCst);
        Some(records.len())
    }

    fn staged_world_records(&self) -> usize {
        self.staged_world_records.load(Ordering::SeqCst)
    }
}

/// Неуспех отдельной стадии `UpdateOnlineUser2DB` без account/credentials.
#[derive(Debug)]
pub(crate) enum OnlineUserDatabaseFailure {
    Runtime(io::Error),
    Connect(RsCdKeyDatabaseError),
    Delete(RsCdKeyDatabaseError),
    Insert {
        account_index: usize,
        error: RsCdKeyDatabaseError,
    },
    WorkerPanicked,
}

/// Итог одного owned-аналога исходного `UpdateOnlineUser2DB`.
#[derive(Debug)]
pub(crate) struct OnlineUserDatabaseReport {
    pub(crate) world_id: i32,
    pub(crate) requested_accounts: usize,
    pub(crate) delete_attempted: bool,
    pub(crate) insert_attempts: usize,
    pub(crate) failures: Vec<OnlineUserDatabaseFailure>,
}

/// Неуспех постановки независимого DB worker без изменения уже собранных карт.
#[derive(Debug)]
pub(crate) enum OnlineUserUpdateStartError {
    SettingsMissing,
    Spawn(io::Error),
}

impl fmt::Display for OnlineUserUpdateStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SettingsMissing => {
                formatter.write_str("DB-настройки UpdateOnlineUser2DB ещё не присоединены к CGame")
            }
            Self::Spawn(error) => write!(
                formatter,
                "не создан owned worker UpdateOnlineUser2DB: {error}"
            ),
        }
    }
}

impl Error for OnlineUserUpdateStartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn(error) => Some(error),
            Self::SettingsMissing => None,
        }
    }
}

struct OnlineUserUpdateTask {
    world_id: i32,
    requested_accounts: usize,
    handle: ThreadJoinHandle<OnlineUserDatabaseReport>,
}

/// Typed-замена трёх достигнутых вызовов `AddLogText` World lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldOperatorLogRecord {
    /// Точная строка оригинала: `INVALID World Server Connectting!`.
    InvalidConnection,
    /// Точный формат оригинала: `WorldServer [%s] connected...ok!`.
    Connected { world_name: Vec<u8> },
    /// Точный формат оригинала: `WorldServer [%s] lost!`.
    Lost { world_name: Vec<u8> },
}

/// Typed-форма одной записи исходной `_acc_logs` из `AccountEnterLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccountEnterRecord {
    /// Byte-exact account.
    pub(crate) account: Vec<u8>,
    /// Исходное 32-битное представление IPv4 до строкового SQL-форматирования.
    pub(crate) client_ip: u32,
    /// Local-time метка, снятая в исходной позиции `AccountEnterLog`.
    pub(crate) recorded_at: NaiveDateTime,
}

/// Typed-форма одной записи исходной `_acc_logs` из `AccountLeaveLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccountLeaveRecord {
    /// Byte-exact account.
    pub(crate) account: Vec<u8>,
    /// Local-time метка, снятая в исходной позиции `AccountLeaveLog`.
    pub(crate) recorded_at: NaiveDateTime,
}

/// Typed-форма одной записи исходной `_acc_logs` из `RoleEnterLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RoleEnterRecord {
    /// Byte-exact account.
    pub(crate) account: Vec<u8>,
    /// Byte-exact имя роли.
    pub(crate) role_name: Vec<u8>,
    /// Исходный `uchar` уровня роли.
    pub(crate) role_level: u8,
    /// Signed World number, переданный исходному `RoleEnterLog`.
    pub(crate) world_number: i32,
    /// Local-time метка, снятая в исходной позиции `RoleEnterLog`.
    pub(crate) recorded_at: NaiveDateTime,
}

/// Typed-форма одной записи исходной `_acc_logs` из `LeaveLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SessionLeaveRecord {
    /// Byte-exact account.
    pub(crate) account: Vec<u8>,
    /// Единая local-time метка для RoleLeaveTime и AccountLeaveTime.
    pub(crate) recorded_at: NaiveDateTime,
}

/// Восстановленные варианты единой FIFO исходной `_acc_logs`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AccountLogRecord {
    Enter(AccountEnterRecord),
    RoleEnter(RoleEnterRecord),
    SessionLeave(SessionLeaveRecord),
    Leave(AccountLeaveRecord),
}

/// Управление `HandlePwdChecked` после доказанной части `PrepareEnter`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrepareEnterOutcome {
    /// Нужно немедленно выполнить `EnterGame` для той же записи.
    Continue,
    /// Запись полностью обработана без `EnterGame`.
    Finished,
    /// Caller должен выполнить `CLoginQueue::matrix_register` и пропустить вход.
    MatrixRegistrationRequired,
}

/// Событие, переданное узким Auth lifecycle остальному будущему `CGame`.
pub(crate) enum AuthGameEvent {
    /// Обычное Auth/close сообщение ожидает своего доменного `Run`.
    Message(CMessage),
    /// Типизированный reconnect был применён в исходной позиции FIFO.
    Reassigned {
        endpoint: AuthServerConfig,
        login_server_info: Result<i32, SendMessageError>,
    },
}

/// Ошибка безопасной границы частично восстановленного Auth lifecycle.
#[derive(Debug)]
pub(crate) enum AuthLifecycleError {
    /// Операция требует существующий старый Auth client.
    MissingCurrentClient,
    /// Reconnect event содержит client без успешного endpoint.
    ReplacementEndpointMissing,
    /// Управляемая reconnect-задача вызвана вне Tokio runtime.
    RuntimeUnavailable,
}

impl fmt::Display for AuthLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCurrentClient => {
                formatter.write_str("текущий Auth-клиент LoginServer отсутствует")
            }
            Self::ReplacementEndpointMissing => {
                formatter.write_str("reconnect-клиент не содержит успешный Auth endpoint")
            }
            Self::RuntimeUnavailable => {
                formatter.write_str("для reconnect-задачи LoginServer не запущен Tokio runtime")
            }
        }
    }
}

impl Error for AuthLifecycleError {}

/// Ошибка безопасной Rust-границы `CGame::ProcessMessage`.
#[derive(Debug)]
pub(crate) enum ProcessMessageError<HandlerError> {
    /// Typed reconnect-событие не может быть применено без отсутствующего факта.
    AuthLifecycle(AuthLifecycleError),
    /// Конкретный component handler достиг собственной доказанной ошибки.
    Handler(HandlerError),
}

/// Ошибка безопасной границы одного достигнутого `CGame::MainLoop` turn.
#[derive(Debug)]
pub(crate) enum LoginMainLoopError {
    /// Конкретный message-handler либо typed Auth lifecycle завершился ошибкой.
    ProcessMessage(ProcessMessageError<LoginComponentMessageError>),
    /// Частично прочитанный setup не определил cadence, достигнутый этим turn.
    MissingSetupField(&'static str),
    /// Достигнутый `ServerInfoLog` требует уже созданный World listener.
    MissingWorldServerForServerInfo,
    /// Внешний `Init` ещё не собрал server-info connection settings.
    MissingServerInfoDatabaseSettings,
}

impl fmt::Display for LoginMainLoopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessMessage(error) => error.fmt(formatter),
            Self::MissingSetupField(field) => {
                write!(
                    formatter,
                    "не инициализировано поле MainLoop setup: {field}"
                )
            }
            Self::MissingWorldServerForServerInfo => {
                formatter.write_str("для ServerInfoLog ещё не создан World listener LoginServer")
            }
            Self::MissingServerInfoDatabaseSettings => formatter
                .write_str("для ServerInfoLog ещё не присоединены DB-настройки из CGame::Init"),
        }
    }
}

impl Error for LoginMainLoopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProcessMessage(error) => Some(error),
            Self::MissingSetupField(_)
            | Self::MissingWorldServerForServerInfo
            | Self::MissingServerInfoDatabaseSettings => None,
        }
    }
}

impl From<ProcessMessageError<LoginComponentMessageError>> for LoginMainLoopError {
    fn from(error: ProcessMessageError<LoginComponentMessageError>) -> Self {
        Self::ProcessMessage(error)
    }
}

/// Один operator-visible результат ping-части исходного `MainLoop`.
#[derive(Debug)]
pub(crate) enum LoginPingWorldEvent {
    /// Начат новый опрос после очистки прежнего telemetry-вектора.
    Requested {
        cleared_responses: usize,
        send_result: Result<i32, GameRouteError>,
    },
    /// Число ответов достигло полного размера setup-карты миров.
    Completed {
        expected_worlds: usize,
        responses: Vec<PingWorldServerInfo>,
    },
    /// Strict error-timeout завершил текущий опрос с накопленными ответами.
    TimedOut {
        expected_worlds: usize,
        responses: Vec<PingWorldServerInfo>,
    },
}

/// Результат достигнутой server-info позиции одного turn.
#[derive(Debug)]
pub(crate) enum LoginServerInfoTurn {
    Disabled,
    NotDue,
    DueWithoutData,
    WorkerStarted {
        append_acquired_lock: bool,
        copied_telemetry_records: usize,
        staged_telemetry_records: usize,
        queued_log_records: i32,
    },
    WorkerStartFailed {
        append_acquired_lock: bool,
        copied_telemetry_records: usize,
        staged_telemetry_records: usize,
        queued_log_records: i32,
        error: io::Error,
    },
}

/// Наблюдаемый итог одного вызова восстановленного `CGame::MainLoop`.
#[derive(Debug)]
pub(crate) enum LoginMainLoopOutcome {
    /// `m_bExit` был установлен; message/queue/ping позиции не выполнялись.
    Exit { refresh_info_due: bool },
    /// Полный достигнутый turn завершился исходным результатом `1`.
    Continue {
        refresh_info_due: bool,
        gas: Vec<GasProcessOutcome>,
        account_logs: Vec<AccLogThreadNotice>,
        messages: LoginProcessMessageOutcome,
        login_queue: Box<LoginQueueRunReport>,
        ping_events: Vec<LoginPingWorldEvent>,
        server_info: LoginServerInfoTurn,
    },
}

/// Конкретное входящее направление managed Login runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoginNetworkDirection {
    World,
    Client,
}

/// Один технический World network turn до доменного `ProcessMessage`.
#[derive(Debug, Default)]
pub(crate) struct LoginWorldNetworkTurn {
    pub(crate) admissions: Vec<AdmissionOutcome>,
    pub(crate) accept_errors: Vec<io::Error>,
    pub(crate) io_completions: Vec<ServerIoCompletion>,
    pub(crate) processed_commands: i32,
    pub(crate) snapshot_errors: Vec<ServerSnapshotError<WorldReceiveError>>,
}

/// Один технический Client network turn до доменного `ProcessMessage`.
#[derive(Debug, Default)]
pub(crate) struct LoginClientNetworkTurn {
    pub(crate) admissions: Vec<AdmissionOutcome>,
    pub(crate) accept_errors: Vec<io::Error>,
    pub(crate) io_completions: Vec<ServerIoCompletion>,
    pub(crate) processed_commands: i32,
    pub(crate) snapshot_errors: Vec<ServerSnapshotError<ClientReceiveError>>,
}

/// Итог concurrent network cadence перед одним `CGame::MainLoop`.
#[derive(Debug, Default)]
pub(crate) struct LoginNetworkTurn {
    pub(crate) world: LoginWorldNetworkTurn,
    pub(crate) client: LoginClientNetworkTurn,
    /// `None` означает отсутствие подключённого Auth socket либо readiness.
    pub(crate) auth: Option<Result<AuthClientIoStep, AuthClientIoError>>,
}

/// Фатальная ошибка owned-задачи либо отсутствующего init-owner.
#[derive(Debug)]
pub(crate) enum LoginNetworkRuntimeError {
    MissingNetworkOwner(LoginNetworkDirection),
    Task {
        direction: LoginNetworkDirection,
        error: JoinError,
    },
}

impl fmt::Display for LoginNetworkRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNetworkOwner(direction) => {
                write!(
                    formatter,
                    "не создан {:?} network owner LoginServer",
                    direction
                )
            }
            Self::Task { direction, error } => {
                write!(
                    formatter,
                    "завершилась {:?} I/O-задача LoginServer: {error}",
                    direction
                )
            }
        }
    }
}

impl Error for LoginNetworkRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Task { error, .. } => Some(error),
            Self::MissingNetworkOwner(_) => None,
        }
    }
}

/// Ошибка одного внешнего `GameThreadFunc` turn.
#[derive(Debug)]
pub(crate) enum LoginRuntimeError {
    Network(LoginNetworkRuntimeError),
    MainLoop(LoginMainLoopError),
}

impl fmt::Display for LoginRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => error.fmt(formatter),
            Self::MainLoop(error) => error.fmt(formatter),
        }
    }
}

impl Error for LoginRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::MainLoop(error) => Some(error),
        }
    }
}

/// Полный отчёт единственного owned аналога исходного `GameThreadFunc`.
#[derive(Debug)]
pub(crate) struct LoginGameThreadReport {
    pub(crate) initialization: Result<LoginInitializationReport, LoginInitializationError>,
    pub(crate) completed_turns: u64,
    pub(crate) runtime_error: Option<LoginRuntimeError>,
    pub(crate) release: LoginReleaseReport,
}

#[derive(Default)]
struct LoginMainLoopState {
    now_ms: Option<u32>,
    last_refresh_info_ms: Option<u32>,
    last_ping_world_ms: Option<u32>,
    last_server_info_log_ms: Option<u32>,
    ping_in_flight: bool,
}

impl<HandlerError: fmt::Display> fmt::Display for ProcessMessageError<HandlerError> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthLifecycle(error) => error.fmt(formatter),
            Self::Handler(error) => error.fmt(formatter),
        }
    }
}

impl<HandlerError> Error for ProcessMessageError<HandlerError>
where
    HandlerError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AuthLifecycle(error) => Some(error),
            Self::Handler(error) => Some(error),
        }
    }
}

/// Ошибка доказанной GMA-маршрутизации из `CGame`.
#[derive(Debug)]
pub(crate) enum GameRouteError {
    /// Текущий Auth client ещё не создан.
    MissingAuthClient,
    /// World net-owner ещё не присоединён к `CGame`.
    MissingWorldServer,
    /// Client net-owner ещё не присоединён к `CGame`.
    MissingClientServer,
    /// Построение legacy envelope завершилось ошибкой.
    Message(SendMessageError),
}

impl fmt::Display for GameRouteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAuthClient => formatter.write_str("Auth-клиент LoginServer отсутствует"),
            Self::MissingWorldServer => {
                formatter.write_str("World net-owner LoginServer отсутствует")
            }
            Self::MissingClientServer => {
                formatter.write_str("Client net-owner LoginServer отсутствует")
            }
            Self::Message(error) => error.fmt(formatter),
        }
    }
}

impl Error for GameRouteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Message(error) => Some(error),
            Self::MissingAuthClient | Self::MissingWorldServer | Self::MissingClientServer => None,
        }
    }
}

impl From<SendMessageError> for GameRouteError {
    fn from(error: SendMessageError) -> Self {
        Self::Message(error)
    }
}

/// Итог изменения исходного `m_mapPWError` после default Auth-отказа.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PasswordFailureOutcome {
    /// `lforbitTime == 0`: счётчик и DB-владелец не затронуты.
    Disabled,
    /// Создан либо увеличен счётчик текущего account.
    Counted { failures: i32 },
    /// Достигнутый ранее лимит вызвал синхронный `CDKeyBan`.
    BanAttempted { succeeded: bool },
    /// DB-owner ещё не присоединён к частично восстановленному `CGame`.
    BanOwnerMissing,
}

/// Структурированная замена исходных проигнорированных send-результатов и log.
#[derive(Debug)]
pub(crate) enum AuthHandlerNotice {
    /// Не удалось передать `0xAF501` текущему client net-owner.
    ClientResponseFailed {
        /// Исходный signed socket ID.
        socket_id: i32,
        /// Причина ошибки фактической сетевой границы.
        error: GameRouteError,
    },
    /// Duplicate queue-entry дошёл до `KickOut`, но его маршрут не выполнен.
    KickOutFailed {
        /// Byte-exact account прежней записи.
        account: Vec<u8>,
        /// Причина ошибки фактической сетевой границы.
        error: GameRouteError,
    },
    /// Исходный `CDKeyBan` вернул `false`.
    CdKeyBanFailed { account: Vec<u8> },
    /// Частично восстановленный Login lifecycle ещё не присоединил `CRsCDKey`.
    CdKeyBanOwnerMissing { account: Vec<u8> },
}

#[derive(Clone)]
struct AuthReconnectPlan {
    bind_ip: Vec<u8>,
    bind_port: u32,
    auth_servers: Vec<AuthServerConfig>,
}

/// Источник, который исходный `LoadSetup` смог открыть первым.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoginSetupSource {
    /// Обычный positional `setup.ini` новой формы.
    Plain,
    /// Декодированный `setup.dat` старой сокращённой формы.
    Encoded,
}

/// Нечувствительный итог одного исходного `LoadSetup`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoginSetupLoadReport {
    /// Фактически выбранный файл без его содержимого.
    pub(crate) source: LoginSetupSource,
    /// Число полностью прочитанных пар `label + value` до stream fail-state.
    pub(crate) parsed_pairs: usize,
    /// Первая пара, которую stream уже не смог извлечь.
    pub(crate) stopped_at_pair: Option<usize>,
    /// Были ли безопасно пересчитаны четыре исходных world state-level.
    pub(crate) state_levels_applied: bool,
    /// Получила ли очередь все три обязательных значения `OnInitial`.
    pub(crate) login_queue_initialized: bool,
}

/// Оба исходных setup-файла недоступны; их содержимое в ошибку не попадает.
#[derive(Debug)]
pub(crate) struct LoginSetupOpenError {
    plain: io::Error,
    encoded: io::Error,
}

impl fmt::Display for LoginSetupOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не открыты setup.ini ({}) и setup.dat ({})",
            self.plain, self.encoded
        )
    }
}

impl Error for LoginSetupOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.encoded)
    }
}

/// Нечувствительный итог positional-чтения `setupex.ini`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoginSetupExLoadReport {
    /// Число полностью прочитанных пар `label + value` до stream fail-state.
    pub(crate) parsed_pairs: usize,
    /// Первая пара, которую stream уже не смог извлечь.
    pub(crate) stopped_at_pair: Option<usize>,
}

/// Итог positional-чтения отдельного `port.ini`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoginListenPortLoadReport {
    /// Число полностью прочитанных пар `label + u32`.
    pub(crate) parsed_pairs: usize,
    /// Первая пара, на которой старый stream вошёл бы в fail-state.
    pub(crate) stopped_at_pair: Option<usize>,
}

/// Итог чтения отдельного `WorldInfoSetup.ini` без публикации имён миров.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoginWorldSetupLoadReport {
    /// Число полностью прочитанных записей `# + id + name + state`.
    pub(crate) parsed_records: usize,
    /// Число уникальных numeric ID после исходной map-замены duplicates.
    pub(crate) unique_worlds: usize,
    /// Первая запись с неполной либо нечисловой тройкой.
    pub(crate) stopped_at_record: Option<usize>,
}

/// Operator-visible итог вложенного `CLoginQueue::LoadNoQueueCdkeyList`.
#[derive(Debug)]
pub(crate) enum LoginNoQueueAccountsLoadDisposition {
    Loaded(NoQueueAccountsLoadReport),
    Failed(NoQueueAccountsLoadError),
}

/// Итог исходной безусловно успешной `CGame::LoadNoQueueCDKeyList`.
#[derive(Debug)]
pub(crate) struct LoginNoQueueAccountsReloadOutcome {
    /// Всегда равен исходному `return true` после существующего queue-owner.
    pub(crate) legacy_result: bool,
    pub(crate) load: LoginNoQueueAccountsLoadDisposition,
}

/// Safe Rust-предусловие GUI-вызова после полного `CGame::Init`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoginQueueOwnerMissing;

impl fmt::Display for LoginQueueOwnerMissing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CLoginQueue ещё не создан либо уже освобождён")
    }
}

impl Error for LoginQueueOwnerMissing {}

/// Итог `ReLoadSetup`: ошибка чтения не отменяет применение прежнего setup.
#[derive(Debug)]
pub(crate) struct LoginSetupReloadOutcome {
    pub(crate) legacy_result: bool,
    pub(crate) load: Result<LoginSetupLoadReport, LoginSetupOpenError>,
}

/// Безопасная граница `ReLoadSetup` вне доказанного post-Init вызова.
#[derive(Debug)]
pub(crate) enum LoginSetupReloadError {
    SetupField(&'static str),
    ClientOwner,
    WorldOwner,
}

impl fmt::Display for LoginSetupReloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetupField(field) => {
                write!(formatter, "ReLoadSetup не имеет прежнего значения {field}")
            }
            Self::ClientOwner => formatter.write_str("Client network owner не создан"),
            Self::WorldOwner => formatter.write_str("World network owner не создан"),
        }
    }
}

impl Error for LoginSetupReloadError {}

/// Итог безусловно успешной `ReLoadWorldSetup` после существующего queue-owner.
#[derive(Debug)]
pub(crate) struct LoginWorldSetupReloadOutcome {
    pub(crate) legacy_result: bool,
    pub(crate) load: Result<LoginWorldSetupLoadReport, io::Error>,
    pub(crate) broadcast_failures: Vec<WorldInfoBroadcastFailure>,
}

/// Итог доказанного префикса `CGame::Init` до создания двух worker threads.
#[derive(Debug)]
pub(crate) struct LoginInitializationPrefixReport {
    /// Результат обязательного позиционного `LoadSetup`.
    pub(crate) setup: LoginSetupLoadReport,
    /// Результат обязательного чтения отдельного `port.ini`.
    pub(crate) listen_ports: LoginListenPortLoadReport,
    /// Нефатальный результат `LoadSetupEx`; ошибка открытия не останавливала Init.
    pub(crate) setup_ex: Result<LoginSetupExLoadReport, io::Error>,
    /// Результат обязательного чтения setup-карты миров.
    pub(crate) world_setup: LoginWorldSetupLoadReport,
    /// Результат обязательного чтения Auth endpoint-списка.
    pub(crate) auth_servers: LoadAsListOutcome,
    /// Число runtime World-записей после `SetListWorldInfoBySetup`.
    pub(crate) world_routes: usize,
    /// Выполнен ли явный `CLoginQueue::OnInitial` из собственной позиции Init.
    pub(crate) login_queue_initialized: bool,
    /// Нефатальный итог constructor-загрузки `NoQueueAccounts.conf`.
    pub(crate) no_queue_accounts: Result<NoQueueAccountsLoadReport, NoQueueAccountsLoadError>,
    /// Нефатальный результат исходного проигнорированного DB-clear.
    pub(crate) online_user_clear: Result<(), RsCdKeyDatabaseError>,
    /// Полный исходный результат `InitAuthClient`, включая неуспешные endpoints.
    pub(crate) auth: AuthInitializationReport,
    /// Поставлена ли стартовая запись в `_serv_logs`.
    pub(crate) startup_log: ServerInfoLogDisposition,
}

/// Итог доказанного префикса `CGame::Init` через запуск `CGasThread`.
#[derive(Debug)]
pub(crate) struct LoginInitializationThroughGasReport {
    /// Полный уже доказанный префикс до worker-tail.
    pub(crate) prefix: LoginInitializationPrefixReport,
    /// Исходно проигнорированный результат `Thread::Start(m_pGasThread)`.
    pub(crate) gas_thread_start: Result<(), io::Error>,
}

/// Полный итог восстановленного `CGame::Init`, чей исходный результат равен `1`.
#[derive(Debug)]
pub(crate) struct LoginInitializationReport {
    /// Префикс и нефатальный результат запуска `CGasThread`.
    pub(crate) through_gas: LoginInitializationThroughGasReport,
    /// Исходно проигнорированный результат `Thread::Start(AccLogThread)`.
    pub(crate) account_log_thread_start: Result<(), io::Error>,
}

/// Две исходные `PutDebugString`-позиции, выполняемые при каждом `Release`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoginReleaseOperatorNotice {
    Exiting,
    Exited,
}

/// Итог полного обратного прохода `CGame::Release` без credential values.
#[derive(Debug)]
pub(crate) struct LoginReleaseReport {
    /// Всегда равен исходному `return 1`.
    pub(crate) legacy_result: bool,
    /// `false` означает повторный вызов после уже установленного `m_bExit`.
    pub(crate) performed: bool,
    /// Обе debug-позиции существуют даже у повторного no-op вызова.
    pub(crate) operator_notices: [LoginReleaseOperatorNotice; 2],
    pub(crate) world_network_was_present: bool,
    pub(crate) client_network_was_present: bool,
    pub(crate) world_network_shutdown_snapshots: u32,
    pub(crate) client_network_shutdown_snapshots: u32,
    pub(crate) world_network_task_errors: Vec<JoinError>,
    pub(crate) client_network_task_errors: Vec<JoinError>,
    pub(crate) auth_client_was_present: bool,
    pub(crate) account_log_thread_was_present: bool,
    pub(crate) gas_thread_was_present: bool,
    pub(crate) database_owner_was_present: bool,
    /// `None` у повторного no-op; DB-ошибка исходно не меняла результат.
    pub(crate) online_user_clear: Option<Result<(), RsCdKeyDatabaseError>>,
    pub(crate) login_queue_was_present: bool,
    /// Owned Rust workers, присоединённые перед уничтожением shared state.
    pub(crate) online_user_workers_joined: usize,
    pub(crate) server_info_workers_joined: usize,
}

impl LoginReleaseReport {
    fn already_released() -> Self {
        Self {
            legacy_result: true,
            performed: false,
            operator_notices: [
                LoginReleaseOperatorNotice::Exiting,
                LoginReleaseOperatorNotice::Exited,
            ],
            world_network_was_present: false,
            client_network_was_present: false,
            world_network_shutdown_snapshots: 0,
            client_network_shutdown_snapshots: 0,
            world_network_task_errors: Vec::new(),
            client_network_task_errors: Vec::new(),
            auth_client_was_present: false,
            account_log_thread_was_present: false,
            gas_thread_was_present: false,
            database_owner_was_present: false,
            online_user_clear: None,
            login_queue_was_present: false,
            online_user_workers_joined: 0,
            server_info_workers_joined: 0,
        }
    }
}

/// Фатальная граница доказанного префикса `CGame::Init`.
#[derive(Debug)]
pub(crate) enum LoginInitializationError {
    /// Обязательный основной setup не открыт.
    Setup(LoginSetupOpenError),
    /// Обязательный `port.ini` не открыт.
    ListenPort(io::Error),
    /// Exact single-instance ключ уже занят либо не может быть создан.
    SingleInstance(io::Error),
    /// Обязательный `WorldInfoSetup.ini` не открыт.
    WorldSetup(io::Error),
    /// Обязательный `aslist.ini` не открыт.
    AuthServerList(io::Error),
    /// Частично прочитанный setup не дал обязательного поля.
    MissingSetupField(&'static str),
    /// Не создан основной DB-owner CD-key.
    DatabaseOwner(RsCdKeyInitializationError),
    /// Не создан WorldServer listener.
    WorldNetwork(LoginNetworkInitializationError),
    /// Не создан client listener.
    ClientNetwork(LoginNetworkInitializationError),
}

impl fmt::Display for LoginInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setup(error) => write!(formatter, "не загружен Login setup: {error}"),
            Self::ListenPort(error) => write!(formatter, "не загружен port.ini: {error}"),
            Self::SingleInstance(error) => {
                write!(
                    formatter,
                    "уже занят single-instance ключ LoginServer: {error}"
                )
            }
            Self::WorldSetup(error) => {
                write!(formatter, "не загружен WorldInfoSetup.ini: {error}")
            }
            Self::AuthServerList(error) => write!(formatter, "не загружен aslist.ini: {error}"),
            Self::MissingSetupField(field) => write!(
                formatter,
                "не инициализировано обязательное поле Login Init: {field}"
            ),
            Self::DatabaseOwner(error) => {
                write!(formatter, "не создан DB-owner LoginServer: {error}")
            }
            Self::WorldNetwork(error) => {
                write!(formatter, "не инициализирован World listener: {error}")
            }
            Self::ClientNetwork(error) => {
                write!(formatter, "не инициализирован Client listener: {error}")
            }
        }
    }
}

impl Error for LoginInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Setup(error) => Some(error),
            Self::ListenPort(error)
            | Self::SingleInstance(error)
            | Self::WorldSetup(error)
            | Self::AuthServerList(error) => Some(error),
            Self::DatabaseOwner(error) => Some(error),
            Self::WorldNetwork(error) | Self::ClientNetwork(error) => Some(error),
            Self::MissingSetupField(_) => None,
        }
    }
}

/// Ошибка безопасной границы одного `InitNetServer_*`.
#[derive(Debug)]
pub(crate) enum LoginNetworkInitializationError {
    /// Обязательное поле частично прочитанной конфигурации не инициализировано.
    MissingSetupField(&'static str),
    /// Общий Linux listener не смог выполнить исходный `Host`.
    Host(ServerHostError),
}

impl fmt::Display for LoginNetworkInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSetupField(field) => {
                write!(
                    formatter,
                    "не инициализировано обязательное поле конфигурации LoginServer: {field}"
                )
            }
            Self::Host(error) => write!(formatter, "не создан Login listener: {error}"),
        }
    }
}

impl Error for LoginNetworkInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingSetupField(_) => None,
            Self::Host(error) => Some(error),
        }
    }
}

/// Ошибка Rust-предусловий достигнутого `ReLoadSetupEx`.
#[derive(Debug)]
pub(crate) enum LoginSetupExReloadError {
    /// Повторное чтение `setupex.ini` завершилось ошибкой открытия.
    Load(io::Error),
    /// Client owner ещё не создан частично восстановленным `Init`.
    MissingClientServer,
    /// World owner ещё не создан частично восстановленным `Init`.
    MissingWorldServer,
}

impl fmt::Display for LoginSetupExReloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "не открыт setupex.ini: {error}"),
            Self::MissingClientServer => formatter.write_str("Client network owner не создан"),
            Self::MissingWorldServer => formatter.write_str("World network owner не создан"),
        }
    }
}

impl Error for LoginSetupExReloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::MissingClientServer | Self::MissingWorldServer => None,
        }
    }
}

impl From<io::Error> for LoginSetupExReloadError {
    fn from(error: io::Error) -> Self {
        Self::Load(error)
    }
}

/// Owned-форма исходного `CGame::tagSetupEx` с точной signedness полей.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoginSetupEx {
    area_id: i32,
    client_max_block_connections: i32,
    client_valid_receive_delay_ms: i32,
    world_max_block_connections: i32,
    world_valid_receive_delay_ms: i32,
    quest_player_data_interval_ms: i32,
    matrix_timeout_ms: i32,
    valid_code: i32,
    valid_code_overtime_ms: i32,
    valid_error_upper_limit: i32,
    valid_error_stay_time_ms: u32,
}

impl LoginSetupEx {
    /// Создаёт setup-ex с доказанными defaults и явным area ID Rust-границы.
    const fn new(area_id: i32) -> Self {
        Self {
            area_id,
            client_max_block_connections: 10,
            client_valid_receive_delay_ms: 4_000,
            world_max_block_connections: 5,
            world_valid_receive_delay_ms: 4_000,
            quest_player_data_interval_ms: 3_000,
            matrix_timeout_ms: 60_000,
            valid_code: 0,
            valid_code_overtime_ms: 60_000,
            valid_error_upper_limit: 3,
            valid_error_stay_time_ms: 180_000,
        }
    }

    fn parse_positional(&mut self, bytes: &[u8]) -> LoginSetupExLoadReport {
        let mut tokens = SetupTokens::new(bytes);

        macro_rules! read_number {
            ($field:ident, $type:ty) => {{
                let Some(raw) = tokens.next_value() else {
                    return Self::report(&tokens);
                };
                let Some(value) = parse_ascii::<$type>(raw) else {
                    // Malformed numeric token безопасно сохраняет прежнее
                    // значение поля и останавливает positional parsing.
                    return Self::report(&tokens);
                };
                self.$field = value;
                tokens.parsed();
            }};
        }

        read_number!(area_id, i32);
        read_number!(client_max_block_connections, i32);
        read_number!(client_valid_receive_delay_ms, i32);
        read_number!(world_max_block_connections, i32);
        read_number!(world_valid_receive_delay_ms, i32);
        read_number!(quest_player_data_interval_ms, i32);
        read_number!(matrix_timeout_ms, i32);
        read_number!(valid_code, i32);
        read_number!(valid_code_overtime_ms, i32);
        read_number!(valid_error_upper_limit, i32);
        read_number!(valid_error_stay_time_ms, u32);

        Self::report(&tokens)
    }

    const fn report(tokens: &SetupTokens<'_>) -> LoginSetupExLoadReport {
        let (parsed_pairs, stopped_at_pair) = tokens.outcome();
        LoginSetupExLoadReport {
            parsed_pairs,
            stopped_at_pair,
        }
    }
}

/// Owned-форма исходного `CGame::tagSetup` с явными uninitialized-границами.
#[derive(Clone, Debug)]
pub(crate) struct LoginSetup {
    server_version: Option<i32>,
    listen_port_client: Option<u32>,
    listen_port_world: Option<u32>,
    sql_connection_type: Vec<u8>,
    sql_server_ip: Vec<u8>,
    sql_user_name: Vec<u8>,
    sql_password: Vec<u8>,
    database_name: Vec<u8>,
    client_check_net: Option<bool>,
    client_max_byte_num: Option<u32>,
    client_max_message_len: Option<u32>,
    client_ban_ip_time: Option<u32>,
    client_check_message_content: Option<bool>,
    client_max_connections: Option<i32>,
    client_max_io_send: Option<i32>,
    client_max_send_buffer: Option<i32>,
    world_check_net: Option<bool>,
    world_max_byte_num: Option<u32>,
    world_max_message_len: Option<u32>,
    world_ban_ip_time: Option<u32>,
    world_check_message_content: Option<bool>,
    world_max_connections: Option<i32>,
    world_max_io_send: Option<i32>,
    world_max_send_buffer: Option<i32>,
    refresh_info_time: Option<u32>,
    save_info_time: Option<u32>,
    do_queue_interval: Option<u32>,
    send_to_queue_interval: Option<u32>,
    world_max_players: Option<u32>,
    world_busy_scale: Option<f32>,
    world_full_scale: Option<f32>,
    ping_world_time: Option<u32>,
    ping_world_error_time: Option<u32>,
    check_forbidden_ip: Option<bool>,
    check_allowed_ip: Option<bool>,
    check_between_ip: Option<bool>,
    server_info_log_time: Option<u32>,
    server_info_log_provider: Vec<u8>,
    server_info_log_user: Vec<u8>,
    server_info_log_password: Vec<u8>,
    server_info_log_ip: Vec<u8>,
    server_info_log_database: Vec<u8>,
    auth_timeout: Option<u32>,
    bind_ip: Vec<u8>,
    bind_port: Option<u16>,
    forbidden_account_time: Option<i32>,
    forbidden_password_errors: Option<i32>,
    mode: Option<i32>,
    gas_area_user: Vec<u8>,
    gas_area_password: Vec<u8>,
    billing_database_ip: Vec<u8>,
    billing_database_name: Vec<u8>,
    billing_database_user: Vec<u8>,
    billing_database_password: Vec<u8>,
    inside_use: Option<i32>,
    verification_address: Vec<u8>,
    verification_sign_upper: Option<i32>,
}

#[derive(Clone, Copy)]
struct LoginClientNetworkConfig {
    check_receive_rate: bool,
    check_content_crc: bool,
    maximum_bytes_per_second: u32,
    forbid_time_ms: u32,
    maximum_message_length: u32,
    maximum_clients: i32,
    maximum_in_flight_sends: i32,
    permitted_send_bytes: i32,
}

#[derive(Clone, Copy)]
struct LoginWorldNetworkConfig {
    check_receive_rate: bool,
    maximum_bytes_per_second: u32,
    forbid_time_ms: u32,
    maximum_clients: i32,
    maximum_in_flight_sends: i32,
    permitted_send_bytes: i32,
}

impl LoginSetup {
    fn database_settings(&self) -> LoginDatabaseSettings {
        LoginDatabaseSettings::new(
            self.sql_server_ip.clone(),
            self.database_name.clone(),
            self.sql_user_name.clone(),
            self.sql_password.clone(),
        )
    }

    fn server_info_database_settings(&self) -> LoginDatabaseSettings {
        LoginDatabaseSettings::new(
            self.server_info_log_ip.clone(),
            self.server_info_log_database.clone(),
            self.server_info_log_user.clone(),
            self.server_info_log_password.clone(),
        )
    }

    fn client_network_config(&self) -> Result<LoginClientNetworkConfig, &'static str> {
        Ok(LoginClientNetworkConfig {
            check_receive_rate: self.client_check_net.ok_or("bCheckNet")?,
            check_content_crc: self.client_check_message_content.ok_or("bCheckMsgCon")?,
            maximum_bytes_per_second: self.client_max_byte_num.ok_or("dwMaxByteNum")?,
            forbid_time_ms: self.client_ban_ip_time.ok_or("dwBanIPTime")?,
            maximum_message_length: self.client_max_message_len.ok_or("dwMaxMsgLen")?,
            maximum_clients: self.client_max_connections.ok_or("lMaxConnectNum")?,
            maximum_in_flight_sends: self.client_max_io_send.ok_or("lMaxIOSendNum")?,
            permitted_send_bytes: self.client_max_send_buffer.ok_or("lMaxClientSendBuf")?,
        })
    }

    fn world_network_config(&self) -> Result<LoginWorldNetworkConfig, &'static str> {
        Ok(LoginWorldNetworkConfig {
            check_receive_rate: self.world_check_net.ok_or("bWorldCheckNet")?,
            maximum_bytes_per_second: self.world_max_byte_num.ok_or("dwWorldMaxByteNum")?,
            forbid_time_ms: self.world_ban_ip_time.ok_or("dwWorldBanIPTime")?,
            maximum_clients: self.world_max_connections.ok_or("lWorldMaxConnectNum")?,
            maximum_in_flight_sends: self.world_max_io_send.ok_or("lWorldMaxIOSendNum")?,
            permitted_send_bytes: self.world_max_send_buffer.ok_or("lWorldMaxClientSendBuf")?,
        })
    }

    /// Возвращает версию client protocol только после успешного extraction.
    pub(crate) const fn server_version(&self) -> Option<i32> {
        self.server_version
    }

    /// Возвращает три исходных IP admission-флага без назначения defaults.
    pub(crate) const fn ip_checks(&self) -> Option<(bool, bool, bool)> {
        match (
            self.check_allowed_ip,
            self.check_forbidden_ip,
            self.check_between_ip,
        ) {
            (Some(allowed), Some(forbidden), Some(between)) => Some((allowed, forbidden, between)),
            _ => None,
        }
    }

    /// Возвращает режим внешней GAS-очереди, сохраняя constructor-default `1`.
    pub(crate) const fn inside_use(&self) -> Option<i32> {
        self.inside_use
    }

    fn parse_plain(&mut self, bytes: &[u8]) -> (usize, Option<usize>) {
        self.parse_positional(bytes, true, true)
    }

    fn parse_encoded(&mut self, bytes: &[u8]) -> (usize, Option<usize>) {
        self.parse_positional(bytes, false, false)
    }

    fn parse_positional(
        &mut self,
        bytes: &[u8],
        include_server_version: bool,
        include_extended_tail: bool,
    ) -> (usize, Option<usize>) {
        let mut tokens = SetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.outcome();
                };
                let Some(value) = $parser(raw) else {
                    // Malformed numeric token безопасно сохраняет прежнее
                    // значение поля и останавливает positional parsing.
                    return tokens.outcome();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }

        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw).map(Some));
            };
        }

        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, |raw| parse_legacy_bool(raw).map(Some));
            };
        }

        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        if include_server_version {
            read_number!(server_version, i32);
        }
        read_number!(listen_port_client, u32);
        read_number!(listen_port_world, u32);
        read_bytes!(sql_connection_type);
        read_bytes!(sql_server_ip);
        read_bytes!(sql_user_name);
        read_bytes!(sql_password);
        read_bytes!(database_name);
        read_bool!(client_check_net);
        read_number!(client_max_byte_num, u32);
        read_number!(client_max_message_len, u32);
        read_number!(client_ban_ip_time, u32);
        read_bool!(client_check_message_content);
        read_number!(client_max_connections, i32);
        read_number!(client_max_io_send, i32);
        read_number!(client_max_send_buffer, i32);
        read_bool!(world_check_net);
        read_number!(world_max_byte_num, u32);
        read_number!(world_max_message_len, u32);
        read_number!(world_ban_ip_time, u32);
        read_bool!(world_check_message_content);
        read_number!(world_max_connections, i32);
        read_number!(world_max_io_send, i32);
        read_number!(world_max_send_buffer, i32);
        read_number!(refresh_info_time, u32);
        read_number!(save_info_time, u32);
        read_number!(do_queue_interval, u32);
        read_number!(send_to_queue_interval, u32);
        read_number!(world_max_players, u32);
        read_number!(world_busy_scale, f32);
        read_number!(world_full_scale, f32);
        read_number!(ping_world_time, u32);
        read_number!(ping_world_error_time, u32);
        read_bool!(check_forbidden_ip);
        read_bool!(check_allowed_ip);
        read_bool!(check_between_ip);
        read_number!(server_info_log_time, u32);
        read_bytes!(server_info_log_provider);
        read_bytes!(server_info_log_user);
        read_bytes!(server_info_log_password);
        read_bytes!(server_info_log_ip);
        read_bytes!(server_info_log_database);
        read_number!(auth_timeout, u32);
        read_bytes!(bind_ip);
        read_number!(bind_port, u16);

        if include_extended_tail {
            read_number!(forbidden_account_time, i32);
            read_number!(forbidden_password_errors, i32);
            read_number!(mode, i32);
            read_bytes!(gas_area_user);
            read_bytes!(gas_area_password);
            read_bytes!(billing_database_ip);
            read_bytes!(billing_database_name);
            read_bytes!(billing_database_user);
            read_bytes!(billing_database_password);
            read_number!(inside_use, i32);
            read_bytes!(verification_address);
            read_number!(verification_sign_upper, i32);
        }

        tokens.outcome()
    }
}

fn parse_ascii<T: std::str::FromStr>(raw: &[u8]) -> Option<T> {
    std::str::from_utf8(raw).ok()?.parse().ok()
}

fn parse_legacy_bool(raw: &[u8]) -> Option<bool> {
    match raw {
        b"0" => Some(false),
        b"1" => Some(true),
        _ => None,
    }
}

pub(crate) fn resolve_legacy_ascii_case(
    runtime_directory: &Path,
    requested_name: &str,
) -> Result<PathBuf, io::Error> {
    let requested_path = runtime_directory.join(requested_name);
    match fs::metadata(&requested_path) {
        Ok(metadata) if metadata.is_file() => return Ok(requested_path),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    for entry in fs::read_dir(runtime_directory)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.eq_ignore_ascii_case(requested_name) && entry.file_type()?.is_file() {
            return Ok(entry.path());
        }
    }
    Ok(requested_path)
}

fn required_setup<T: Copy>(
    value: Option<T>,
    field: &'static str,
) -> Result<T, LoginNetworkInitializationError> {
    value.ok_or(LoginNetworkInitializationError::MissingSetupField(field))
}

impl Default for LoginSetup {
    fn default() -> Self {
        Self {
            server_version: None,
            listen_port_client: None,
            listen_port_world: None,
            sql_connection_type: Vec::new(),
            sql_server_ip: Vec::new(),
            sql_user_name: Vec::new(),
            sql_password: Vec::new(),
            database_name: Vec::new(),
            client_check_net: Some(true),
            client_max_byte_num: Some(5_000),
            client_max_message_len: Some(0x19_000),
            client_ban_ip_time: Some(10),
            client_check_message_content: Some(true),
            client_max_connections: None,
            client_max_io_send: None,
            client_max_send_buffer: None,
            world_check_net: None,
            world_max_byte_num: None,
            world_max_message_len: None,
            world_ban_ip_time: None,
            world_check_message_content: None,
            world_max_connections: None,
            world_max_io_send: None,
            world_max_send_buffer: None,
            refresh_info_time: Some(1_000),
            save_info_time: Some(60_000),
            do_queue_interval: None,
            send_to_queue_interval: None,
            world_max_players: None,
            world_busy_scale: None,
            world_full_scale: None,
            ping_world_time: None,
            ping_world_error_time: None,
            check_forbidden_ip: None,
            check_allowed_ip: None,
            check_between_ip: None,
            server_info_log_time: None,
            server_info_log_provider: Vec::new(),
            server_info_log_user: Vec::new(),
            server_info_log_password: Vec::new(),
            server_info_log_ip: Vec::new(),
            server_info_log_database: Vec::new(),
            auth_timeout: Some(5_000),
            bind_ip: b"0.0.0.0".to_vec(),
            bind_port: Some(0),
            forbidden_account_time: None,
            forbidden_password_errors: None,
            mode: Some(0),
            gas_area_user: Vec::new(),
            gas_area_password: Vec::new(),
            billing_database_ip: Vec::new(),
            billing_database_name: Vec::new(),
            billing_database_user: Vec::new(),
            billing_database_password: Vec::new(),
            inside_use: Some(1),
            verification_address: Vec::new(),
            verification_sign_upper: None,
        }
    }
}

struct SetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    attempted_pairs: usize,
    parsed_pairs: usize,
}

impl<'a> SetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            attempted_pairs: 0,
            parsed_pairs: 0,
        }
    }

    fn next_value(&mut self) -> Option<&'a [u8]> {
        self.attempted_pairs += 1;
        let _label = self.tokens.get(self.next)?;
        let value = self.tokens.get(self.next + 1).copied()?;
        self.next += 2;
        Some(value)
    }

    fn parsed(&mut self) {
        self.parsed_pairs += 1;
    }

    const fn outcome(&self) -> (usize, Option<usize>) {
        let stopped = if self.parsed_pairs < self.attempted_pairs {
            Some(self.attempted_pairs)
        } else {
            None
        };
        (self.parsed_pairs, stopped)
    }
}

/// Восстановленный `CGame` LoginServer с owned lifecycle и network cadence.
pub(crate) struct CGame {
    instance_guard: Option<UnixListener>,
    auth_servers: Vec<AuthServerConfig>,
    current_auth_server: Option<AuthServerConfig>,
    auth_client: Option<CMyNetClientAuth>,
    reconnect_task: Option<JoinHandle<()>>,
    listen_ports: LoginListenPorts,
    world_setup_routes: BTreeMap<i32, WorldSetupEntry>,
    world_routes: BTreeMap<i32, WorldRoute>,
    ping_world_server_info: Vec<PingWorldServerInfo>,
    client_server: Option<CMyNetServerClient>,
    world_server: Option<CMyNetServerWorld>,
    world_sender: Option<ServerCommandHandle>,
    client_sender: Option<ServerCommandHandle>,
    world_accept_task: Option<JoinHandle<io::Result<(TcpStream, SocketAddrV4)>>>,
    client_accept_task: Option<JoinHandle<io::Result<(TcpStream, SocketAddrV4)>>>,
    world_io_tasks: JoinSet<ServerIoCompletion>,
    client_io_tasks: JoinSet<ServerIoCompletion>,
    next_world_accept_at: Instant,
    next_client_accept_at: Instant,
    login_queue: Option<Arc<CLoginQueue>>,
    gas_thread: Option<CGasThread>,
    account_log_queue: Arc<AccLogQueue>,
    account_log_thread: Option<AccLogThread>,
    setup: LoginSetup,
    setup_ex: LoginSetupEx,
    state_levels: [i32; 4],
    login_cdkey_world: BTreeMap<Vec<u8>, Vec<u8>>,
    world_cdkeys: BTreeMap<i32, Vec<Vec<u8>>>,
    password_errors: BTreeMap<Vec<u8>, i32>,
    password_ban_minutes: i32,
    password_error_limit: i32,
    rs_cdkey_owner: Option<Box<dyn RsCdKeyOwner>>,
    online_user_database_settings: Option<LoginDatabaseSettings>,
    online_user_update_tasks: Vec<OnlineUserUpdateTask>,
    online_user_database_reports: VecDeque<OnlineUserDatabaseReport>,
    server_info_database_settings: Option<LoginDatabaseSettings>,
    server_info_log_shared: Arc<ServerInfoLogShared>,
    server_info_log_tasks: Vec<ServerInfoLogTask>,
    server_info_database_reports: VecDeque<ServerInfoDatabaseReport>,
    online_cdkey_count: Arc<AtomicU32>,
    auth_handler_notices: VecDeque<AuthHandlerNotice>,
    world_operator_log_records: VecDeque<WorldOperatorLogRecord>,
    main_loop_state: LoginMainLoopState,
    exit_requested: bool,
}

impl CGame {
    /// Создаёт единственный owned `CGame` до чтения setup в исходном `Init`.
    pub(crate) fn new(area_id: i32) -> Self {
        Self {
            instance_guard: None,
            auth_servers: Vec::new(),
            current_auth_server: None,
            auth_client: None,
            reconnect_task: None,
            listen_ports: LoginListenPorts::default(),
            world_setup_routes: BTreeMap::new(),
            world_routes: BTreeMap::new(),
            ping_world_server_info: Vec::new(),
            client_server: None,
            world_server: None,
            world_sender: None,
            client_sender: None,
            world_accept_task: None,
            client_accept_task: None,
            world_io_tasks: JoinSet::new(),
            client_io_tasks: JoinSet::new(),
            next_world_accept_at: Instant::now(),
            next_client_accept_at: Instant::now(),
            login_queue: None,
            gas_thread: None,
            account_log_queue: Arc::new(AccLogQueue::new()),
            account_log_thread: None,
            setup: LoginSetup::default(),
            setup_ex: LoginSetupEx::new(area_id),
            state_levels: [-1, 10_000, 20_000, 30_000],
            login_cdkey_world: BTreeMap::new(),
            world_cdkeys: BTreeMap::new(),
            password_errors: BTreeMap::new(),
            password_ban_minutes: 0,
            password_error_limit: 0,
            rs_cdkey_owner: None,
            online_user_database_settings: None,
            online_user_update_tasks: Vec::new(),
            online_user_database_reports: VecDeque::new(),
            server_info_database_settings: None,
            server_info_log_shared: Arc::new(ServerInfoLogShared::new()),
            server_info_log_tasks: Vec::new(),
            server_info_database_reports: VecDeque::new(),
            online_cdkey_count: Arc::new(AtomicU32::new(0)),
            auth_handler_notices: VecDeque::new(),
            world_operator_log_records: VecDeque::new(),
            main_loop_state: LoginMainLoopState::default(),
            exit_requested: false,
        }
    }

    /// Очищает прежний список и читает whitespace-пары `host + u16 port`.
    ///
    /// Ошибка открытия возвращается после очистки, как у оригинала. Ошибка
    /// очередной пары только завершает чтение; ранее прочитанные entries и
    /// общий успешный результат файла сохраняются.
    pub(crate) fn load_as_list(&mut self, path: &Path) -> Result<LoadAsListOutcome, io::Error> {
        self.auth_servers.clear();
        let bytes = fs::read(path)?;
        let mut tokens = bytes.split(|byte| byte.is_ascii_whitespace());
        let mut stopped_on_invalid_pair = false;

        while let Some(host) = tokens.find(|token| !token.is_empty()) {
            let Some(port) = tokens.find(|token| !token.is_empty()) else {
                stopped_on_invalid_pair = true;
                break;
            };
            let parsed_port = std::str::from_utf8(port)
                .ok()
                .and_then(|port| port.parse::<u16>().ok());
            let Some(port) = parsed_port else {
                stopped_on_invalid_pair = true;
                break;
            };
            self.auth_servers
                .push(AuthServerConfig::new(host.to_vec(), port));
        }

        Ok(LoadAsListOutcome {
            loaded: self.auth_servers.len(),
            stopped_on_invalid_pair,
        })
    }

    /// Создаёт Auth client, последовательно пробует список и всегда ставит
    /// LoginServer-info, сохраняя исходный безусловный успех функции.
    pub(crate) async fn init_auth_client(&mut self) -> AuthInitializationReport {
        self.stop_reconnect_task().await;
        self.auth_client = None;
        let (mut client, attempts, connected) = self.connect_new_auth_client().await;
        client.enable_control_send();
        self.current_auth_server = connected.clone();
        self.auth_client = Some(client);
        let login_server_info = self.send_login_server_info_to_auth();

        AuthInitializationReport {
            legacy_result: true,
            attempts,
            connected,
            login_server_info,
        }
    }

    /// Сообщает буквальный `mASClient != nullptr && IsConnect()`.
    pub(crate) fn is_connect_as(&self) -> bool {
        self.auth_client
            .as_ref()
            .is_some_and(CMyNetClientAuth::is_connected)
    }

    /// Выполняет явный close текущего Auth client без synthetic `0xCF301`.
    pub(crate) fn disconnect_as(&mut self) {
        if let Some(client) = self.auth_client.as_mut() {
            let _legacy_result = client.close();
        }
    }

    /// Пробует новый client по тому же списку и публикует typed reconnect в
    /// FIFO старого клиента; фактическая замена произойдёт при обработке FIFO.
    pub(crate) async fn reconnect_as(&mut self) -> Result<AuthReconnectReport, AuthLifecycleError> {
        if self.auth_client.is_none() {
            return Err(AuthLifecycleError::MissingCurrentClient);
        }
        let (client, attempts, connected) = self.connect_new_auth_client().await;
        if connected.is_some() {
            self.auth_client
                .as_ref()
                .expect("наличие старого Auth client проверено")
                .publish_reconnected(client);
            Ok(AuthReconnectReport {
                connected: true,
                attempts,
            })
        } else {
            Ok(AuthReconnectReport {
                connected: false,
                attempts,
            })
        }
    }

    /// Останавливает прежний reconnect-owner и запускает немедленную попытку.
    ///
    /// После неуспеха задача ждёт ровно исходные пять секунд. Успешный client
    /// передаётся в FIFO прежнего клиента типизированным replacement-событием;
    /// фактический `ReassignAS` остаётся обычной обработкой этой FIFO.
    pub(crate) async fn start_reconnect_thread(&mut self) -> Result<(), AuthLifecycleError> {
        let publisher = self
            .auth_client
            .as_ref()
            .ok_or(AuthLifecycleError::MissingCurrentClient)?
            .event_publisher();
        self.stop_reconnect_task().await;

        let plan = self.reconnect_plan();
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| AuthLifecycleError::RuntimeUnavailable)?;
        self.reconnect_task = Some(runtime.spawn(run_reconnect_task(plan, publisher)));
        Ok(())
    }

    /// Обрабатывает одно Auth FIFO-событие без преждевременного доменного Run.
    pub(crate) fn process_next_auth_event(
        &mut self,
    ) -> Result<Option<AuthGameEvent>, AuthLifecycleError> {
        let event = self
            .auth_client
            .as_ref()
            .and_then(CMyNetClientAuth::pop_event);
        match event {
            None => Ok(None),
            Some(AuthClientEvent::Message(message)) => Ok(Some(AuthGameEvent::Message(message))),
            Some(AuthClientEvent::Reconnected(client)) => {
                let (endpoint, login_server_info) = self.reassign_as(client)?;
                Ok(Some(AuthGameEvent::Reassigned {
                    endpoint,
                    login_server_info,
                }))
            }
        }
    }

    /// Выполняет один awaitable read/send шаг текущего Auth client.
    pub(crate) async fn run_auth_io_once(&mut self) -> Result<AuthClientIoStep, AuthClientIoError> {
        let client = self
            .auth_client
            .as_mut()
            .ok_or(AuthClientIoError::NotConnected)?;
        client.run_io_once().await
    }

    /// Возвращает send-очередь текущего Auth client доменным producers.
    pub(crate) fn auth_send_queue(&self) -> Option<&ClientSendQueue> {
        self.auth_client.as_ref().map(CMyNetClientAuth::send_queue)
    }

    /// Возвращает resolved endpoint текущего назначенного AuthServer.
    pub(crate) fn current_auth_server(&self) -> Option<&AuthServerConfig> {
        self.current_auth_server.as_ref()
    }

    /// Возвращает число событий в текущей Auth FIFO.
    pub(crate) fn pending_auth_events(&self) -> i32 {
        self.auth_client
            .as_ref()
            .map_or(0, CMyNetClientAuth::pending_events)
    }

    /// Обрабатывает один исходный snapshot трёх входных FIFO.
    ///
    /// Runner вызывается немедленно в позиции старого virtual `Run`; сообщения,
    /// добавленные в текущую очередь во время callback, остаются следующему
    /// проходу. Typed Auth reconnect выполняется в том же snapshot без вызова
    /// runner, потому что он заменяет внутрипроцессный pointer, а не `CMessage`.
    pub(crate) async fn process_message<Runner, HandlerError>(
        &mut self,
        mut run_message: Runner,
    ) -> Result<i32, ProcessMessageError<HandlerError>>
    where
        Runner: for<'a> AsyncFnMut(&'a mut CGame, &'a mut CMessage) -> Result<(), HandlerError>,
    {
        let mut remaining = self
            .world_server
            .as_ref()
            .map_or(0, CMyNetServerWorld::pending_messages);
        while remaining > 0 {
            let message = self
                .world_server
                .as_ref()
                .and_then(CMyNetServerWorld::pop_received_message);
            if let Some(mut message) = message {
                run_message(self, &mut message)
                    .await
                    .map_err(ProcessMessageError::Handler)?;
            }
            remaining -= 1;
        }

        remaining = self
            .client_server
            .as_ref()
            .map_or(0, CMyNetServerClient::pending_messages);
        while remaining > 0 {
            let message = self
                .client_server
                .as_ref()
                .and_then(CMyNetServerClient::pop_received_message);
            if let Some(mut message) = message {
                run_message(self, &mut message)
                    .await
                    .map_err(ProcessMessageError::Handler)?;
            }
            remaining -= 1;
        }

        remaining = self.pending_auth_events();
        while remaining > 0 {
            let event = self
                .process_next_auth_event()
                .map_err(ProcessMessageError::AuthLifecycle)?;
            if let Some(AuthGameEvent::Message(mut message)) = event {
                run_message(self, &mut message)
                    .await
                    .map_err(ProcessMessageError::Handler)?;
            }
            remaining -= 1;
        }

        Ok(1)
    }

    /// Выполняет один достигнутый turn исходного `CGame::MainLoop`.
    ///
    /// При ошибке message-handler уже выполненные более ранние side effects не
    /// откатываются. Server-info telemetry копируется неблокирующе, а SQL-
    /// consumer запускается отдельным сериализованным owned worker.
    pub(crate) async fn main_loop_turn(
        &mut self,
        auth_manager: &mut AuthManager,
    ) -> Result<LoginMainLoopOutcome, LoginMainLoopError> {
        let initial_now = match self.main_loop_state.now_ms {
            Some(now) => now,
            None => {
                let now = legacy_tick_ms();
                self.main_loop_state.now_ms = Some(now);
                now
            }
        };
        self.main_loop_state
            .last_refresh_info_ms
            .get_or_insert(initial_now);

        let now = legacy_tick_ms();
        self.main_loop_state.now_ms = Some(now);
        self.main_loop_state
            .last_server_info_log_ms
            .get_or_insert(now);

        let refresh_interval = self
            .setup
            .refresh_info_time
            .ok_or(LoginMainLoopError::MissingSetupField("dwRefeashInfoTime"))?;
        let last_refresh = self
            .main_loop_state
            .last_refresh_info_ms
            .expect("MainLoop только что инициализировал refresh cadence");
        let refresh_info_due = refresh_interval < now.wrapping_sub(last_refresh);
        if refresh_info_due {
            self.main_loop_state.last_refresh_info_ms = Some(now);
        }

        if self.exit_requested {
            return Ok(LoginMainLoopOutcome::Exit { refresh_info_due });
        }

        let gas_events = self
            .gas_thread
            .as_mut()
            .map(CGasThread::drain_events)
            .unwrap_or_default();
        let gas = gas_events
            .into_iter()
            .map(|event| apply_worker_event(self, event))
            .collect();
        let account_logs = self
            .account_log_thread
            .as_mut()
            .map(AccLogThread::drain_notices)
            .unwrap_or_default();
        let messages = process_login_messages(self, auth_manager).await?;
        let login_queue = self.run_login_queue(auth_manager);

        self.main_loop_state.last_ping_world_ms.get_or_insert(now);
        let mut ping_events = Vec::new();
        if !self.main_loop_state.ping_in_flight {
            let ping_interval =
                self.setup
                    .ping_world_time
                    .ok_or(LoginMainLoopError::MissingSetupField(
                        "dwPingWorldServerTime",
                    ))?;
            let last_ping = self
                .main_loop_state
                .last_ping_world_ms
                .expect("MainLoop только что инициализировал ping cadence");
            if ping_interval < now.wrapping_sub(last_ping) {
                self.main_loop_state.last_ping_world_ms = Some(now);
                self.main_loop_state.ping_in_flight = true;
                let cleared_responses = self.ping_world_server_info.len();
                self.ping_world_server_info.clear();
                let request = CMessage::new(WORLD_PING_REQUEST_MESSAGE_TYPE);
                let send_result = self.send_all_world(&request);
                ping_events.push(LoginPingWorldEvent::Requested {
                    cleared_responses,
                    send_result,
                });
            }
        } else {
            let expected_worlds = self.world_routes.len();
            if expected_worlds <= self.ping_world_server_info.len() {
                self.main_loop_state.ping_in_flight = false;
                ping_events.push(LoginPingWorldEvent::Completed {
                    expected_worlds,
                    responses: self.ping_world_server_info.clone(),
                });
            }

            let error_interval =
                self.setup
                    .ping_world_error_time
                    .ok_or(LoginMainLoopError::MissingSetupField(
                        "dwPingWorldServerErrorTime",
                    ))?;
            let last_ping = self
                .main_loop_state
                .last_ping_world_ms
                .expect("активный ping уже имеет исходный start tick");
            if error_interval < now.wrapping_sub(last_ping) {
                self.main_loop_state.ping_in_flight = false;
                ping_events.push(LoginPingWorldEvent::TimedOut {
                    expected_worlds,
                    responses: self.ping_world_server_info.clone(),
                });
            }
        }

        self.collect_finished_server_info_logs();
        let server_info_interval = self
            .setup
            .server_info_log_time
            .ok_or(LoginMainLoopError::MissingSetupField("dwServerInfoLogTime"))?;
        let server_info = if server_info_interval == 0 {
            LoginServerInfoTurn::Disabled
        } else {
            let last_server_info = self
                .main_loop_state
                .last_server_info_log_ms
                .expect("MainLoop только что инициализировал server-info cadence");
            if server_info_interval <= now.wrapping_sub(last_server_info) {
                self.main_loop_state.last_server_info_log_ms = Some(now);
                let copied = self
                    .server_info_log_shared
                    .append_without_waiting(&self.ping_world_server_info);
                let append_acquired_lock = copied.is_some();
                let copied_telemetry_records = copied.unwrap_or(0);
                let staged_telemetry_records = self.server_info_log_shared.staged_world_records();
                let queued_log_records = self.server_info_log_shared.logs.size();
                if staged_telemetry_records == 0 && queued_log_records == 0 {
                    LoginServerInfoTurn::DueWithoutData
                } else {
                    let login_ip = self
                        .world_server
                        .as_ref()
                        .ok_or(LoginMainLoopError::MissingWorldServerForServerInfo)?
                        .local_ip()
                        .to_vec();
                    let settings = self
                        .server_info_database_settings
                        .clone()
                        .ok_or(LoginMainLoopError::MissingServerInfoDatabaseSettings)?;
                    match self.start_server_info_log_worker(settings, login_ip) {
                        Ok(()) => LoginServerInfoTurn::WorkerStarted {
                            append_acquired_lock,
                            copied_telemetry_records,
                            staged_telemetry_records,
                            queued_log_records,
                        },
                        Err(error) => LoginServerInfoTurn::WorkerStartFailed {
                            append_acquired_lock,
                            copied_telemetry_records,
                            staged_telemetry_records,
                            queued_log_records,
                            error,
                        },
                    }
                }
            } else {
                LoginServerInfoTurn::NotDue
            }
        };

        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(LoginMainLoopOutcome::Continue {
            refresh_info_due,
            gas,
            account_logs,
            messages,
            login_queue: Box::new(login_queue),
            ping_events,
            server_info,
        })
    }

    /// Устанавливает исходный `m_bExit` для следующего `MainLoop` turn.
    pub(crate) fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    /// Выполняет один concurrent network cadence перед доменным `MainLoop`.
    ///
    /// World и Client snapshots идут в порядке будущего `ProcessMessage`.
    /// Auth read/send polling выполняется один раз без ожидания readiness:
    /// старые направления обслуживались независимыми threads, поэтому между
    /// ними не вводится новый блокирующий порядок.
    pub(crate) async fn run_network_turn(
        &mut self,
    ) -> Result<LoginNetworkTurn, LoginNetworkRuntimeError> {
        let mut turn = LoginNetworkTurn::default();
        self.run_world_network_turn(&mut turn.world).await?;
        self.run_client_network_turn(&mut turn.client).await?;
        turn.auth = match self.auth_client.as_mut() {
            Some(client) if client.is_connected() => poll_once(client.run_io_once()).await,
            Some(_) | None => None,
        };
        Ok(turn)
    }

    async fn run_world_network_turn(
        &mut self,
        turn: &mut LoginWorldNetworkTurn,
    ) -> Result<(), LoginNetworkRuntimeError> {
        if self.world_server.is_none() {
            return Err(LoginNetworkRuntimeError::MissingNetworkOwner(
                LoginNetworkDirection::World,
            ));
        }
        if self
            .world_accept_task
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            let result = self
                .world_accept_task
                .take()
                .expect("завершившаяся World accept-задача уже проверена")
                .await
                .map_err(|error| LoginNetworkRuntimeError::Task {
                    direction: LoginNetworkDirection::World,
                    error,
                })?;
            self.next_world_accept_at = Instant::now() + ACCEPT_THREAD_DELAY;
            match result {
                Ok((stream, peer)) => turn.admissions.push(
                    self.world_server
                        .as_mut()
                        .expect("World owner проверен в начале turn")
                        .queue_accepted(stream, peer, legacy_tick_ms()),
                ),
                Err(error) => turn.accept_errors.push(error),
            }
        }
        drain_io_tasks(
            &mut self.world_io_tasks,
            LoginNetworkDirection::World,
            &mut turn.io_completions,
        )?;
        if self.world_accept_task.is_none() && Instant::now() >= self.next_world_accept_at {
            match self
                .world_server
                .as_ref()
                .expect("World owner проверен в начале turn")
                .begin_accept()
            {
                AcceptStart::NotListening => {
                    return Err(LoginNetworkRuntimeError::MissingNetworkOwner(
                        LoginNetworkDirection::World,
                    ));
                }
                AcceptStart::AtCapacity => {
                    self.next_world_accept_at = Instant::now() + ACCEPT_AT_CAPACITY_DELAY;
                }
                AcceptStart::Pending(accept) => {
                    self.world_accept_task = Some(tokio::spawn(accept.accept()));
                }
            }
        }
        let (snapshot, commands) = {
            let network = self
                .world_server
                .as_mut()
                .expect("World owner проверен в начале turn");
            let snapshot = network.process_network_snapshot(legacy_tick_ms());
            (snapshot, network.command_handle())
        };
        turn.processed_commands = snapshot.processed_commands();
        let (actions, errors) = snapshot.into_parts();
        turn.snapshot_errors = errors;
        spawn_io_actions(&mut self.world_io_tasks, actions, commands);
        Ok(())
    }

    async fn run_client_network_turn(
        &mut self,
        turn: &mut LoginClientNetworkTurn,
    ) -> Result<(), LoginNetworkRuntimeError> {
        if self.client_server.is_none() {
            return Err(LoginNetworkRuntimeError::MissingNetworkOwner(
                LoginNetworkDirection::Client,
            ));
        }
        if self
            .client_accept_task
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            let result = self
                .client_accept_task
                .take()
                .expect("завершившаяся Client accept-задача уже проверена")
                .await
                .map_err(|error| LoginNetworkRuntimeError::Task {
                    direction: LoginNetworkDirection::Client,
                    error,
                })?;
            self.next_client_accept_at = Instant::now() + ACCEPT_THREAD_DELAY;
            match result {
                Ok((stream, peer)) => turn.admissions.push(
                    self.client_server
                        .as_mut()
                        .expect("Client owner проверен в начале turn")
                        .queue_accepted(stream, peer, legacy_tick_ms()),
                ),
                Err(error) => turn.accept_errors.push(error),
            }
        }
        drain_io_tasks(
            &mut self.client_io_tasks,
            LoginNetworkDirection::Client,
            &mut turn.io_completions,
        )?;
        if self.client_accept_task.is_none() && Instant::now() >= self.next_client_accept_at {
            match self
                .client_server
                .as_ref()
                .expect("Client owner проверен в начале turn")
                .begin_accept()
            {
                AcceptStart::NotListening => {
                    return Err(LoginNetworkRuntimeError::MissingNetworkOwner(
                        LoginNetworkDirection::Client,
                    ));
                }
                AcceptStart::AtCapacity => {
                    self.next_client_accept_at = Instant::now() + ACCEPT_AT_CAPACITY_DELAY;
                }
                AcceptStart::Pending(accept) => {
                    self.client_accept_task = Some(tokio::spawn(accept.accept()));
                }
            }
        }
        let (snapshot, commands) = {
            let network = self
                .client_server
                .as_mut()
                .expect("Client owner проверен в начале turn");
            let snapshot = network.process_network_snapshot(legacy_tick_ms());
            (snapshot, network.command_handle())
        };
        turn.processed_commands = snapshot.processed_commands();
        let (actions, errors) = snapshot.into_parts();
        turn.snapshot_errors = errors;
        spawn_io_actions(&mut self.client_io_tasks, actions, commands);
        Ok(())
    }

    async fn release_world_network(&mut self) -> (u32, Vec<JoinError>) {
        let mut task_errors = stop_accept_task(&mut self.world_accept_task).await;
        if let Some(network) = &self.world_server {
            network.command_handle().quit_all();
        }
        let mut snapshots = 0_u32;
        while self
            .world_server
            .as_ref()
            .is_some_and(CMyNetServerWorld::has_clients)
        {
            let (snapshot, commands) = {
                let network = self
                    .world_server
                    .as_mut()
                    .expect("World shutdown проверил наличие owner");
                let snapshot = network.process_network_snapshot(legacy_tick_ms());
                (snapshot, network.command_handle())
            };
            snapshots = snapshots.wrapping_add(1);
            let (actions, _errors) = snapshot.into_parts();
            spawn_io_actions(&mut self.world_io_tasks, actions, commands);
            collect_io_task_errors(&mut self.world_io_tasks, &mut task_errors);
            tokio::time::sleep(ACCEPT_THREAD_DELAY).await;
        }
        self.world_io_tasks.shutdown().await;
        (snapshots, task_errors)
    }

    async fn release_client_network(&mut self) -> (u32, Vec<JoinError>) {
        let mut task_errors = stop_accept_task(&mut self.client_accept_task).await;
        if let Some(network) = &self.client_server {
            network.command_handle().quit_all();
        }
        let mut snapshots = 0_u32;
        while self
            .client_server
            .as_ref()
            .is_some_and(CMyNetServerClient::has_clients)
        {
            let (snapshot, commands) = {
                let network = self
                    .client_server
                    .as_mut()
                    .expect("Client shutdown проверил наличие owner");
                let snapshot = network.process_network_snapshot(legacy_tick_ms());
                (snapshot, network.command_handle())
            };
            snapshots = snapshots.wrapping_add(1);
            let (actions, _errors) = snapshot.into_parts();
            spawn_io_actions(&mut self.client_io_tasks, actions, commands);
            collect_io_task_errors(&mut self.client_io_tasks, &mut task_errors);
            tokio::time::sleep(ACCEPT_THREAD_DELAY).await;
        }
        self.client_io_tasks.shutdown().await;
        (snapshots, task_errors)
    }

    /// Выполняет полный обратный порядок исходного `CGame::Release`.
    ///
    /// Все исходные ошибки cleanup игнорировались и функция всегда возвращала
    /// `1`; Rust сохраняет их в отчёте. Если `m_bExit` уже установлен, обе
    /// operator-позиции остаются, но владельцы повторно не затрагиваются.
    pub(crate) async fn release(&mut self) -> LoginReleaseReport {
        if self.exit_requested {
            return LoginReleaseReport::already_released();
        }
        self.exit_requested = true;

        let world_network_was_present = self.world_server.is_some();
        let (world_network_shutdown_snapshots, world_network_task_errors) =
            self.release_world_network().await;
        let client_network_was_present = self.client_server.is_some();
        let (client_network_shutdown_snapshots, client_network_task_errors) =
            self.release_client_network().await;

        // Managed reconnect заменяет ещё один небезопасный внешний thread,
        // способный обратиться к mASClient после его исходной позиции delete.
        self.stop_reconnect_task().await;
        let auth_client_was_present = self.auth_client.is_some();
        self.disconnect_as();
        self.auth_client.take();
        self.current_auth_server = None;

        self.account_log_queue.clear();
        let account_log_thread_was_present = self.account_log_thread.is_some();
        self.account_log_thread.take();

        let gas_thread_was_present = self.gas_thread.is_some();
        self.gas_thread.take();

        self.client_sender = None;
        self.client_server.take();
        self.world_sender = None;
        self.world_server.take();

        // Tokio sockets уже владеют platform cleanup, поэтому отдельный
        // CMySocket::MySocketClearUp здесь не имеет живого Rust-эффекта.
        let database_owner_was_present = self.rs_cdkey_owner.take().is_some();
        let online_user_clear = clear_online_user_database(self.setup.database_settings()).await;
        self.online_user_database_settings = None;
        self.server_info_database_settings = None;

        // ADO/COM globals и CBaseMessage static buffers заменены Tiberius и
        // локальными buffers; не создаём пустых lifecycle-вызовов.
        let login_queue_was_present = self.login_queue.take().is_some();

        // Старые DB workers были detached. Rust ждёт их в позиции перед
        // уничтожением shared synchronization, не запрещая им конкурировать с
        // уже выполненным выше ClearOnlineUserDatabase.
        let online_user_workers_joined = self.join_all_online_user_updates();
        let server_info_workers_joined = self.join_all_server_info_logs();

        LoginReleaseReport {
            legacy_result: true,
            performed: true,
            operator_notices: [
                LoginReleaseOperatorNotice::Exiting,
                LoginReleaseOperatorNotice::Exited,
            ],
            world_network_was_present,
            client_network_was_present,
            world_network_shutdown_snapshots,
            client_network_shutdown_snapshots,
            world_network_task_errors,
            client_network_task_errors,
            auth_client_was_present,
            account_log_thread_was_present,
            gas_thread_was_present,
            database_owner_was_present,
            online_user_clear: Some(online_user_clear),
            login_queue_was_present,
            online_user_workers_joined,
            server_info_workers_joined,
        }
    }

    /// Выдаёт cadence-owner право публикации в FIFO текущего Auth-клиента.
    pub(crate) fn auth_event_publisher(
        &self,
    ) -> Result<AuthClientEventPublisher, AuthLifecycleError> {
        self.auth_client
            .as_ref()
            .map(CMyNetClientAuth::event_publisher)
            .ok_or(AuthLifecycleError::MissingCurrentClient)
    }

    /// Возвращает исходный `m_SetupEx.iAreaId` для GMA server-info запроса.
    pub(crate) const fn area_id(&self) -> i32 {
        self.setup_ex.area_id
    }

    /// Пересоздаёт Client listener и применяет исходные setup-группы.
    ///
    /// Ошибка `Host` сохраняет новый, но не слушающий owner. Частично
    /// прочитанный setup без обязательного поля блокируется до замены старого
    /// owner: безопасный Rust не воспроизводит чтение uninitialized памяти.
    pub(crate) fn init_net_server_client(&mut self) -> Result<(), LoginNetworkInitializationError> {
        let port = required_setup(self.listen_ports.client, "_listen_port.dwListenPort_Client")?;
        let config = self
            .setup
            .client_network_config()
            .map_err(LoginNetworkInitializationError::MissingSetupField)?;
        let setup_ex = self.setup_ex;

        drop(self.client_server.take());
        self.client_sender = None;
        let server = CMyNetServerClient::new(legacy_tick_ms());
        self.client_sender = Some(server.command_handle());
        self.client_server = Some(server);

        let server = self
            .client_server
            .as_mut()
            .expect("Client owner только что сохранён");
        server
            .host(port, None, DEFAULT_SOCKET_TYPE, true)
            .map_err(LoginNetworkInitializationError::Host)?;
        if let Some(address) = resolve_first_local_ipv4() {
            let dotted = address.to_string();
            server.set_local_identity(dotted.as_bytes(), legacy_ipv4_word(address));
        }
        server.configure_receive(
            config.check_receive_rate,
            config.check_content_crc,
            config.maximum_bytes_per_second,
            config.forbid_time_ms,
            config.maximum_message_length,
        );
        server.configure_connection_limits(
            config.maximum_clients,
            config.maximum_in_flight_sends,
            config.permitted_send_bytes,
        );
        server.configure_accept_limits_after_host(
            setup_ex.client_max_block_connections,
            setup_ex.client_valid_receive_delay_ms,
        );
        Ok(())
    }

    /// Пересоздаёт World listener и применяет исходные setup-группы.
    ///
    /// Ошибка `Host` сохраняет новый, но не слушающий owner. Неиспользуемые
    /// `bWorldCheckMsgCon/dwWorldMaxMsgLen` не получают притворного состояния:
    /// World parser этой сборки их не читал.
    pub(crate) fn init_net_server_world(&mut self) -> Result<(), LoginNetworkInitializationError> {
        let port = required_setup(self.listen_ports.world, "_listen_port.dwListenPort_World")?;
        let config = self
            .setup
            .world_network_config()
            .map_err(LoginNetworkInitializationError::MissingSetupField)?;
        let setup_ex = self.setup_ex;

        drop(self.world_server.take());
        self.world_sender = None;
        let server = CMyNetServerWorld::new(legacy_tick_ms());
        self.world_sender = Some(server.command_handle());
        self.world_server = Some(server);

        let server = self
            .world_server
            .as_mut()
            .expect("World owner только что сохранён");
        server
            .host(port, None, DEFAULT_SOCKET_TYPE, true)
            .map_err(LoginNetworkInitializationError::Host)?;
        if let Some(address) = resolve_first_local_ipv4() {
            let dotted = address.to_string();
            server.set_local_identity(dotted.as_bytes(), legacy_ipv4_word(address));
        }
        server.configure_receive_guard(
            config.check_receive_rate,
            config.maximum_bytes_per_second,
            config.forbid_time_ms,
        );
        server.configure_connection_limits(
            config.maximum_clients,
            config.maximum_in_flight_sends,
            config.permitted_send_bytes,
        );
        server.configure_accept_limits_after_host(
            setup_ex.world_max_block_connections,
            setup_ex.world_valid_receive_delay_ms,
        );
        Ok(())
    }

    /// Присоединяет фактическую командную границу Client net-owner.
    pub(crate) fn attach_client_sender(&mut self, sender: ServerCommandHandle) {
        self.client_sender = Some(sender);
    }

    /// Присоединяет фактическую командную границу World net-owner.
    pub(crate) fn attach_world_sender(&mut self, sender: ServerCommandHandle) {
        self.world_sender = Some(sender);
    }

    /// Присоединяет единый DB-владелец достигнутых операций `CRsCDKey`.
    pub(crate) fn attach_rs_cdkey_owner(&mut self, owner: Box<dyn RsCdKeyOwner>) {
        self.rs_cdkey_owner = Some(owner);
    }

    /// Возвращает текущую owned-копию исходного `m_Setup`.
    pub(crate) const fn login_setup(&self) -> &LoginSetup {
        &self.setup
    }

    /// Возвращает текущую owned-копию исходного `m_SetupEx`.
    pub(crate) const fn login_setup_ex(&self) -> &LoginSetupEx {
        &self.setup_ex
    }

    /// Позиционно читает `setupex.ini`, игнорируя имена слева от значений.
    ///
    /// Ошибка открытия не меняет setup-ex. Успешное открытие остаётся успехом
    /// при последующем stream fail-state: уже извлечённые поля меняются, а
    /// текущие и последующие сохраняют прежние значения.
    pub(crate) fn load_setup_ex(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<LoginSetupExLoadReport, io::Error> {
        let bytes = fs::read(runtime_directory.join("setupex.ini"))?;
        Ok(self.setup_ex.parse_positional(&bytes))
    }

    /// Читает отдельный `port.ini` как две positional-пары `label + u32`.
    ///
    /// Ошибка открытия не меняет прежние порты. После успешного открытия
    /// malformed-хвост сохраняет уже извлечённые значения и общий legacy-успех;
    /// отчёт лишь показывает границу stream fail-state.
    pub(crate) fn load_listen_port(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<LoginListenPortLoadReport, io::Error> {
        let bytes = fs::read(runtime_directory.join("port.ini"))?;
        let mut tokens = bytes
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty());
        let mut parsed_pairs = 0;

        let Some(_client_label) = tokens.next() else {
            return Ok(LoginListenPortLoadReport {
                parsed_pairs,
                stopped_at_pair: Some(1),
            });
        };
        let Some(client) = tokens.next().and_then(parse_ascii::<u32>) else {
            return Ok(LoginListenPortLoadReport {
                parsed_pairs,
                stopped_at_pair: Some(1),
            });
        };
        self.listen_ports.client = Some(client);
        parsed_pairs += 1;

        let Some(_world_label) = tokens.next() else {
            return Ok(LoginListenPortLoadReport {
                parsed_pairs,
                stopped_at_pair: Some(2),
            });
        };
        let Some(world) = tokens.next().and_then(parse_ascii::<u32>) else {
            return Ok(LoginListenPortLoadReport {
                parsed_pairs,
                stopped_at_pair: Some(2),
            });
        };
        self.listen_ports.world = Some(world);
        parsed_pairs += 1;

        Ok(LoginListenPortLoadReport {
            parsed_pairs,
            stopped_at_pair: None,
        })
    }

    /// Очищает и читает `WorldInfoSetup.ini` до точного маркера `#`.
    ///
    /// Старый Windows lookup был нечувствителен к ASCII-регистру; Linux-owner
    /// сохраняет это только для имени файла. Повторный ID заменяет setup-запись,
    /// как `std::map::operator[]`; имя остаётся byte-exact.
    pub(crate) fn load_world_setup(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<LoginWorldSetupLoadReport, io::Error> {
        self.world_setup_routes.clear();
        let path = resolve_legacy_ascii_case(runtime_directory, "WorldInfoSetup.ini")?;
        let bytes = fs::read(path)?;
        let mut tokens = bytes
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty());
        let mut parsed_records = 0;
        let mut stopped_at_record = None;

        while read_to(&mut tokens, b"#") {
            let record = parsed_records + 1;
            let fields = (tokens.next(), tokens.next(), tokens.next());
            let (Some(world_id), Some(name), Some(configured_state)) = fields else {
                stopped_at_record = Some(record);
                break;
            };
            let (Some(world_id), Some(configured_state)) = (
                parse_ascii::<i32>(world_id),
                parse_ascii::<i32>(configured_state),
            ) else {
                // Старый extraction оставлял локальную переменную без
                // определённого значения; safe Rust не создаёт malformed route.
                stopped_at_record = Some(record);
                break;
            };
            self.world_setup_routes.insert(
                world_id,
                WorldSetupEntry {
                    name: name.to_vec(),
                    configured_state,
                },
            );
            parsed_records += 1;
        }

        Ok(LoginWorldSetupLoadReport {
            parsed_records,
            unique_worlds: self.world_setup_routes.len(),
            stopped_at_record,
        })
    }

    /// Перестраивает runtime `m_listWorldInfo` из setup-карты со state `0`.
    pub(crate) fn set_list_world_info_by_setup(&mut self) -> usize {
        self.world_routes.clear();
        for (&world_id, setup) in &self.world_setup_routes {
            self.world_routes
                .insert(world_id, WorldRoute::new(world_id, setup.name.clone(), 0));
        }
        self.world_routes.len()
    }

    /// Проверяет ненулевой setup-state независимо от runtime подключения World.
    pub(crate) fn world_server_is_open_state(&self, world_id: i32) -> bool {
        self.world_setup_routes
            .get(&world_id)
            .is_some_and(|setup| setup.configured_state != 0)
    }

    /// Выполняет доказанный префикс `CGame::Init` до запуска GAS/AccLog threads.
    ///
    /// Успешный результат не означает полный исходный `Init`: оба готовых
    /// worker-owner запускаются следующими двумя позициями [`Self::initialize`].
    /// При любой ошибке уже выполненные мутации и созданные owners намеренно
    /// сохраняются.
    pub(crate) async fn initialize_before_worker_threads(
        &mut self,
        runtime_directory: &Path,
        auth_manager: &mut AuthManager,
    ) -> Result<LoginInitializationPrefixReport, LoginInitializationError> {
        let setup = self
            .load_setup(runtime_directory)
            .map_err(LoginInitializationError::Setup)?;
        let listen_ports = self
            .load_listen_port(runtime_directory)
            .map_err(LoginInitializationError::ListenPort)?;
        self.acquire_single_instance_guard()
            .map_err(LoginInitializationError::SingleInstance)?;

        // Старый pointer появлялся после успешного FindWindowA-check. Rust
        // создаёт owner здесь; constructor в своей последней позиции читает
        // no-queue файл, но до этого места LoadSetup не вызывает OnInitial.
        let (login_queue, no_queue_accounts) = CLoginQueue::new(runtime_directory);
        self.login_queue = Some(Arc::new(login_queue));
        let setup_ex = self.load_setup_ex(runtime_directory);
        let world_setup = self
            .load_world_setup(runtime_directory)
            .map_err(LoginInitializationError::WorldSetup)?;

        let auth_timeout = self
            .setup
            .auth_timeout
            .ok_or(LoginInitializationError::MissingSetupField("authTimeOut"))?;
        let _legacy_auth_manager_result = auth_manager.init(auth_timeout);
        let auth_servers = self
            .load_as_list(&runtime_directory.join("aslist.ini"))
            .map_err(LoginInitializationError::AuthServerList)?;
        let world_routes = self.set_list_world_info_by_setup();
        self.initialize_login_queue_from_setup()
            .map_err(LoginInitializationError::MissingSetupField)?;
        let login_queue_initialized = true;

        let database_settings = self.setup.database_settings();
        let online_user_clear = clear_online_user_database(database_settings).await;
        self.initialize_rs_cdkey_from_setup()
            .map_err(LoginInitializationError::DatabaseOwner)?;

        // `CBaseMessage::Initial` и WinSock startup были техническими globals;
        // static dispatch/local buffers и Tokio transport уже выражают эффект.
        self.init_net_server_world()
            .map_err(LoginInitializationError::WorldNetwork)?;
        self.init_net_server_client()
            .map_err(LoginInitializationError::ClientNetwork)?;
        let auth = self.init_auth_client().await;
        self.initialize_server_info_database_from_setup();
        let startup_log = self.queue_login_started_server_info_log()?;

        Ok(LoginInitializationPrefixReport {
            setup,
            listen_ports,
            setup_ex,
            world_setup,
            auth_servers,
            world_routes,
            login_queue_initialized,
            no_queue_accounts,
            online_user_clear,
            auth,
            startup_log,
        })
    }

    /// Продолжает префикс `CGame::Init` ровно через создание `CGasThread`.
    ///
    /// Как оригинальный `Init`, эта ступень не делает ошибку запуска worker
    /// фатальной и возвращает её только в operator-visible отчёте.
    pub(crate) async fn initialize_through_gas_thread(
        &mut self,
        runtime_directory: &Path,
        auth_manager: &mut AuthManager,
    ) -> Result<LoginInitializationThroughGasReport, LoginInitializationError> {
        let prefix = self
            .initialize_before_worker_threads(runtime_directory, auth_manager)
            .await?;
        let gas_thread_start = self.start_gas_thread();
        Ok(LoginInitializationThroughGasReport {
            prefix,
            gas_thread_start,
        })
    }

    /// Выполняет полный доказанный `CGame::Init` до исходного `return 1`.
    ///
    /// Оба `Thread::Start` не влияли на возвращаемое значение оригинала;
    /// ошибки публикации Rust workers поэтому остаются только в отчёте.
    pub(crate) async fn initialize(
        &mut self,
        runtime_directory: &Path,
        auth_manager: &mut AuthManager,
    ) -> Result<LoginInitializationReport, LoginInitializationError> {
        let through_gas = self
            .initialize_through_gas_thread(runtime_directory, auth_manager)
            .await?;
        let account_log_thread_start = self.start_account_log_thread();
        Ok(LoginInitializationReport {
            through_gas,
            account_log_thread_start,
        })
    }

    fn start_gas_thread(&mut self) -> Result<(), io::Error> {
        if self.gas_thread.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "CGasThread уже принадлежит этому CGame",
            ));
        }
        let config = GasVerificationConfig {
            address: self.setup.verification_address.clone(),
            signature_uppercase: self.setup.verification_sign_upper,
        };
        let queue = self.login_queue.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "m_pLoginQueue ещё не создан в позиции CGame::Init",
            )
        })?;
        self.gas_thread = Some(CGasThread::start(Arc::clone(queue), config)?);
        Ok(())
    }

    fn start_account_log_thread(&mut self) -> Result<(), io::Error> {
        if self.account_log_thread.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "AccLogThread уже принадлежит этому CGame",
            ));
        }
        self.account_log_thread = Some(AccLogThread::start(
            Arc::clone(&self.account_log_queue),
            self.setup.database_settings(),
        )?);
        Ok(())
    }

    fn acquire_single_instance_guard(&mut self) -> Result<(), io::Error> {
        if self.instance_guard.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                "single-instance guard уже принадлежит этому CGame",
            ));
        }
        let client_port = self.listen_ports.client.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "port.ini не инициализировал Client port",
            )
        })?;
        let title = format!(
            "LoginServer[{}][{}]-FengYun",
            self.setup_ex.area_id, client_port
        );
        let address = UnixSocketAddr::from_abstract_name(title.as_bytes())?;
        self.instance_guard = Some(UnixListener::bind_addr(&address)?);
        Ok(())
    }

    fn queue_login_started_server_info_log(
        &self,
    ) -> Result<ServerInfoLogDisposition, LoginInitializationError> {
        match self.setup.server_info_log_time {
            Some(0) => Ok(ServerInfoLogDisposition::Disabled),
            Some(_) => {
                let world_server = self
                    .world_server
                    .as_ref()
                    .expect("startup log достигается после успешного World Host");
                let date = Local::now().format("%m/%d/%y").to_string();
                let time = Local::now().format("%H:%M:%S").to_string();
                let description = format!("LoingServer started on {date} {time}.").into_bytes();
                self.server_info_log_shared.logs.push(ServLog::new(
                    world_server.local_ipv4_word(),
                    -1,
                    0,
                    description,
                ));
                Ok(ServerInfoLogDisposition::Queued)
            }
            None => Err(LoginInitializationError::MissingSetupField(
                "dwServerInfoLogTime",
            )),
        }
    }

    /// Передаёт reload фиксированного файла фактическому `CLoginQueue` owner.
    ///
    /// Ошибка вложенного чтения не меняет исходный безусловный `return true`;
    /// она возвращается как operator-visible disposition. Вызов вне интервала
    /// между созданием queue в `Init` и `Release` является Rust-ошибкой.
    pub(crate) fn load_no_queue_cdkey_list(
        &self,
        runtime_directory: &Path,
    ) -> Result<LoginNoQueueAccountsReloadOutcome, LoginQueueOwnerMissing> {
        let queue = self.login_queue.as_ref().ok_or(LoginQueueOwnerMissing)?;
        let load = match queue.load_no_queue_cdkey_list(runtime_directory) {
            Ok(report) => LoginNoQueueAccountsLoadDisposition::Loaded(report),
            Err(error) => LoginNoQueueAccountsLoadDisposition::Failed(error),
        };
        Ok(LoginNoQueueAccountsReloadOutcome {
            legacy_result: true,
            load,
        })
    }

    /// Суммирует размеры всех `s_listCdkey` в numeric World-порядке.
    ///
    /// Исходный `long` накапливался без проверки; Rust сохраняет его 32-битное
    /// wrapping, включая усечение каждого MSVC `size_t` до наблюдаемого слова.
    pub(crate) fn get_login_world_cdkey_numbers(&self) -> i32 {
        self.world_cdkeys.values().fold(0_i32, |total, accounts| {
            total.wrapping_add(accounts.len() as u32 as i32)
        })
    }

    /// Перечитывает основной setup и применяет runtime-поля без restart listener.
    ///
    /// Ошибка открытия обоих файлов исходно игнорировалась: уже существующий
    /// setup всё равно повторно записывался в Client, затем World и AuthManager.
    /// Missing-field/owner возможен только вне доказанного post-Init вызова и
    /// блокирует ровно ещё не выполненную часть этого порядка.
    pub(crate) fn reload_setup(
        &mut self,
        runtime_directory: &Path,
        auth_manager: &mut AuthManager,
    ) -> Result<LoginSetupReloadOutcome, LoginSetupReloadError> {
        let load = self.load_setup(runtime_directory);

        let client = self
            .setup
            .client_network_config()
            .map_err(LoginSetupReloadError::SetupField)?;
        let client_server = self
            .client_server
            .as_mut()
            .ok_or(LoginSetupReloadError::ClientOwner)?;
        client_server.configure_receive(
            client.check_receive_rate,
            client.check_content_crc,
            client.maximum_bytes_per_second,
            client.forbid_time_ms,
            client.maximum_message_length,
        );
        client_server.configure_connection_limits(
            client.maximum_clients,
            client.maximum_in_flight_sends,
            client.permitted_send_bytes,
        );

        let world = self
            .setup
            .world_network_config()
            .map_err(LoginSetupReloadError::SetupField)?;
        let world_server = self
            .world_server
            .as_mut()
            .ok_or(LoginSetupReloadError::WorldOwner)?;
        world_server.configure_receive_guard(
            world.check_receive_rate,
            world.maximum_bytes_per_second,
            world.forbid_time_ms,
        );
        world_server.configure_connection_limits(
            world.maximum_clients,
            world.maximum_in_flight_sends,
            world.permitted_send_bytes,
        );
        // `bWorldCheckMsgCon/dwWorldMaxMsgLen` записывались в CServer, но этот
        // компонентный parser всегда проверяет content CRC и не читает предел.

        let auth_timeout = self
            .setup
            .auth_timeout
            .ok_or(LoginSetupReloadError::SetupField("authTimeOut"))?;
        auth_manager.set_timeout(auth_timeout);
        Ok(LoginSetupReloadOutcome {
            legacy_result: true,
            load,
        })
    }

    /// Перечитывает setup-карту миров и рассылает её текущим Login clients.
    ///
    /// Ошибка файла не останавливает рассылку: `LoadWorldSetup` уже очистил
    /// setup-map, а исходная `ReLoadWorldSetup` игнорировала его `bool`.
    /// MFC `UpdateDisplayWorldInfo` не имеет отдельного Linux runtime-эффекта.
    pub(crate) fn reload_world_setup(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<LoginWorldSetupReloadOutcome, LoginQueueOwnerMissing> {
        let load = self.load_world_setup(runtime_directory);
        if self.login_queue.is_none() {
            return Err(LoginQueueOwnerMissing);
        }
        let broadcast_failures = self.update_world_info_to_all_clients();
        Ok(LoginWorldSetupReloadOutcome {
            legacy_result: true,
            load,
            broadcast_failures,
        })
    }

    /// Перечитывает setup-ex и применяет четыре поздние сетевые записи.
    ///
    /// Отсутствующий network owner сообщает нарушение доказанного вызова после
    /// полного `Init`, не разыменовывая nullable raw pointer.
    pub(crate) fn reload_setup_ex(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<LoginSetupExLoadReport, LoginSetupExReloadError> {
        let report = self.load_setup_ex(runtime_directory)?;
        let setup_ex = self.setup_ex;
        let client_server = self
            .client_server
            .as_mut()
            .ok_or(LoginSetupExReloadError::MissingClientServer)?;
        client_server.configure_accept_limits_after_host(
            setup_ex.client_max_block_connections,
            setup_ex.client_valid_receive_delay_ms,
        );
        let world_server = self
            .world_server
            .as_mut()
            .ok_or(LoginSetupExReloadError::MissingWorldServer)?;
        world_server.configure_accept_limits_after_host(
            setup_ex.world_max_block_connections,
            setup_ex.world_valid_receive_delay_ms,
        );
        Ok(report)
    }

    /// Выполняет позиционный `LoadSetup`: сначала `setup.ini`, затем `setup.dat`.
    ///
    /// Успешное открытие остаётся успехом даже при последующем stream fail-state:
    /// уже извлечённые поля меняются, а текущие и последующие сохраняют прежнее
    /// значение. Имена слева от значений намеренно игнорируются. Метод не
    /// публикует содержимое setup-файла и не запускает ни один сервис.
    pub(crate) fn load_setup(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<LoginSetupLoadReport, LoginSetupOpenError> {
        let plain_path = runtime_directory.join("setup.ini");
        let encoded_path = runtime_directory.join("setup.dat");

        let (source, parsed_pairs, stopped_at_pair) = match fs::read(&plain_path) {
            Ok(bytes) => {
                let (parsed, stopped) = self.setup.parse_plain(&bytes);
                (LoginSetupSource::Plain, parsed, stopped)
            }
            Err(plain) => match fs::read(&encoded_path) {
                Ok(bytes) => {
                    let decoded = ini_decode(&bytes);
                    let c_string_len = decoded
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(decoded.len());
                    let (parsed, stopped) = self.setup.parse_encoded(&decoded[..c_string_len]);
                    (LoginSetupSource::Encoded, parsed, stopped)
                }
                Err(encoded) => return Err(LoginSetupOpenError { plain, encoded }),
            },
        };

        let (state_levels_applied, login_queue_initialized) =
            self.apply_loaded_setup_side_effects();
        Ok(LoginSetupLoadReport {
            source,
            parsed_pairs,
            stopped_at_pair,
            state_levels_applied,
            login_queue_initialized,
        })
    }

    fn apply_loaded_setup_side_effects(&mut self) -> (bool, bool) {
        let state_levels = match (
            self.setup.world_max_players,
            self.setup.world_busy_scale,
            self.setup.world_full_scale,
        ) {
            (Some(max_players), Some(busy_scale), Some(full_scale)) => {
                let max_players_i32 = i32::try_from(max_players).ok();
                let busy = legacy_ftol(busy_scale * max_players as f32);
                let full = legacy_ftol(full_scale * max_players as f32);
                match (max_players_i32, busy, full) {
                    (Some(max_players), Some(busy), Some(full)) => {
                        Some([-1, busy, full, max_players])
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        let state_levels_applied = if let Some(state_levels) = state_levels {
            self.state_levels = state_levels;
            self.change_all_world_state();
            true
        } else {
            // Safe Rust не передаёт uninitialized/out-of-range setup-значения
            // в старую `_ftol2`-границу и сохраняет прежние state levels.
            false
        };

        let login_queue_initialized =
            self.login_queue.is_some() && self.initialize_login_queue_from_setup().is_ok();

        if let (Some(ban_minutes), Some(error_limit)) = (
            self.setup.forbidden_account_time,
            self.setup.forbidden_password_errors,
        ) {
            self.configure_password_failure_policy(ban_minutes, error_limit);
        }

        (state_levels_applied, login_queue_initialized)
    }

    fn initialize_login_queue_from_setup(&self) -> Result<(), &'static str> {
        let interval = self.setup.do_queue_interval.ok_or("dwDoQueueInter")?;
        let send_interval = self
            .setup
            .send_to_queue_interval
            .ok_or("dwSendMsgToQueInter")?;
        let max_players = self.setup.world_max_players.ok_or("dwWorldMaxPlayers")?;
        self.login_queue
            .as_ref()
            .ok_or("m_pLoginQueue")?
            .on_initial(interval, send_interval, max_players);
        Ok(())
    }

    fn change_all_world_state(&mut self) {
        for (world_id, route) in &mut self.world_routes {
            let Some(accounts) = self.world_cdkeys.get(world_id) else {
                continue;
            };
            let Ok(player_count) = i32::try_from(accounts.len()) else {
                // Практически недостижимая коллекция > i32::MAX не получает
                // придуманной интерпретации исходного 32-битного `_Mysize`.
                continue;
            };
            route.state_level = if player_count < self.state_levels[1] {
                1
            } else if player_count < self.state_levels[2] {
                2
            } else {
                3
            };
        }
    }

    /// Создаёт `CRsCDKey` из четырёх byte-exact полей уже загруженного setup.
    ///
    /// Это достигнутый участок `CGame::Init` сразу после
    /// `CMyAdoBase::Initialize` и до создания message/network owners. Tiberius-
    /// owner соединяется только при первой DB-операции; сам метод сервисы не
    /// запускает и credential values наружу не возвращает.
    pub(crate) fn initialize_rs_cdkey_from_setup(
        &mut self,
    ) -> Result<(), RsCdKeyInitializationError> {
        let settings = self.setup.database_settings();
        self.attach_tiberius_rs_cdkey(settings)
    }

    /// Сохраняет четыре server-info DB-поля в достигнутой позиции `Init`.
    ///
    /// Старый код после `InitAuthClient` один раз собирал глобальную ADO-
    /// connection string. Снимок не меняется при последующем чтении setup;
    /// provider не нужен нативному TDS-клиенту и не выдаётся за серверный факт.
    pub(crate) fn initialize_server_info_database_from_setup(&mut self) {
        self.server_info_database_settings = Some(self.setup.server_info_database_settings());
    }

    /// Создаёт и присоединяет Linux/TDS-владельца из исходных DB setup-полей.
    ///
    /// Низкоуровневая граница принимает уже позиционно прочитанные byte-exact
    /// значения и не публикует credentials.
    pub(crate) fn attach_tiberius_rs_cdkey(
        &mut self,
        settings: LoginDatabaseSettings,
    ) -> Result<(), RsCdKeyInitializationError> {
        let owner = TiberiusRsCdKey::new(settings.clone())?;
        self.online_user_database_settings = Some(settings);
        self.attach_rs_cdkey_owner(Box::new(owner));
        Ok(())
    }

    /// Забирает следующее структурированное DB/file-log событие `CRsCDKey`.
    pub(crate) fn pop_rs_cdkey_notice(&mut self) -> Option<RsCdKeyNotice> {
        self.rs_cdkey_owner
            .as_mut()
            .and_then(|owner| owner.pop_notice())
    }

    /// Обновляет два исходных setup-поля политики неверного пароля.
    pub(crate) fn configure_password_failure_policy(&mut self, ban_minutes: i32, error_limit: i32) {
        self.password_ban_minutes = ban_minutes;
        self.password_error_limit = error_limit;
    }

    /// Обновляет два setup-поля, используемые password-check FIFO.
    pub(crate) fn configure_valid_code_policy(&mut self, enabled: bool, error_limit: i32) {
        self.setup_ex.valid_code = i32::from(enabled);
        self.setup_ex.valid_error_upper_limit = error_limit;
    }

    /// Обновляет исходный `m_SetupEx.dwValidErrStayTime` в миллисекундах.
    pub(crate) const fn configure_valid_error_stay_time(&mut self, stay_time_ms: u32) {
        self.setup_ex.valid_error_stay_time_ms = stay_time_ms;
    }

    /// Обновляет исходные timeout-поля matrix и legacy valid-code setup.
    pub(crate) const fn configure_login_queue_timeouts(
        &mut self,
        matrix_timeout_ms: i32,
        valid_code_overtime_ms: i32,
    ) {
        self.setup_ex.matrix_timeout_ms = matrix_timeout_ms;
        self.setup_ex.valid_code_overtime_ms = valid_code_overtime_ms;
    }

    /// Обновляет исходный `m_SetupEx.lQuestPlayerDataInterval`.
    pub(crate) const fn configure_quest_player_data_interval(&mut self, interval_ms: i32) {
        self.setup_ex.quest_player_data_interval_ms = interval_ms;
    }

    /// Возвращает текущий интервал защиты повторного запроса `player_id`.
    pub(crate) const fn quest_player_data_interval_ms(&self) -> u32 {
        self.setup_ex.quest_player_data_interval_ms as u32
    }

    /// Возвращает исходный `m_SetupEx.bValidCode`.
    pub(crate) const fn valid_code_enabled(&self) -> bool {
        self.setup_ex.valid_code != 0
    }

    /// Возвращает исходный `m_SetupEx.iValidErrUpperLimit`.
    pub(crate) const fn valid_error_limit(&self) -> i32 {
        self.setup_ex.valid_error_upper_limit
    }

    /// Возвращает исходный `m_SetupEx.dwValidErrStayTime`.
    pub(crate) const fn valid_error_stay_time_ms(&self) -> u32 {
        self.setup_ex.valid_error_stay_time_ms
    }

    /// Передаёт matrix-card проверку фактическому `CRsCDKey`.
    ///
    /// `None` означает, что частично восстановленный lifecycle ещё не
    /// присоединил DB-owner; это не считается успешной проверкой.
    pub(crate) fn validate_matrix_card(
        &mut self,
        account: &[u8],
        positions: &[u8; 3],
        answer: &[u8; 3],
    ) -> Option<MatrixValidation> {
        self.rs_cdkey_owner
            .as_mut()
            .map(|owner| owner.matrix_validate(account, positions, answer))
    }

    /// Даёт LoginQueue краткий mutable borrow единственного `CRsCDKey`.
    ///
    /// `None` сохраняет незавершённую lifecycle-границу и не назначает
    /// временный успех ни одной DB-проверке.
    pub(crate) fn rs_cdkey_owner_mut(&mut self) -> Option<&mut (dyn RsCdKeyOwner + '_)> {
        match self.rs_cdkey_owner.as_mut() {
            Some(owner) => Some(owner.as_mut()),
            None => None,
        }
    }

    /// Выполняет GAS stored procedure и сохраняет исходный безусловный `false`.
    ///
    /// `CGame::ExecuteProce` RVA 0x407AB0 открывал отдельное соединение,
    /// вызывал `getAccInfoEx` с тремя `varchar(200)` input и одним `int` output,
    /// закрывал соединение и возвращал `false` даже после успеха. DB-ошибка
    /// остаётся доступна через notice единственного `CRsCDKey` owner.
    pub(crate) fn execute_proce(
        &mut self,
        user_id: &[u8],
        user_ip: &[u8],
        user_password: &[u8],
        initial_result: i32,
    ) -> bool {
        let _legacy_log_result = initial_result;
        if let Some(owner) = self.rs_cdkey_owner.as_mut() {
            let _executed = owner.execute_get_acc_info_ex(user_id, user_ip, user_password);
        }
        false
    }

    /// Заменяет одну запись `m_listWorldInfo` для будущего World lifecycle.
    pub(crate) fn upsert_world_route(&mut self, route: WorldRoute) {
        self.world_routes.insert(route.world_id, route);
    }

    /// Добавляет World telemetry в конец исходного `m_vectorPingWorldServerInfo`.
    pub(crate) fn append_ping_world_server_info(&mut self, info: PingWorldServerInfo) -> usize {
        self.ping_world_server_info.push(info);
        self.ping_world_server_info.len() - 1
    }

    /// Добавляет account только в существующий список World без дубликата.
    pub(crate) fn add_cdkey(&mut self, account: &[u8], world_id: i32) -> bool {
        let account = legacy_c_string_prefix(account);
        let Some(accounts) = self.world_cdkeys.get_mut(&world_id) else {
            return false;
        };
        if accounts
            .iter()
            .any(|candidate| legacy_c_string_prefix(candidate) == account)
        {
            return false;
        }
        accounts.push(account.to_vec());
        self.refresh_online_cdkey_count();
        self.update_world_state_from_cdkey_count(world_id);
        true
    }

    /// Возвращает первый World ID с C-string account либо исходный sentinel `-1`.
    pub(crate) fn find_cdkey(&self, account: &[u8]) -> i32 {
        let account = legacy_c_string_prefix(account);
        self.world_cdkeys
            .iter()
            .find_map(|(world_id, accounts)| {
                accounts
                    .iter()
                    .any(|candidate| legacy_c_string_prefix(candidate) == account)
                    .then_some(*world_id)
            })
            .unwrap_or(-1)
    }

    /// Удаляет C-string account только из исходного `m_LoginCdkeyWorld`.
    pub(crate) fn clear_login_cdkey(&mut self, account: &[u8]) -> bool {
        self.login_cdkey_world
            .remove(legacy_c_string_prefix(account))
            .is_some()
    }

    /// Удаляет первое совпадение в порядке World ID либо login-map запись.
    pub(crate) fn clear_cdkey(&mut self, account: &[u8]) {
        let account = legacy_c_string_prefix(account);
        let world_id = self.world_cdkeys.iter().find_map(|(world_id, accounts)| {
            accounts
                .iter()
                .any(|candidate| legacy_c_string_prefix(candidate) == account)
                .then_some(*world_id)
        });
        let Some(world_id) = world_id else {
            self.clear_login_cdkey(account);
            return;
        };
        let accounts = self
            .world_cdkeys
            .get_mut(&world_id)
            .expect("World ID найден в этой же карте");
        let position = accounts
            .iter()
            .position(|candidate| legacy_c_string_prefix(candidate) == account)
            .expect("совпадение только что найдено");
        accounts.remove(position);
        self.refresh_online_cdkey_count();
        self.update_world_state_from_cdkey_count(world_id);
    }

    fn update_world_state_from_cdkey_count(&mut self, world_id: i32) {
        let Some(player_count) = self
            .world_cdkeys
            .get(&world_id)
            .and_then(|accounts| i32::try_from(accounts.len()).ok())
        else {
            // Исходный размер был 32-битным; коллекции больше `i32::MAX`
            // практически недостижимы и не получают придуманного state-level.
            return;
        };
        let state_level = if player_count < self.state_levels[1] {
            1
        } else if player_count < self.state_levels[2] {
            2
        } else {
            3
        };
        if let Some(route) = self.world_routes.get_mut(&world_id) {
            route.state_level = state_level;
        }
    }

    fn refresh_online_cdkey_count(&self) {
        self.online_cdkey_count.store(
            self.get_login_world_cdkey_numbers() as u32,
            Ordering::SeqCst,
        );
    }

    /// Запускает независимую DB-замену списка и сохраняет owned join handle.
    pub(crate) fn start_online_user_database_update(
        &mut self,
        world_id: i32,
        accounts: Vec<Vec<u8>>,
    ) -> Result<(), OnlineUserUpdateStartError> {
        self.collect_finished_online_user_updates();
        let settings = self
            .online_user_database_settings
            .clone()
            .ok_or(OnlineUserUpdateStartError::SettingsMissing)?;
        let requested_accounts = accounts.len();
        let handle = thread::Builder::new()
            .name("login-online-user-update".to_owned())
            .spawn(move || update_online_user_database(settings, world_id, accounts))
            .map_err(OnlineUserUpdateStartError::Spawn)?;
        self.online_user_update_tasks.push(OnlineUserUpdateTask {
            world_id,
            requested_accounts,
            handle,
        });
        Ok(())
    }

    /// Извлекает завершённый DB-отчёт, не ожидая ещё работающие workers.
    pub(crate) fn pop_online_user_database_report(&mut self) -> Option<OnlineUserDatabaseReport> {
        self.collect_finished_online_user_updates();
        self.online_user_database_reports.pop_front()
    }

    fn collect_finished_online_user_updates(&mut self) {
        let mut index = 0;
        while index < self.online_user_update_tasks.len() {
            if !self.online_user_update_tasks[index].handle.is_finished() {
                index += 1;
                continue;
            }
            let task = self.online_user_update_tasks.remove(index);
            let report = task
                .handle
                .join()
                .unwrap_or_else(|_| OnlineUserDatabaseReport {
                    world_id: task.world_id,
                    requested_accounts: task.requested_accounts,
                    delete_attempted: false,
                    insert_attempts: 0,
                    failures: vec![OnlineUserDatabaseFailure::WorkerPanicked],
                });
            self.online_user_database_reports.push_back(report);
        }
    }

    fn join_all_online_user_updates(&mut self) -> usize {
        let tasks = std::mem::take(&mut self.online_user_update_tasks);
        let joined = tasks.len();
        for task in tasks {
            let report = task
                .handle
                .join()
                .unwrap_or_else(|_| OnlineUserDatabaseReport {
                    world_id: task.world_id,
                    requested_accounts: task.requested_accounts,
                    delete_attempted: false,
                    insert_attempts: 0,
                    failures: vec![OnlineUserDatabaseFailure::WorkerPanicked],
                });
            self.online_user_database_reports.push_back(report);
        }
        joined
    }

    fn start_server_info_log_worker(
        &mut self,
        settings: LoginDatabaseSettings,
        login_ip: Vec<u8>,
    ) -> Result<(), io::Error> {
        self.collect_finished_server_info_logs();
        let shared = Arc::clone(&self.server_info_log_shared);
        let online_cdkey_count = Arc::clone(&self.online_cdkey_count);
        let staged_world_records = shared.staged_world_records();
        let staged_game_records = shared.staging.try_lock().map_or(0, |records| {
            records.iter().map(|record| record.game_servers.len()).sum()
        });
        let queued_log_records = shared.logs.size();
        let handle = thread::Builder::new()
            .name("login-server-info-log".to_owned())
            .spawn(move || {
                server_info_log_database(shared, settings, login_ip, online_cdkey_count)
            })?;
        self.server_info_log_tasks.push(ServerInfoLogTask {
            staged_world_records,
            staged_game_records,
            queued_log_records,
            handle,
        });
        Ok(())
    }

    /// Извлекает завершённый `ServerInfoLog`, не ожидая работающие workers.
    pub(crate) fn pop_server_info_database_report(&mut self) -> Option<ServerInfoDatabaseReport> {
        self.collect_finished_server_info_logs();
        self.server_info_database_reports.pop_front()
    }

    fn collect_finished_server_info_logs(&mut self) {
        let mut index = 0;
        while index < self.server_info_log_tasks.len() {
            if !self.server_info_log_tasks[index].handle.is_finished() {
                index += 1;
                continue;
            }
            let task = self.server_info_log_tasks.remove(index);
            let report = task.handle.join().unwrap_or(ServerInfoDatabaseReport {
                staged_world_records: task.staged_world_records,
                staged_game_records: task.staged_game_records,
                queued_log_records: task.queued_log_records,
                attempted_log_records: 0,
                failure: Some(ServerInfoDatabaseFailure::WorkerPanicked),
            });
            self.server_info_database_reports.push_back(report);
        }
    }

    fn join_all_server_info_logs(&mut self) -> usize {
        let tasks = std::mem::take(&mut self.server_info_log_tasks);
        let joined = tasks.len();
        for task in tasks {
            let report = task.handle.join().unwrap_or(ServerInfoDatabaseReport {
                staged_world_records: task.staged_world_records,
                staged_game_records: task.staged_game_records,
                queued_log_records: task.queued_log_records,
                attempted_log_records: 0,
                failure: Some(ServerInfoDatabaseFailure::WorkerPanicked),
            });
            self.server_info_database_reports.push_back(report);
        }
        joined
    }

    /// Ставит numeric identity текущему World socket до изменения game-state.
    pub(crate) fn set_world_socket_map_id(
        &self,
        socket_id: i32,
        world_id: i32,
    ) -> Result<i32, GameRouteError> {
        let sender = self
            .world_sender
            .as_ref()
            .ok_or(GameRouteError::MissingWorldServer)?;
        Ok(sender.set_client_map_id(socket_id, world_id))
    }

    /// Выполняет буквальную полезную часть `CGame::AddWorld`.
    pub(crate) fn activate_world(
        &mut self,
        world_id: i32,
        world_name: &[u8],
    ) -> WorldActivationOutcome {
        let Some(route) = self.world_routes.get(&world_id) else {
            self.world_operator_log_records
                .push_back(WorldOperatorLogRecord::InvalidConnection);
            return WorldActivationOutcome::Rejected(WorldActivationFailure::UnknownWorld);
        };
        if route.name != world_name {
            self.world_operator_log_records
                .push_back(WorldOperatorLogRecord::InvalidConnection);
            return WorldActivationOutcome::Rejected(WorldActivationFailure::NameMismatch);
        }
        self.world_routes
            .get_mut(&world_id)
            .expect("World только что проверен")
            .state_level = 1;

        let broadcast_failures = self.update_world_info_to_all_clients();
        let world_count = self.world_routes.len() as i32;
        self.login_queue_ref().set_world_count(world_count as u32);
        self.world_cdkeys.insert(world_id, Vec::new());
        self.refresh_online_cdkey_count();
        WorldActivationOutcome::Activated {
            world_count,
            broadcast_failures,
        }
    }

    /// Возвращает byte-exact имя настроенного мира по numeric ID.
    pub(crate) fn world_name_by_id(&self, world_id: i32) -> Option<&[u8]> {
        self.world_routes
            .get(&world_id)
            .map(|route| route.name.as_slice())
    }

    /// Очищает список вошедших CD-key, сохраняя сам World entry.
    pub(crate) fn clear_cdkeys_by_world_id(&mut self, world_id: i32) {
        if let Some(accounts) = self.world_cdkeys.get_mut(&world_id) {
            accounts.clear();
            self.refresh_online_cdkey_count();
        }
    }

    /// Ставит известный connect-log после `AddWorld` и до `0x4FC03`.
    pub(crate) fn queue_world_connected_operator_log(&mut self, world_name: &[u8]) {
        self.world_operator_log_records
            .push_back(WorldOperatorLogRecord::Connected {
                world_name: world_name.to_vec(),
            });
    }

    /// Ставит известный lost-log до очистки World CD-key.
    pub(crate) fn queue_world_lost_operator_log(&mut self, world_name: &[u8]) {
        self.world_operator_log_records
            .push_back(WorldOperatorLogRecord::Lost {
                world_name: world_name.to_vec(),
            });
    }

    /// Выполняет буквальную полезную часть `CGame::DelWorld`.
    pub(crate) fn deactivate_world(&mut self, world_id: i32) -> WorldDeactivationOutcome {
        if let Some(route) = self.world_routes.get_mut(&world_id) {
            route.state_level = 0;
        }
        let broadcast_failures = self.update_world_info_to_all_clients();
        let world_count = self.world_routes.len() as i32;
        self.login_queue_ref().set_world_count(world_count as u32);
        self.world_cdkeys.remove(&world_id);
        self.refresh_online_cdkey_count();
        WorldDeactivationOutcome {
            world_count,
            broadcast_failures,
        }
    }

    /// Рассылает новый список миров только account, ещё не выбравшим World.
    fn update_world_info_to_all_clients(&self) -> Vec<WorldInfoBroadcastFailure> {
        let mut failures = Vec::new();
        for (account, world_name) in &self.login_cdkey_world {
            if !world_name.is_empty() {
                continue;
            }
            let mut message = CMessage::new(WORLD_INFO_MESSAGE_TYPE);
            self.add_world_info_to_message(
                &mut message,
                self.login_queue_ref().is_no_queue_account(account),
            );
            if let Err(error) = self.send_to_client_cdkey(&message, account) {
                failures.push(WorldInfoBroadcastFailure {
                    account: account.clone(),
                    error,
                });
            }
        }
        failures
    }

    /// Отправляет connect-ack конкретному World socket.
    pub(crate) fn send_to_world_socket(
        &self,
        message: &CMessage,
        socket_id: i32,
    ) -> Result<i32, GameRouteError> {
        let sender = self
            .world_sender
            .as_ref()
            .ok_or(GameRouteError::MissingWorldServer)?;
        message
            .send_to_world_socket(sender, socket_id)
            .map_err(GameRouteError::Message)
    }

    /// Ставит точную legacy connect-строку в исходную `_serv_logs`.
    pub(crate) fn queue_world_connected_server_info_log(
        &mut self,
        source_ip: u32,
        world_id: i32,
        world_name: &[u8],
    ) -> ServerInfoLogDisposition {
        match self.server_info_log_disposition() {
            ServerInfoLogDisposition::Queued => {
                let date = Local::now().format("%m/%d/%y").to_string();
                let time = Local::now().format("%H:%M:%S").to_string();
                let mut description =
                    Vec::with_capacity(2 + world_name.len() + 2 + date.len() + 1 + time.len() + 7);
                description.extend_from_slice(b"WS");
                description.extend_from_slice(world_name);
                description.extend_from_slice(&[0xD4, 0xDA]);
                description.extend_from_slice(date.as_bytes());
                description.push(b' ');
                description.extend_from_slice(time.as_bytes());
                description.extend_from_slice(&[0xC1, 0xAC, 0xBD, 0xD3]);
                description.extend_from_slice(b"LS.");
                self.server_info_log_shared.logs.push(ServLog::new(
                    source_ip,
                    -2,
                    world_id,
                    description,
                ));
                ServerInfoLogDisposition::Queued
            }
            disposition => disposition,
        }
    }

    /// Возвращает исходный setup-gate до чтения server-log payload.
    pub(crate) fn server_info_log_disposition(&self) -> ServerInfoLogDisposition {
        match self.setup.server_info_log_time {
            None => ServerInfoLogDisposition::SetupValueMissing,
            Some(0) => ServerInfoLogDisposition::Disabled,
            Some(_) => ServerInfoLogDisposition::Queued,
        }
    }

    /// Ставит готовую запись после выполненного вызывающим setup-gate.
    pub(crate) fn push_server_info_log(
        &self,
        source_ip: u32,
        server_type: i32,
        server_number: i32,
        description: Vec<u8>,
    ) {
        self.server_info_log_shared.logs.push(ServLog::new(
            source_ip,
            server_type,
            server_number,
            description,
        ));
    }

    /// Извлекает старейшую typed-запись исходной `_serv_logs`.
    pub(crate) fn pop_server_info_log_record(&self) -> Option<ServLog> {
        self.server_info_log_shared.logs.pop()
    }

    /// Извлекает старейший World lifecycle вызов исходного `AddLogText`.
    pub(crate) fn pop_world_operator_log_record(&mut self) -> Option<WorldOperatorLogRecord> {
        self.world_operator_log_records.pop_front()
    }

    /// Ищет первое точное byte-name совпадение в порядке numeric world ID.
    ///
    /// Отсутствующая либо отключённая запись возвращает исходный sentinel `-1`.
    pub(crate) fn world_id_by_name(&self, name: &[u8]) -> i32 {
        self.world_routes
            .values()
            .find(|route| route.name == name)
            .filter(|route| route.state_level != 0)
            .map_or(-1, |route| route.world_id)
    }

    /// Проверяет существование включённого WorldServer по byte-exact имени.
    pub(crate) fn is_exit_world(&self, name: &[u8]) -> bool {
        self.world_id_by_name(name) != -1
    }

    /// Возвращает число известных account выбранного World либо sentinel `-1`.
    pub(crate) fn login_world_player_num_by_name(&self, name: &[u8]) -> i32 {
        let world_id = self.world_id_by_name(name);
        if world_id == -1 {
            return -1;
        }
        self.world_cdkeys
            .get(&world_id)
            .and_then(|accounts| i32::try_from(accounts.len()).ok())
            .unwrap_or(-1)
    }

    /// Возвращает выбранный World из исходного `m_LoginCdkeyWorld`.
    pub(crate) fn login_cdkey_world_server(&self, account: &[u8]) -> Option<&[u8]> {
        self.login_cdkey_world.get(account).map(Vec::as_slice)
    }

    /// Заменяет выбранный World для byte-exact account.
    pub(crate) fn set_login_cdkey_world_server(&mut self, account: &[u8], world_name: &[u8]) {
        self.login_cdkey_world
            .insert(account.to_vec(), world_name.to_vec());
    }

    /// Копирует сообщение в send-команду WorldServer по numeric map ID.
    pub(crate) fn send_msg_to_world(
        &self,
        message: &CMessage,
        world_id: i32,
    ) -> Result<i32, GameRouteError> {
        let sender = self
            .world_sender
            .as_ref()
            .ok_or(GameRouteError::MissingWorldServer)?;
        message
            .send_to_world_map(sender, world_id)
            .map_err(GameRouteError::Message)
    }

    /// Копирует RLE-ответ клиенту по исходному signed socket ID.
    pub(crate) fn send_to_client(
        &self,
        message: &CMessage,
        socket_id: i32,
    ) -> Result<i32, GameRouteError> {
        let sender = self
            .client_sender
            .as_ref()
            .ok_or(GameRouteError::MissingClientServer)?;
        message
            .send_to_client_socket(sender, socket_id)
            .map_err(GameRouteError::Message)
    }

    /// Копирует RLE-ответ клиенту по строковой CD-key identity.
    pub(crate) fn send_to_client_cdkey(
        &self,
        message: &CMessage,
        account: &[u8],
    ) -> Result<i32, GameRouteError> {
        let sender = self
            .client_sender
            .as_ref()
            .ok_or(GameRouteError::MissingClientServer)?;
        message
            .send_to_client_cdkey(sender, account)
            .map_err(GameRouteError::Message)
    }

    /// Выполняет исходный `KickOut`: client identity имеет приоритет над world.
    pub(crate) fn kick_out(&self, account: &[u8]) -> Result<bool, GameRouteError> {
        let account = legacy_c_string_prefix(account);
        if self.login_cdkey_world.contains_key(account) {
            self.quit_client_by_cdkey(account)?;
            return Ok(true);
        }

        let world_id = self.find_cdkey(account);
        if world_id == -1 {
            return Ok(false);
        }

        let mut message = CMessage::new(KICK_PLAYER_MESSAGE_TYPE);
        add_legacy_string(&mut message, account);
        self.send_msg_to_world(&message, world_id)?;
        Ok(true)
    }

    /// Ставит client `QUIT` по C-string CD-key identity.
    pub(crate) fn quit_client_by_cdkey(&self, account: &[u8]) -> Result<i32, GameRouteError> {
        let sender = self
            .client_sender
            .as_ref()
            .ok_or(GameRouteError::MissingClientServer)?;
        Ok(sender.quit_by_map_name(legacy_c_string_prefix(account)))
    }

    /// Передаёт owned password-check в очередь и сохраняет порядок duplicate.
    pub(crate) fn push_back_pwd_checked(&mut self, checked: TagPwdChecked) {
        let queue = Arc::clone(self.login_queue_ref());
        queue.push_back_pwd_checked(checked, |account| {
            if let Err(error) = self.kick_out(account) {
                self.auth_handler_notices
                    .push_back(AuthHandlerNotice::KickOutFailed {
                        account: account.to_vec(),
                        error,
                    });
            }
        });
    }

    /// Возвращает владеющую ссылку на embedded `CLoginQueue` этого `CGame`.
    pub(crate) fn login_queue(&self) -> Arc<CLoginQueue> {
        Arc::clone(self.login_queue_ref())
    }

    /// Запускает один полный исходный `HandlePwdChecked` у queue-owner.
    pub(crate) fn handle_pwd_checked(&mut self) -> HandlePwdCheckedReport {
        let queue = Arc::clone(self.login_queue_ref());
        queue.handle_pwd_checked(self)
    }

    /// Выполняет один полный проход восстановленного `CLoginQueue::Run`.
    pub(crate) fn run_login_queue(
        &mut self,
        auth_manager: &mut AuthManager,
    ) -> LoginQueueRunReport {
        let queue = Arc::clone(self.login_queue_ref());
        let matrix_timeout_ms = self.setup_ex.matrix_timeout_ms as u32;
        let valid_code_overtime_ms = self.setup_ex.valid_code_overtime_ms as u32;
        queue.run(
            self,
            auth_manager,
            matrix_timeout_ms,
            valid_code_overtime_ms,
        )
    }

    /// Выполняет доказанный timeout-хвост отдельно для диагностического owner.
    ///
    /// Он намеренно остаётся доступен отдельно от полного прохода только для
    /// точечной проверки трёх cadence-владельцев.
    pub(crate) fn run_login_queue_timeout_tail(&self) -> LoginQueueTimeoutReport {
        self.login_queue_ref().run_timeout_tail(
            self,
            self.setup_ex.matrix_timeout_ms as u32,
            self.setup_ex.valid_code_overtime_ms as u32,
        )
    }

    fn login_queue_ref(&self) -> &Arc<CLoginQueue> {
        self.login_queue
            .as_ref()
            .expect("CLoginQueue доступна только между успешной позицией Init и Release")
    }

    /// Выполняет доказанную часть `PrepareEnter` до `EnterGame`.
    pub(crate) fn prepare_enter(
        &mut self,
        checked: &TagPwdChecked,
    ) -> Result<PrepareEnterOutcome, GameRouteError> {
        if checked.client_ip() == 0 || checked.socket_id() == 0 {
            return Ok(PrepareEnterOutcome::Finished);
        }

        if checked.world_server().is_empty() && self.kick_out(checked.account())? {
            let mut message = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
            message.base_mut().add_char(8);
            self.send_to_client(&message, checked.socket_id())?;
            return Ok(PrepareEnterOutcome::Finished);
        }

        let sender = self
            .client_sender
            .as_ref()
            .ok_or(GameRouteError::MissingClientServer)?;
        sender.set_client_map_name(checked.socket_id(), checked.account());
        self.account_log_queue
            .push(AccountLogRecord::Enter(AccountEnterRecord {
                account: checked.account().to_vec(),
                client_ip: checked.client_ip(),
                recorded_at: Local::now().naive_local(),
            }));
        self.login_cdkey_world
            .insert(checked.account().to_vec(), checked.world_server().to_vec());

        if checked.world_server().is_empty() && checked.has_matrix() {
            return Ok(PrepareEnterOutcome::MatrixRegistrationRequired);
        }
        Ok(PrepareEnterOutcome::Continue)
    }

    /// Выполняет доказанную обычную ветвь `EnterGame`.
    pub(crate) fn enter_game(
        &self,
        checked: &TagPwdChecked,
        no_queue_account: bool,
    ) -> Result<(), GameRouteError> {
        if checked.client_ip() == 0 || checked.socket_id() == 0 {
            return Ok(());
        }

        if checked.world_server().is_empty() {
            let mut message = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
            message.base_mut().add_char(2);
            add_legacy_string(&mut message, checked.account());
            self.add_world_info_to_message(&mut message, no_queue_account);
            self.send_to_client(&message, checked.socket_id())?;
        } else {
            self.l2w_player_base_send(checked.world_server(), checked.account(), no_queue_account)?;
        }
        Ok(())
    }

    /// Продолжает matrix-вход только при сохранённой записи account/world.
    ///
    /// Исходный handler проверял `GetLoginCdkeyWorldServer(account)`, но затем
    /// намеренно передавал в `EnterGame` пустое имя мира.
    pub(crate) fn enter_after_matrix(
        &self,
        account: &[u8],
        client_ip: u32,
        socket_id: i32,
        no_queue_account: bool,
    ) -> Result<bool, GameRouteError> {
        if !self.login_cdkey_world.contains_key(account) {
            return Ok(false);
        }
        let checked = TagPwdChecked::new(socket_id, client_ip, account.to_vec(), Vec::new(), false);
        self.enter_game(&checked, no_queue_account)?;
        Ok(true)
    }

    /// Ставит account-leave запись в общую FIFO исходной `_acc_logs`.
    pub(crate) fn account_leave_log(&mut self, account: &[u8]) {
        let end = account
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(account.len());
        self.account_log_queue
            .push(AccountLogRecord::Leave(AccountLeaveRecord {
                account: account[..end].to_vec(),
                recorded_at: Local::now().naive_local(),
            }));
    }

    /// Ставит role-enter запись в общую FIFO исходной `_acc_logs`.
    pub(crate) fn role_enter_log(
        &mut self,
        account: &[u8],
        role_name: &[u8],
        role_level: u8,
        world_number: i32,
    ) {
        self.account_log_queue
            .push(AccountLogRecord::RoleEnter(RoleEnterRecord {
                account: legacy_c_string_prefix(account).to_vec(),
                role_name: legacy_c_string_prefix(role_name).to_vec(),
                role_level,
                world_number,
                recorded_at: Local::now().naive_local(),
            }));
    }

    /// Ставит `LeaveLog`-запись в общую FIFO исходной `_acc_logs`.
    pub(crate) fn leave_log(&mut self, account: &[u8]) {
        self.account_log_queue
            .push(AccountLogRecord::SessionLeave(SessionLeaveRecord {
                account: legacy_c_string_prefix(account).to_vec(),
                recorded_at: Local::now().naive_local(),
            }));
    }

    fn add_world_info_to_message(&self, message: &mut CMessage, no_queue_account: bool) {
        message.base_mut().add_short(self.world_routes.len() as i16);
        for route in self.world_routes.values() {
            let visible_state =
                if self.world_server_is_open_state(route.world_id) || no_queue_account {
                    route.state_level
                } else {
                    0
                };
            message.base_mut().add_long(visible_state);
            add_legacy_string(message, &route.name);
        }
    }

    /// Дописывает список миров с исходной проверкой no-queue для account.
    pub(crate) fn add_world_info_to_message_for_account(
        &self,
        message: &mut CMessage,
        account: &[u8],
    ) {
        let no_queue_account = self
            .login_queue_ref()
            .is_no_queue_account(legacy_c_string_prefix(account));
        self.add_world_info_to_message(message, no_queue_account);
    }

    pub(crate) fn l2w_player_base_send(
        &self,
        world_name: &[u8],
        account: &[u8],
        no_queue_account: bool,
    ) -> Result<bool, GameRouteError> {
        if account.is_empty() {
            return Ok(false);
        }
        let world_id = self.world_id_by_name(world_name);
        if world_id == -1 {
            return Ok(false);
        }
        if !self.world_server_is_open_state(world_id) && !no_queue_account {
            return Ok(false);
        }

        let mut message = CMessage::new(WORLD_PLAYER_BASE_MESSAGE_TYPE);
        add_legacy_string(&mut message, account);
        self.send_msg_to_world(&message, world_id)?;
        Ok(true)
    }

    /// Пересылает исходный create-role payload как `0x4FB04 + account`.
    ///
    /// `None` либо неизвестный/отключённый World сохраняет исходный no-op и
    /// оставляет входное сообщение неизменным.
    pub(crate) fn l2w_create_role_send(
        &self,
        world_name: Option<&[u8]>,
        account: &[u8],
        message: &mut CMessage,
    ) -> Result<bool, GameRouteError> {
        let Some(world_name) = world_name else {
            return Ok(false);
        };
        let world_id = self.world_id_by_name(world_name);
        if world_id == -1 {
            return Ok(false);
        }

        message.set_message_type(WORLD_CREATE_ROLE_MESSAGE_TYPE);
        add_legacy_string(message, account);
        self.send_msg_to_world(message, world_id)?;
        Ok(true)
    }

    /// Отправляет `0x4FB02 + account + player ID + IPv4`.
    pub(crate) fn l2w_delete_role_send(
        &self,
        world_name: Option<&[u8]>,
        account: &[u8],
        player_id: i32,
        client_ip: u32,
    ) -> Result<bool, GameRouteError> {
        let Some(world_name) = world_name else {
            return Ok(false);
        };
        if account.first().copied().unwrap_or(0) == 0 {
            return Ok(false);
        }
        let world_id = self.world_id_by_name(world_name);
        if world_id == -1 {
            return Ok(false);
        }

        let mut message = CMessage::new(WORLD_DELETE_ROLE_MESSAGE_TYPE);
        add_legacy_string(&mut message, account);
        message.base_mut().add_long(player_id);
        message.base_mut().add_long(client_ip as i32);
        self.send_msg_to_world(&message, world_id)?;
        Ok(true)
    }

    /// Отправляет `0x4FB03 + account + player ID`.
    pub(crate) fn l2w_restore_role_send(
        &self,
        world_name: Option<&[u8]>,
        account: &[u8],
        player_id: u32,
    ) -> Result<bool, GameRouteError> {
        let Some(world_name) = world_name else {
            return Ok(false);
        };
        let world_id = self.world_id_by_name(world_name);
        if world_id == -1 {
            return Ok(false);
        }

        let mut message = CMessage::new(WORLD_RESTORE_ROLE_MESSAGE_TYPE);
        add_legacy_string(&mut message, account);
        message.base_mut().add_long(player_id as i32);
        self.send_msg_to_world(&message, world_id)?;
        Ok(true)
    }

    /// Отправляет WorldServer запрос деталей персонажа `0x4FB05`.
    ///
    /// `None` сохраняет исходную nullable-границу World lookup и даёт no-op.
    pub(crate) fn l2w_quest_detail_send(
        &self,
        world_name: Option<&[u8]>,
        account: &[u8],
        player_id: i32,
        client_ip: u32,
    ) -> Result<bool, GameRouteError> {
        let Some(world_name) = world_name else {
            return Ok(false);
        };
        let world_id = self.world_id_by_name(world_name);
        if world_id == -1 {
            return Ok(false);
        }

        let mut message = CMessage::new(WORLD_PLAYER_DETAIL_MESSAGE_TYPE);
        message.base_mut().add_long(player_id);
        add_legacy_string(&mut message, account);
        message.base_mut().add_long(client_ip as i32);
        self.send_msg_to_world(&message, world_id)?;
        Ok(true)
    }

    /// Применяет default-ветвь исходного счётчика и синхронного `CDKeyBan`.
    pub(crate) fn register_password_failure(&mut self, account: &[u8]) -> PasswordFailureOutcome {
        if self.password_ban_minutes == 0 {
            return PasswordFailureOutcome::Disabled;
        }

        match self.password_errors.get_mut(account) {
            None => {
                self.password_errors.insert(account.to_vec(), 1);
                PasswordFailureOutcome::Counted { failures: 1 }
            }
            Some(failures) if *failures < self.password_error_limit => {
                *failures += 1;
                PasswordFailureOutcome::Counted {
                    failures: *failures,
                }
            }
            Some(_) => {
                let Some(owner) = self.rs_cdkey_owner.as_mut() else {
                    self.password_errors.remove(account);
                    return PasswordFailureOutcome::BanOwnerMissing;
                };
                let succeeded = owner.cd_key_ban(account, self.password_ban_minutes);
                self.password_errors.remove(account);
                PasswordFailureOutcome::BanAttempted { succeeded }
            }
        }
    }

    /// Добавляет структурированное событие в исходной позиции side effect.
    pub(crate) fn push_auth_handler_notice(&mut self, notice: AuthHandlerNotice) {
        self.auth_handler_notices.push_back(notice);
    }

    /// Извлекает самое старое событие `AuthHandler`.
    pub(crate) fn pop_auth_handler_notice(&mut self) -> Option<AuthHandlerNotice> {
        self.auth_handler_notices.pop_front()
    }

    /// Копирует сообщение всем текущим WorldServer.
    pub(crate) fn send_all_world(&self, message: &CMessage) -> Result<i32, GameRouteError> {
        let sender = self
            .world_sender
            .as_ref()
            .ok_or(GameRouteError::MissingWorldServer)?;
        message
            .send_all_world(sender)
            .map_err(GameRouteError::Message)
    }

    /// Копирует сообщение текущему AuthServer.
    pub(crate) fn send_to_auth(&self, message: &CMessage) -> Result<i32, GameRouteError> {
        let sender = self
            .auth_send_queue()
            .ok_or(GameRouteError::MissingAuthClient)?;
        message
            .send_to_auth(sender)
            .map_err(GameRouteError::Message)
    }

    fn send_login_server_info_to_auth(&self) -> Result<i32, SendMessageError> {
        let mut message = CMessage::new(LOGIN_SERVER_INFO_MESSAGE_TYPE);
        message.base_mut().add_long(self.setup_ex.area_id);
        let client = self
            .auth_client
            .as_ref()
            .expect("Init/Reassign устанавливает Auth client до отправки server-info");
        message.send_to_auth(client.send_queue())
    }

    fn reassign_as(
        &mut self,
        mut client: CMyNetClientAuth,
    ) -> Result<(AuthServerConfig, Result<i32, SendMessageError>), AuthLifecycleError> {
        let endpoint = client
            .connected_endpoint()
            .map(AuthServerConfig::from_resolved)
            .ok_or(AuthLifecycleError::ReplacementEndpointMissing)?;
        self.auth_client = None;
        client.enable_control_send();
        self.current_auth_server = Some(endpoint.clone());
        self.auth_client = Some(client);
        let login_server_info = self.send_login_server_info_to_auth();
        Ok((endpoint, login_server_info))
    }

    async fn connect_new_auth_client(
        &self,
    ) -> (
        CMyNetClientAuth,
        Vec<AuthConnectAttempt>,
        Option<AuthServerConfig>,
    ) {
        connect_new_auth_client(&self.reconnect_plan()).await
    }

    fn reconnect_plan(&self) -> AuthReconnectPlan {
        AuthReconnectPlan {
            bind_ip: self.setup.bind_ip.clone(),
            bind_port: u32::from(self.setup.bind_port.unwrap_or(0)),
            auth_servers: self.auth_servers.clone(),
        }
    }

    async fn stop_reconnect_task(&mut self) {
        if let Some(task) = self.reconnect_task.take() {
            task.abort();
            let _ = task.await;
        }
    }
}

/// Очищает всю runtime-таблицу `online_user` через отдельное DB-соединение.
///
/// Ошибка подключения либо DELETE возвращается вызывающему; исходные `Init` и
/// `Release` намеренно продолжали работу после проигнорированного `false`.
pub(crate) async fn clear_online_user_database(
    settings: LoginDatabaseSettings,
) -> Result<(), RsCdKeyDatabaseError> {
    let mut client = connect_login_database(settings).await?;
    client.execute("DELETE FROM online_user", &[]).await?;
    Ok(())
}

fn update_online_user_database(
    settings: LoginDatabaseSettings,
    world_id: i32,
    accounts: Vec<Vec<u8>>,
) -> OnlineUserDatabaseReport {
    let requested_accounts = accounts.len();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            return OnlineUserDatabaseReport {
                world_id,
                requested_accounts,
                delete_attempted: false,
                insert_attempts: 0,
                failures: vec![OnlineUserDatabaseFailure::Runtime(error)],
            };
        }
    };
    runtime.block_on(update_online_user_database_async(
        settings, world_id, accounts,
    ))
}

async fn update_online_user_database_async(
    settings: LoginDatabaseSettings,
    world_id: i32,
    accounts: Vec<Vec<u8>>,
) -> OnlineUserDatabaseReport {
    let requested_accounts = accounts.len();
    let mut report = OnlineUserDatabaseReport {
        world_id,
        requested_accounts,
        delete_attempted: false,
        insert_attempts: 0,
        failures: Vec::new(),
    };
    let mut client = match connect_login_database(settings).await {
        Ok(client) => client,
        Err(error) => {
            report
                .failures
                .push(OnlineUserDatabaseFailure::Connect(error));
            return report;
        }
    };

    report.delete_attempted = true;
    let delete_sql = format!("DELETE FROM online_user WHERE worldNumber={world_id}");
    if let Err(error) = client.execute(delete_sql, &[]).await {
        report
            .failures
            .push(OnlineUserDatabaseFailure::Delete(error.into()));
    }

    for (account_index, account) in accounts.into_iter().enumerate() {
        report.insert_attempts += 1;
        let (account, _, _) = WINDOWS_1251.decode(&account);
        // Login RVA 0x000060A0 строил SQL через `sprintf` без escaping.
        // Буквальная строка сохраняет реакцию доверенного WorldServer и не
        // назначает новый результат account с апострофом.
        let insert_sql = format!(
            "INSERT INTO online_user(userAccount, worldNumber) VALUES('{account}',{world_id})"
        );
        if let Err(error) = client.execute(insert_sql, &[]).await {
            report.failures.push(OnlineUserDatabaseFailure::Insert {
                account_index,
                error: error.into(),
            });
        }
    }
    report
}

fn server_info_log_database(
    shared: Arc<ServerInfoLogShared>,
    settings: LoginDatabaseSettings,
    login_ip: Vec<u8>,
    online_cdkey_count: Arc<AtomicU32>,
) -> ServerInfoDatabaseReport {
    // `ServerInfoLog` держал `g_csServerInfoLog` от EnterCriticalSection до
    // финальной очистки staging, включая connect и все ADO-вызовы.
    let mut staging = shared.staging.lock();
    let staged_world_records = staging.len();
    let staged_game_records = staging.iter().map(|world| world.game_servers.len()).sum();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            staging.clear();
            shared.staged_world_records.store(0, Ordering::SeqCst);
            return ServerInfoDatabaseReport {
                staged_world_records,
                staged_game_records,
                queued_log_records: 0,
                attempted_log_records: 0,
                failure: Some(ServerInfoDatabaseFailure::Runtime(error)),
            };
        }
    };
    let report = runtime.block_on(server_info_log_database_async(
        settings,
        &login_ip,
        &staging,
        &shared.logs,
        &online_cdkey_count,
    ));

    // Catch RVA 0x000139BC возвращался в общий хвост RVA 0x000138A6, поэтому
    // COM-ошибка, как и успех, очищала весь telemetry staging перед unlock.
    staging.clear();
    shared.staged_world_records.store(0, Ordering::SeqCst);
    report
}

async fn server_info_log_database_async(
    settings: LoginDatabaseSettings,
    login_ip: &[u8],
    staging: &[PingWorldServerInfo],
    logs: &ServLogQueue,
    online_cdkey_count: &AtomicU32,
) -> ServerInfoDatabaseReport {
    let staged_world_records = staging.len();
    let staged_game_records = staging.iter().map(|world| world.game_servers.len()).sum();
    let mut report = ServerInfoDatabaseReport {
        staged_world_records,
        staged_game_records,
        queued_log_records: 0,
        attempted_log_records: 0,
        failure: None,
    };
    let mut client = match connect_login_database(settings).await {
        Ok(client) => client,
        Err(error) => {
            report.failure = Some(ServerInfoDatabaseFailure::Database {
                operation: ServerInfoDatabaseOperation::Connect,
                error,
            });
            return report;
        }
    };

    let (login_ip, _, _) = WINDOWS_1251.decode(login_ip);
    let login_lookup =
        format!("SELECT * FROM server_info WHERE server_ip = '{login_ip}' AND server_type = 0");
    let login_exists = match server_info_row_exists(
        &mut client,
        login_lookup,
        ServerInfoDatabaseOperation::LoginLookup,
    )
    .await
    {
        Ok(exists) => exists,
        Err(failure) => {
            report.failure = Some(failure);
            return report;
        }
    };
    // GetCdkeyCount RVA 0x00007930 выполнялся именно после lookup и перед
    // sprintf выбранной UPDATE/INSERT ветви; `%d` читал uint bit-pattern как int.
    let online_user = online_cdkey_count.load(Ordering::SeqCst) as i32;
    let login_write = if login_exists {
        format!(
            "UPDATE server_info SET online_user={online_user}, last_time=getdate(), server_state=0 WHERE server_ip = '{login_ip}' AND server_type = 0"
        )
    } else {
        format!(
            "INSERT INTO server_info(server_ip, server_type, server_num, online_user, last_time, server_state, world_id) VALUES('{login_ip}', 0, 0, {online_user}, getdate(), 0, 0)"
        )
    };
    if let Err(failure) = server_info_execute(
        &mut client,
        login_write,
        ServerInfoDatabaseOperation::LoginWrite,
    )
    .await
    {
        report.failure = Some(failure);
        return report;
    }

    for (world_index, world) in staging.iter().enumerate() {
        let (world_ip, _, _) = WINDOWS_1251.decode(&world.ip);
        let world_port = world.port as i32;
        let world_players = world.player_count as i32;
        let world_lookup = format!(
            "SELECT * FROM server_info WHERE server_ip = '{world_ip}' AND server_num = {world_port} AND server_type = 1"
        );
        let world_exists = match server_info_row_exists(
            &mut client,
            world_lookup,
            ServerInfoDatabaseOperation::WorldLookup { world_index },
        )
        .await
        {
            Ok(exists) => exists,
            Err(failure) => {
                report.failure = Some(failure);
                return report;
            }
        };
        let world_write = if world_exists {
            format!(
                "UPDATE server_info SET online_user={world_players}, last_time=getdate(), server_state=0 WHERE server_ip = '{world_ip}' AND server_num = {world_port} AND server_type = 1"
            )
        } else {
            format!(
                "INSERT INTO server_info(server_ip, server_type, server_num, online_user, last_time, server_state, world_id) VALUES('{world_ip}',1,{world_port},{world_players},getdate(),0,{world_port})"
            )
        };
        if let Err(failure) = server_info_execute(
            &mut client,
            world_write,
            ServerInfoDatabaseOperation::WorldWrite { world_index },
        )
        .await
        {
            report.failure = Some(failure);
            return report;
        }

        for (game_index, game) in world.game_servers.iter().enumerate() {
            let (game_ip, _, _) = WINDOWS_1251.decode(&game.ip);
            let game_port = game.port as i32;
            let game_players = game.player_count as i32;
            let game_lookup = format!(
                "SELECT * FROM server_info WHERE server_ip = '{game_ip}' AND server_type = 2 AND server_num = {game_port} AND world_id = {world_port}"
            );
            let game_exists = match server_info_row_exists(
                &mut client,
                game_lookup,
                ServerInfoDatabaseOperation::GameLookup {
                    world_index,
                    game_index,
                },
            )
            .await
            {
                Ok(exists) => exists,
                Err(failure) => {
                    report.failure = Some(failure);
                    return report;
                }
            };
            let game_write = if game_exists {
                format!(
                    "UPDATE server_info SET online_user={game_players}, last_time=getdate(), server_state=0 WHERE server_ip = '{game_ip}' AND server_type = 2 AND server_num = {game_port} AND world_id = {world_port}"
                )
            } else {
                format!(
                    "INSERT INTO server_info(server_ip, server_type, server_num, online_user, last_time, server_state, world_id) VALUES('{game_ip}', 2, {game_port}, {game_players}, getdate(), 0, {world_port})"
                )
            };
            if let Err(failure) = server_info_execute(
                &mut client,
                game_write,
                ServerInfoDatabaseOperation::GameWrite {
                    world_index,
                    game_index,
                },
            )
            .await
            {
                report.failure = Some(failure);
                return report;
            }
        }
    }

    // Исходник снимал размер FIFO один раз после ReleaseRs: новые записи во
    // время цикла оставались следующему worker. Каждая запись pop-алась до SQL.
    report.queued_log_records = logs.size();
    for log_index in 0..report.queued_log_records.max(0) as usize {
        let Some(record) = logs.pop() else {
            continue;
        };
        report.attempted_log_records += 1;
        let octets = record.source_ip.to_le_bytes();
        let (description, _, _) = WINDOWS_1251.decode(&record.description);
        let sql = format!(
            "INSERT INTO server_log(serv_ip,serv_type,serv_id,description) VALUES('{}.{}.{}.{}',{},{},'{}');",
            octets[0],
            octets[1],
            octets[2],
            octets[3],
            record.server_type,
            record.server_number,
            description,
        );
        if let Err(failure) = server_info_execute(
            &mut client,
            sql,
            ServerInfoDatabaseOperation::ServerLogWrite { log_index },
        )
        .await
        {
            // Запись уже снята: COM catch её не возвращал в `_serv_logs`.
            report.failure = Some(failure);
            return report;
        }
    }
    report
}

async fn server_info_row_exists(
    client: &mut TdsClient,
    sql: String,
    operation: ServerInfoDatabaseOperation,
) -> Result<bool, ServerInfoDatabaseFailure> {
    client
        .query(sql, &[])
        .await
        .map_err(|error| ServerInfoDatabaseFailure::Database {
            operation,
            error: error.into(),
        })?
        .into_row()
        .await
        .map(|row| row.is_some())
        .map_err(|error| ServerInfoDatabaseFailure::Database {
            operation,
            error: error.into(),
        })
}

async fn server_info_execute(
    client: &mut TdsClient,
    sql: String,
    operation: ServerInfoDatabaseOperation,
) -> Result<(), ServerInfoDatabaseFailure> {
    client.execute(sql, &[]).await.map(|_| ()).map_err(|error| {
        ServerInfoDatabaseFailure::Database {
            operation,
            error: error.into(),
        }
    })
}

fn resolve_first_local_ipv4() -> Option<Ipv4Addr> {
    let hostname = uname();
    let hostname = hostname.nodename().to_str().ok()?;
    (hostname, 0)
        .to_socket_addrs()
        .ok()?
        .find_map(|address| match address {
            SocketAddr::V4(address) => Some(*address.ip()),
            SocketAddr::V6(_) => None,
        })
}

fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u128).wrapping_mul(1000);
    let nanoseconds_ms = (now.tv_nsec as u128) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

fn legacy_ftol(value: f32) -> Option<i32> {
    let value = value.trunc();
    (value.is_finite()
        && f64::from(value) >= f64::from(i32::MIN)
        && f64::from(value) <= f64::from(i32::MAX))
    .then_some(value as i32)
}

impl AuthListener for CGame {
    fn on_quest(&mut self, quest: &AuthQuest) {
        AuthHandler::on_quest(quest);
    }

    fn on_response(&mut self, result: &AuthResult) {
        AuthHandler::on_response(self, result);
    }
}

impl Drop for CGame {
    fn drop(&mut self) {
        if let Some(task) = self.world_accept_task.take() {
            task.abort();
        }
        if let Some(task) = self.client_accept_task.take() {
            task.abort();
        }
        self.account_log_queue.clear();
        self.account_log_thread.take();
        self.gas_thread.take();
        if let Some(task) = self.reconnect_task.take() {
            task.abort();
        }
        let _ = self.join_all_online_user_updates();
        let _ = self.join_all_server_info_logs();
    }
}

fn spawn_io_actions(
    tasks: &mut JoinSet<ServerIoCompletion>,
    actions: Vec<ServerIoAction>,
    commands: ServerCommandHandle,
) {
    for action in actions {
        let commands = commands.clone();
        tasks.spawn(async move { action.run(commands).await });
    }
}

fn drain_io_tasks(
    tasks: &mut JoinSet<ServerIoCompletion>,
    direction: LoginNetworkDirection,
    output: &mut Vec<ServerIoCompletion>,
) -> Result<(), LoginNetworkRuntimeError> {
    while let Some(completion) = tasks.try_join_next() {
        output
            .push(completion.map_err(|error| LoginNetworkRuntimeError::Task { direction, error })?);
    }
    Ok(())
}

fn collect_io_task_errors(tasks: &mut JoinSet<ServerIoCompletion>, errors: &mut Vec<JoinError>) {
    while let Some(completion) = tasks.try_join_next() {
        if let Err(error) = completion {
            errors.push(error);
        }
    }
}

async fn stop_accept_task(
    task: &mut Option<JoinHandle<io::Result<(TcpStream, SocketAddrV4)>>>,
) -> Vec<JoinError> {
    let Some(task) = task.take() else {
        return Vec::new();
    };
    task.abort();
    match task.await {
        Ok(result) => {
            drop(result);
            Vec::new()
        }
        Err(error) if error.is_cancelled() => Vec::new(),
        Err(error) => vec![error],
    }
}

async fn poll_once<Output>(future: impl Future<Output = Output>) -> Option<Output> {
    let mut future = Box::pin(future);
    poll_fn(move |context| {
        Poll::Ready(match future.as_mut().poll(context) {
            Poll::Ready(output) => Some(output),
            Poll::Pending => None,
        })
    })
    .await
}

fn add_legacy_string(message: &mut CMessage, value: &[u8]) {
    let value = legacy_c_string_prefix(value);
    message.base_mut().add(value);
    message.base_mut().add_byte(0);
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn resolve_baseline_bind_ipv4(raw: &[u8]) -> Result<Ipv4Addr, AuthConnectFailure> {
    legacy_inet_addr(legacy_c_string_prefix(raw))
        .ok_or_else(|| AuthConnectFailure::BindAddressInvalid {
            host: raw.to_vec(),
        })
}

async fn run_reconnect_task(plan: AuthReconnectPlan, publisher: AuthClientEventPublisher) {
    loop {
        let (client, _attempts, connected) = connect_new_auth_client(&plan).await;
        if connected.is_some() {
            publisher.publish_reconnected(client);
            return;
        }
        tokio::time::sleep(AUTH_RECONNECT_DELAY).await;
    }
}

async fn connect_new_auth_client(
    plan: &AuthReconnectPlan,
) -> (
    CMyNetClientAuth,
    Vec<AuthConnectAttempt>,
    Option<AuthServerConfig>,
) {
    let mut client = CMyNetClientAuth::new();
    let mut attempts = Vec::new();
    let mut connected = None;

    for configured in &plan.auth_servers {
        let result = match configured.resolve_ipv4().await {
            Ok(remote) => match resolve_baseline_bind_ipv4(&plan.bind_ip) {
                Ok(bind_address) => match bind_tcp_ipv4(Some(bind_address), plan.bind_port) {
                    Ok(socket) => client
                        .connect(socket, remote)
                        .await
                        .map_err(AuthConnectFailure::Connect),
                    Err(error) => Err(AuthConnectFailure::Bind(error)),
                },
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };

        match result {
            Ok(()) => {
                let selected = client
                    .connected_endpoint()
                    .map(AuthServerConfig::from_resolved)
                    .expect("успешный connect сохраняет endpoint");
                attempts.push(AuthConnectAttempt {
                    endpoint: configured.clone(),
                    failure: None,
                });
                connected = Some(selected);
                break;
            }
            Err(failure) => attempts.push(AuthConnectAttempt {
                endpoint: configured.clone(),
                failure: Some(failure),
            }),
        }
    }

    (client, attempts, connected)
}

/// Owned Linux-форма исходного `GameThreadFunc` RVA 0x00014680.
///
/// Единственный `CGame` всегда проходит `Release` перед уничтожением, включая
/// фатальный partial `Init` и ошибку runtime turn. `shutdown` заменяет внешний
/// `g_bGameThreadExit`; он не подменяет его полем `m_bExit` внутри владельца.
pub(crate) async fn game_thread_func<Shutdown>(
    runtime_directory: &Path,
    area_id: i32,
    shutdown: Shutdown,
) -> LoginGameThreadReport
where
    Shutdown: Future<Output = ()>,
{
    let mut game = CGame::new(area_id);
    let mut auth_manager = AuthManager::new();
    let initialization = game.initialize(runtime_directory, &mut auth_manager).await;
    let mut completed_turns = 0_u64;
    let runtime_error = if initialization.is_ok() {
        tokio::pin!(shutdown);
        loop {
            let turn = async {
                let _network = game
                    .run_network_turn()
                    .await
                    .map_err(LoginRuntimeError::Network)?;
                game.main_loop_turn(&mut auth_manager)
                    .await
                    .map_err(LoginRuntimeError::MainLoop)
            };
            let outcome = tokio::select! {
                biased;
                () = &mut shutdown => break None,
                result = turn => result,
            };
            match outcome {
                Ok(LoginMainLoopOutcome::Continue { .. }) => {
                    completed_turns = completed_turns.wrapping_add(1);
                }
                Ok(LoginMainLoopOutcome::Exit { .. }) => {
                    completed_turns = completed_turns.wrapping_add(1);
                    break None;
                }
                Err(error) => break Some(error),
            }
        }
    } else {
        None
    };
    let release = game.release().await;
    drop(game);
    LoginGameThreadReport {
        initialization,
        completed_turns,
        runtime_error,
        release,
    }
}
