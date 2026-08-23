//! Владелец союза исторического `WorldServer`.
//!
//! Достигнутая save-часть `CUnion`, `CUnion::Save` RVA `0x000C1A00`,
//! `CUnion::CloneSaveData` RVA `0x000C6380`, `SetChangeData` RVA `0x000C17D0`
//! и `CUnion::IsMember` RVA
//! `0x000BD840`, `CUnion::GetPlayerHeader` RVA `0x000C6320`, а также
//! `CUnion::DelMember` RVA `0x000C2B30`, inert virtual-ы
//! `DubAndSetJobLvl/EditLeaveWord/OperatorTax/SetControbuter/Upgrade` RVA
//! `0x000C18B0/0x000C1EC0/0x000C1ED0/0x000C1F20/0x000C1F30`,
//! `GetEstablishedTime` RVA `0x000C1F00`, `Exit` RVA `0x000C3DD0`,
//! `Disband` RVA `0x000C43F0`, `FireOut` RVA `0x000C49D0`, `Demise` RVA
//! `0x000C5060` и
//! compiler-owned destructor RVA
//! `0x000C1F40`, обе перегрузки `GetMemberList` RVA
//! `0x000C1BB0/0x000C2710`, `IsUsingPV/SetMemPV/AbolishMemPV` RVA
//! `0x000C1D00/0x000C1D70/0x000C1DD0` и обе `CheckOperValidate` RVA
//! `0x000C18D0/0x000C1970`, `ClearApplyList` RVA `0x000C1840`, empty
//! enemy-set getter-ы RVA `0x000C1C50`,
//! `IsOwnedCity/IsEnemyFaction/GetOwnedCities/IsHaveEnymyFaction/
//! IsHaveCityEnemyFaction` RVA
//! `0x000C2340/0x000C23A0/0x000C2650/0x000C2810/0x000C2870`, обе
//! `AddOwnedCity`, `DelOwnedCity/ClearOwnedCity/SetOwnedCity` RVA
//! `0x000C2400/0x000C2490/0x000C2510/0x000C2570/0x000C25F0`,
//! `ClearEnemyFation/GetEnemyLeaderOrgnizingID` RVA
//! `0x000C28D0/0x000C2950` и три victor fan-out RVA
//! `0x000C29B0/0x000C2A30/0x000C2AB0`, `UpdatePlayerFactionInfo` RVA
//! `0x000C5770` и `UpdateEnemyFactionToClient/
//! UpdateCityWarEnemyFactionToClient/UpdateOwnedCityToClient` RVA
//! `0x000C5FF0/0x000C60D0/0x000C61B0` и `SendInfoToAllMember` RVA
//! `0x000C6290`, `DeleteOrgaToClient` RVA `0x000C5D20` и
//! `UpdateMemberInfoToClient` RVA `0x000C5840`, `AddMembersToByteArray` RVA
//! `0x000C21D0`, `AddToByteArray` RVA `0x000C6590`, public-конструктор RVA
//! `0x000C64D0`, `Initial` RVA `0x000C1FE0` и `AddFaction` RVA
//! `0x000C3170`, `ApplyForJoin` RVA `0x000C2B90`, его локальные
//! `PlayerApplyForJoinConfeder` constructor/`DoAsyncCall`/`OnAsyncCallback`
//! RVA `0x000C19C0/0x000C1A70/0x000C2EA0`, `DoJoin` RVA `0x000C66A0` и
//! `Invite` RVA `0x000C3660`, его локальные
//! `InviteJoinConfeder` constructor/`DoAsyncCall`/`OnAsyncCallback` RVA
//! `0x000C1870/0x000C39B0/0x000C3B10`,
//! `Disband/FireOut` RVA `0x000C43F0/0x000C49D0` —
//! реализованы в Rust. Точная пара доказательных артефактов:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h`
//! и `union.cpp:559,615,621,636,680,785,1442`.
//!
//! Точный PDB задаёт старый `CUnion` размером `0x50`: signed ID `+0x4`, имя
//! `+0x8`, signed master ID `+0x24`, ordered member-map `+0x28`, `tagTime`
//! `+0x38` и signed dirty-mask `+0x48`. `CloneSaveData` возвращает null при
//! нулевой mask; иначе создаёт отдельного `CUnion`, копирует ровно эти поля и
//! весь member-map в signed key-order. `m_ApplyPerson` clone не назначает:
//! save-проекция хранит `None`, а live `Initial` — доказанный `Some(0)`.
//! `m_dwLastDemiseTime` clone не назначает и DB-owner не читает. При этом
//! public-конструктор и `Initial` машинно тоже не инициализируют offset `+0x4C`,
//! хотя `Demise` читает его до первой записи. Safe Rust устраняет этот
//! внутренний uninitialized-read: live и DB-reached owner начинают с `0`, как
//! явно делал поздний Linux-донор; поле остаётся transient и не считается
//! частью save-контракта.
//!
//! `Vec<u8>`, `BTreeMap` и обычный `Clone/Drop` заменяют только MSVC string,
//! tree и `new/delete`. Rust-layout не выдаётся за старый ABI. Узкий
//! `from_reached_save_state` создаёт save-проекцию, а `from_live_state`
//! выполняет public-конструктор и `Initial` с явными callbacks. Private default
//! constructor материализован как общий safe baseline: старые string/map/time
//! создаются, а неинициализированные data-поля нормализуются в нули/`None`.
//! Это исправляет внутренний lifetime/initialization-дефект без выдумывания
//! наблюдаемого состояния: каждая достигнутая фабрика затем явно назначает
//! свой ID, время, members и transient `apply_person`.
//!
//! PDB публикует `CUnion::IsMember(long)` на том же RVA `0x000BD840`, что и
//! faction-вариант: identical-code folding допустим, потому что `m_lID` и
//! `m_Members` у обоих concrete owners имеют одинаковые offsets `+0x4/+0x28`.
//! Exact EXE ищет входной signed ID и возвращает собственный ID либо `0`.
//!
//! `DelMember` не читает union receiver. Машинный проход
//! `0x004C2B30..0x004C2B8B` подтверждает единый входной faction ID для `find`
//! и `operator[]`, вопреки повреждённому RAW имени stack-slot. Поэтому lookup
//! и два faction-callback-а материализованы у владельца map как
//! `COrganizingCtrl::detach_union_member`; это технический перенос singleton-
//! доступа, а не изменение контракта `CUnion`.
//! `GetPlayerHeader` при неположительном master-faction ID возвращает `0`.
//! Иначе он ищет master-faction в controller map и вызывает именно
//! `GetMasterID`, не рекурсивный `GetPlayerHeader`; miss/null map-value также
//! дают `0`. Недостигнутый Rust master ID у найденной concrete faction остаётся
//! typed-блокировкой caller-а, а не выдуманным нулём.
//! Пять inert virtual-ов машинно не читают receiver или аргументы: title/job
//! возвращает `true`, edit/tax/upgrade — `false`, contributor-mutator ничего не
//! делает. Эти результаты сохранены буквально, а не заменены предполагаемой
//! faction-логикой. `GetEstablishedTime` возвращал const reference на точные
//! `0x10` bytes; Copy-значение Rust отделяет lifetime без изменения данных.
//! Destructor до следующего PDB-символа освобождает только member-map, string и
//! base storage. Обычный `Drop` Rust-полей заменяет MSVC tree/string cleanup;
//! ручного destructor callback-а нет.
//! `Demise` сохраняет машинный порядок gates и mutations, включая странные
//! `member count < 6` и сравнение old player ID с new faction ID. Exact ASM
//! исправляет повреждённый RAW: после переноса обе faction публикуются как
//! `OP_Update`, а `WS0282` получает old name/ID и new name/ID. Два вызова
//! `timeGetTime`, wrapping cooldown `10_800_000`, notices, fan-out, dirty-mask
//! `3` и общий player refresh сохранены. Неограниченные `strcpy/sprintf`
//! заменены фиксированными typed-границами; повреждённый master-key, который
//! старый `operator[]` превращал бы в неинициализированный member, также
//! останавливается до UB.
//! Обе `GetMemberList` сначала очищают caller-list, затем проходят member-map в
//! signed key-order. ID-вариант возвращает ключи; organizing-вариант для
//! каждого положительного ключа выполняет nullable faction lookup, пропускает
//! miss/null и не добавляет один pointer дважды. Свежий `Vec` заменяет только
//! list clear/nodes, а lookup передаётся явно вместо singleton-а.
//! Три purview-owner-а проверяют индекс `0..10` до map lookup. Query принимает
//! только точный `PST_Permit`; grant меняет лишь `PST_No`, revoke — любое
//! ненулевое состояние. Общий typed-результат перенесён к owner-у
//! `COrganizing::tagMemInfo`, но concrete map и переходы остаются в `CUnion`.
//! Validation сначала преобразует player master-а через controller-wide scan в
//! положительный faction ID. Простая перегрузка требует union-membership и
//! право; target-перегрузка дополнительно отвергает literal equality входных
//! ID, нулевой target и master-faction союза, затем требует членство обоих и
//! ровно `Permit` у manager при отсутствии `Permit` у target.
//! Оба enemy-set getter-а folded в один owner, который не читает receiver и
//! всегда строит пустой set. Остальные read-only proxy используют только
//! положительный master-faction ID и nullable controller lookup; miss даёт
//! `0`, `false` либо пустой owned-city list. Старый miss `GetOwnedCities`
//! возвращал ссылку на уже уничтоженный локальный list. Safe Rust исправляет
//! этот чистый lifetime/UB-дефект, возвращая owned snapshot либо пустой
//! `VecDeque`, не меняя задуманное значение нормальных ветвей.
//! Owned-city fan-out проходит member-map в signed key-order, пропускает
//! неположительные ID и nullable faction lookup. Одиночное и списочное
//! добавление вызывают каждую найденную faction; clear делает то же и всегда
//! возвращал `true`; set обращается только к положительной найденной
//! master-faction. Exact ASM `0x004C2510..0x004C256C` отдельно подтверждает
//! внешний legacy-баг: `DelOwnedCity(region)` не читает `region`, вызывает у
//! master-фракции безаргументный slot `ClearOwnedCity` и возвращает `true`
//! только при найденном ненулевом pointer. Старый Linux-донор исправлял это на
//! обычный delete, но Rust сохраняет машинное поведение целевой версии явно.
//! Enemy clear и три victor-owner-а снова проходят member-map в signed order,
//! пропускают неположительные ID и miss/null faction. Exact vtable slots
//! `+0xC8/+0x130/+0x134/+0x138` подтверждают concrete clear и порядок
//! defence/offense/village счётчиков. Enemy-leader proxy использует только
//! положительную master-faction и slot `+0xC4`; exact `CFaction`-реализация
//! этого slot-а возвращает literal `0`.
//! `UpdatePlayerFactionInfo(0)` проходит все положительные member-faction ID;
//! положительный аргумент выбирает ровно этот ID без membership-check, а
//! отрицательный не вызывает ничего. Каждый найденный target получает
//! concrete `CFaction::UpdatePlayerFactionInfo(0)`; lookup miss/null тихо
//! пропускается.
//! Три client-snapshot wrapper-а выбирают targets по тому же правилу и
//! полностью игнорируют входной `operation`: найденной faction всегда
//! передаётся `(player=0, OP_Update=2)`. Rust вызывает уже восстановленные
//! concrete snapshot-owner-ы напрямую и сохраняет результаты всех send;
//! owned-city serialization block останавливает обход после выполненного
//! prefix-а вместо выдуманного продолжения после safe-границы.
//! `SendInfoToAllMember` всегда проходит все положительные member-faction ID.
//! Exact ASM `0x004C62B7..0x004C62FA` подтверждает, что повреждённый RAW
//! `find(&param_3)` на самом деле использует сохранённый текущий member ID и для
//! `find`, и для `operator[]`; найденная faction получает исходные text/title,
//! information type и color без перестановки. Online-фильтра у union-owner-а
//! нет: он остаётся внутри уже восстановленной faction-рассылки.
//! `DeleteOrgaToClient` при положительном faction ID обходит только найденную
//! faction, а при любом `<= 0` — все положительные union member ID. Внутри
//! каждой faction member-player-ы идут в signed order; допускаются только
//! online player с ненулевым GameServer ID и полученными faction data. Wire
//! `0x7FE05` содержит recipient player ID, затем именно union ID.
//! `UpdateMemberInfoToClient` RVA `0x000C5840` использует `0x7FE0E`.
//! Delete-ветвь рассылает только `recipient/OP_Delete/target faction`; любой
//! non-delete сначала требует target key в union map, обновляет cached level
//! через target faction и снимает один local `tagTime`. Перед каждым реальным
//! send level ещё раз разрешается через `tagMemInfo::lID`, после чего wire идёт
//! как `recipient/operator/target/name/level/occupation/job/title/0x2C PV/`
//! пустая region C-строка/тот же `0x10` time. Stored region, contribute и
//! LastOnlineTime этот union-owner не отправляет.
//! Оба snapshot-сериализатора машинно возвращают literal `true` и используют
//! `Vec<u8>` только как безопасную замену MSVC vector. Полный union snapshot
//! пишет `id/name/master id/master faction name`, затем member snapshot.
//! Member map идёт в signed key-order; каждый record пишет `id/job/title/PV`,
//! обновляет cached faction name/level через именно `tagMemInfo::lID`, затем
//! пишет `level/occupation/name/contribute`, пустую region C-строку и сохранённое
//! `LastOnlineTime`. Исходный безграничный `strcpy` в 32-байтовое member name
//! заменён typed-остановкой только для недопустимого переполнения; временный
//! 256-байтовый master-name buffer устранён прямой C-string сериализацией.
//! Public-конструктор назначает ID, входные name/master ID, снимает отдельное
//! established time и вызывает `Initial`. Тот сбрасывает apply-person, снимает
//! member time, строит master record с job `1` и правами
//! `2,0,0,2,2,0,0,0,0,0,0`, а title получает по внешнему StringTable ID
//! `WS0154`. Найденная master-faction заменяет union name и задаёт level; EXE,
//! в отличие от Linux-донора, не копирует её name в member record. Старые
//! неинициализированные name/region bytes заменены нулями без добавления этой
//! отсутствующей копии. Затем record вставляется под master key, faction
//! получает union ID, dirty-mask сбрасывается и обновляются её online players.
//! `AddFaction` снимает member time и разрешает `WS0268` до faction lookup.
//! Miss/non-positive ID форматирует `WS0269(faction, union)` и пишет `war`, не
//! меняя state. Success создаёт job `99`, права
//! `0,2,0,0,0,0,0,0,0,0,0`, имя/level найденной faction и перезаписывает
//! существующий key без duplicate-gate. Затем строго идут superior assignment,
//! `RefreshOwnCityInfo`, `WS0270(faction name, union name)` с title `WS0188`
//! всем union members и player refresh только добавленной faction. Dirty-bit
//! этот owner не ставит. Fixed name/title сохраняют layout с typed overflow;
//! внутренние 256-байтовые `_sprintf` buffers заменены владеющим formatter-ом.
//! `DoJoin` до чтения заявки отвергает `member_count >= 50`: exact data-word
//! EXE исправляет устаревший лимит `5` Linux-донора. Manager/applicant должны
//! быть ненулевыми, а applicant — совпасть с единственным `m_ApplyPerson`;
//! approve/time не читаются. После совпадения заявка немедленно сбрасывается и
//! остаётся сброшенной при уже последующих membership/authority отказах. Затем
//! строго идут `IsFreeFaction`, право `PV_ConMem`, `AddFaction`, initial union
//! snapshot новой faction, `UpdateMemberInfoToClient(OP_Add)` и dirty bit `2`.
//! Bool snapshot исходно игнорируется, что сохранено отдельно от typed safe-
//! блокировок повреждённого state; внешние эффекты до такой границы не
//! откатываются.
//! `FireOut` сначала проверяет standard/city enemy proxy master-faction и
//! сообщает manager-у соответственно `WS0271/WS0121` либо `WS0272/WS0121`.
//! Затем target-validation требует право `PV_FireOut=4`. Success форматирует
//! `WS0277(target faction name, union name)` с title `WS0188` и color
//! `0x0087A238` всем ещё полным members, вызывает `DelMember`, удаляет target
//! из union-map, последовательно делает `DeleteOrgaToClient(target)`,
//! `UpdateMemberInfoToClient(target, OP_Delete)`, dirty bit `2` и refresh
//! player-ов target faction. Только после этого nullable target/master faction
//! lookup может записать `war` строку `WS0278(target name, target ID, master
//! name, union ID)`. Rust заменяет `_sprintf` на внешний formatter и сохраняет
//! уже выполненные эффекты в typed-блоках вместо продолжения после malformed
//! fixed string или callback-state. Exact ASM `0x004C49D0..0x004C5050` также
//! подтверждает, что erase использует сохранённый target ID; повреждённое имя
//! stack-slot в RAW не является отдельным аргументом.
//! `Disband` требует `PV_Disband`, повторно блокирует обе войны через единый
//! `WS0275/WS0121`, затем публикует `WS0276(union name)`/`WS0188`, пишет ту же
//! строку в `war`, вызывает `DelMember` для snapshot всех keys в signed order,
//! сбрасывает apply-person и удаляет union-state у клиентов всех member-
//! фракций. Member-map сам concrete owner не очищает; его целиком уничтожает
//! последующий controller delete. Snapshot `Vec` заменяет только небезопасную
//! итерацию MSVC map поверх reentrant faction callbacks.
//! `Exit` проверяет войны раньше права `PV_Exit`, используя отдельные
//! `WS0246/WS0247` и общий title `WS0121`. После успешной проверки он повторно
//! разрешает master-faction игрока, публикует `WS0273(leaving faction name)`
//! всем ещё полным members и до мутаций при двух найденных faction-owner-ах
//! пишет `WS0274(leaving name, leaving ID, union ID, master faction name)` в
//! `war`. Затем строго идут `DelMember`, erase, `DeleteOrgaToClient`,
//! `UpdateMemberInfoToClient(OP_Delete)`, dirty bit `2` и player refresh.
//! Exact ASM `0x004C3DD0..0x004C43E9` подтверждает четыре log-аргумента:
//! push master-name остаётся на стеке через virtual `GetID`, после чего идут
//! union ID, leaving ID и leaving-name. Rust не повторяет ошибку RAW, который
//! потерял последний аргумент из-за неверного восстановления thiscall/cdecl.
//! `ClearApplyList` сначала через тот же virtual validation требует у manager-а
//! право `PV_ConMem`; отказ не меняет заявку. Успех записывает literal `0` в
//! `m_ApplyPerson` и возвращает `true`. Exact ASM
//! `0x004C1840..0x004C1866` подтверждает аргумент, slot `+0x1AC`, purview `3`
//! и отсутствие других эффектов; безаргументный helper Linux-донора не является
//! контрактом этой функции.
//! `ApplyForJoin` отвергает уже pending/нулевую/зарезервированную/состоящую в
//! union faction, затем требует оба faction-owner-а и online master-player.
//! Offline master получает applicant-header notice `WS0264/WS0193`. Только
//! после online lookup inline `GetNetExID` расходует process ID; затем exact
//! лимит `50` может вернуть `WS0265/WS0193`. Success сначала записывает
//! applicant в `m_ApplyPerson`, затем добавляет его в establishment-list и
//! запускает session `1000` с confirmation kind `2`, master player ID, полным
//! applicant faction name, union/applicant ID и выделенным NetEx ID. Второй и
//! третий входные `long` не читаются. Existing `CNetSessionManager` остаётся
//! универсальным transport-owner-ом; Rust trait передаёт ему точный доменный
//! request, а ошибки безопасной session-границы сохраняют уже выполненные
//! assignment/list эффекты без выдуманного rollback. Локальный callback-owner
//! хранит только union/applicant ID; `Box/Arc/Drop` заменяют два interface-
//! под-объекта и ручной `Release`. `DoAsyncCall` строит exact `0x7FE17`:
//! master player ID, literal confirmation kind `2`, applicant name как
//! C-строку, signed session ID и второй cookie, после чего выбирает GameServer
//! по master player и игнорирует send result. Result читает decision; approval
//! вызывает `DoJoin(GetPlayerHeader(), applicant, 1, time)`, отказ сообщает
//! именно faction ID через `WS0266/WS0193`, а timeout/non-result не сообщает
//! ничего.
//! Exact ingress `0x004A74C6..0x004A7509` читает только однобайтовый result и
//! старый manager передаёт адрес своего 4-байтового stack-slot. Поэтому
//! последующее копирование `0x10` bytes в callback-е захватывало соседний
//! технический стек; `DoJoin` этот time не читает. Rust устраняет внутреннее
//! out-of-bounds чтение и передаёт нулевой технический `TagTimeValue`, не меняя
//! ни одного достигнутого эффекта.
//! После normal return найденный union сбрасывает заявку и list-owner удаляет
//! только первое совпадение; union miss всё равно очищает list. Неверный erased
//! Rust payload и недостигнутое malformed state останавливаются typed-блоком
//! без чтения чужой памяти и без выдуманного terminal continuation.
//! `Invite` использует тот же pending/reservation/member-limit/session-контракт,
//! но направляет `0x7FE17` master-у приглашённой faction с kind `1` и именем
//! приглашающей faction. Exact `OnPlayerInviteFaction` передаёт первым
//! аргументом faction ID, а `Invite` передаёт его в `CheckOperValidate`, который
//! интерпретирует значение как master-player ID; странный gate сохранён как
//! наблюдаемая семантика. Offline и limit notices идут player-header первой
//! faction как `WS0264/WS0265 + WS0193`; NetEx ID расходуется до limit gate.
//! Invitation callback на approve вызывает `DoJoin` с тем же faction ID в
//! manager-позиции и нулевым техническим `TagTimeValue`: ingress передаёт лишь
//! 4-байтовый result, а машинное чтение следующих `0x10` stack bytes является
//! тем же неиспользуемым OOB, что у apply-callback. Deny отправляет
//! `WS0245/WS0193` через virtual player-header приглашающей faction;
//! terminal сбрасывает pending и удаляет первую reservation приглашённой.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use super::faction::{
    current_local_member_time, FactionEnemyDelivery, FactionInitialPropertyBlock,
    FactionMemberInfoReport, FactionMemberInfoRequest, FactionOwnedCityDelivery,
    FactionOwnedCityRefreshBlock, FactionOwnedCityRefreshReport,
    FactionOwnedCityUpdateBuildError, FactionPropertyDelivery,
    FactionSuperiorOrganizingBlock, OwnedCityMutationBuildError,
};
use super::organizing::{
    EOperator, EPurview, EPurviewOwnState, MemberPurviewMutation, TagMemInfo, TagTimeValue,
    UnterminatedMemberField,
};
use super::organizingparam::COrganizingParam;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::netsession::{
    NetSessionAsyncResult, NetSessionAsyncResultKind, NetSessionEndpoint,
};
use crate::public::netsessionmanager::{
    CNetSessionManager, CreatedNetSession, NetSessionCreateBlock, NetSessionManagerBeginBlock,
    NetSessionSetCallbackBlock,
};
use crate::worldserver::worldserver::game::CGame;

const DELETE_UNION_ORGANIZING_MESSAGE_TYPE: i32 = 0x7FE05;
const UNION_MEMBER_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0E;
const UNION_APPLICATION_CONFIRMATION_MESSAGE_TYPE: i32 = 0x7FE17;
const MAX_UNION_MEMBER_COUNT: i32 = 50;
const UNION_DEMISE_MEMBER_LIMIT: usize = 6;
const UNION_DEMISE_COOLDOWN_MS: u32 = 10_800_000;

/// Узкая read-only граница controller-wide `IsFactionMaster`.
pub(crate) trait UnionOperatorValidationContext {
    type Block;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block>;
}

/// Узкая read-only граница master-faction proxy без singleton-а.
pub(crate) trait UnionMasterFactionQueryContext {
    fn faction_is_owned_city(&self, faction_id: i32, region_id: i32) -> Option<i32>;

    fn faction_is_enemy_faction(&self, faction_id: i32, enemy_id: i32) -> Option<i32>;

    fn faction_owned_cities(&self, faction_id: i32) -> Option<VecDeque<i32>>;

    fn faction_has_enemy(&self, faction_id: i32) -> Option<bool>;

    fn faction_has_city_war_enemy(&self, faction_id: i32) -> Option<bool>;

    fn faction_enemy_leader_organizing_id(&self, faction_id: i32) -> Option<i32>;
}

/// Узкая mutable-граница faction-map для owned-city virtual dispatch.
pub(crate) trait UnionOwnedCityMutationContext {
    fn faction_add_owned_city(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_add_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_ids: &VecDeque<i32>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_clear_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_set_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_ids: &VecDeque<i32>,
    ) -> Result<bool, OwnedCityMutationBuildError>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCityFanoutReport {
    pub(crate) invoked_faction_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCityMutationBlock {
    pub(crate) faction_id: i32,
    pub(crate) completed_faction_ids: Vec<i32>,
    pub(crate) source: OwnedCityMutationBuildError,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCityBooleanMutationReport {
    pub(crate) legacy_result: bool,
    pub(crate) invoked_faction_ids: Vec<i32>,
}

/// Узкая mutable-граница standard enemy и victor virtual dispatch.
pub(crate) trait UnionFactionStateMutationContext {
    fn faction_clear_enemy_factions(&mut self, faction_id: i32) -> bool;

    fn faction_clear_city_war_enemy_factions(&mut self, faction_id: i32) -> bool;

    fn faction_add_defence_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;

    fn faction_add_offense_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;

    fn faction_add_village_war_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionFactionFanoutReport {
    pub(crate) invoked_faction_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionVictorFactionReport {
    pub(crate) faction_id: i32,
    pub(crate) deliveries: Vec<FactionPropertyDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionVictorFanoutReport {
    pub(crate) factions: Vec<UnionVictorFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionVictorMutationBlock {
    pub(crate) faction_id: i32,
    pub(crate) completed_factions: Vec<UnionVictorFactionReport>,
    pub(crate) source: FactionInitialPropertyBlock,
}

/// Read-only faction callback для обновления online player-состояния.
pub(crate) trait UnionPlayerRefreshContext {
    fn faction_update_player_info(
        &self,
        faction_id: i32,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Option<Vec<i32>>;
}

/// Mutable callback `Initial` для назначения union ID master-faction.
pub(crate) trait UnionInitialMutationContext {
    fn faction_set_superior_organizing(
        &mut self,
        faction_id: i32,
        union_id: i32,
        union_master_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<bool, FactionSuperiorOrganizingBlock>;
}

/// Faction-side callbacks полного `CUnion::AddFaction` owner-а.
pub(crate) trait UnionFactionJoinContext:
    UnionFactionMemberContext
    + UnionInitialMutationContext
    + UnionPlayerRefreshContext
    + UnionSendInfoContext
{
    fn faction_refresh_owned_city_info(
        &self,
        faction_id: i32,
        refresh_owned_city: &mut dyn FnMut(i32, i32, i32, Option<u8>),
    ) -> Result<Option<FactionOwnedCityRefreshReport>, FactionOwnedCityRefreshBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

/// StringTable, `war` log и внешние эффекты, не принадлежащие union state.
pub(crate) trait UnionAddFactionEffects {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;

    fn put_war_log(&mut self, text: &[u8]);

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    );

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
}

/// Controller callbacks, которые `CUnion::DoJoin` вызывал через singleton.
pub(crate) trait UnionDoJoinContext {
    type FreeFactionBlock;
    type InitialSnapshotBlock;

    fn union_id_for_joining_faction(
        &self,
        faction_id: i32,
    ) -> Result<i32, Self::FreeFactionBlock>;

    fn add_current_union_to_client_by_faction_id(
        &mut self,
        game: &CGame,
        union: &mut CUnion,
        faction_id: i32,
    ) -> Result<bool, Self::InitialSnapshotBlock>;
}

/// Controller callbacks, которые `CUnion::FireOut` вызывал через singleton.
pub(crate) trait UnionFireOutContext:
    UnionOperatorValidationContext
    + UnionMasterFactionQueryContext
    + UnionSendInfoContext
    + UnionFactionMemberContext
    + UnionPlayerRefreshContext
{
    type DetachBlock;
    type DetachOutcome;

    fn detach_union_member_for_fire_out(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<Self::DetachOutcome, Self::DetachBlock>;
}

/// Controller callbacks, которые `CUnion::Demise` вызывал через singleton.
pub(crate) trait UnionDemiseContext:
    UnionOperatorValidationContext
    + UnionMasterFactionQueryContext
    + UnionSendInfoContext
    + UnionFactionMemberContext
    + UnionPlayerRefreshContext
    + UnionMasterProjectionMutationContext
{
}

impl<T> UnionDemiseContext for T where
    T: UnionOperatorValidationContext
        + UnionMasterFactionQueryContext
        + UnionSendInfoContext
        + UnionFactionMemberContext
        + UnionPlayerRefreshContext
        + UnionMasterProjectionMutationContext
{
}

/// Поддерживает faction-local проекцию master ID после смены главы союза.
pub(crate) trait UnionMasterProjectionMutationContext {
    fn set_union_master_projection(
        &mut self,
        faction_ids: &[i32],
        union_master_id: i32,
    );
}

/// Controller callbacks, которые `CUnion::Exit` вызывал через singleton.
pub(crate) trait UnionExitContext:
    UnionOperatorValidationContext
    + UnionMasterFactionQueryContext
    + UnionSendInfoContext
    + UnionFactionMemberContext
    + UnionPlayerRefreshContext
{
    type DetachBlock;
    type DetachOutcome;

    fn detach_union_member_for_exit(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<Self::DetachOutcome, Self::DetachBlock>;
}

/// StringTable, organizing-info и `war` log вне union state.
pub(crate) trait UnionFireOutEffects {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);

    fn put_war_log(&mut self, text: &[u8]);

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    );
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionApplicationFactionSnapshot {
    pub(crate) faction_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) player_header: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnionApplicationFactionBlock {
    pub(crate) faction_id: i32,
}

/// Controller state, который синхронная часть `ApplyForJoin` читает и резервирует.
pub(crate) trait UnionApplyForJoinContext {
    fn union_application_is_reserved(&self, faction_id: i32) -> bool;

    fn union_application_faction(
        &self,
        faction_id: i32,
    ) -> Result<Option<UnionApplicationFactionSnapshot>, UnionApplicationFactionBlock>;

    fn reserve_union_application(&mut self, faction_id: i32);
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionApplicationSessionRequest {
    pub(crate) union_id: i32,
    pub(crate) applicant_faction_id: i32,
    pub(crate) recipient_player_id: i32,
    pub(crate) requested_session_id: i32,
    pub(crate) timeout_ticks: u32,
    pub(crate) confirmation_kind: i32,
    pub(crate) applicant_faction_name: Vec<u8>,
}

/// Нормальные ветви локального `OnAsyncCallback` после typed decode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionApplicationTerminal {
    Approved,
    Denied,
    NonResult { kind: NetSessionAsyncResultKind },
}

/// Ошибка erased payload на границе, где старый callback читал raw pointer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionApplicationEndpointBlock {
    BeginPayloadType,
    ResultPayloadType,
}

/// Живые эффекты локального callback-owner-а, требующие interior synchronization.
pub(crate) trait UnionApplicationSessionRuntime: Send + Sync {
    /// Маршрутизирует сообщение через GameServer текущего online player-а.
    fn send_union_application_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    );

    /// Немедленно выполняет terminal mutation organizing owner-а.
    fn finish_union_application(
        &self,
        union_id: i32,
        applicant_faction_id: i32,
        terminal: UnionApplicationTerminal,
    );

    /// Фиксирует safe-остановку вместо чтения значения неверного Rust-типа.
    fn block_union_application_endpoint(&self, block: UnionApplicationEndpointBlock);
}

/// Safe owner локального `PlayerApplyForJoinConfeder` с двумя старыми interface.
pub(crate) struct PlayerApplyForJoinConfeder {
    union_id: i32,
    applicant_faction_id: i32,
    runtime: Arc<dyn UnionApplicationSessionRuntime>,
}

impl PlayerApplyForJoinConfeder {
    pub(crate) fn new(
        union_id: i32,
        applicant_faction_id: i32,
        runtime: Arc<dyn UnionApplicationSessionRuntime>,
    ) -> Self {
        Self {
            union_id,
            applicant_faction_id,
            runtime,
        }
    }
}

impl NetSessionEndpoint for PlayerApplyForJoinConfeder {
    fn do_async_call(&self, session_id: i64, cookie_second: i32, payload: &dyn Any) {
        let Some(request) = payload.downcast_ref::<UnionApplicationSessionRequest>() else {
            self.runtime.block_union_application_endpoint(
                UnionApplicationEndpointBlock::BeginPayloadType,
            );
            return;
        };

        let mut message = CMessage::new(UNION_APPLICATION_CONFIRMATION_MESSAGE_TYPE);
        message.base_mut().add_long(request.recipient_player_id);
        message.base_mut().add_long(request.confirmation_kind);
        message
            .base_mut()
            .add(legacy_c_string_visible_bytes(&request.applicant_faction_name));
        message.base_mut().add_byte(0);
        message.base_mut().add_long64(session_id);
        message.base_mut().add_long(cookie_second);
        self.runtime
            .send_union_application_confirmation(request.recipient_player_id, &message);
    }

    fn on_async_callback(&self, result: NetSessionAsyncResult<'_>) {
        let terminal = if result.kind == NetSessionAsyncResultKind::Result {
            let Some(decision) = result
                .payload
                .and_then(|payload| payload.downcast_ref::<i32>())
            else {
                self.runtime.block_union_application_endpoint(
                    UnionApplicationEndpointBlock::ResultPayloadType,
                );
                return;
            };
            if *decision == 1 {
                UnionApplicationTerminal::Approved
            } else {
                UnionApplicationTerminal::Denied
            }
        } else {
            UnionApplicationTerminal::NonResult { kind: result.kind }
        };
        self.runtime.finish_union_application(
            self.union_id,
            self.applicant_faction_id,
            terminal,
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnionApplicationSessionReport {
    pub(crate) session: CreatedNetSession,
}

pub(crate) enum UnionApplicationSessionBlock {
    Create(NetSessionCreateBlock),
    SetCallback {
        session: CreatedNetSession,
        source: NetSessionSetCallbackBlock,
    },
    Begin {
        session: CreatedNetSession,
        source: NetSessionManagerBeginBlock,
    },
}

impl std::fmt::Debug for UnionApplicationSessionBlock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(source) => formatter.debug_tuple("Create").field(source).finish(),
            Self::SetCallback { session, source } => {
                let source = match source {
                    NetSessionSetCallbackBlock::SessionNotFound { .. } => "SessionNotFound",
                    NetSessionSetCallbackBlock::AlreadyAssigned(_) => "AlreadyAssigned",
                };
                formatter
                    .debug_struct("SetCallback")
                    .field("session", session)
                    .field("source", &source)
                    .finish()
            }
            Self::Begin { session, source } => formatter
                .debug_struct("Begin")
                .field("session", session)
                .field("source", source)
                .finish(),
        }
    }
}

/// Связывает доменный request с уже восстановленным session manager в exact order.
pub(crate) fn begin_union_application_session(
    manager: &CNetSessionManager,
    request: UnionApplicationSessionRequest,
    runtime: Arc<dyn UnionApplicationSessionRuntime>,
    random: impl FnMut(i32) -> i32,
) -> Result<UnionApplicationSessionReport, UnionApplicationSessionBlock> {
    let session = manager
        .create_session(
            request.recipient_player_id,
            request.requested_session_id,
            random,
        )
        .map_err(UnionApplicationSessionBlock::Create)?;
    let endpoint = Box::new(PlayerApplyForJoinConfeder::new(
        request.union_id,
        request.applicant_faction_id,
        runtime,
    ));
    manager
        .set_callback_handle(session.id, endpoint)
        .map_err(|source| UnionApplicationSessionBlock::SetCallback { session, source })?;
    manager
        .beging(session.id, request.timeout_ticks, &request)
        .map_err(|source| UnionApplicationSessionBlock::Begin { session, source })?;
    Ok(UnionApplicationSessionReport { session })
}

/// StringTable, client notice и адаптер уже восстановленного `CNetSessionManager`.
pub(crate) trait UnionApplyForJoinEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);

    fn begin_union_application_session(
        &mut self,
        request: UnionApplicationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionInvitationSessionRequest {
    pub(crate) union_id: i32,
    pub(crate) inviter_faction_id: i32,
    pub(crate) invited_faction_id: i32,
    pub(crate) recipient_player_id: i32,
    pub(crate) requested_session_id: i32,
    pub(crate) timeout_ticks: u32,
    pub(crate) inviter_faction_name: Vec<u8>,
}

pub(crate) trait UnionInvitationSessionRuntime: Send + Sync {
    fn send_union_invitation_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    );

    fn finish_union_invitation(
        &self,
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        terminal: UnionApplicationTerminal,
    );

    fn block_union_invitation_endpoint(&self, block: UnionApplicationEndpointBlock);
}

pub(crate) struct InviteJoinConfeder {
    union_id: i32,
    inviter_faction_id: i32,
    invited_faction_id: i32,
    runtime: Arc<dyn UnionInvitationSessionRuntime>,
}

impl NetSessionEndpoint for InviteJoinConfeder {
    fn do_async_call(&self, session_id: i64, cookie_second: i32, payload: &dyn Any) {
        let Some(request) = payload.downcast_ref::<UnionInvitationSessionRequest>() else {
            self.runtime.block_union_invitation_endpoint(
                UnionApplicationEndpointBlock::BeginPayloadType,
            );
            return;
        };

        let mut message = CMessage::new(UNION_APPLICATION_CONFIRMATION_MESSAGE_TYPE);
        message.base_mut().add_long(request.recipient_player_id);
        message.base_mut().add_long(1);
        message
            .base_mut()
            .add(legacy_c_string_visible_bytes(&request.inviter_faction_name));
        message.base_mut().add_byte(0);
        message.base_mut().add_long64(session_id);
        message.base_mut().add_long(cookie_second);
        self.runtime
            .send_union_invitation_confirmation(request.recipient_player_id, &message);
    }

    fn on_async_callback(&self, result: NetSessionAsyncResult<'_>) {
        let terminal = if result.kind == NetSessionAsyncResultKind::Result {
            let Some(decision) = result
                .payload
                .and_then(|payload| payload.downcast_ref::<i32>())
            else {
                self.runtime.block_union_invitation_endpoint(
                    UnionApplicationEndpointBlock::ResultPayloadType,
                );
                return;
            };
            if *decision == 1 {
                UnionApplicationTerminal::Approved
            } else {
                UnionApplicationTerminal::Denied
            }
        } else {
            UnionApplicationTerminal::NonResult { kind: result.kind }
        };
        self.runtime.finish_union_invitation(
            self.union_id,
            self.inviter_faction_id,
            self.invited_faction_id,
            terminal,
        );
    }
}

pub(crate) fn begin_union_invitation_session(
    manager: &CNetSessionManager,
    request: UnionInvitationSessionRequest,
    runtime: Arc<dyn UnionInvitationSessionRuntime>,
    random: impl FnMut(i32) -> i32,
) -> Result<UnionApplicationSessionReport, UnionApplicationSessionBlock> {
    let session = manager
        .create_session(
            request.recipient_player_id,
            request.requested_session_id,
            random,
        )
        .map_err(UnionApplicationSessionBlock::Create)?;
    let endpoint = Box::new(InviteJoinConfeder {
        union_id: request.union_id,
        inviter_faction_id: request.inviter_faction_id,
        invited_faction_id: request.invited_faction_id,
        runtime,
    });
    manager
        .set_callback_handle(session.id, endpoint)
        .map_err(|source| UnionApplicationSessionBlock::SetCallback { session, source })?;
    manager
        .beging(session.id, request.timeout_ticks, &request)
        .map_err(|source| UnionApplicationSessionBlock::Begin { session, source })?;
    Ok(UnionApplicationSessionReport { session })
}

pub(crate) trait UnionInviteEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn begin_union_invitation_session(
        &mut self,
        request: UnionInvitationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionInviteRejection {
    PendingApplication,
    ZeroFactionId,
    InvitedFactionReserved,
    InviterNotPermitted,
    InvitedAlreadyInUnion { union_id: i32 },
    InviterFactionMissing,
    InvitedFactionMissing,
    InvitedMasterOffline { notice_sent: bool },
    MemberLimit {
        member_count: i32,
        maximum: i32,
        allocated_net_exchange_id: i32,
        notice_sent: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionInviteOutcome<SessionReport> {
    Rejected(UnionInviteRejection),
    Started {
        invited_faction_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionInviteBlock<OperatorBlock, MembershipBlock, SessionBlock> {
    PendingApplicationUninitialized,
    Operator(OperatorBlock),
    Membership(MembershipBlock),
    FactionSnapshot(UnionApplicationFactionBlock),
    Session {
        source: SessionBlock,
        invited_faction_id: i32,
        net_exchange_id: i32,
        application_assigned: bool,
        establishment_reserved: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionApplyForJoinRejection {
    PendingApplication,
    ZeroApplicantFactionId,
    ApplicantAlreadyReserved,
    ApplicantAlreadyInUnion { union_id: i32 },
    ApplicantFactionMissing,
    MasterFactionMissing,
    MasterPlayerOffline { notice_sent: bool },
    MemberLimit {
        member_count: i32,
        maximum: i32,
        allocated_net_exchange_id: i32,
        notice_sent: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionApplyForJoinOutcome<SessionReport> {
    Rejected(UnionApplyForJoinRejection),
    Started {
        applicant_faction_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionApplyForJoinBlock<MembershipBlock, SessionBlock> {
    PendingApplicationUninitialized,
    MembershipScan(MembershipBlock),
    FactionSnapshot(UnionApplicationFactionBlock),
    Session {
        source: SessionBlock,
        applicant_faction_id: i32,
        net_exchange_id: i32,
        application_assigned: bool,
        establishment_reserved: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionFactionPlayerRefreshReport {
    pub(crate) faction_id: i32,
    pub(crate) refreshed_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionPlayerRefreshReport {
    pub(crate) factions: Vec<UnionFactionPlayerRefreshReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionInitialReport {
    pub(crate) master_faction_found: bool,
    pub(crate) superior_assigned: bool,
    pub(crate) player_refresh: UnionPlayerRefreshReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionInitialBlock {
    MasterTitleWouldOverflow {
        visible_length: usize,
        capacity: usize,
    },
    MissingFactionLevel(UnionFactionLevelBlock),
    SuperiorOrganizing {
        faction_id: i32,
        source: FactionSuperiorOrganizingBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionAddFactionReport {
    pub(crate) faction_id: i32,
    pub(crate) replaced_existing_member: bool,
    pub(crate) superior_assigned: bool,
    pub(crate) owned_city_refresh: FactionOwnedCityRefreshReport,
    pub(crate) member_information: UnionInfoFanoutReport,
    pub(crate) player_refresh: UnionPlayerRefreshReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionAddFactionOutcome {
    Added(UnionAddFactionReport),
    FactionUnavailable {
        faction_id: i32,
        war_log: Vec<u8>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionAddFactionBlock {
    MemberTitleWouldOverflow {
        visible_length: usize,
        capacity: usize,
    },
    FactionNameWouldOverflow {
        faction_id: i32,
        visible_length: usize,
        capacity: usize,
    },
    MissingFactionLevel(UnionFactionLevelBlock),
    SuperiorOrganizing {
        faction_id: i32,
        member_inserted: bool,
        source: FactionSuperiorOrganizingBlock,
    },
    OwnedCityRefresh {
        faction_id: i32,
        member_inserted: bool,
        superior_assigned: bool,
        source: FactionOwnedCityRefreshBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionDoJoinRejection {
    MemberLimit {
        member_count: i32,
        maximum: i32,
    },
    InvalidApplication {
        manager_id: i32,
        applicant_faction_id: i32,
        pending_applicant_faction_id: Option<i32>,
    },
    ApplicantAlreadyInUnion {
        applicant_faction_id: i32,
        union_id: i32,
    },
    OperatorNotAuthorized,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionDoJoinReport {
    pub(crate) add_faction: UnionAddFactionOutcome,
    pub(crate) initial_snapshot_legacy_result: bool,
    pub(crate) member_update: UnionMemberUpdateReport,
    pub(crate) dirty_set: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionDoJoinOutcome {
    Rejected {
        reason: UnionDoJoinRejection,
        application_cleared: bool,
    },
    Joined(UnionDoJoinReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionDoJoinBlock<FreeFactionBlock, OperatorBlock, InitialSnapshotBlock> {
    FreeFactionScan {
        source: FreeFactionBlock,
        application_cleared: bool,
    },
    OperatorValidation {
        source: OperatorBlock,
        application_cleared: bool,
    },
    AddFaction {
        source: UnionAddFactionBlock,
        application_cleared: bool,
    },
    InitialSnapshot {
        source: InitialSnapshotBlock,
        application_cleared: bool,
        add_faction: UnionAddFactionOutcome,
    },
    MemberUpdate {
        source: UnionMemberUpdateBlock,
        application_cleared: bool,
        add_faction: UnionAddFactionOutcome,
        initial_snapshot_legacy_result: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionFireOutRejection {
    StandardWar,
    CityWar,
    OperatorValidationFailed,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionFireOutReport<DetachOutcome> {
    pub(crate) member_information: UnionInfoFanoutReport,
    pub(crate) detach: DetachOutcome,
    pub(crate) member_removed: bool,
    pub(crate) delete_organizing: UnionDeleteOrganizingReport,
    pub(crate) member_update: UnionMemberUpdateReport,
    pub(crate) player_refresh: UnionPlayerRefreshReport,
    pub(crate) war_log: Option<Vec<u8>>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionFireOutOutcome<DetachOutcome> {
    Rejected(UnionFireOutRejection),
    Fired(UnionFireOutReport<DetachOutcome>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionFireOutBlock<OperatorBlock, DetachBlock> {
    OperatorValidation(OperatorBlock),
    UnterminatedTargetName {
        target_faction_id: i32,
        source: UnterminatedMemberField,
    },
    Detach {
        target_faction_id: i32,
        source: DetachBlock,
        member_information: UnionInfoFanoutReport,
    },
    MemberUpdate {
        target_faction_id: i32,
        source: UnionMemberUpdateBlock,
        member_information: UnionInfoFanoutReport,
        member_removed: bool,
        delete_organizing: UnionDeleteOrganizingReport,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionDemiseRejection {
    MemberCountLimit {
        member_count: usize,
        limit: usize,
    },
    LegacyIdentifiersEqual,
    OldMasterFactionNotFound,
    ZeroNewMasterFaction,
    Cooldown {
        elapsed_ms: u32,
        minimum_ms: u32,
        notice_sent: bool,
    },
    CityWar {
        notice_sent: bool,
    },
    StandardWar {
        notice_sent: bool,
    },
    OldFactionIsNotCurrentMaster {
        old_faction_id: i32,
        current_master_faction_id: i32,
    },
    OldMemberNotFound {
        old_faction_id: i32,
    },
    NewMemberNotFound {
        new_faction_id: i32,
    },
    OldMasterOffline,
    OldMasterOperatingFactionWar,
    CurrentMasterFactionMissing {
        faction_id: i32,
    },
    NewMasterFactionMissing {
        faction_id: i32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionDemiseReport {
    pub(crate) old_faction_id: i32,
    pub(crate) new_faction_id: i32,
    pub(crate) old_member_update: UnionMemberUpdateReport,
    pub(crate) new_member_update: UnionMemberUpdateReport,
    pub(crate) member_information: UnionInfoFanoutReport,
    pub(crate) player_refresh: UnionPlayerRefreshReport,
    pub(crate) war_log: Option<Vec<u8>>,
    pub(crate) completed_at_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionDemiseOutcome {
    Rejected(UnionDemiseRejection),
    Transferred(UnionDemiseReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionDemiseBlock<OperatorBlock> {
    MasterLookup(OperatorBlock),
    UnterminatedMemberField {
        member_id: i32,
        source: UnterminatedMemberField,
    },
    DemotedTitleWouldOverflow {
        visible_length: usize,
        capacity: usize,
        master_changed: bool,
    },
    MemberUpdate {
        target_faction_id: i32,
        source: UnionMemberUpdateBlock,
        old_member_update: Option<UnionMemberUpdateReport>,
    },
    InformationWouldOverflow {
        visible_length: usize,
        capacity: usize,
        old_member_update: UnionMemberUpdateReport,
        new_member_update: UnionMemberUpdateReport,
    },
    WarLogWouldOverflow {
        visible_length: usize,
        capacity: usize,
        old_member_update: UnionMemberUpdateReport,
        new_member_update: UnionMemberUpdateReport,
        member_information: UnionInfoFanoutReport,
        player_refresh: UnionPlayerRefreshReport,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionExitRejection {
    StandardWar,
    CityWar,
    OperatorValidationFailed,
    FactionMasterNotFound,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionExitReport<DetachOutcome> {
    pub(crate) leaving_faction_id: i32,
    pub(crate) member_information: UnionInfoFanoutReport,
    pub(crate) war_log: Option<Vec<u8>>,
    pub(crate) detach: DetachOutcome,
    pub(crate) member_removed: bool,
    pub(crate) delete_organizing: UnionDeleteOrganizingReport,
    pub(crate) member_update: UnionMemberUpdateReport,
    pub(crate) player_refresh: UnionPlayerRefreshReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionExitOutcome<DetachOutcome> {
    Rejected(UnionExitRejection),
    Exited(UnionExitReport<DetachOutcome>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionExitBlock<OperatorBlock, DetachBlock> {
    OperatorValidation(OperatorBlock),
    MasterLookup(OperatorBlock),
    MissingLeavingMember { leaving_faction_id: i32 },
    UnterminatedLeavingName {
        leaving_faction_id: i32,
        source: UnterminatedMemberField,
    },
    Detach {
        leaving_faction_id: i32,
        source: DetachBlock,
        member_information: UnionInfoFanoutReport,
        war_log: Option<Vec<u8>>,
    },
    MemberUpdate {
        leaving_faction_id: i32,
        source: UnionMemberUpdateBlock,
        member_information: UnionInfoFanoutReport,
        war_log: Option<Vec<u8>>,
        member_removed: bool,
        delete_organizing: UnionDeleteOrganizingReport,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionDisbandRejection {
    OperatorValidationFailed,
    War,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionDisbandReport<DetachOutcome> {
    pub(crate) member_information: UnionInfoFanoutReport,
    pub(crate) war_log: Vec<u8>,
    pub(crate) detached_members: Vec<(i32, DetachOutcome)>,
    pub(crate) application_cleared: bool,
    pub(crate) delete_organizing: UnionDeleteOrganizingReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionDisbandOutcome<DetachOutcome> {
    Rejected(UnionDisbandRejection),
    Disbanded(UnionDisbandReport<DetachOutcome>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionDisbandBlock<OperatorBlock, DetachBlock, DetachOutcome> {
    OperatorValidation(OperatorBlock),
    Detach {
        faction_id: i32,
        source: DetachBlock,
        member_information: UnionInfoFanoutReport,
        war_log: Vec<u8>,
        completed: Vec<(i32, DetachOutcome)>,
    },
}

/// Read-only faction callback для полных enemy/owned-city snapshot-ов.
pub(crate) trait UnionClientSnapshotContext {
    fn faction_update_enemy_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Option<Vec<FactionEnemyDelivery>>;

    fn faction_update_city_war_enemy_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Option<Vec<FactionEnemyDelivery>>;

    fn faction_update_owned_city_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionOwnedCityDelivery>>, FactionOwnedCityUpdateBuildError>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionEnemySnapshotFactionReport {
    pub(crate) faction_id: i32,
    pub(crate) deliveries: Vec<FactionEnemyDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionEnemySnapshotReport {
    pub(crate) factions: Vec<UnionEnemySnapshotFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCitySnapshotFactionReport {
    pub(crate) faction_id: i32,
    pub(crate) deliveries: Vec<FactionOwnedCityDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCitySnapshotReport {
    pub(crate) factions: Vec<UnionOwnedCitySnapshotFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCitySnapshotBlock {
    pub(crate) faction_id: i32,
    pub(crate) completed_factions: Vec<UnionOwnedCitySnapshotFactionReport>,
    pub(crate) source: FactionOwnedCityUpdateBuildError,
}

/// Read-only faction callback для organizing-info fan-out.
pub(crate) trait UnionSendInfoContext {
    fn faction_send_info_to_members<'a>(
        &self,
        faction_id: i32,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        color: u32,
        send_organizing_info: &mut dyn FnMut(FactionMemberInfoRequest<'a>),
    ) -> Option<FactionMemberInfoReport>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionFactionInfoFanout {
    pub(crate) faction_id: i32,
    pub(crate) recipient_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionInfoFanoutReport {
    pub(crate) factions: Vec<UnionFactionInfoFanout>,
}

/// Read-only доступ к ordered faction member-player ID.
pub(crate) trait UnionFactionMemberContext {
    fn faction_member_player_ids(&self, faction_id: i32) -> Option<Vec<i32>>;

    fn faction_name(&self, faction_id: i32) -> Option<Vec<u8>>;

    fn faction_level(
        &self,
        faction_id: i32,
    ) -> Result<Option<i32>, UnionFactionLevelBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnionFactionLevelBlock {
    pub(crate) faction_id: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionMemberSnapshotBlock {
    UnterminatedMemberField {
        member_id: i32,
        field: UnterminatedMemberField,
        completed_members: usize,
    },
    FactionNameWouldOverflow {
        faction_id: i32,
        visible_length: usize,
        capacity: usize,
        completed_members: usize,
    },
    MissingFactionLevel {
        source: UnionFactionLevelBlock,
        completed_members: usize,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionDeleteOrganizingDelivery {
    pub(crate) faction_id: i32,
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionDeleteOrganizingReport {
    pub(crate) visited_faction_ids: Vec<i32>,
    pub(crate) deliveries: Vec<UnionDeleteOrganizingDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionMemberUpdateDelivery {
    pub(crate) recipient_faction_id: i32,
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionMemberUpdateReport {
    pub(crate) target_found: Option<bool>,
    pub(crate) target_level: Option<i32>,
    pub(crate) deliveries: Vec<UnionMemberUpdateDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionMemberUpdateBlock {
    MissingFactionLevel {
        source: UnionFactionLevelBlock,
        target_level: i32,
        completed_deliveries: Vec<UnionMemberUpdateDelivery>,
    },
    UnterminatedField {
        field: UnterminatedMemberField,
        target_level: i32,
        recipient_faction_id: i32,
        recipient_player_id: i32,
        game_server_id: i32,
        completed_deliveries: Vec<UnionMemberUpdateDelivery>,
    },
}

/// Поля `CUnion`, которые буквально копирует и читает save-цепочка.
pub(crate) struct CUnion {
    union_id: i32,
    name: Vec<u8>,
    master_id: i32,
    members: BTreeMap<i32, TagMemInfo>,
    established_time: TagTimeValue,
    apply_person: Option<i32>,
    change_data_type: i32,
    last_demise_time_ms: u32,
}

impl CUnion {
    /// Материализует достигнутую инфраструктурную часть private `CUnion::CUnion`.
    ///
    /// Старая функция конструирует пустые string/map и `tagTime`, но не пишет
    /// scalar-поля. Последующее чтение таких полей было внутренним UB, поэтому
    /// safe baseline задаёт нейтральные значения. Время приходит от concrete
    /// фабрики, сохраняя её доказанный момент снятия local clock.
    fn with_private_constructor_defaults(established_time: TagTimeValue) -> Self {
        Self {
            union_id: 0,
            name: Vec::new(),
            master_id: 0,
            members: BTreeMap::new(),
            established_time,
            apply_person: None,
            change_data_type: 0,
            last_demise_time_ms: 0,
        }
    }

    /// Строит concrete union на DB-пути `LoadAllConfederation`.
    ///
    /// Public constructor точного EXE вызывает `Initial` ещё до
    /// `LoadConfeMembers`: поэтому `m_ApplyPerson` становится `0`, а в map
    /// появляется fallback master-member с `WS0154` и current local time.
    /// Переданные DB members затем заменяют соответствующие ключи, как старый
    /// `map::operator[]`. Faction-map на этой стадии ещё пуста, поэтому
    /// constructor не меняет имя/union ID живой faction и не делает player
    /// refresh; эти side effect старого null lookup отсутствуют.
    pub(crate) fn from_database_load_state(
        union_id: i32,
        name: Vec<u8>,
        master_id: i32,
        master_title: &[u8],
        members: BTreeMap<i32, TagMemInfo>,
    ) -> Result<Self, UnionInitialBlock> {
        let visible_title = legacy_c_string_visible_bytes(master_title);
        let mut title = [0; 64];
        if visible_title.len() >= title.len() {
            return Err(UnionInitialBlock::MasterTitleWouldOverflow {
                visible_length: visible_title.len(),
                capacity: title.len(),
            });
        }
        title[..visible_title.len()].copy_from_slice(visible_title);
        let established_time = current_local_member_time();
        let fallback_master = TagMemInfo::from_complete_fields(
            master_id,
            [0; 32],
            0,
            0,
            1,
            title,
            [
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ],
            [0; 64],
            established_time,
            false,
        );
        let mut union = Self::with_private_constructor_defaults(established_time);
        union.union_id = union_id;
        union.name = name;
        union.master_id = master_id;
        union.members.insert(master_id, fallback_master);
        union.members.extend(members);
        union.apply_person = Some(0);
        Ok(union)
    }

    /// Строит live-union и выполняет доказанный `Initial` с явными callbacks.
    ///
    /// При safe-блокировке возвращает сам частично инициализированный owner,
    /// чтобы caller не терял уже выполненные внешние эффекты.
    pub(crate) fn from_live_state<Context>(
        union_id: i32,
        master_id: i32,
        name: Vec<u8>,
        master_title: Option<&[u8]>,
        context: &mut Context,
        parameters: &COrganizingParam,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<(Self, UnionInitialReport), (Self, UnionInitialBlock)>
    where
        Context:
            UnionFactionMemberContext + UnionInitialMutationContext + UnionPlayerRefreshContext,
    {
        let mut union = Self::with_private_constructor_defaults(current_local_member_time());
        union.union_id = union_id;
        union.name = name;
        union.master_id = master_id;
        match union.initial_live(master_title, context, parameters, game, update_player) {
            Ok(report) => Ok((union, report)),
            Err(source) => Err((union, source)),
        }
    }

    /// Создаёт достигнутую save-проекцию, не имитируя live `Initial`.
    pub(crate) fn from_reached_save_state(
        union_id: i32,
        name: Vec<u8>,
        master_id: i32,
        members: BTreeMap<i32, TagMemInfo>,
        established_time: TagTimeValue,
        change_data_type: i32,
    ) -> Self {
        let mut union = Self::with_private_constructor_defaults(established_time);
        union.union_id = union_id;
        union.name = name;
        union.master_id = master_id;
        union.members = members;
        union.change_data_type = change_data_type;
        union
    }

    /// Выполняет машинный `Initial` поверх уже созданного live-owner-а.
    pub(crate) fn initial_live<Context>(
        &mut self,
        master_title: Option<&[u8]>,
        context: &mut Context,
        parameters: &COrganizingParam,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionInitialReport, UnionInitialBlock>
    where
        Context:
            UnionFactionMemberContext + UnionInitialMutationContext + UnionPlayerRefreshContext,
    {
        self.apply_person = Some(0);
        let member_time = current_local_member_time();

        let visible_title = legacy_c_string_visible_bytes(master_title.unwrap_or_default());
        let mut title = [0; 64];
        if visible_title.len() >= title.len() {
            return Err(UnionInitialBlock::MasterTitleWouldOverflow {
                visible_length: visible_title.len(),
                capacity: title.len(),
            });
        }
        title[..visible_title.len()].copy_from_slice(visible_title);

        let master_name = if self.master_id > 0 {
            context.faction_name(self.master_id)
        } else {
            None
        };
        let master_faction_found = master_name.is_some();
        let level = if master_faction_found {
            match context.faction_level(self.master_id) {
                Ok(Some(level)) => level,
                Ok(None) => 0,
                Err(source) => return Err(UnionInitialBlock::MissingFactionLevel(source)),
            }
        } else {
            0
        };
        if let Some(master_name) = master_name {
            self.name = master_name;
        }

        let member = TagMemInfo::from_complete_fields(
            self.master_id,
            [0; 32],
            level,
            0,
            1,
            title,
            [
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ],
            [0; 64],
            member_time,
            false,
        );
        self.members.insert(self.master_id, member);

        let superior_assigned = if master_faction_found {
            context
                .faction_set_superior_organizing(
                    self.master_id,
                    self.union_id,
                    self.master_id,
                    parameters,
                )
                .map_err(|source| UnionInitialBlock::SuperiorOrganizing {
                    faction_id: self.master_id,
                    source,
                })?
        } else {
            false
        };
        self.change_data_type = 0;
        let player_refresh = self.update_player_faction_info(
            self.master_id,
            context,
            game,
            update_player,
        );
        Ok(UnionInitialReport {
            master_faction_found,
            superior_assigned,
            player_refresh,
        })
    }

    /// Воспроизводит nullable результат virtual `CloneSaveData`.
    pub(crate) fn clone_save_data(&self) -> Option<Self> {
        (self.change_data_type != 0).then(|| Self {
            union_id: self.union_id,
            name: self.name.clone(),
            master_id: self.master_id,
            members: self.members.clone(),
            established_time: self.established_time,
            apply_person: None,
            change_data_type: self.change_data_type,
            last_demise_time_ms: 0,
        })
    }

    /// Вызывает exact DB-owner с live union и игнорирует его результат.
    ///
    /// Оригинал разрешал `CGame::m_pRsConfederation`, один раз вызывал
    /// `SaveConfederation(this)` и всегда возвращал `true`. Rust передаёт
    /// внешний DB-owner явным one-shot callback-ом; snapshot-фаза `savedb`
    /// остаётся отдельным асинхронным owner-ом.
    pub(crate) fn save(&self, save_confederation: impl FnOnce(&Self)) -> bool {
        save_confederation(self);
        true
    }

    /// Применяет точную bit-mask семантику virtual `SetChangeData`.
    pub(crate) fn set_change_data(&mut self, change_data_type: i32) {
        if change_data_type == 0 {
            self.change_data_type = 0;
        } else if self.change_data_type & change_data_type == 0 {
            self.change_data_type |= change_data_type;
        }
    }

    pub(crate) const fn union_id(&self) -> i32 {
        self.union_id
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) const fn master_id(&self) -> i32 {
        self.master_id
    }

    /// Возвращает byte-exact время учреждения; старый const-reference был Copy.
    pub(crate) const fn established_time(&self) -> TagTimeValue {
        self.established_time
    }

    /// Save-проекция не выдумывает transient apply-person; live `Initial` — 0.
    pub(crate) const fn apply_person(&self) -> Option<i32> {
        self.apply_person
    }

    /// Сбрасывает pending applicant после нормального terminal callback-а.
    pub(crate) fn finish_union_application_callback(&mut self) {
        self.apply_person = Some(0);
    }

    /// Запускает подтверждение заявки faction на вступление в союз.
    pub(crate) fn apply_for_join<Context, Effects>(
        &mut self,
        game: &CGame,
        applicant_faction_id: i32,
        _second_parameter: i32,
        _third_parameter: i32,
        context: &mut Context,
        effects: &mut Effects,
    ) -> Result<
        UnionApplyForJoinOutcome<Effects::SessionReport>,
        UnionApplyForJoinBlock<
            <Context as UnionDoJoinContext>::FreeFactionBlock,
            Effects::SessionBlock,
        >,
    >
    where
        Context: UnionApplyForJoinContext + UnionDoJoinContext,
        Effects: UnionApplyForJoinEffects,
    {
        let Some(pending_application) = self.apply_person else {
            return Err(UnionApplyForJoinBlock::PendingApplicationUninitialized);
        };
        if pending_application >= 1 {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::PendingApplication,
            ));
        }
        if applicant_faction_id == 0 {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ZeroApplicantFactionId,
            ));
        }
        if context.union_application_is_reserved(applicant_faction_id) {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ApplicantAlreadyReserved,
            ));
        }

        let existing_union_id = context
            .union_id_for_joining_faction(applicant_faction_id)
            .map_err(UnionApplyForJoinBlock::MembershipScan)?;
        if existing_union_id > 0 {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ApplicantAlreadyInUnion {
                    union_id: existing_union_id,
                },
            ));
        }

        let applicant = context
            .union_application_faction(applicant_faction_id)
            .map_err(UnionApplyForJoinBlock::FactionSnapshot)?;
        let Some(applicant) = applicant else {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::ApplicantFactionMissing,
            ));
        };
        let master = context
            .union_application_faction(self.master_id)
            .map_err(UnionApplyForJoinBlock::FactionSnapshot)?;
        let Some(master) = master else {
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::MasterFactionMissing,
            ));
        };

        let Some(master_player) = game.online_player_by_id(master.player_header as u32) else {
            send_union_application_notice(
                effects,
                applicant.player_header,
                b"WS0264",
            );
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::MasterPlayerOffline { notice_sent: true },
            ));
        };

        let net_exchange_id = master_player.get_net_exchange_id();
        let member_count = self.members.len() as u32 as i32;
        if member_count >= MAX_UNION_MEMBER_COUNT {
            send_union_application_notice(
                effects,
                applicant.player_header,
                b"WS0265",
            );
            return Ok(UnionApplyForJoinOutcome::Rejected(
                UnionApplyForJoinRejection::MemberLimit {
                    member_count,
                    maximum: MAX_UNION_MEMBER_COUNT,
                    allocated_net_exchange_id: net_exchange_id,
                    notice_sent: true,
                },
            ));
        }

        self.apply_person = Some(applicant_faction_id);
        context.reserve_union_application(applicant_faction_id);
        let request = UnionApplicationSessionRequest {
            union_id: self.union_id,
            applicant_faction_id,
            recipient_player_id: master.player_header,
            requested_session_id: net_exchange_id,
            timeout_ticks: 1_000,
            confirmation_kind: 2,
            applicant_faction_name: applicant.name,
        };
        let session = effects
            .begin_union_application_session(request)
            .map_err(|source| UnionApplyForJoinBlock::Session {
                source,
                applicant_faction_id,
                net_exchange_id,
                application_assigned: true,
                establishment_reserved: true,
            })?;

        Ok(UnionApplyForJoinOutcome::Started {
            applicant_faction_id,
            net_exchange_id,
            session,
        })
    }

    /// Запускает подтверждение приглашения свободной faction в этот союз.
    pub(crate) fn invite<Context, Effects>(
        &mut self,
        game: &CGame,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
    ) -> Result<
        UnionInviteOutcome<Effects::SessionReport>,
        UnionInviteBlock<
            <Context as UnionOperatorValidationContext>::Block,
            <Context as UnionDoJoinContext>::FreeFactionBlock,
            Effects::SessionBlock,
        >,
    >
    where
        Context: UnionOperatorValidationContext
            + UnionApplyForJoinContext
            + UnionDoJoinContext,
        Effects: UnionInviteEffects,
    {
        let Some(pending_application) = self.apply_person else {
            return Err(UnionInviteBlock::PendingApplicationUninitialized);
        };
        if pending_application > 0 {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::PendingApplication,
            ));
        }
        if inviter_faction_id == 0 || invited_faction_id == 0 {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::ZeroFactionId,
            ));
        }
        if context.union_application_is_reserved(invited_faction_id) {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedFactionReserved,
            ));
        }

        // Exact caller передаёт сюда faction ID, хотя validation owner
        // интерпретирует аргумент как master-player ID. Наблюдаемый gate
        // сохраняется без подмены исходным player ID ingress-а.
        let permitted = self
            .check_operator_validate(inviter_faction_id, EPurview::ConMem as i32, context)
            .map_err(UnionInviteBlock::Operator)?;
        if !permitted {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InviterNotPermitted,
            ));
        }

        let existing_union_id = context
            .union_id_for_joining_faction(invited_faction_id)
            .map_err(UnionInviteBlock::Membership)?;
        if existing_union_id > 0 {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedAlreadyInUnion {
                    union_id: existing_union_id,
                },
            ));
        }

        let inviter = context
            .union_application_faction(inviter_faction_id)
            .map_err(UnionInviteBlock::FactionSnapshot)?;
        let Some(inviter) = inviter else {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InviterFactionMissing,
            ));
        };
        let invited = context
            .union_application_faction(invited_faction_id)
            .map_err(UnionInviteBlock::FactionSnapshot)?;
        let Some(invited) = invited else {
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedFactionMissing,
            ));
        };

        let Some(invited_master) = game.online_player_by_id(invited.player_header as u32) else {
            send_union_invitation_notice(effects, inviter.player_header, b"WS0264");
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::InvitedMasterOffline { notice_sent: true },
            ));
        };
        let net_exchange_id = invited_master.get_net_exchange_id();
        let member_count = self.members.len() as u32 as i32;
        if member_count >= MAX_UNION_MEMBER_COUNT {
            send_union_invitation_notice(effects, inviter.player_header, b"WS0265");
            return Ok(UnionInviteOutcome::Rejected(
                UnionInviteRejection::MemberLimit {
                    member_count,
                    maximum: MAX_UNION_MEMBER_COUNT,
                    allocated_net_exchange_id: net_exchange_id,
                    notice_sent: true,
                },
            ));
        }

        self.apply_person = Some(invited_faction_id);
        context.reserve_union_application(invited_faction_id);
        let session = effects
            .begin_union_invitation_session(UnionInvitationSessionRequest {
                union_id: self.union_id,
                inviter_faction_id,
                invited_faction_id,
                recipient_player_id: invited.player_header,
                requested_session_id: net_exchange_id,
                timeout_ticks: 1_000,
                inviter_faction_name: inviter.name,
            })
            .map_err(|source| UnionInviteBlock::Session {
                source,
                invited_faction_id,
                net_exchange_id,
                application_assigned: true,
                establishment_reserved: true,
            })?;

        Ok(UnionInviteOutcome::Started {
            invited_faction_id,
            net_exchange_id,
            session,
        })
    }

    /// Очищает единственную union-заявку только при праве `PV_ConMem`.
    pub(crate) fn clear_apply_list<Context>(
        &mut self,
        manager_player_id: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        if !self.check_operator_validate(
            manager_player_id,
            EPurview::ConMem as i32,
            context,
        )? {
            return Ok(false);
        }
        self.apply_person = Some(0);
        Ok(true)
    }

    /// Добавляет faction-member и выполняет полный исходный callback-порядок.
    pub(crate) fn add_faction<Context, Effects>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        parameters: &COrganizingParam,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionAddFactionOutcome, UnionAddFactionBlock>
    where
        Context: UnionFactionJoinContext,
        Effects: UnionAddFactionEffects,
    {
        let member_time = current_local_member_time();
        let member_title = effects.world_string(b"WS0268");
        let visible_title = legacy_c_string_visible_bytes(&member_title);
        let mut title = [0; 64];
        if visible_title.len() >= title.len() {
            return Err(UnionAddFactionBlock::MemberTitleWouldOverflow {
                visible_length: visible_title.len(),
                capacity: title.len(),
            });
        }
        title[..visible_title.len()].copy_from_slice(visible_title);

        let faction_name = if faction_id > 0 {
            context.faction_name(faction_id)
        } else {
            None
        };
        let Some(faction_name) = faction_name else {
            let war_log = effects.format_world_string(
                b"WS0269",
                &[
                    UnionFormatArgument::Signed(faction_id),
                    UnionFormatArgument::Signed(self.union_id),
                ],
            );
            let war_log = legacy_c_string_visible_bytes(&war_log).to_vec();
            effects.put_war_log(&war_log);
            return Ok(UnionAddFactionOutcome::FactionUnavailable {
                faction_id,
                war_log,
            });
        };

        let visible_faction_name = legacy_c_string_visible_bytes(&faction_name);
        let mut fixed_name = [0; 32];
        if visible_faction_name.len() >= fixed_name.len() {
            return Err(UnionAddFactionBlock::FactionNameWouldOverflow {
                faction_id,
                visible_length: visible_faction_name.len(),
                capacity: fixed_name.len(),
            });
        }
        fixed_name[..visible_faction_name.len()].copy_from_slice(visible_faction_name);
        let level = match context.faction_level(faction_id) {
            Ok(Some(level)) => level,
            Ok(None) => {
                return Err(UnionAddFactionBlock::MissingFactionLevel(
                    UnionFactionLevelBlock { faction_id },
                ));
            }
            Err(source) => return Err(UnionAddFactionBlock::MissingFactionLevel(source)),
        };

        let member = TagMemInfo::from_complete_fields(
            faction_id,
            fixed_name,
            level,
            0,
            99,
            title,
            [
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ],
            [0; 64],
            member_time,
            false,
        );
        let replaced_existing_member = self.members.insert(faction_id, member).is_some();

        let superior_assigned = context
            .faction_set_superior_organizing(
                faction_id,
                self.union_id,
                self.master_id,
                parameters,
            )
            .map_err(|source| UnionAddFactionBlock::SuperiorOrganizing {
                faction_id,
                member_inserted: true,
                source,
            })?;
        let mut refresh_owned_city = |region_id, owner_faction_id, owner_union_id, country_id| {
            effects.refresh_owned_city(
                region_id,
                owner_faction_id,
                owner_union_id,
                country_id,
            );
        };
        let owned_city_refresh = context
            .faction_refresh_owned_city_info(faction_id, &mut refresh_owned_city)
            .map_err(|source| UnionAddFactionBlock::OwnedCityRefresh {
                faction_id,
                member_inserted: true,
                superior_assigned,
                source,
            })?
            .unwrap_or(FactionOwnedCityRefreshReport {
                refreshed_region_ids: Vec::new(),
            });

        let information = effects.format_world_string(
            b"WS0270",
            &[
                UnionFormatArgument::Text(visible_faction_name),
                UnionFormatArgument::Text(legacy_c_string_visible_bytes(&self.name)),
            ],
        );
        let information = legacy_c_string_visible_bytes(&information);
        let information_title = effects.world_string(b"WS0188");
        let information_title = legacy_c_string_visible_bytes(&information_title);
        let member_information = self.send_info_to_all_members(
            information,
            information_title,
            -1,
            0x0087_A238,
            context,
            &mut |request| effects.send_organizing_info(request),
        );
        let player_refresh = self.update_player_faction_info(
            faction_id,
            context,
            game,
            update_player,
        );
        Ok(UnionAddFactionOutcome::Added(UnionAddFactionReport {
            faction_id,
            replaced_existing_member,
            superior_assigned,
            owned_city_refresh,
            member_information,
            player_refresh,
        }))
    }

    /// Принимает pending faction-заявку в точном порядке `CUnion::DoJoin`.
    ///
    /// EXE хранит лимит `50`; значение `5` из старого Linux-донора к этой
    /// сборке не относится. После совпадения заявки она сбрасывается до
    /// membership/authority-проверок. `approve_flag` и `join_time` исходная
    /// функция не читала. Результат initial snapshot игнорируется, но
    /// сохраняется в отчёте; safe-блокировки malformed state не продолжают
    /// путь после места старого UB.
    pub(crate) fn do_join<Context, Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        applicant_faction_id: i32,
        _approve_flag: i32,
        _join_time: TagTimeValue,
        context: &mut Context,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        UnionDoJoinOutcome,
        UnionDoJoinBlock<
            <Context as UnionDoJoinContext>::FreeFactionBlock,
            <Context as UnionOperatorValidationContext>::Block,
            <Context as UnionDoJoinContext>::InitialSnapshotBlock,
        >,
    >
    where
        Context: UnionFactionJoinContext
            + UnionOperatorValidationContext
            + UnionDoJoinContext,
        Effects: UnionAddFactionEffects,
    {
        let member_count = self.members.len() as u32 as i32;
        if member_count >= MAX_UNION_MEMBER_COUNT {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::MemberLimit {
                    member_count,
                    maximum: MAX_UNION_MEMBER_COUNT,
                },
                application_cleared: false,
            });
        }
        if manager_id == 0
            || applicant_faction_id == 0
            || self.apply_person != Some(applicant_faction_id)
        {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::InvalidApplication {
                    manager_id,
                    applicant_faction_id,
                    pending_applicant_faction_id: self.apply_person,
                },
                application_cleared: false,
            });
        }

        self.apply_person = Some(0);
        let existing_union_id = context
            .union_id_for_joining_faction(applicant_faction_id)
            .map_err(|source| UnionDoJoinBlock::FreeFactionScan {
                source,
                application_cleared: true,
            })?;
        if existing_union_id > 0 {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::ApplicantAlreadyInUnion {
                    applicant_faction_id,
                    union_id: existing_union_id,
                },
                application_cleared: true,
            });
        }

        let operator_authorized = self
            .check_operator_validate(manager_id, EPurview::ConMem as i32, context)
            .map_err(|source| UnionDoJoinBlock::OperatorValidation {
                source,
                application_cleared: true,
            })?;
        if !operator_authorized {
            return Ok(UnionDoJoinOutcome::Rejected {
                reason: UnionDoJoinRejection::OperatorNotAuthorized,
                application_cleared: true,
            });
        }

        let add_faction = self
            .add_faction(
                applicant_faction_id,
                context,
                effects,
                parameters,
                game,
                update_player,
            )
            .map_err(|source| UnionDoJoinBlock::AddFaction {
                source,
                application_cleared: true,
            })?;
        let initial_snapshot_legacy_result =
            match context.add_current_union_to_client_by_faction_id(
                game,
                self,
                applicant_faction_id,
            ) {
                Ok(result) => result,
                Err(source) => {
                    return Err(UnionDoJoinBlock::InitialSnapshot {
                        source,
                        application_cleared: true,
                        add_faction,
                    });
                }
            };
        let member_update = match self.update_member_info_to_client(
            game,
            applicant_faction_id,
            EOperator::Add,
            context,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(UnionDoJoinBlock::MemberUpdate {
                    source,
                    application_cleared: true,
                    add_faction,
                    initial_snapshot_legacy_result,
                });
            }
        };
        self.set_change_data(2);

        Ok(UnionDoJoinOutcome::Joined(UnionDoJoinReport {
            add_faction,
            initial_snapshot_legacy_result,
            member_update,
            dirty_set: true,
        }))
    }

    /// Передаёт руководство союза в точном порядке `CUnion::Demise`.
    pub(crate) fn demise<Context, Effects>(
        &mut self,
        game: &CGame,
        old_master_player_id: i32,
        new_master_faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        get_tick: &mut dyn FnMut() -> u32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionDemiseOutcome, UnionDemiseBlock<Context::Block>>
    where
        Context: UnionDemiseContext,
        Effects: UnionFireOutEffects,
    {
        if self.members.len() >= UNION_DEMISE_MEMBER_LIMIT {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::MemberCountLimit {
                    member_count: self.members.len(),
                    limit: UNION_DEMISE_MEMBER_LIMIT,
                },
            ));
        }
        // Exact EXE сравнивает player ID с faction ID. Разные домены здесь
        // намеренно не "исправляются": результат наблюдаем клиентом.
        if old_master_player_id == new_master_faction_id {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::LegacyIdentifiersEqual,
            ));
        }

        let old_faction_id = context
            .faction_id_by_master_player(old_master_player_id)
            .map_err(UnionDemiseBlock::MasterLookup)?;
        if old_faction_id == 0 {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMasterFactionNotFound,
            ));
        }
        if new_master_faction_id == 0 {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::ZeroNewMasterFaction,
            ));
        }

        let now_ms = get_tick();
        let elapsed_ms = now_ms.wrapping_sub(self.last_demise_time_ms);
        if elapsed_ms < UNION_DEMISE_COOLDOWN_MS {
            send_union_demise_notice(effects, old_master_player_id, b"WS0353");
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::Cooldown {
                    elapsed_ms,
                    minimum_ms: UNION_DEMISE_COOLDOWN_MS,
                    notice_sent: true,
                },
            ));
        }
        if self.has_city_war_enemy_faction(context) {
            send_union_demise_notice(effects, old_master_player_id, b"WS0279");
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::CityWar { notice_sent: true },
            ));
        }
        if self.has_enemy_faction(context) {
            send_union_demise_notice(effects, old_master_player_id, b"WS0280");
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::StandardWar { notice_sent: true },
            ));
        }
        if old_faction_id != self.master_id {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldFactionIsNotCurrentMaster {
                    old_faction_id,
                    current_master_faction_id: self.master_id,
                },
            ));
        }
        if !self.members.contains_key(&new_master_faction_id) {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::NewMemberNotFound {
                    new_faction_id: new_master_faction_id,
                },
            ));
        }

        let Some(old_master_player) = game.online_player_by_id(old_master_player_id as u32) else {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMasterOffline,
            ));
        };
        if old_master_player.faction_war_operator() {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMasterOperatingFactionWar,
            ));
        }
        if context.faction_name(self.master_id).is_none() {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::CurrentMasterFactionMissing {
                    faction_id: self.master_id,
                },
            ));
        }
        if context.faction_name(new_master_faction_id).is_none() {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::NewMasterFactionMissing {
                    faction_id: new_master_faction_id,
                },
            ));
        }

        // `IsMaster(old)` не гарантирует наличие повреждённого master-key в
        // map. Старый operator[] тогда создавал частично неинициализированный
        // tagMemInfo; safe owner останавливает этот внутренний UB.
        let Some(old_member) = self.members.get(&old_faction_id) else {
            return Ok(UnionDemiseOutcome::Rejected(
                UnionDemiseRejection::OldMemberNotFound { old_faction_id },
            ));
        };
        let old_title = old_member
            .title_wire_bytes()
            .map_err(|source| UnionDemiseBlock::UnterminatedMemberField {
                member_id: old_faction_id,
                source,
            })?
            .to_vec();
        let old_name = old_member
            .name_wire_bytes()
            .map_err(|source| UnionDemiseBlock::UnterminatedMemberField {
                member_id: old_faction_id,
                source,
            })?;
        let old_name = old_name[..old_name.len() - 1].to_vec();
        let new_member = self
            .members
            .get(&new_master_faction_id)
            .expect("IsMember(new faction) проверен выше");
        let new_name = new_member
            .name_wire_bytes()
            .map_err(|source| UnionDemiseBlock::UnterminatedMemberField {
                member_id: new_master_faction_id,
                source,
            })?;
        let new_name = new_name[..new_name.len() - 1].to_vec();
        let old_purview = old_member.purview;
        let old_job_level = old_member.job_level;

        self.master_id = new_master_faction_id;
        let member_faction_ids = self.members.keys().copied().collect::<Vec<_>>();
        context.set_union_master_projection(&member_faction_ids, new_master_faction_id);
        {
            let new_member = self
                .members
                .get_mut(&new_master_faction_id)
                .expect("new member существует до мутации");
            new_member.title[..old_title.len()].copy_from_slice(&old_title);
            new_member.purview = old_purview;
            new_member.job_level = old_job_level;
        }
        {
            let old_member = self
                .members
                .get_mut(&old_faction_id)
                .expect("old member проверен до мутации");
            old_member.job_level = 99;
            old_member.id = old_faction_id;
        }

        let demoted_title = effects.world_string(b"WS0268");
        let demoted_title = legacy_c_string_visible_bytes(&demoted_title);
        let title_capacity = self
            .members
            .get(&old_faction_id)
            .expect("old member существует при title assignment")
            .title
            .len();
        if demoted_title.len() >= title_capacity {
            return Err(UnionDemiseBlock::DemotedTitleWouldOverflow {
                visible_length: demoted_title.len(),
                capacity: title_capacity,
                master_changed: true,
            });
        }
        {
            let old_member = self
                .members
                .get_mut(&old_faction_id)
                .expect("old member существует при demotion");
            old_member.title[..demoted_title.len()].copy_from_slice(demoted_title);
            old_member.title[demoted_title.len()] = 0;
        }
        self.name = new_name.clone();
        {
            let old_member = self
                .members
                .get_mut(&old_faction_id)
                .expect("old member существует при purview demotion");
            old_member.purview = [
                EPurviewOwnState::No,
                EPurviewOwnState::Permit,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
                EPurviewOwnState::No,
            ];
        }

        let old_member_update = self
            .update_member_info_to_client(game, old_faction_id, EOperator::Update, context)
            .map_err(|source| UnionDemiseBlock::MemberUpdate {
                target_faction_id: old_faction_id,
                source,
                old_member_update: None,
            })?;
        let new_member_update = match self.update_member_info_to_client(
            game,
            new_master_faction_id,
            EOperator::Update,
            context,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(UnionDemiseBlock::MemberUpdate {
                    target_faction_id: new_master_faction_id,
                    source,
                    old_member_update: Some(old_member_update),
                });
            }
        };
        self.set_change_data(3);

        let information = effects.format_world_string(
            b"WS0281",
            &[
                UnionFormatArgument::Text(&old_name),
                UnionFormatArgument::Text(&new_name),
            ],
        );
        let information = legacy_c_string_visible_bytes(&information);
        let title = effects.world_string(b"WS0188");
        let member_information = self.send_info_to_all_members(
            information,
            legacy_c_string_visible_bytes(&title),
            -1,
            0x0087_A238,
            context,
            &mut |request| effects.send_organizing_info(request),
        );
        let player_refresh = self.update_player_faction_info(0, context, game, update_player);

        // EXE после player refresh дважды повторяет nullable faction lookup.
        // После назначения master оба lookup относятся к одной new faction.
        let war_log = if context.faction_name(new_master_faction_id).is_some()
            && context.faction_name(self.master_id).is_some()
        {
            let war_log = effects.format_world_string(
                b"WS0282",
                &[
                    UnionFormatArgument::Text(&old_name),
                    UnionFormatArgument::Signed(old_faction_id),
                    UnionFormatArgument::Text(&new_name),
                    UnionFormatArgument::Signed(new_master_faction_id),
                ],
            );
            let war_log = legacy_c_string_visible_bytes(&war_log).to_vec();
            effects.put_war_log(&war_log);
            Some(war_log)
        } else {
            None
        };
        let completed_at_ms = get_tick();
        self.last_demise_time_ms = completed_at_ms;

        Ok(UnionDemiseOutcome::Transferred(UnionDemiseReport {
            old_faction_id,
            new_faction_id: new_master_faction_id,
            old_member_update,
            new_member_update,
            member_information,
            player_refresh,
            war_log,
            completed_at_ms,
        }))
    }

    /// Исключает member-faction в точном порядке `CUnion::FireOut`.
    pub(crate) fn fire_out<Context, Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        target_faction_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        UnionFireOutOutcome<Context::DetachOutcome>,
        UnionFireOutBlock<Context::Block, Context::DetachBlock>,
    >
    where
        Context: UnionFireOutContext,
        Effects: UnionFireOutEffects,
    {
        if self.has_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0271");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionFireOutOutcome::Rejected(
                UnionFireOutRejection::StandardWar,
            ));
        }
        if self.has_city_war_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0272");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionFireOutOutcome::Rejected(UnionFireOutRejection::CityWar));
        }

        let operation_valid = self
            .check_operator_validate_target(
                manager_id,
                target_faction_id,
                EPurview::FireOut as i32,
                context,
            )
            .map_err(UnionFireOutBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(UnionFireOutOutcome::Rejected(
                UnionFireOutRejection::OperatorValidationFailed,
            ));
        }

        let target = self
            .members
            .get(&target_faction_id)
            .expect("успешный CheckOperValidate гарантирует target member");
        let target_name_wire = target.name_wire_bytes().map_err(|source| {
            UnionFireOutBlock::UnterminatedTargetName {
                target_faction_id,
                source,
            }
        })?;
        let target_name = target_name_wire[..target_name_wire.len() - 1].to_vec();
        let information = effects.format_world_string(
            b"WS0277",
            &[
                UnionFormatArgument::Text(&target_name),
                UnionFormatArgument::Text(legacy_c_string_visible_bytes(&self.name)),
            ],
        );
        let title = effects.world_string(b"WS0188");
        let member_information = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&information),
            legacy_c_string_visible_bytes(&title),
            -1,
            0x0087_A238,
            context,
            &mut |request| effects.send_organizing_info(request),
        );

        let detach = match context.detach_union_member_for_fire_out(
            game,
            parameters,
            target_faction_id,
        ) {
            Ok(detach) => detach,
            Err(source) => {
                return Err(UnionFireOutBlock::Detach {
                    target_faction_id,
                    source,
                    member_information,
                });
            }
        };
        let member_removed = self.members.remove(&target_faction_id).is_some();
        let delete_organizing =
            self.delete_organizing_to_client(target_faction_id, context, game);
        let member_update = match self.update_member_info_to_client(
            game,
            target_faction_id,
            EOperator::Delete,
            context,
        ) {
            Ok(member_update) => member_update,
            Err(source) => {
                return Err(UnionFireOutBlock::MemberUpdate {
                    target_faction_id,
                    source,
                    member_information,
                    member_removed,
                    delete_organizing,
                });
            }
        };
        self.set_change_data(2);
        let player_refresh =
            self.update_player_faction_info(target_faction_id, context, game, update_player);

        let war_log = match (
            context.faction_name(target_faction_id),
            context.faction_name(self.master_id),
        ) {
            (Some(target_faction_name), Some(master_faction_name)) => {
                let text = effects.format_world_string(
                    b"WS0278",
                    &[
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &target_faction_name,
                        )),
                        UnionFormatArgument::Signed(target_faction_id),
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &master_faction_name,
                        )),
                        UnionFormatArgument::Signed(self.union_id),
                    ],
                );
                effects.put_war_log(&text);
                Some(text)
            }
            _ => None,
        };

        Ok(UnionFireOutOutcome::Fired(UnionFireOutReport {
            member_information,
            detach,
            member_removed,
            delete_organizing,
            member_update,
            player_refresh,
            war_log,
        }))
    }

    /// Выводит member-faction в точном порядке `CUnion::Exit`.
    pub(crate) fn exit<Context, Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        context: &mut Context,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        UnionExitOutcome<Context::DetachOutcome>,
        UnionExitBlock<Context::Block, Context::DetachBlock>,
    >
    where
        Context: UnionExitContext,
        Effects: UnionFireOutEffects,
    {
        if self.has_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0246");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionExitOutcome::Rejected(UnionExitRejection::StandardWar));
        }
        if self.has_city_war_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0247");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionExitOutcome::Rejected(UnionExitRejection::CityWar));
        }

        let operation_valid = self
            .check_operator_validate(player_id, EPurview::Exit as i32, context)
            .map_err(UnionExitBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(UnionExitOutcome::Rejected(
                UnionExitRejection::OperatorValidationFailed,
            ));
        }

        // Оригинал повторно вызывал IsFactionMaster после validation.
        let leaving_faction_id = context
            .faction_id_by_master_player(player_id)
            .map_err(UnionExitBlock::MasterLookup)?;
        if leaving_faction_id == 0 {
            return Ok(UnionExitOutcome::Rejected(
                UnionExitRejection::FactionMasterNotFound,
            ));
        }
        let leaving = self.members.get(&leaving_faction_id).ok_or(
            UnionExitBlock::MissingLeavingMember { leaving_faction_id },
        )?;
        let leaving_name_wire = leaving.name_wire_bytes().map_err(|source| {
            UnionExitBlock::UnterminatedLeavingName {
                leaving_faction_id,
                source,
            }
        })?;
        let leaving_name = leaving_name_wire[..leaving_name_wire.len() - 1].to_vec();
        let information = effects.format_world_string(
            b"WS0273",
            &[UnionFormatArgument::Text(&leaving_name)],
        );
        let title = effects.world_string(b"WS0188");
        let member_information = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&information),
            legacy_c_string_visible_bytes(&title),
            -1,
            0x0087_A238,
            context,
            &mut |request| effects.send_organizing_info(request),
        );

        let war_log = match (
            context.faction_name(leaving_faction_id),
            context.faction_name(self.master_id),
        ) {
            (Some(leaving_faction_name), Some(master_faction_name)) => {
                let text = effects.format_world_string(
                    b"WS0274",
                    &[
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &leaving_faction_name,
                        )),
                        UnionFormatArgument::Signed(leaving_faction_id),
                        UnionFormatArgument::Signed(self.union_id),
                        UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                            &master_faction_name,
                        )),
                    ],
                );
                effects.put_war_log(&text);
                Some(text)
            }
            _ => None,
        };

        let detach = match context.detach_union_member_for_exit(
            game,
            parameters,
            leaving_faction_id,
        ) {
            Ok(detach) => detach,
            Err(source) => {
                return Err(UnionExitBlock::Detach {
                    leaving_faction_id,
                    source,
                    member_information,
                    war_log,
                });
            }
        };
        let member_removed = self.members.remove(&leaving_faction_id).is_some();
        let delete_organizing =
            self.delete_organizing_to_client(leaving_faction_id, context, game);
        let member_update = match self.update_member_info_to_client(
            game,
            leaving_faction_id,
            EOperator::Delete,
            context,
        ) {
            Ok(member_update) => member_update,
            Err(source) => {
                return Err(UnionExitBlock::MemberUpdate {
                    leaving_faction_id,
                    source,
                    member_information,
                    war_log,
                    member_removed,
                    delete_organizing,
                });
            }
        };
        self.set_change_data(2);
        let player_refresh =
            self.update_player_faction_info(leaving_faction_id, context, game, update_player);

        Ok(UnionExitOutcome::Exited(UnionExitReport {
            leaving_faction_id,
            member_information,
            war_log,
            detach,
            member_removed,
            delete_organizing,
            member_update,
            player_refresh,
        }))
    }

    /// Распускает union в точном порядке concrete `CUnion::Disband`.
    pub(crate) fn disband<Context, Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        context: &mut Context,
        effects: &mut Effects,
    ) -> Result<
        UnionDisbandOutcome<Context::DetachOutcome>,
        UnionDisbandBlock<Context::Block, Context::DetachBlock, Context::DetachOutcome>,
    >
    where
        Context: UnionFireOutContext,
        Effects: UnionFireOutEffects,
    {
        let operation_valid = self
            .check_operator_validate(manager_id, EPurview::Disband as i32, context)
            .map_err(UnionDisbandBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(UnionDisbandOutcome::Rejected(
                UnionDisbandRejection::OperatorValidationFailed,
            ));
        }
        if self.has_enemy_faction(context) || self.has_city_war_enemy_faction(context) {
            let first_text = effects.world_string(b"WS0275");
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(UnionDisbandOutcome::Rejected(UnionDisbandRejection::War));
        }

        let information = effects.format_world_string(
            b"WS0276",
            &[UnionFormatArgument::Text(legacy_c_string_visible_bytes(
                &self.name,
            ))],
        );
        let title = effects.world_string(b"WS0188");
        let member_information = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&information),
            legacy_c_string_visible_bytes(&title),
            -1,
            0x0087_A238,
            context,
            &mut |request| effects.send_organizing_info(request),
        );
        effects.put_war_log(&information);

        let member_ids: Vec<i32> = self.members.keys().copied().collect();
        let mut detached_members = Vec::with_capacity(member_ids.len());
        for faction_id in member_ids {
            match context.detach_union_member_for_fire_out(game, parameters, faction_id) {
                Ok(outcome) => detached_members.push((faction_id, outcome)),
                Err(source) => {
                    return Err(UnionDisbandBlock::Detach {
                        faction_id,
                        source,
                        member_information,
                        war_log: information,
                        completed: detached_members,
                    });
                }
            }
        }
        self.apply_person = Some(0);
        let delete_organizing = self.delete_organizing_to_client(0, context, game);
        Ok(UnionDisbandOutcome::Disbanded(UnionDisbandReport {
            member_information,
            war_log: information,
            detached_members,
            application_cleared: true,
            delete_organizing,
        }))
    }

    /// Legacy union не менял title/job и всегда сообщал успех.
    pub(crate) const fn dub_and_set_job_level(
        &self,
        _manager_id: i32,
        _target_id: i32,
        _title: &[u8],
        _job_level: i32,
    ) -> bool {
        true
    }

    /// У union нет leave-word мутации; virtual всегда возвращал `false`.
    pub(crate) const fn edit_leave_word(
        &self,
        _player_id: i32,
        _word_id: i32,
        _operation: EOperator,
    ) -> bool {
        false
    }

    /// У union нет tax-операции; оба аргумента игнорировались.
    pub(crate) const fn operator_tax(&self, _player_id: i32, _operation: i32) -> bool {
        false
    }

    /// Virtual присутствовал в общем interface, но union не менял state.
    pub(crate) const fn set_contributor(
        &self,
        _requester_id: i32,
        _target_id: i32,
        _enabled: bool,
    ) {}

    /// Union-upgrade отсутствовал и всегда возвращал `false`.
    pub(crate) const fn upgrade(&self, _player_id: i32) -> bool {
        false
    }

    /// Разрешает header через master-faction, сохраняя legacy `0` на miss.
    pub(crate) fn player_header<Lookup, Block>(
        &self,
        master_player_id_by_faction: Lookup,
    ) -> Result<i32, Block>
    where
        Lookup: FnOnce(i32) -> Result<Option<i32>, Block>,
    {
        if self.master_id <= 0 {
            return Ok(0);
        }
        Ok(master_player_id_by_faction(self.master_id)?.unwrap_or(0))
    }

    pub(crate) const fn members(&self) -> &BTreeMap<i32, TagMemInfo> {
        &self.members
    }

    /// Возвращает новый снимок member-ID в исходном signed map-order.
    pub(crate) fn member_ids_snapshot(&self) -> Vec<i32> {
        self.members.keys().copied().collect()
    }

    /// Дописывает ordered union-member snapshot в точном клиентском формате.
    pub(crate) fn add_members_to_byte_array<Context>(
        &mut self,
        output: &mut Vec<u8>,
        context: &Context,
    ) -> Result<bool, UnionMemberSnapshotBlock>
    where
        Context: UnionFactionMemberContext,
    {
        output.extend_from_slice(&(self.members.len() as u32).to_le_bytes());
        for (completed_members, member) in self.members.values_mut().enumerate() {
            append_i32(output, member.id);
            append_i32(output, member.job_level);
            let title = member.title_wire_bytes().map_err(|field| {
                UnionMemberSnapshotBlock::UnterminatedMemberField {
                    member_id: member.id,
                    field,
                    completed_members,
                }
            })?;
            output.extend_from_slice(title);
            output.extend_from_slice(&member.purview_wire_bytes());

            if member.id > 0 {
                if let Some(faction_name) = context.faction_name(member.id) {
                    let visible_name = legacy_c_string_visible_bytes(&faction_name);
                    if visible_name.len() >= member.name.len() {
                        return Err(UnionMemberSnapshotBlock::FactionNameWouldOverflow {
                            faction_id: member.id,
                            visible_length: visible_name.len(),
                            capacity: member.name.len(),
                            completed_members,
                        });
                    }
                    member.name[..visible_name.len()].copy_from_slice(visible_name);
                    member.name[visible_name.len()] = 0;
                    match context.faction_level(member.id) {
                        Ok(Some(level)) => member.level = level,
                        Ok(None) => {}
                        Err(source) => {
                            return Err(UnionMemberSnapshotBlock::MissingFactionLevel {
                                source,
                                completed_members,
                            });
                        }
                    }
                }
            }

            append_i32(output, member.level);
            append_i32(output, member.occupation);
            let name = member.name_wire_bytes().map_err(|field| {
                UnionMemberSnapshotBlock::UnterminatedMemberField {
                    member_id: member.id,
                    field,
                    completed_members,
                }
            })?;
            output.extend_from_slice(name);
            output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
            output.push(0);
            output.extend_from_slice(&member.last_online_wire_bytes());
        }
        Ok(true)
    }

    /// Дописывает полный union snapshot и его member records.
    pub(crate) fn add_to_byte_array<Context>(
        &mut self,
        output: &mut Vec<u8>,
        context: &Context,
    ) -> Result<bool, UnionMemberSnapshotBlock>
    where
        Context: UnionFactionMemberContext,
    {
        append_i32(output, self.union_id);
        append_legacy_c_string(output, &self.name);
        append_i32(output, self.master_id);
        if self.master_id > 0 {
            if let Some(master_name) = context.faction_name(self.master_id) {
                append_legacy_c_string(output, &master_name);
            } else {
                output.push(0);
            }
        } else {
            output.push(0);
        }
        self.add_members_to_byte_array(output, context)?;
        Ok(true)
    }

    /// Union не хранит отдельный standard enemy-set и всегда возвращал пустой.
    pub(crate) fn enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        BTreeSet::new()
    }

    /// Folded city-war getter также всегда возвращал новый пустой set.
    pub(crate) fn city_war_enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        BTreeSet::new()
    }

    /// Разрешает member faction-объекты и сохраняет pointer-identity dedupe.
    pub(crate) fn member_organizings<'a, T, Lookup>(&self, mut lookup: Lookup) -> Vec<&'a T>
    where
        T: ?Sized + 'a,
        Lookup: FnMut(i32) -> Option<&'a T>,
    {
        let mut output: Vec<&'a T> = Vec::new();
        for &member_id in self.members.keys() {
            if member_id < 1 {
                continue;
            }
            let Some(organizing) = lookup(member_id) else {
                continue;
            };
            if output
                .iter()
                .any(|current| std::ptr::eq(*current, organizing))
            {
                continue;
            }
            output.push(organizing);
        }
        output
    }

    /// Возвращает union ID только для существующего faction-member key.
    pub(crate) fn is_member(&self, faction_id: i32) -> i32 {
        if self.members.contains_key(&faction_id) {
            self.union_id
        } else {
            0
        }
    }

    /// Проверяет точное состояние `PST_Permit` одного faction-member права.
    pub(crate) fn is_using_purview(&self, faction_id: i32, purview: i32) -> bool {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return false;
        };
        if faction_id == 0 {
            return false;
        }
        self.members
            .get(&faction_id)
            .is_some_and(|member| member.purview[purview.index()] == EPurviewOwnState::Permit)
    }

    /// Переводит только `PST_No` в `PST_Permit`.
    pub(crate) fn set_member_purview(
        &mut self,
        faction_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&faction_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state != EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::Permit;
        MemberPurviewMutation::Changed
    }

    /// Переводит любое не-`PST_No` состояние в `PST_No`.
    pub(crate) fn abolish_member_purview(
        &mut self,
        faction_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&faction_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state == EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::No;
        MemberPurviewMutation::Changed
    }

    /// Проверяет право player-а через faction-master scan и union membership.
    pub(crate) fn check_operator_validate<Context>(
        &self,
        manager_player_id: i32,
        purview: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        let manager_faction_id = context.faction_id_by_master_player(manager_player_id)?;
        if manager_faction_id == 0 || self.is_member(manager_faction_id) == 0 {
            return Ok(false);
        }
        Ok(self.is_using_purview(manager_faction_id, purview))
    }

    /// Проверяет manager-а относительно другой member-faction.
    ///
    /// Это полный owner `CUnion::CheckOperValidate(manager, target, purview)`
    /// RVA `0x000C18D0`: выделенный Ghidra хвост `0x000C18DB` не является
    /// самостоятельной функцией и продолжает те же проверки membership и
    /// противоположности purview.
    pub(crate) fn check_operator_validate_target<Context>(
        &self,
        manager_player_id: i32,
        target_faction_id: i32,
        purview: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        if manager_player_id == target_faction_id {
            return Ok(false);
        }
        let manager_faction_id = context.faction_id_by_master_player(manager_player_id)?;
        if manager_faction_id == 0
            || target_faction_id == 0
            || target_faction_id == self.master_id
            || self.is_member(manager_faction_id) == 0
            || self.is_member(target_faction_id) == 0
        {
            return Ok(false);
        }
        let manager_permitted = self.is_using_purview(manager_faction_id, purview);
        if !manager_permitted {
            return Ok(false);
        }
        Ok(manager_permitted != self.is_using_purview(target_faction_id, purview))
    }

    /// Делегирует owned-city predicate только найденной master-faction.
    pub(crate) fn is_owned_city<Context>(&self, region_id: i32, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_is_owned_city(self.master_id, region_id)
            .unwrap_or(0)
    }

    /// Делегирует исторический enemy predicate только master-faction.
    pub(crate) fn is_enemy_faction<Context>(&self, enemy_id: i32, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_is_enemy_faction(self.master_id, enemy_id)
            .unwrap_or(0)
    }

    /// Возвращает независимый owned-city snapshot вместо dangling reference.
    pub(crate) fn owned_cities_snapshot<Context>(&self, context: &Context) -> VecDeque<i32>
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return VecDeque::new();
        }
        context
            .faction_owned_cities(self.master_id)
            .unwrap_or_default()
    }

    pub(crate) fn has_enemy_faction<Context>(&self, context: &Context) -> bool
    where
        Context: UnionMasterFactionQueryContext,
    {
        self.master_id > 0
            && context
                .faction_has_enemy(self.master_id)
                .unwrap_or(false)
    }

    pub(crate) fn has_city_war_enemy_faction<Context>(&self, context: &Context) -> bool
    where
        Context: UnionMasterFactionQueryContext,
    {
        self.master_id > 0
            && context
                .faction_has_city_war_enemy(self.master_id)
                .unwrap_or(false)
    }

    /// Делегирует enemy-leader query только найденной master-faction.
    pub(crate) fn enemy_leader_organizing_id<Context>(&self, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_enemy_leader_organizing_id(self.master_id)
            .unwrap_or(0)
    }

    fn fan_out_owned_city_mutation<Context, Dispatch>(
        &self,
        context: &mut Context,
        mut dispatch: Dispatch,
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
        Dispatch: FnMut(
            &mut Context,
            i32,
        ) -> Result<bool, OwnedCityMutationBuildError>,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            match dispatch(context, faction_id) {
                Ok(true) => invoked_faction_ids.push(faction_id),
                Ok(false) => {}
                Err(source) => {
                    return Err(UnionOwnedCityMutationBlock {
                        faction_id,
                        completed_faction_ids: invoked_faction_ids,
                        source,
                    });
                }
            }
        }
        Ok(UnionOwnedCityFanoutReport {
            invoked_faction_ids,
        })
    }

    /// Добавляет region каждой найденной положительной member-faction.
    pub(crate) fn add_owned_city<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_add_owned_city(faction_id, game, region_id, update_player)
        })
    }

    /// Добавляет исходный list каждой найденной положительной member-faction.
    pub(crate) fn add_owned_cities<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        region_ids: &VecDeque<i32>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_add_owned_cities(faction_id, game, region_ids, update_player)
        })
    }

    /// Очищает owned-city state всех найденных member-фракций.
    pub(crate) fn clear_owned_cities<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityBooleanMutationReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        let fanout = self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_clear_owned_cities(faction_id, game, update_player)
        })?;
        Ok(UnionOwnedCityBooleanMutationReport {
            legacy_result: true,
            invoked_faction_ids: fanout.invoked_faction_ids,
        })
    }

    /// Заменяет owned-city list только у найденной положительной master-faction.
    pub(crate) fn set_owned_cities<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        region_ids: &VecDeque<i32>,
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        if self.master_id <= 0 {
            return Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: Vec::new(),
            });
        }
        match context.faction_set_owned_cities(self.master_id, game, region_ids) {
            Ok(true) => Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: vec![self.master_id],
            }),
            Ok(false) => Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: Vec::new(),
            }),
            Err(source) => Err(UnionOwnedCityMutationBlock {
                faction_id: self.master_id,
                completed_faction_ids: Vec::new(),
                source,
            }),
        }
    }

    /// Сохраняет подтверждённый баг: аргумент игнорируется, master-list очищается.
    pub(crate) fn delete_owned_city<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        _region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityBooleanMutationReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        if self.master_id <= 0 {
            return Ok(UnionOwnedCityBooleanMutationReport {
                legacy_result: false,
                invoked_faction_ids: Vec::new(),
            });
        }
        match context.faction_clear_owned_cities(self.master_id, game, update_player) {
            Ok(found) => Ok(UnionOwnedCityBooleanMutationReport {
                legacy_result: found,
                invoked_faction_ids: found.then_some(self.master_id).into_iter().collect(),
            }),
            Err(source) => Err(UnionOwnedCityMutationBlock {
                faction_id: self.master_id,
                completed_faction_ids: Vec::new(),
                source,
            }),
        }
    }

    /// Очищает standard enemy-set каждой найденной member-faction.
    pub(crate) fn clear_enemy_factions<Context>(
        &self,
        context: &mut Context,
    ) -> UnionFactionFanoutReport
    where
        Context: UnionFactionStateMutationContext,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id > 0 && context.faction_clear_enemy_factions(faction_id) {
                invoked_faction_ids.push(faction_id);
            }
        }
        UnionFactionFanoutReport {
            invoked_faction_ids,
        }
    }

    /// Очищает city-war enemy-set каждой найденной member-faction.
    pub(crate) fn clear_city_war_enemy_factions<Context>(
        &self,
        context: &mut Context,
    ) -> UnionFactionFanoutReport
    where
        Context: UnionFactionStateMutationContext,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id > 0 && context.faction_clear_city_war_enemy_factions(faction_id) {
                invoked_faction_ids.push(faction_id);
            }
        }
        UnionFactionFanoutReport {
            invoked_faction_ids,
        }
    }

    fn fan_out_victor_mutation<Context, Dispatch>(
        &self,
        context: &mut Context,
        game: &CGame,
        mut dispatch: Dispatch,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
        Dispatch: FnMut(
            &mut Context,
            i32,
            &CGame,
        ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>,
    {
        let mut factions = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            match dispatch(context, faction_id, game) {
                Ok(Some(deliveries)) => factions.push(UnionVictorFactionReport {
                    faction_id,
                    deliveries,
                }),
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionVictorMutationBlock {
                        faction_id,
                        completed_factions: factions,
                        source,
                    });
                }
            }
        }
        Ok(UnionVictorFanoutReport { factions })
    }

    pub(crate) fn add_defence_victor_counts<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, game, |context, faction_id, game| {
            context.faction_add_defence_victor_count(faction_id, game)
        })
    }

    pub(crate) fn add_offense_victor_counts<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, game, |context, faction_id, game| {
            context.faction_add_offense_victor_count(faction_id, game)
        })
    }

    pub(crate) fn add_village_war_victor_counts<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, game, |context, faction_id, game| {
            context.faction_add_village_war_victor_count(faction_id, game)
        })
    }

    fn target_faction_ids(&self, faction_id: i32) -> Vec<i32> {
        if faction_id == 0 {
            self.members.keys().copied().collect()
        } else {
            vec![faction_id]
        }
    }

    /// Обновляет online player-ов одной faction либо всех member-фракций.
    pub(crate) fn update_player_faction_info<Context>(
        &self,
        faction_id: i32,
        context: &Context,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> UnionPlayerRefreshReport
    where
        Context: UnionPlayerRefreshContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(refreshed_player_ids) = context.faction_update_player_info(
                target_faction_id,
                game,
                update_player,
            ) else {
                continue;
            };
            factions.push(UnionFactionPlayerRefreshReport {
                faction_id: target_faction_id,
                refreshed_player_ids,
            });
        }
        UnionPlayerRefreshReport { factions }
    }

    /// Публикует полный standard enemy snapshot выбранных faction-target-ов.
    pub(crate) fn update_enemy_factions_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
        game: &CGame,
    ) -> UnionEnemySnapshotReport
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(deliveries) =
                context.faction_update_enemy_snapshot(target_faction_id, game)
            else {
                continue;
            };
            factions.push(UnionEnemySnapshotFactionReport {
                faction_id: target_faction_id,
                deliveries,
            });
        }
        UnionEnemySnapshotReport { factions }
    }

    /// Публикует полный city-war enemy snapshot выбранных faction-target-ов.
    pub(crate) fn update_city_war_enemy_factions_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
        game: &CGame,
    ) -> UnionEnemySnapshotReport
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(deliveries) =
                context.faction_update_city_war_enemy_snapshot(target_faction_id, game)
            else {
                continue;
            };
            factions.push(UnionEnemySnapshotFactionReport {
                faction_id: target_faction_id,
                deliveries,
            });
        }
        UnionEnemySnapshotReport { factions }
    }

    /// Публикует полный owned-city snapshot выбранных faction-target-ов.
    pub(crate) fn update_owned_cities_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
        game: &CGame,
    ) -> Result<UnionOwnedCitySnapshotReport, UnionOwnedCitySnapshotBlock>
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            match context.faction_update_owned_city_snapshot(target_faction_id, game) {
                Ok(Some(deliveries)) => factions.push(UnionOwnedCitySnapshotFactionReport {
                    faction_id: target_faction_id,
                    deliveries,
                }),
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionOwnedCitySnapshotBlock {
                        faction_id: target_faction_id,
                        completed_factions: factions,
                        source,
                    });
                }
            }
        }
        Ok(UnionOwnedCitySnapshotReport { factions })
    }

    /// Передаёт organizing-info всем найденным member-фракциям.
    pub(crate) fn send_info_to_all_members<'a, Context>(
        &self,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        color: u32,
        context: &Context,
        send_organizing_info: &mut dyn FnMut(FactionMemberInfoRequest<'a>),
    ) -> UnionInfoFanoutReport
    where
        Context: UnionSendInfoContext,
    {
        let mut factions = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            let Some(report) = context.faction_send_info_to_members(
                faction_id,
                first_text,
                second_text,
                information_type,
                color,
                send_organizing_info,
            ) else {
                continue;
            };
            factions.push(UnionFactionInfoFanout {
                faction_id,
                recipient_player_ids: report.recipient_player_ids,
            });
        }
        UnionInfoFanoutReport { factions }
    }

    /// Удаляет union-state у готовых клиентов одной либо всех member-фракций.
    pub(crate) fn delete_organizing_to_client<Context>(
        &self,
        faction_id: i32,
        context: &Context,
        game: &CGame,
    ) -> UnionDeleteOrganizingReport
    where
        Context: UnionFactionMemberContext,
    {
        let target_faction_ids: Vec<i32> = if faction_id > 0 {
            vec![faction_id]
        } else {
            self.members.keys().copied().collect()
        };
        let mut visited_faction_ids = Vec::new();
        let mut deliveries = Vec::new();
        for target_faction_id in target_faction_ids {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(recipient_player_ids) =
                context.faction_member_player_ids(target_faction_id)
            else {
                continue;
            };
            visited_faction_ids.push(target_faction_id);
            for recipient_player_id in recipient_player_ids {
                let player = game.online_player_by_id(recipient_player_id as u32);
                let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
                if player.is_none_or(|player| !player.faction_data_received())
                    || game_server_id == 0
                {
                    continue;
                }
                let mut message = CMessage::new(DELETE_UNION_ORGANIZING_MESSAGE_TYPE);
                message.base_mut().add_long(recipient_player_id);
                message.base_mut().add_long(self.union_id);
                deliveries.push(UnionDeleteOrganizingDelivery {
                    faction_id: target_faction_id,
                    recipient_player_id,
                    game_server_id,
                    result: game.send_msg_to_game_server(game_server_id, &message),
                });
            }
        }
        UnionDeleteOrganizingReport {
            visited_faction_ids,
            deliveries,
        }
    }

    /// Публикует delete либо полный union member-faction record всем клиентам.
    pub(crate) fn update_member_info_to_client<Context>(
        &mut self,
        game: &CGame,
        target_faction_id: i32,
        operator: EOperator,
        context: &Context,
    ) -> Result<UnionMemberUpdateReport, UnionMemberUpdateBlock>
    where
        Context: UnionFactionMemberContext,
    {
        let update_time = if operator == EOperator::Delete {
            None
        } else {
            let Some(current_level) = self.members.get(&target_faction_id).map(|member| member.level)
            else {
                return Ok(UnionMemberUpdateReport {
                    target_found: Some(false),
                    target_level: None,
                    deliveries: Vec::new(),
                });
            };
            match context.faction_level(target_faction_id) {
                Ok(Some(level)) => {
                    self.members
                        .get_mut(&target_faction_id)
                        .expect("target key проверен до faction lookup")
                        .level = level;
                }
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionMemberUpdateBlock::MissingFactionLevel {
                        source,
                        target_level: current_level,
                        completed_deliveries: Vec::new(),
                    });
                }
            }
            Some(current_local_member_time())
        };

        let recipient_faction_ids: Vec<i32> = self.members.keys().copied().collect();
        let mut deliveries = Vec::new();
        for recipient_faction_id in recipient_faction_ids {
            if recipient_faction_id <= 0 {
                continue;
            }
            let Some(recipient_player_ids) =
                context.faction_member_player_ids(recipient_faction_id)
            else {
                continue;
            };
            for recipient_player_id in recipient_player_ids {
                let player = game.online_player_by_id(recipient_player_id as u32);
                let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
                if player.is_none_or(|player| !player.faction_data_received())
                    || game_server_id == 0
                {
                    continue;
                }

                let mut message = CMessage::new(UNION_MEMBER_UPDATE_MESSAGE_TYPE);
                message.base_mut().add_long(recipient_player_id);
                message.base_mut().add_long(operator.wire_value());
                message.base_mut().add_long(target_faction_id);

                if let Some(update_time) = update_time {
                    let member_faction_id = self
                        .members
                        .get(&target_faction_id)
                        .expect("non-delete target существует до recipient-прохода")
                        .id;
                    if member_faction_id > 0 {
                        let current_level = self
                            .members
                            .get(&target_faction_id)
                            .expect("target существует до level refresh")
                            .level;
                        match context.faction_level(member_faction_id) {
                            Ok(Some(level)) => {
                                self.members
                                    .get_mut(&target_faction_id)
                                    .expect("target существует до level assignment")
                                    .level = level;
                            }
                            Ok(None) => {}
                            Err(source) => {
                                return Err(UnionMemberUpdateBlock::MissingFactionLevel {
                                    source,
                                    target_level: current_level,
                                    completed_deliveries: deliveries,
                                });
                            }
                        }
                    }

                    let target = self
                        .members
                        .get(&target_faction_id)
                        .expect("non-delete target существует до wire build");
                    let mut fields = Vec::new();
                    if let Err(field) =
                        append_union_member_update_fields(&mut fields, target, update_time)
                    {
                        return Err(UnionMemberUpdateBlock::UnterminatedField {
                            field,
                            target_level: target.level,
                            recipient_faction_id,
                            recipient_player_id,
                            game_server_id,
                            completed_deliveries: deliveries,
                        });
                    }
                    message.base_mut().add(&fields);
                }

                deliveries.push(UnionMemberUpdateDelivery {
                    recipient_faction_id,
                    recipient_player_id,
                    game_server_id,
                    result: game.send_msg_to_game_server(game_server_id, &message),
                });
            }
        }

        Ok(UnionMemberUpdateReport {
            target_found: (operator != EOperator::Delete).then_some(true),
            target_level: (operator != EOperator::Delete).then(|| {
                self.members
                    .get(&target_faction_id)
                    .expect("non-delete target существует после recipient-прохода")
                    .level
            }),
            deliveries,
        })
    }

    pub(crate) const fn change_data_type(&self) -> i32 {
        self.change_data_type
    }

    pub(crate) fn member_count(&self) -> usize {
        self.members.len()
    }
}

fn append_union_member_update_fields(
    output: &mut Vec<u8>,
    member: &TagMemInfo,
    update_time: TagTimeValue,
) -> Result<(), UnterminatedMemberField> {
    output.extend_from_slice(member.name_wire_bytes()?);
    output.extend_from_slice(&member.level.to_le_bytes());
    output.extend_from_slice(&member.occupation.to_le_bytes());
    output.extend_from_slice(&member.job_level.to_le_bytes());
    output.extend_from_slice(member.title_wire_bytes()?);
    output.extend_from_slice(&member.purview_wire_bytes());
    output.push(0);
    output.extend_from_slice(&update_time.wire_bytes());
    Ok(())
}

fn send_union_application_notice<Effects>(
    effects: &mut Effects,
    recipient_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: UnionApplyForJoinEffects,
{
    let second_text = effects.world_string(b"WS0193");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_union_invitation_notice<Effects>(
    effects: &mut Effects,
    recipient_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: UnionInviteEffects,
{
    let second_text = effects.world_string(b"WS0193");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_union_demise_notice<Effects>(
    effects: &mut Effects,
    recipient_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: UnionFireOutEffects,
{
    let second_text = effects.world_string(b"WS0119");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn append_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn legacy_c_string_visible_bytes(value: &[u8]) -> &[u8] {
    match value.iter().position(|byte| *byte == 0) {
        Some(terminator) => &value[..terminator],
        None => value,
    }
}

fn append_legacy_c_string(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(legacy_c_string_visible_bytes(value));
    output.push(0);
}
