//! Владелец organizing-control исторического `WorldServer`.
//!
//! Статус `COrganizingCtrl::AddOneTopInfo` RVA `0x00036960`,
//! `SendTopInfoToClient` RVA `0x00033FC0` и
//! `SendAllTopInfoToInfoToOneClient` RVA `0x000352C0` — `IMPLEMENTED`;
//! полный `COrganizingCtrl::Run` RVA `0x0003A550` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`, `DisbandFaction` RVA `0x00038550` и
//! `UpdateOtherFacInfoToClient` RVA `0x00034980` — `IMPLEMENTED`;
//! `IsFreePlayer` RVA `0x000343A0`, `IsFreeFaction` RVA `0x00034420`,
//! `RemovePersonFromApplyFactionList/GetFactionByPlayerInApplyList` RVA
//! `0x00034880/0x000348F0`,
//! `SetPlayerOrganizing` RVA `0x000370A0` и callback-цепочки
//! `OnPlayerEnterGame/OnPlayerExitGame` RVA `0x00037B70/0x00037BD0` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`; `GenerateSaveData` RVA `0x00034A10` —
//! `IMPLEMENTED`, `GetpFactionById` RVA `0x00034080` и
//! `ReInitialFacFactionByLvl` RVA `0x00034C80` — `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:73,164,242,725,1352,1641,1655,1910,1952,1960,1970,1998`.
//!
//! `GenerateSaveData` проходит faction-map, затем union-map в signed key-order.
//! `force_all=true` ставит bits `1/2/4/8` четырьмя virtual-вызовами; concrete
//! clone дописывается в `CGame::tagDBData`, после чего live mask сбрасывается.
//! Затем delete-faction и delete-union ID переносятся в list-order, и только
//! после полного переноса каждый исходный список очищается. Все достигнутые
//! caller-ы полного `CGame::GenerateDBData` передают `false`, но доказанная
//! bool-ветвь сохранена. Null map-value исходно разыменовывался при clone и
//! остаётся локальной typed-блокировкой; уже выполненные append/reset эффекты
//! не откатываются. `BTreeMap`, `VecDeque`, `Clone` и `Drop` заменяют только
//! MSVC tree/list/RTTI/allocator и compiler cleanup.
//!
//! `GetpFactionById` ищет signed ID только в `m_FacOrg`. Miss возвращает
//! `nullptr`; найденный null value возвращается тем же `nullptr` без
//! разыменования. Узкая Rust-граница возвращает borrow найденного concrete
//! `CFaction`; текущий DB-caller читает у него только Goods War count и затем
//! переносит его в save-копию. `BTreeMap::get`, `Option` и borrow заменяют
//! только MSVC iterator и nullable pointer; raw owner-блок после реализации
//! удалён.
//!
//! `IsFreePlayer` проходит `m_FacOrg` в порядке исходного ordered map и для
//! каждого `COrganizing*` вызывает virtual slot `+0xDC`. Точный PDB
//! именует faction-реализацию `CFaction::IsMember(long)`, а exact EXE
//! `0x004BD840..0x004BD865` подтверждает: ключом `m_Members.find` служит входной
//! player ID; при наличии возвращается signed `m_lID`, иначе `0`. Первый
//! положительный faction ID немедленно завершает обход. В отличие от двух
//! последующих callbacks, сам scan не проверяет map-value на null. Rust
//! сохраняет такую запись через `Option<Box<CFaction>>`, но останавливает её
//! локальным `BLOCKED_MISSING_FACT`, а не назначает старому null-dereference
//! продолжение либо fail-closed результат.
//!
//! `IsFreeFaction` симметрично проходит `m_ConfedeOrganizings`; PDB публикует
//! `CUnion::IsMember(long)` на том же folded RVA `0x000BD840`, поскольку ID и
//! member-map обоих concrete owners лежат по одинаковым offsets. Он ищет
//! входной faction ID и возвращает первый положительный union ID. Null map-
//! value остаётся такой же локальной неизвестностью старого разыменования.
//!
//! `RemovePersonFromApplyFactionList` вызывает `RemoveApplyMember(player)` у
//! каждой faction в signed map-order, игнорирует все concrete return values и
//! после полного прохода всегда возвращает `0`. Каждое фактическое удаление
//! поэтому успевает опубликовать пустой `OP_Delete`-record и поставить dirty
//! bit `8`, прежде чем обход продолжится. `GetFactionByPlayerInApplyList`
//! проходит тот же map и возвращает `GetID()` первой faction, чей
//! `IsInApplyMembers(player)` дал положительное значение; miss возвращает `0`.
//! Exact ASM `0x00434880..0x004348EC/0x004348F0..0x0043496F` подтверждает
//! virtual slots, ранний выход lookup и порядок. Rust отчёты сохраняют исходно
//! игнорировавшиеся результаты удалений; null map-value остаётся локальной
//! typed-границей после уже завершённого prefix-а.
//!
//! Exact `SetPlayerOrganizing` `0x004370A0..0x0043737A` сначала всегда пишет
//! результат `IsFreePlayer`. Для найденной faction он в точном порядке пишет
//! level, exp, contribute, name, member-title, master, оба enemy-set, очищает
//! owned-регионы и проходит faction city-list. Только существующий ненулевой
//! `tagRegion::pRegion` добавляется с младшими 16 битами `REGION_TYPE`; затем
//! независимо выполняются `IsFreeFaction` и условная запись union-master.
//! Exact key-dataflow исправляет повреждённые raw stack-slot имена для обоих
//! map lookup, `IsControbute` и `GetTitleByID`.
//!
//! `CPlayer::AddOwnedRegion` копировал все восемь байт локального `tagOwnedReg`,
//! хотя PDB задаёт только `long +0` и `unsigned short +4`: два последних байта
//! не инициализировались и позже наблюдались в player-wire. Rust сохраняет все
//! предшествующие мутации, но останавливается перед первым таким добавлением с
//! `BLOCKED_MISSING_FACT`; нули или иное значение padding не выдумываются.
//!
//! Constructor RVA `0x00036C90` создаёт `m_FacOrg` пустым. `CreateFaction` RVA
//! `0x000381A0` выделяет concrete `CFaction` и сохраняет base-pointer в map, а
//! `Release` RVA `0x00036F40` virtual-удаляет каждое ненулевое значение и
//! обнуляет slot. `BTreeMap<i32, Option<Box<CFaction>>>` заменяет MSVC tree,
//! pointer/null и доказанное владение, сохраняя signed key-порядок. Достигнутый
//! `m_ConfedeOrganizings` выражен симметричным map `CUnion`; Rust-layout не
//! выдаётся за Windows ABI, а остальные поля singleton-а остаются raw.
//!
//! `ReInitialFacFactionByLvl` проходит тот же signed faction-map. Exact EXE
//! `0x00434C80..0x00434CFE` делает RTTI cast каждого value, пропускает null либо
//! иной concrete type и вызывает `CFaction::ReInitialPropertyByLvl`; typed map
//! исключает посторонний concrete owner, а `Option::None` сохраняет skip.
//! Результаты уже выполненных property-send не откатываются при safe-границе
//! неполного Rust-owner-а.
//!
//! Exact `CUnion::DelMember` `0x004C2B30..0x004C2B8B` не читает union receiver:
//! для положительного входного ID он дважды использует один stack-slot как key
//! faction-map, затем вызывает `SetSuperiorOrganizing(0)` и
//! `UpdatePropertyToClient`; miss/null и неположительный ID пропускаются, а
//! возврат всегда `true`. Singleton lookup перенесён к фактическому map-owner-у
//! `COrganizingCtrl::detach_union_member`, без изменения порядка эффектов.
//!
//! Оба callback-а сначала сохраняют исходный player ID, вызывают
//! `IsFreePlayer`, а при положительном результате ищут именно этот faction ID
//! в `m_FacOrg`. Декомпилят ошибочно подставил this вместо ключа поиска find:
//! exact EXE `0x00437B7A..0x00437BA6` и `0x00437BDA..0x00437C06` сохраняет
//! результат в stack-slot и передаёт его и в `find`, и в `operator[]`.
//! Ненулевой faction получает исходный player ID: enter через slot `+0x13C`,
//! exit через `+0x140`; PDB именует их
//! `CFaction::OnMemberEnterGame/OnMemberExitGame`. Enter после любой
//! faction-ветви безусловно вызывает `SendAllTopInfoToInfoToOneClient`, exit
//! дополнительных эффектов не имеет. «Безусловно» относится к нормальному
//! возврату faction callback-а: если safe Rust достигает уже локализованного
//! старого UB внутри faction, top-info не исполняется как выдуманное
//! продолжение после невозвратившегося исходного пути. Exit так же передаёт
//! blocked-результат наружу, чтобы будущий `CGame::RemoveOnlinePlayer` не мог
//! продолжить операции после исходного невозврата.
//!
//! Typed dispatch различает отсутствие membership, отсутствие возвращённого
//! key, явно null target и выполненный concrete callback. Faction slots мутируют
//! полный `COrganizing::tagMemInfo` и публикуют member-update; enter
//! дополнительно читает online-player/region, затем на нормальном возврате
//! выполняет уже готовую top-info рассылку. Внутренности MSVC tree/string/list
//! и compiler cleanup не являются отдельной Rust-семантикой.
//!
//! `stTopInfo` содержит четыре 32-битных поля `lID/lTimerFlag/lParam/
//! dwStartTime` и byte-exact `std::string strInfo`; list-node размером `0x34`
//! подтверждает значение размером `0x2C`. Rust не копирует старый ABI:
//! `VecDeque<StTopInfo>` сохраняет хвостовую вставку и порядок обхода, `Vec<u8>`
//! — полное содержимое строки, а `Drop` — освобождение list/string storage.
//! Public Rust-граница принимает строковые bytes как `&[u8]`, потому что
//! by-value уничтожение входного MSVC `std::string` было только механизмом
//! владения; наблюдаемое копирование в запись либо сообщение сохранено.
//! Process-static `GetTopInfoID::lID` по exact адресу `0x0056A888` начинается с
//! `1`. Exact EXE `0x00436987..0x00436A68` возвращает прежний ID, wrapping
//! увеличивает static до снятия tick и не возвращает результат
//! `__security_check_cookie`; безопасный `AtomicI32::fetch_add` сохраняет
//! process-lifetime и 32-битное переполнение без исходной data race.
//!
//! Broadcast `SendTopInfoToClient` строит `0x7FA04` с player ID `0`, затем
//! `ID/timer/param/info\0`. Персональная отправка при непустом списке один раз
//! получает GameServer ID игрока и один wrapping boot tick, проходит записи в
//! порядке list и для `timer == 2` пропускает `elapsed >= param`; иначе в wire
//! попадает wrapping `param - elapsed`. Остальные timer-флаги передают исходный
//! param. Внутренний NUL сохранённой `std::string` обрезает именно C-string wire,
//! но не storage. Готовые World `CMessage` и `CGame` routing сохраняют
//! `SendAll/SendToMapID`; их исходно игнорировавшиеся результаты лишь входят в
//! typed Rust-отчёт и не останавливают следующий элемент. Пустой список не
//! читает GameServer registry и не снимает tick.
//!
//! В отличие от персональной отправки, top-info хвост `Run` безусловно снимает
//! ровно один tick даже при пустом списке. Exact EXE `0x0043A6BD..0x0043A742`
//! подтверждает unsigned условие удаления `timer == 2 && param <= now-start`,
//! предварительное сохранение следующего узла, уменьшение list-size и полный
//! проход до sentinel после каждого erase. `VecDeque::retain` сохраняет порядок
//! оставшихся записей, а `Drop` заменяет освобождение node/string. Этот участок
//! не читает и не меняет process-static top-info ID. Остальные поля полного
//! constructor-а остаются raw. Первая часть полного `Run` проходит faction-map
//! в signed order, уменьшает только положительный `m_lDelRemainTime` через
//! wrapping subtraction, собирает достигшие `<= 0` пары `(faction ID,
//! master ID)` во временный ordered map и лишь после traversal вызывает
//! `DisbandFaction(master, faction)`. `Run` принимает этот достигнутый вызов как
//! явный callback, чтобы caller мог передать Game/context без самозаимствования
//! controller-а; concrete `disband_faction` теперь материализован ниже.
//!
//! `UpdateOtherFacInfoToClient` проходит оставшиеся faction-owner-ы в signed
//! map-order, пропускает null value и вызывает concrete update slot `+0x170`.
//! `DisbandFaction` сначала повторяет standard/village и city/attack gates с
//! собственными `WS0235/WS0236 + WS0121`, очищает city enemy-set и вызывает
//! внутренний `CFaction::Disband`. После success exact order: erase map-owner,
//! второй `DeleteOrgaToClient(0)`, append ID в `m_DeleteFactions`, broadcast
//! empty-name `OP_Delete` оставшимся faction-ам, сброс player faction-data flag,
//! optional log и virtual delete. Exact ASM `0x00438550..0x004389CA`
//! подтверждает двойную delete-рассылку и исправляет raw dataflow: faction ID
//! в update/log — второй аргумент, player ID — первый. Наблюдаемая дубликация
//! сохранена; утечка 256-байтового SQL buffer при offline player устранена как
//! чисто внутренний дефект, а DB/war/player owners оставлены узким context.

use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicI32, Ordering};

use rustix::time::{ClockId, clock_gettime};

use super::faction::{
    CFaction, FactionCloneSaveBlock, FactionDeleteOrganizingBuildError,
    FactionDeleteOrganizingOutcome, FactionDisbandBlock, FactionDisbandContext,
    FactionDisbandOutcome, FactionDisbandProgress, FactionDisbandRejection,
    FactionInitialPropertyBlock, FactionMemberInfoRequest, FactionOrganizingInfoContext,
    FactionOtherInfoBuildError, FactionOtherInfoDelivery, FactionPropertyDelivery,
    FactionPropertyReinitialization, FactionRemoveApplyMemberOutcome,
    FactionSuperiorOrganizingBlock, MemberEnterOutcome, MemberExitOutcome,
};
use super::organizing::EOperator;
use super::organizingparam::COrganizingParam;
use super::union::CUnion;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::appworld::player::{
    PlayerOrganizingState, PlayerOrganizingUpdateError, PlayerOrganizingUpdater,
};
use crate::worldserver::worldserver::game::CGame;

const TOP_INFO_MESSAGE_TYPE: i32 = 0x7FA04;
const EXPIRING_TIMER_FLAG: i32 = 2;

static NEXT_TOP_INFO_ID: AtomicI32 = AtomicI32::new(1);

/// Достигнутая семантика исходного `stTopInfo` без копирования Windows ABI.
struct StTopInfo {
    id: i32,
    timer_flag: i32,
    param: i32,
    started_at_ms: u32,
    info: Vec<u8>,
}

/// Результат одной исходно игнорировавшейся отправки top-info.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TopInfoDelivery {
    pub(crate) top_info_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

/// Отчёт полного list-прохода для одного игрока.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TopInfoDeliveryReport {
    /// `None` означает точную раннюю ветвь пустого `m_TopInfos`.
    pub(crate) game_server_id: Option<i32>,
    pub(crate) skipped_expired: usize,
    pub(crate) deliveries: Vec<TopInfoDelivery>,
}

/// Typed-результат ordered `m_FacOrg` scan вместо старого null-dereference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FreePlayerLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}

/// Typed-результат ordered `m_ConfedeOrganizings` scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FreeFactionLookup {
    NoUnion,
    Union(i32),
    BlockedNullConfederation { map_key: i32 },
}

/// Результат одного вызова `CFaction::RemoveApplyMember` в map-order.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ApplyFactionRemoval {
    pub(crate) map_key: i32,
    pub(crate) outcome: FactionRemoveApplyMemberOutcome,
}

/// Полный normal-return либо точная null-pointer граница ordered прохода.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RemovePersonFromApplyFactionListOutcome {
    Completed {
        removals: Vec<ApplyFactionRemoval>,
    },
    BlockedNullFaction {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
}

/// Результат поиска первой faction, содержащей player в apply-list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApplyFactionLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}

/// Локальная safe-граница ordered `GenerateSaveData` traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingSaveDataBlock {
    NullFaction {
        map_key: i32,
    },
    FactionClone {
        map_key: i32,
        reason: FactionCloneSaveBlock,
    },
    NullConfederation {
        map_key: i32,
    },
}

/// Счётчики полностью завершённого `GenerateSaveData` прохода.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingSaveDataReport {
    pub(crate) saved_factions: usize,
    pub(crate) saved_unions: usize,
    pub(crate) deleted_factions: usize,
    pub(crate) deleted_unions: usize,
}

/// Локальная safe-граница полного faction countdown traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingRunBlock {
    NullFaction { map_key: i32 },
    DeleteRemainTimeAbsent { map_key: i32 },
    MasterIdAbsent { map_key: i32, faction_id: i32 },
}

/// Один вызов отдельного `DisbandFaction(master, faction)` owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingRunDisband {
    pub(crate) faction_id: i32,
    pub(crate) master_id: i32,
    pub(crate) result: bool,
}

/// Полный результат `COrganizingCtrl::Run` после normal return callbacks.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingRunReport {
    pub(crate) decremented_factions: usize,
    pub(crate) disbands: Vec<OrganizingRunDisband>,
    pub(crate) expired_top_infos: usize,
}

/// Фактически выбранная faction-ветвь enter callback-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionEnterDispatch {
    NoFactionMembership,
    FactionEntryMissing {
        faction_id: i32,
    },
    NullFactionPointer {
        faction_id: i32,
    },
    Called {
        faction_id: i32,
        outcome: MemberEnterOutcome,
    },
}

/// Полный результат `COrganizingCtrl::OnPlayerEnterGame`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PlayerEnterGameOutcome {
    BlockedDuringFactionScan {
        map_key: i32,
    },
    BlockedDuringFactionCallback {
        faction_id: i32,
        outcome: MemberEnterOutcome,
    },
    Completed {
        faction: FactionEnterDispatch,
        top_info: TopInfoDeliveryReport,
    },
}

/// Фактически выбранная faction-ветвь exit callback-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionExitDispatch {
    NoFactionMembership,
    FactionEntryMissing {
        faction_id: i32,
    },
    NullFactionPointer {
        faction_id: i32,
    },
    Called {
        faction_id: i32,
        outcome: MemberExitOutcome,
    },
}

/// Полный результат `COrganizingCtrl::OnPlayerExitGame`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PlayerExitGameOutcome {
    BlockedDuringFactionScan {
        map_key: i32,
    },
    BlockedDuringFactionCallback {
        faction_id: i32,
        outcome: MemberExitOutcome,
    },
    Dispatched(FactionExitDispatch),
}

/// Один успешно переинициализированный faction-owner в signed map-order.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionReinitializationEntry {
    pub(crate) map_key: i32,
    pub(crate) result: FactionPropertyReinitialization,
}

/// Safe-граница неполного concrete `CFaction` внутри старого pointer-map.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionReinitializationBlock {
    pub(crate) map_key: i32,
    pub(crate) source: FactionInitialPropertyBlock,
    /// Уже выполненные публикации исходный ordered проход не откатывал бы.
    pub(crate) completed: Vec<FactionReinitializationEntry>,
}

/// Нормальный результат исходного `CUnion::DelMember`, всегда возвращавшего true.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionMemberDetachOutcome {
    NonPositiveFactionId,
    FactionEntryMissing,
    NullFactionPointer,
    Detached {
        deliveries: Vec<FactionPropertyDelivery>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionMemberDetachBlockSource {
    SuperiorOrganizing(FactionSuperiorOrganizingBlock),
    Property(FactionInitialPropertyBlock),
}

/// Safe-граница частично материализованного faction-owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnionMemberDetachBlock {
    pub(crate) faction_id: i32,
    pub(crate) source: UnionMemberDetachBlockSource,
}

/// Один concrete faction-result controller-wide other-faction broadcast-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingOtherFactionUpdate {
    pub(crate) map_key: i32,
    pub(crate) deliveries: Vec<FactionOtherInfoDelivery>,
}

/// Safe-граница после уже выполненного prefix-а signed map traversal.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingOtherFactionUpdateBlock {
    pub(crate) map_key: i32,
    pub(crate) source: FactionOtherInfoBuildError,
    pub(crate) completed: Vec<OrganizingOtherFactionUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDisbandPlayer {
    pub(crate) player_id: i32,
    pub(crate) player_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDisbandRejection {
    FactionEntryMissing,
    StandardWar,
    CityWar,
    Faction(FactionDisbandRejection),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDisbandProgress {
    pub(crate) cleared_city_war_enemies: usize,
    pub(crate) faction: FactionDisbandProgress,
    pub(crate) map_removed: bool,
    pub(crate) second_delete_organizing: Option<FactionDeleteOrganizingOutcome>,
    pub(crate) delete_faction_queued: bool,
    pub(crate) other_faction_updates: Option<Vec<OrganizingOtherFactionUpdate>>,
    pub(crate) player: Option<OrganizingDisbandPlayer>,
    pub(crate) log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDisbandOutcome {
    Rejected {
        reason: OrganizingDisbandRejection,
        notice_sent: bool,
        cleared_city_war_enemies: usize,
    },
    Disbanded(OrganizingDisbandProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDisbandBlock {
    NullFaction {
        faction_id: i32,
    },
    Faction {
        source: FactionDisbandBlock,
        cleared_city_war_enemies: usize,
    },
    SecondDeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        progress: OrganizingDisbandProgress,
    },
    OtherFactionUpdate {
        source: OrganizingOtherFactionUpdateBlock,
        progress: OrganizingDisbandProgress,
    },
}

/// Player/log continuation внешнего `COrganizingCtrl::DisbandFaction`.
pub(crate) trait OrganizingDisbandContext: FactionDisbandContext {
    fn clear_player_faction_data_received(
        &mut self,
        player_id: i32,
    ) -> Option<OrganizingDisbandPlayer>;

    fn faction_disband_log_enabled(&self) -> bool;

    fn write_faction_disband_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
    );
}

/// Достигнутые faction-callback и top-info части исходного singleton owner-а.
pub(crate) struct COrganizingCtrl {
    factions: BTreeMap<i32, Option<Box<CFaction>>>,
    confederations: BTreeMap<i32, Option<Box<CUnion>>>,
    delete_factions: VecDeque<i32>,
    delete_unions: VecDeque<i32>,
    top_infos: VecDeque<StTopInfo>,
}

/// Явная per-call замена singleton/controller и `CGame::s_mapRegionList`.
pub(crate) struct COrganizingPlayerUpdater<'a> {
    controller: &'a COrganizingCtrl,
    region_types: &'a BTreeMap<i32, Option<u16>>,
}

impl COrganizingCtrl {
    /// Создаёт доказанные пустые organizing-map и `m_TopInfos`.
    pub(crate) const fn with_reached_callback_state() -> Self {
        Self {
            factions: BTreeMap::new(),
            confederations: BTreeMap::new(),
            delete_factions: VecDeque::new(),
            delete_unions: VecDeque::new(),
            top_infos: VecDeque::new(),
        }
    }

    /// Материализует organizing save/delete очереди в `CGame::tagDBData`.
    ///
    /// Оба map обходятся в signed key-order. После успешного clone live-mask
    /// сбрасывается, а delete-list очищается только после полного переноса.
    pub(crate) fn generate_save_data(
        &mut self,
        game: &CGame,
        force_all: bool,
    ) -> Result<OrganizingSaveDataReport, OrganizingSaveDataBlock> {
        let mut saved_factions = 0;
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                // BLOCKED_MISSING_FACT: RVA 0x00034A10 вызывает virtual
                // CloneSaveData через null map-value. Продолжение не доказано.
                return Err(OrganizingSaveDataBlock::NullFaction { map_key });
            };
            if force_all {
                for change_data_type in [1, 2, 4, 8] {
                    faction.set_change_data(change_data_type);
                }
            }
            let save_copy = faction
                .clone_save_data()
                .map_err(|reason| OrganizingSaveDataBlock::FactionClone { map_key, reason })?;
            if let Some(save_copy) = save_copy {
                game.append_save_faction(Box::new(save_copy));
                faction.set_change_data(0);
                saved_factions += 1;
            }
        }

        let mut saved_unions = 0;
        for (&map_key, union) in &mut self.confederations {
            let Some(union) = union.as_deref_mut() else {
                return Err(OrganizingSaveDataBlock::NullConfederation { map_key });
            };
            if force_all {
                for change_data_type in [1, 2, 4, 8] {
                    union.set_change_data(change_data_type);
                }
            }
            if let Some(save_copy) = union.clone_save_data() {
                game.append_save_union(Box::new(save_copy));
                union.set_change_data(0);
                saved_unions += 1;
            }
        }

        let deleted_factions = self.delete_factions.len();
        for &faction_id in &self.delete_factions {
            game.append_delete_faction(faction_id);
        }
        self.delete_factions.clear();

        let deleted_unions = self.delete_unions.len();
        for &union_id in &self.delete_unions {
            game.append_delete_union(union_id);
        }
        self.delete_unions.clear();

        Ok(OrganizingSaveDataReport {
            saved_factions,
            saved_unions,
            deleted_factions,
            deleted_unions,
        })
    }

    /// Возвращает ту же canonical faction, которую искал `GetpFactionById`.
    ///
    /// Отсутствующий key и сохранённый null pointer оба дают исходный
    /// `nullptr`; caller сам сохраняет последующую pointer-семантику.
    pub(crate) fn faction_by_id(&self, faction_id: i32) -> Option<&CFaction> {
        self.factions.get(&faction_id).and_then(Option::as_deref)
    }

    /// Публикует одно other-faction изменение всем concrete faction-owner-ам.
    pub(crate) fn update_other_faction_info_to_client(
        &self,
        game: &CGame,
        faction_id: i32,
        faction_name: &[u8],
        operator: EOperator,
    ) -> Result<Vec<OrganizingOtherFactionUpdate>, OrganizingOtherFactionUpdateBlock> {
        let mut completed = Vec::new();
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                continue;
            };
            let deliveries = match faction.update_other_faction_info_to_client(
                game,
                faction_id,
                faction_name,
                operator,
            ) {
                Ok(deliveries) => deliveries,
                Err(source) => {
                    return Err(OrganizingOtherFactionUpdateBlock {
                        map_key,
                        source,
                        completed,
                    });
                }
            };
            completed.push(OrganizingOtherFactionUpdate {
                map_key,
                deliveries,
            });
        }
        Ok(completed)
    }

    /// Выполняет оба исходных disband-слоя и удаляет concrete faction-owner.
    pub(crate) fn disband_faction<Context>(
        &mut self,
        game: &CGame,
        player_id: i32,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<OrganizingDisbandOutcome, OrganizingDisbandBlock>
    where
        Context: OrganizingDisbandContext,
    {
        let Some(faction) = self.factions.get_mut(&faction_id) else {
            return Ok(OrganizingDisbandOutcome::Rejected {
                reason: OrganizingDisbandRejection::FactionEntryMissing,
                notice_sent: false,
                cleared_city_war_enemies: 0,
            });
        };
        let Some(faction) = faction.as_deref_mut() else {
            return Err(OrganizingDisbandBlock::NullFaction { faction_id });
        };

        if faction.has_enemy_faction() || context.village_war_declared(faction_id) {
            send_disband_information(context, player_id, b"WS0235");
            return Ok(OrganizingDisbandOutcome::Rejected {
                reason: OrganizingDisbandRejection::StandardWar,
                notice_sent: true,
                cleared_city_war_enemies: 0,
            });
        }
        if faction.has_city_war_enemy_faction() || context.city_war_declared(faction_id) {
            send_disband_information(context, player_id, b"WS0236");
            return Ok(OrganizingDisbandOutcome::Rejected {
                reason: OrganizingDisbandRejection::CityWar,
                notice_sent: true,
                cleared_city_war_enemies: 0,
            });
        }

        let cleared_city_war_enemies = faction.city_war_enemy_factions().len();
        faction.clear_city_war_enemy_factions();
        let faction_progress = match faction.disband(game, player_id, context) {
            Ok(FactionDisbandOutcome::Rejected {
                reason,
                notice_sent,
            }) => {
                return Ok(OrganizingDisbandOutcome::Rejected {
                    reason: OrganizingDisbandRejection::Faction(reason),
                    notice_sent,
                    cleared_city_war_enemies,
                });
            }
            Ok(FactionDisbandOutcome::Disbanded(progress)) => progress,
            Err(source) => {
                return Err(OrganizingDisbandBlock::Faction {
                    source,
                    cleared_city_war_enemies,
                });
            }
        };

        let faction = self
            .factions
            .remove(&faction_id)
            .and_then(|faction| faction)
            .expect("успешный faction Disband не меняет controller map");
        let mut progress = OrganizingDisbandProgress {
            cleared_city_war_enemies,
            faction: faction_progress,
            map_removed: true,
            second_delete_organizing: None,
            delete_faction_queued: false,
            other_faction_updates: None,
            player: None,
            log_written: false,
        };
        progress.second_delete_organizing = Some(
            match faction.delete_organizing_to_client(game, 0, context) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Err(OrganizingDisbandBlock::SecondDeleteOrganizing {
                        source,
                        progress,
                    });
                }
            },
        );

        self.delete_factions.push_back(faction.faction_id());
        progress.delete_faction_queued = true;
        progress.other_faction_updates = Some(
            match self.update_other_faction_info_to_client(game, faction_id, b"", EOperator::Delete)
            {
                Ok(updates) => updates,
                Err(source) => {
                    return Err(OrganizingDisbandBlock::OtherFactionUpdate { source, progress });
                }
            },
        );

        progress.player = context.clear_player_faction_data_received(player_id);
        if context.faction_disband_log_enabled()
            && let Some(player) = progress.player.as_ref()
        {
            context.write_faction_disband_log(
                faction_id,
                legacy_c_string_prefix(faction.name()),
                player.player_id,
                legacy_c_string_prefix(&player.player_name),
            );
            progress.log_written = true;
        }

        Ok(OrganizingDisbandOutcome::Disbanded(progress))
    }

    /// Пересчитывает и публикует property всех concrete faction-owner-ов.
    pub(crate) fn reinitialize_factions_by_level(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> Result<Vec<FactionReinitializationEntry>, FactionReinitializationBlock> {
        let mut completed = Vec::new();
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                // Exact RTTI cast null/non-CFaction pointer пропускал.
                continue;
            };
            let result = match faction.reinitialize_property_by_level(game, parameters) {
                Ok(result) => result,
                Err(source) => {
                    return Err(FactionReinitializationBlock {
                        map_key,
                        source,
                        completed,
                    });
                }
            };
            completed.push(FactionReinitializationEntry { map_key, result });
        }
        Ok(completed)
    }

    /// Отвязывает faction от union в точном порядке `CUnion::DelMember`.
    pub(crate) fn detach_union_member(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<UnionMemberDetachOutcome, UnionMemberDetachBlock> {
        if faction_id <= 0 {
            return Ok(UnionMemberDetachOutcome::NonPositiveFactionId);
        }
        let Some(faction) = self.factions.get_mut(&faction_id) else {
            return Ok(UnionMemberDetachOutcome::FactionEntryMissing);
        };
        let Some(faction) = faction.as_deref_mut() else {
            return Ok(UnionMemberDetachOutcome::NullFactionPointer);
        };

        faction
            .set_superior_organizing(0, parameters)
            .map_err(|source| UnionMemberDetachBlock {
                faction_id,
                source: UnionMemberDetachBlockSource::SuperiorOrganizing(source),
            })?;
        let deliveries = faction.update_property_to_client(game).map_err(|source| {
            UnionMemberDetachBlock {
                faction_id,
                source: UnionMemberDetachBlockSource::Property(source),
            }
        })?;
        Ok(UnionMemberDetachOutcome::Detached { deliveries })
    }

    /// Ищет первый положительный faction ID в signed map-порядке.
    pub(crate) fn is_free_player(&self, player_id: i32) -> FreePlayerLookup {
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                // BLOCKED_MISSING_FACT: `IsFreePlayer` RVA `0x000343A0`
                // разыменовывает map-value без null-check. Достижимость и
                // наблюдаемая реакция null не доказаны.
                return FreePlayerLookup::BlockedNullFaction { map_key };
            };
            let faction_id = faction.is_member(player_id);
            if faction_id > 0 {
                return FreePlayerLookup::Faction(faction_id);
            }
        }
        FreePlayerLookup::NoFaction
    }

    /// Ищет первый положительный union ID в signed map-порядке.
    pub(crate) fn is_free_faction(&self, faction_id: i32) -> FreeFactionLookup {
        for (&map_key, union) in &self.confederations {
            let Some(union) = union.as_deref() else {
                // BLOCKED_MISSING_FACT: `IsFreeFaction` RVA `0x00034420`
                // разыменовывает map-value без null-check. Достижимость и
                // наблюдаемая реакция null не доказаны.
                return FreeFactionLookup::BlockedNullConfederation { map_key };
            };
            let union_id = union.is_member(faction_id);
            if union_id > 0 {
                return FreeFactionLookup::Union(union_id);
            }
        }
        FreeFactionLookup::NoUnion
    }

    /// Удаляет player из apply-list каждой faction и на normal return даёт `0`.
    pub(crate) fn remove_person_from_apply_faction_list(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> RemovePersonFromApplyFactionListOutcome {
        let mut removals = Vec::with_capacity(self.factions.len());
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                return RemovePersonFromApplyFactionListOutcome::BlockedNullFaction {
                    map_key,
                    completed_removals: removals,
                };
            };
            removals.push(ApplyFactionRemoval {
                map_key,
                outcome: faction.remove_apply_member(game, player_id),
            });
        }
        RemovePersonFromApplyFactionListOutcome::Completed { removals }
    }

    /// Возвращает ID первой faction с положительным apply-membership.
    pub(crate) fn faction_by_player_in_apply_list(&self, player_id: i32) -> ApplyFactionLookup {
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                return ApplyFactionLookup::BlockedNullFaction { map_key };
            };
            if faction.is_in_apply_members(player_id) > 0 {
                return ApplyFactionLookup::Faction(faction.faction_id());
            }
        }
        ApplyFactionLookup::NoFaction
    }

    /// Связывает один точный SetPlayerOrganizing с достигнутым region snapshot.
    pub(crate) const fn player_updater<'a>(
        &'a self,
        region_types: &'a BTreeMap<i32, Option<u16>>,
    ) -> COrganizingPlayerUpdater<'a> {
        COrganizingPlayerUpdater {
            controller: self,
            region_types,
        }
    }

    /// Выполняет faction enter-ветвь и затем безусловную top-info отправку.
    pub(crate) fn on_player_enter_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> PlayerEnterGameOutcome {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => None,
            FreePlayerLookup::Faction(faction_id) => Some(faction_id),
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return PlayerEnterGameOutcome::BlockedDuringFactionScan { map_key };
            }
        };

        let faction = match faction_id {
            None => FactionEnterDispatch::NoFactionMembership,
            Some(faction_id) => match self.factions.get_mut(&faction_id) {
                None => FactionEnterDispatch::FactionEntryMissing { faction_id },
                Some(None) => FactionEnterDispatch::NullFactionPointer { faction_id },
                Some(Some(faction)) => {
                    let outcome = faction.on_member_enter_game(game, player_id);
                    if matches!(
                        &outcome,
                        MemberEnterOutcome::Blocked(_) | MemberEnterOutcome::Published(Err(_))
                    ) {
                        // Старый faction callback на этих UB-границах не
                        // возвращался бы к последующей top-info отправке.
                        return PlayerEnterGameOutcome::BlockedDuringFactionCallback {
                            faction_id,
                            outcome,
                        };
                    }
                    FactionEnterDispatch::Called {
                        faction_id,
                        outcome,
                    }
                }
            },
        };

        let top_info = self.send_all_top_info_to_one_client(game, player_id);
        PlayerEnterGameOutcome::Completed { faction, top_info }
    }

    /// Выполняет единственную faction exit-ветвь без дополнительных эффектов.
    pub(crate) fn on_player_exit_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> PlayerExitGameOutcome {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => None,
            FreePlayerLookup::Faction(faction_id) => Some(faction_id),
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return PlayerExitGameOutcome::BlockedDuringFactionScan { map_key };
            }
        };

        let faction = match faction_id {
            None => FactionExitDispatch::NoFactionMembership,
            Some(faction_id) => match self.factions.get_mut(&faction_id) {
                None => FactionExitDispatch::FactionEntryMissing { faction_id },
                Some(None) => FactionExitDispatch::NullFactionPointer { faction_id },
                Some(Some(faction)) => {
                    let outcome = faction.on_member_exit_game(game, player_id);
                    if matches!(&outcome, MemberExitOutcome::Published(Err(_))) {
                        return PlayerExitGameOutcome::BlockedDuringFactionCallback {
                            faction_id,
                            outcome,
                        };
                    }
                    FactionExitDispatch::Called {
                        faction_id,
                        outcome,
                    }
                }
            },
        };
        PlayerExitGameOutcome::Dispatched(faction)
    }

    /// Сохраняет новую top-info запись в хвост и возвращает process-static ID.
    pub(crate) fn add_one_top_info(&mut self, timer_flag: i32, param: i32, info: &[u8]) -> i32 {
        let id = NEXT_TOP_INFO_ID.fetch_add(1, Ordering::Relaxed);
        let started_at_ms = legacy_tick_ms();
        self.top_infos.push_back(StTopInfo {
            id,
            timer_flag,
            param,
            started_at_ms,
            info: info.to_vec(),
        });
        id
    }

    /// Удаляет все истёкшие timer-2 записи, сохраняя порядок остальных.
    pub(crate) fn run_top_info_expiry(&mut self) -> usize {
        let now_ms = legacy_tick_ms();
        let old_len = self.top_infos.len();
        self.top_infos.retain(|top_info| {
            top_info.timer_flag != EXPIRING_TIMER_FLAG
                || (top_info.param as u32) > now_ms.wrapping_sub(top_info.started_at_ms)
        });
        old_len - self.top_infos.len()
    }

    /// Выполняет полный ordered countdown, disband-dispatch и top-info expiry.
    pub(crate) fn run<Disband>(
        &mut self,
        minute_delta: i32,
        mut disband_faction: Disband,
    ) -> Result<OrganizingRunReport, OrganizingRunBlock>
    where
        Disband: FnMut(&mut Self, i32, i32) -> bool,
    {
        let mut pending_disbands = BTreeMap::new();
        let mut decremented_factions = 0;
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                return Err(OrganizingRunBlock::NullFaction { map_key });
            };
            let Some(remaining) = faction.delete_remain_time() else {
                return Err(OrganizingRunBlock::DeleteRemainTimeAbsent { map_key });
            };
            if 0 < remaining {
                let remaining = remaining.wrapping_sub(minute_delta);
                faction.set_delete_remain_time(remaining);
                decremented_factions += 1;
                if remaining < 1 {
                    let faction_id = faction.faction_id();
                    let master_id =
                        faction
                            .master_id()
                            .ok_or(OrganizingRunBlock::MasterIdAbsent {
                                map_key,
                                faction_id,
                            })?;
                    pending_disbands.insert(faction_id, master_id);
                }
            }
        }

        let mut disbands = Vec::with_capacity(pending_disbands.len());
        for (faction_id, master_id) in pending_disbands {
            let result = disband_faction(self, master_id, faction_id);
            disbands.push(OrganizingRunDisband {
                faction_id,
                master_id,
                result,
            });
        }
        let expired_top_infos = self.run_top_info_expiry();
        Ok(OrganizingRunReport {
            decremented_factions,
            disbands,
            expired_top_infos,
        })
    }

    /// Рассылает одну top-info запись всем GameServer с player ID `0`.
    pub(crate) fn send_top_info_to_client(
        &self,
        game: &CGame,
        top_info_id: i32,
        timer_flag: i32,
        param: i32,
        info: &[u8],
    ) -> Result<i32, SendMessageError> {
        let message = top_info_message(0, top_info_id, timer_flag, param, info);
        let sender = game.current_game_server_sender();
        message.send_all(sender.as_ref())
    }

    /// Отправляет одному игроку все неистёкшие top-info записи в list-порядке.
    pub(crate) fn send_all_top_info_to_one_client(
        &self,
        game: &CGame,
        player_id: i32,
    ) -> TopInfoDeliveryReport {
        if self.top_infos.is_empty() {
            return TopInfoDeliveryReport {
                game_server_id: None,
                skipped_expired: 0,
                deliveries: Vec::new(),
            };
        }

        let game_server_id = game.game_server_number_by_player_id(player_id);
        let now_ms = legacy_tick_ms();
        let mut skipped_expired = 0;
        let mut deliveries = Vec::with_capacity(self.top_infos.len());

        for top_info in &self.top_infos {
            let param = if top_info.timer_flag == EXPIRING_TIMER_FLAG {
                let elapsed_ms = now_ms.wrapping_sub(top_info.started_at_ms);
                if elapsed_ms >= top_info.param as u32 {
                    skipped_expired += 1;
                    continue;
                }
                (top_info.param as u32).wrapping_add(top_info.started_at_ms.wrapping_sub(now_ms))
                    as i32
            } else {
                top_info.param
            };
            let message = top_info_message(
                player_id,
                top_info.id,
                top_info.timer_flag,
                param,
                &top_info.info,
            );
            deliveries.push(TopInfoDelivery {
                top_info_id: top_info.id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }

        TopInfoDeliveryReport {
            game_server_id: Some(game_server_id),
            skipped_expired,
            deliveries,
        }
    }
}

impl PlayerOrganizingUpdater for COrganizingPlayerUpdater<'_> {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        let faction_id = match self.controller.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => 0,
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(
                    PlayerOrganizingUpdateError::NullFactionDuringMembershipScan { map_key },
                );
            }
        };
        organizing.faction_id = faction_id;

        if faction_id > 0
            && let Some(Some(faction)) = self.controller.factions.get(&faction_id)
        {
            let level =
                faction
                    .level()
                    .ok_or(PlayerOrganizingUpdateError::UninitializedFactionField {
                        faction_id,
                        field: "m_Property.lLvl",
                    })?;
            organizing.faction_level = level as u16;

            organizing.faction_experience = faction.experience().ok_or(
                PlayerOrganizingUpdateError::UninitializedFactionField {
                    faction_id,
                    field: "m_Property.lExp",
                },
            )?;
            organizing.faction_contribute = faction.is_contribute(player_id);
            organizing.faction_name = faction.name().to_vec();
            organizing.faction_title = faction.member_title(player_id).map_err(|_| {
                PlayerOrganizingUpdateError::UnterminatedFactionMemberTitle {
                    faction_id,
                    player_id,
                }
            })?;
            organizing.faction_master_id = faction.master_id().ok_or(
                PlayerOrganizingUpdateError::UninitializedFactionField {
                    faction_id,
                    field: "m_lMastterID",
                },
            )?;
            organizing.enemy_factions = faction.enemy_factions().clone();
            organizing.city_war_enemy_factions = faction.city_war_enemy_factions().clone();
            organizing.owned_regions.clear();

            for &region_id in faction.owned_cities() {
                let Some(region_type) = self.region_types.get(&region_id) else {
                    continue;
                };
                let Some(_region_type) = *region_type else {
                    return Err(PlayerOrganizingUpdateError::UninitializedRegionType { region_id });
                };
                // BLOCKED_MISSING_FACT: AddOwnedRegion RVA `0x0005DD10`
                // копирует два неинициализированных padding-байта local
                // tagOwnedReg. Они входят в последующий player-wire.
                return Err(
                    PlayerOrganizingUpdateError::UninitializedOwnedRegionPadding { region_id },
                );
            }
        }

        let union_id = match self.controller.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => 0,
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(
                    PlayerOrganizingUpdateError::NullConfederationDuringMembershipScan { map_key },
                );
            }
        };
        organizing.union_id = union_id;

        if union_id > 0
            && let Some(Some(union)) = self.controller.confederations.get(&union_id)
        {
            organizing.union_master_id = union.master_id();
        }
        Ok(())
    }
}

fn send_disband_information<Context>(
    context: &mut Context,
    player_id: i32,
    first_string_id: &'static [u8],
) where
    Context: FactionOrganizingInfoContext + ?Sized,
{
    let second_text = context.world_string(b"WS0121").unwrap_or_default();
    let first_text = context.world_string(first_string_id).unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn top_info_message(
    player_id: i32,
    top_info_id: i32,
    timer_flag: i32,
    param: i32,
    info: &[u8],
) -> CMessage {
    let mut message = CMessage::new(TOP_INFO_MESSAGE_TYPE);
    let payload = message.base_mut();
    payload.add_long(player_id);
    payload.add_long(top_info_id);
    payload.add_long(timer_flag);
    payload.add_long(param);
    let info = legacy_c_string_prefix(info);
    payload.add(info);
    payload.add_byte(0);
    message
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1_000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h:117
// RVA: 0x0001C050
// ADDRESS: 0041c050
// PROTOTYPE: COrganizing * __thiscall GetFactionOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionNumber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:445
// RVA: 0x00033560
// ADDRESS: 00433560
// PROTOTYPE: long __thiscall GetFactionNumber(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:937
// RVA: 0x00033680
// ADDRESS: 00433680
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SendOrgaInfoToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1923
// RVA: 0x00033750
// ADDRESS: 00433750
// PROTOTYPE: void __thiscall SendOrgaInfoToClient(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, long param_4, ulong param_5, ulong param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SendOrgaInfoToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1939
// RVA: 0x00033840
// ADDRESS: 00433840
// PROTOTYPE: void __thiscall SendOrgaInfoToClient(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionMemBillboardToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:333
// RVA: 0x000339D0
// ADDRESS: 004339d0
// PROTOTYPE: void __thiscall AddFactionMemBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionOffBillboardToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:348
// RVA: 0x00033A50
// ADDRESS: 00433a50
// PROTOTYPE: void __thiscall AddFactionOffBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionDefBillboardToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:363
// RVA: 0x00033AD0
// ADDRESS: 00433ad0
// PROTOTYPE: void __thiscall AddFactionDefBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsInEstaList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h:143
// RVA: 0x00033BA0
// ADDRESS: 00433ba0
// PROTOTYPE: bool __thiscall IsInEstaList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionBillboardToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:312
// RVA: 0x00033CC0
// ADDRESS: 00433cc0
// PROTOTYPE: void __thiscall AddFactionBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionNumber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:451
// RVA: 0x00033D10
// ADDRESS: 00433d10
// PROTOTYPE: long __thiscall GetFactionNumber(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionListToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:492
// RVA: 0x00033D90
// ADDRESS: 00433d90
// PROTOTYPE: void __thiscall AddFactionListToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, long param_2, uchar param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::PlayerTransferOwnerCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1211
// RVA: 0x00033F40
// ADDRESS: 00433f40
// PROTOTYPE: undefined __thiscall PlayerTransferOwnerCity(long param_1, long param_2, long param_3, long param_4, long param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GenerateDBOrganizingID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:214
// RVA: 0x000341C0
// ADDRESS: 004341c0
// PROTOTYPE: long __thiscall GenerateDBOrganizingID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SetAllCityFacEnemyChanged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1312
// RVA: 0x00034240
// ADDRESS: 00434240
// PROTOTYPE: void __thiscall SetAllCityFacEnemyChanged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::ClearAllCityFacRelation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1322
// RVA: 0x000342C0
// ADDRESS: 004342c0
// PROTOTYPE: void __thiscall ClearAllCityFacRelation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::UpdateAllCityEneFacRelation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1340
// RVA: 0x00034330
// ADDRESS: 00434330
// PROTOTYPE: void __thiscall UpdateAllCityEneFacRelation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsFreeFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1368
// RVA: 0x00034420
// ADDRESS: 00434420
// PROTOTYPE: long __thiscall IsFreeFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsFactionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1398
// RVA: 0x000344A0
// ADDRESS: 004344a0
// PROTOTYPE: long __thiscall IsFactionMaster(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsConferationMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1414
// RVA: 0x00034520
// ADDRESS: 00434520
// PROTOTYPE: long __thiscall IsConferationMaster(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::FindOrgaByName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1436
// RVA: 0x000345A0
// ADDRESS: 004345a0
// PROTOTYPE: COrganizing * __thiscall FindOrgaByName(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::ReSetPermitDemise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1522
// RVA: 0x00034810
// ADDRESS: 00434810
// PROTOTYPE: void __thiscall ReSetPermitDemise(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::RemovePersonFromApplyFactionList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1531
// RVA: 0x00034880
// ADDRESS: 00434880
// PROTOTYPE: long __thiscall RemovePersonFromApplyFactionList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionByPlayerInApplyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1542
// RVA: 0x000348F0
// ADDRESS: 004348f0
// PROTOTYPE: long __thiscall GetFactionByPlayerInApplyList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::UpdateOtherFacInfoToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1910
// RVA: 0x00034980
// ADDRESS: 00434980
// PROTOTYPE: void __thiscall UpdateOtherFacInfoToClient(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: COrganizingCtrl::GenerateSaveData RVA 0x00034A10.

// ============================================================================
// FUNCTION: COrganizingCtrl::ReInitialFacFactionByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:2050
// RVA: 0x00034C80
// ADDRESS: 00434c80
// PROTOTYPE: void __thiscall ReInitialFacFactionByLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddAllFactinInfoToClientByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:405
// RVA: 0x00034D00
// ADDRESS: 00434d00
// PROTOTYPE: bool __thiscall AddAllFactinInfoToClientByPlayerID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::CreateUnion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:931
// RVA: 0x00034E90
// ADDRESS: 00434e90
// PROTOTYPE: undefined __thiscall CreateUnion(long param_1, long param_2, long param_3, long param_4, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:934
// RVA: 0x00034EF0
// ADDRESS: 00434ef0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1214
// RVA: 0x00034F10
// ADDRESS: 00434f10
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::PushToEstaList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h:149
// RVA: 0x000367C0
// ADDRESS: 004367c0
// PROTOTYPE: void __thiscall PushToEstaList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::~COrganizingCtrl
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:87
// RVA: 0x00036830
// ADDRESS: 00436830
// PROTOTYPE: void __thiscall ~COrganizingCtrl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetConfederationOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h:132
// RVA: 0x00036BF0
// ADDRESS: 00436bf0
// PROTOTYPE: COrganizing * __thiscall GetConfederationOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::COrganizingCtrl
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:73
// RVA: 0x00036C90
// ADDRESS: 00436c90
// PROTOTYPE: undefined __thiscall COrganizingCtrl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:93
// RVA: 0x00036E90
// ADDRESS: 00436e90
// PROTOTYPE: COrganizingCtrl * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:143
// RVA: 0x00036F40
// ADDRESS: 00436f40
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:234
// RVA: 0x00037030
// ADDRESS: 00437030
// PROTOTYPE: void __thiscall AddFactionOrganizing(long param_1, COrganizing * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SetPlayerOrganizing
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:242
// RVA: 0x000370A0
//
// Реализация находится в `COrganizingPlayerUpdater`; exact диапазон
// `0x004370A0..0x0043737A` исправляет raw key/dataflow и восстанавливает
// пропущенные city-war/owned-region операции перед union-ветвью.
//
// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionToClientByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:379
// RVA: 0x00037380
// ADDRESS: 00437380
// PROTOTYPE: bool __thiscall AddFactionToClientByPlayerID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddDeclareWarFactionInfoToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:532
// RVA: 0x000374B0
// ADDRESS: 004374b0
// PROTOTYPE: void __thiscall AddDeclareWarFactionInfoToByteArray(long param_1, vector<unsigned_char,std::allocator<unsigned_char>_> * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddUnionToClientByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:595
// RVA: 0x000376F0
// ADDRESS: 004376f0
// PROTOTYPE: bool __thiscall AddUnionToClientByPlayerID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1217
// RVA: 0x00037810
// ADDRESS: 00437810
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SetEnemyFactionRelation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1296
// RVA: 0x00037A40
// ADDRESS: 00437a40
// PROTOTYPE: void __thiscall SetEnemyFactionRelation(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetUnion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1502
// RVA: 0x00037AF0
// ADDRESS: 00437af0
// PROTOTYPE: COrganizing * __thiscall GetUnion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetCountryByFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1514
// RVA: 0x00037B20
// ADDRESS: 00437b20
// PROTOTYPE: uchar __thiscall GetCountryByFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddOwnedCityToFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1717
// RVA: 0x00037C20
// ADDRESS: 00437c20
// PROTOTYPE: void __thiscall AddOwnedCityToFaction(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::OnAttackCityEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1834
// RVA: 0x00037C70
// ADDRESS: 00437c70
// PROTOTYPE: void __thiscall OnAttackCityEnd(long param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddUnionToClientByFactionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:626
// RVA: 0x00038010
// ADDRESS: 00438010
// PROTOTYPE: bool __thiscall AddUnionToClientByFactionID(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::CreateFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:651
// RVA: 0x000381A0
// ADDRESS: 004381a0
// PROTOTYPE: eCrOrgResult __thiscall CreateFaction(long param_1, long param_2, tagTime * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4, uchar param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::DisbandFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:725
// RVA: 0x00038550
// ADDRESS: 00438550
// PROTOTYPE: bool __thiscall DisbandFaction(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::CreateConfederation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:867
// RVA: 0x000389D0
// ADDRESS: 004389d0
// PROTOTYPE: eCrOrgResult __thiscall CreateConfederation(long param_1, long param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:953
// RVA: 0x00038D80
// ADDRESS: 00438d80
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::DisbandConferation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1053
// RVA: 0x000393B0
// ADDRESS: 004393b0
// PROTOTYPE: bool __thiscall DisbandConferation(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::TransferIOwnerCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1105
// RVA: 0x00039890
// ADDRESS: 00439890
// PROTOTYPE: bool __thiscall TransferIOwnerCity(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1238
// RVA: 0x0003A070
// ADDRESS: 0043a070
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::OnDeleteRole
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1665
// RVA: 0x0003A2C0
// ADDRESS: 0043a2c0
// PROTOTYPE: int __thiscall OnDeleteRole(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::OnNewDay
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:2064
// RVA: 0x0003A490
// ADDRESS: 0043a490
// PROTOTYPE: void __stdcall OnNewDay(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// VERIFIED_DISASSEMBLY, IMPLEMENTED: полный `COrganizingCtrl::Run` RVA
// `0x0003A550` находится выше; temporary-map/STL traversal заменён typed
// callback к материализованному `DisbandFaction` owner-у.

// ============================================================================
// FUNCTION: COrganizingCtrl::OnPlayerInviteFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:790
// RVA: 0x0003A780
// ADDRESS: 0043a780
// PROTOTYPE: bool __thiscall OnPlayerInviteFaction(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatMemberNumBillboard
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1738
// RVA: 0x0003AC70
// ADDRESS: 0043ac70
// PROTOTYPE: void __thiscall StatMemberNumBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatOffenseVictorCountsBillboard
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1768
// RVA: 0x0003B050
// ADDRESS: 0043b050
// PROTOTYPE: void __thiscall StatOffenseVictorCountsBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatDefenceVictorCountsBillboard
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1801
// RVA: 0x0003B430
// ADDRESS: 0043b430
// PROTOTYPE: void __thiscall StatDefenceVictorCountsBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatBillboard
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1731
// RVA: 0x0003B810
// ADDRESS: 0043b810
// PROTOTYPE: void __thiscall StatBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::Initialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:106
// RVA: 0x0003B830
// ADDRESS: 0043b830
// PROTOTYPE: bool __thiscall Initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440d6d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x00040D6D
// ADDRESS: 00440d6d
// PROTOTYPE: undefined Catch@00440d6d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440e6d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x00040E6D
// ADDRESS: 00440e6d
// PROTOTYPE: undefined Catch@00440e6d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00441181
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x00041181
// ADDRESS: 00441181
// PROTOTYPE: undefined Catch@00441181()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044123b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x0004123B
// ADDRESS: 0044123b
// PROTOTYPE: undefined Catch@0044123b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052e0b0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x0012E0B0
// ADDRESS: 0052e0b0
// PROTOTYPE: undefined Unwind@0052e0b0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// COMPONENT_VARIANT_END: WorldServer
