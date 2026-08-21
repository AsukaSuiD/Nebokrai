//! Владелец faction-состояния исторического `WorldServer`.
//!
//! `CFaction::SetChangeData` RVA `0x000B4C60`, `CloneSaveData` RVA
//! `0x000C1210`, достигнутое чтение
//! `m_ChangeDataType` из `CRsFaction::SaveFaction` RVA `0x000FF070`,
//! `GetPronounceData` RVA
//! `0x000B4CA0`, `SetGoodsWarCount` RVA `0x000B4D70`,
//! `AddMembersToByteArray` RVA `0x000B53D0`, PDB-inline
//! `GetMembers/GetMemberNum` RVA `0x000BD7C0/0x000BD830` и
//! `CFaction::IsMember` RVA `0x000BD840` и
//! `UpdateMemberInfoToClient` RVA `0x000BA7C0`, а также
//! оба `CheckOperValidate` RVA `0x000B4CE0/0x000C17F0`,
//! `OnMemberExitGame` RVA `0x000B64D0`,
//! `InitialPropertyByLvl` RVA `0x000B4BA0`,
//! `AddEnemyFactionsToByteArray/AddCityWarEnemyFactionsToByteArray` RVA
//! `0x000B5E40/0x000B5ED0`,
//! `IsHaveEnymyFaction/IsHaveCityEnemyFaction` RVA `0x000B50F0/0x000B5100`,
//! `SetSuperiorOrganizing` RVA `0x000B5110`,
//! `IsOwnedCity` RVA `0x000B5490`, `GetOwnedCities` RVA `0x000BD7D0`,
//! `UpdateExpToClient/SetExp` RVA `0x000B55B0/0x000B61F0`,
//! `DelMember` RVA `0x000B9EF0`,
//! `UpdatePropertyToClient` RVA `0x000B9FB0`,
//! `UpdateEnemyFactionToClient/UpdateCityWarEnemyFactionToClient` RVA
//! `0x000BA0F0/0x000BA210`,
//! `AddDefence/Offense/VillageWarVictorCounts` RVA
//! `0x000BA3B0/0x000BA3D0/0x000BA3F0`,
//! `ReInitialPropertyByLvl` RVA `0x000BA630`,
//! `IsUsingPV/SetMemPV/AbolishMemPV` RVA
//! `0x000BA700/0x000BA760/0x000C1DD0`,
//! `IsSuperiorOrganizing` RVA `0x000BD780`, `IsMaster` RVA `0x000C1EE0` и
//! `OnMemberEnterGame` RVA `0x000C0A10` — `IMPLEMENTED`; спорные ключи lookup
//! имеют статус `VERIFIED_DISASSEMBLY`.
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:185,242,243`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:265,344,1388,2519,2561`.
//!
//! PDB задаёт `m_lID` как signed `long` по offset `+0x4`, а ordered
//! `std::map<long, COrganizing::tagMemInfo> m_Members` — по `+0x28` старого
//! `CFaction`. Оба конструктора создают пустой map; public constructor RVA
//! `0x000C0EE0` до этого принимает и сохраняет faction ID. Rust partial-owner
//! хранит только эти достигнутые поля: `BTreeMap<i32, TagMemInfo>` заменяет
//! MSVC tree-node, sentinel, allocator и ручной cleanup, сохраняя signed
//! numeric порядок. Rust-layout не объявляется копией старого ABI, а полного
//! `CFaction` constructor-а до остальных полей не существует.
//! `InitialPropertyByLvl` принимает восстановленный `COrganizingParam` явно,
//! меняет шесть permission-флагов и maximum до проверки level-record, а
//! upgrade experience — только после неё. Поэтому отсутствующий уровень
//! сохраняет уже выполненный prefix и возвращает старый `false`; отсутствующий
//! live property остаётся отдельной safe-границей узкого Rust-owner-а.
//! `SetSuperiorOrganizing` так же принимает параметры явно. Машинный проход
//! `0x004B5110..0x004B5167` подтверждает, что union ID меняется первым, positive
//! ID только отменяет положительный countdown, а negative/zero ID сравнивает
//! member-count с порогом через unsigned `JNC`. При неполном live-state уже
//! выполненная смена union ID не откатывается.
//! `DelMember` отклоняет master до erase, после erase использует новый unsigned
//! member-count и запускает countdown только для faction без union. Exact
//! `0x004B9EF0..0x004B9F46` подтверждает порядок и unsigned `JNC`.
//! `UpdatePropertyToClient` публикует message `0x7FE0B`: recipient ID, затем
//! все `0x38` байт property вместе с padding. Получатели обходятся по signed
//! member key и допускаются только при online-owner, ненулевом GameServer ID и
//! уже выставленном `m_bGetFactionData`; результат каждого send игнорировался.
//! Три victor-counter owner-а машинно подтверждают единый порядок: wrapping
//! 32-битный `ADD`, property-send, затем dirty-bit `1`.
//! Permission owner-ы используют player ID как map key и PDB enum `0..10` как
//! индекс. `SetMemPV` в EXE единственный не проверял индекс и мог писать за
//! `listPV`; safe Rust явно отклоняет недопустимое значение. Это исправление
//! внутреннего memory bug, а не новая Miracle-семантика допустимых прав.
//! Трёхаргументный `CheckOperValidate` проверяет право requester и запрещает
//! ему управлять target с тем же правом, кроме случая requester-master; exact
//! ASM подтверждает, что финальный `IsMaster` получает requester.
//! Enemy-update сообщения передают полный set, а не delta: `0x7FE11/0x7FE12`,
//! recipient ID, 32-битный count и signed IDs в tree-order. Объявленные
//! `enemy_id/operator` исходные функции не читали и в Rust-интерфейс не входят.
//! Experience-update `0x7FE14` получает только contributor либо master и несёт
//! recipient/current/upgrade exp. `SetExp` ставит dirty-bit до этой рассылки.
//! Достигнутый `SetPlayerOrganizing` дополнительно читает `m_strName`,
//! `m_lMastterID`, `m_Property.lLvl/lExp`, `m_OwnedCities` и два enemy-set.
//! Коллекции, которые constructor действительно создавал пустыми, хранятся
//! пустыми; ещё не назначенные narrow state scalar представлены `Option`, а не
//! выдуманным нулём. Member title/contribute берутся из того же `tagMemInfo`.
//!
//! Достигнутые save-поля расширяют partial-owner без притворного старого
//! layout: полный byte-exact `tagFacBaseProperty[0x38]`, established/delete
//! scalars, `m_ChangeDataType`, `m_lFactionWarWinCount` и
//! `m_szFactionWarLastWinTime`, а также ability-state `m_ApplyPersons`,
//! `m_Pronounce`, `m_LastUploadIconTime` и `m_IconData`. `SetChangeData(0)`
//! очищает всю mask, ненулевое значение добавляет отсутствующие bits через OR.
//! `SetGoodsWarCount` сначала
//! снимает один Windows-local wall-clock, пишет строку без leading zeroes,
//! ставит bit `1`, затем сохраняет `0` для входа `<= 0` либо сам положительный
//! signed `long`. Последняя нормализация подтверждена exact диапазоном
//! `0x004B4DC5..0x004B4DDF`; после ответа reverse прекращён.
//!
//! Для достигнутого `SaveAbility` наблюдаемы только signed keys ordered map
//! `m_ApplyPersons`, поэтому `BTreeSet<i32>` заменяет MSVC map вместе с пока не
//! читаемым value. `GetPronounceData` буквально дописывает все `0x828` bytes
//! `m_Pronounce`; partial-owner хранит их byte-exact, не объявляя Rust-layout
//! формой ещё не восстановленного `tagPronounceWord`. `Vec<u8>` и
//! `TagTimeValue` заменяют только STL-vector и Windows `SYSTEMTIME`-совместимое
//! значение значка. Оба конструктора создавали пустые map/vector, а достигнутый
//! `Initial` обнулял весь pronounce-блок; тот же достигнутый baseline задаёт
//! `with_reached_member_state`.
//! `CloneSaveData` возвращает null при нулевой mask; иначе всегда переносит ID
//! и mask, а группы `property/members/leave-word/ability` копирует только для
//! bits `1/2/4/8`. Property переносится четырнадцатью DWORD вместе с тремя
//! padding-байтами после `btCountry`. Enemy-set и Goods War поля функция не
//! копирует. Ещё не материализованные scalar-поля narrow state дают локальный
//! `BLOCKED_MISSING_FACT`, а не выдуманный default.
//!
//! Достигнутый leave-word save-state хранит исходный list-order через
//! `VecDeque<TagLeaveWord>`. Три блока той же точной пары согласуются по полному
//! layout: list-copy в `CFaction` переносит `0x40` DWORD, `LoadLeaveWords`
//! заполняет `lID/lPlayerID`, `tagTime`, `strContent` и `strName`, а exact
//! `SaveLeaveWords` читает time по `+0x1C` и content по `+0x2C`. Поэтому
//! `tagLeaveWord` имеет размер `0x100`: signed ID по `+0x00/+0x04`,
//! `strName[20]` по `+0x08`, 16-байтовый `tagTime` по `+0x1C` и
//! `strContent[212]` по `+0x2C`. `repr(C)` и compile-time assertions фиксируют
//! этот value-layout; partial `CFaction` по-прежнему не выдаётся за полный x86
//! object-layout. Отсутствующий NUL content локализован до старого `CheckPoint`
//! и не воспроизводится чтением за массивом.
//!
//! `GetMembers` возвращает неизменяемое заимствование map, `GetMemberNum`
//! сохраняет младший 32-битный шаблон старого `_Mysize`, а `IsMember` ищет
//! только map-key и при наличии возвращает faction ID, иначе `0`. Exact EXE
//! `0x004BD840..0x004BD865` подтверждает, что ключом служит входной signed
//! player ID; подстановка this в декомпиляте является ошибкой восстановления stack-slot.
//!
//! `AddMembersToByteArray` первым дописывает 32-битный count и затем проходит
//! map в signed-key порядке. Каждое полное `tagMemInfo` публикуется как
//! `lID, lJobLvl, strTitle\0, listPV[0x2C], lLvl, lOccu, strName\0,
//! uint(bControbute), strRegion\0, LastOnlineTime[0x10]`. Отдельная non-delete
//! проекция `UpdateMemberInfoToClient` имеет порядок
//! `strName\0, lLvl, lOccu, lJobLvl, strTitle\0, listPV[0x2C], strRegion\0,
//! uint(bControbute), LastOnlineTime[0x10]`. Полный update в non-delete ветви
//! сначала ищет target и при наличии одним снимком локального времени заменяет
//! его `LastOnlineTime`; delete не ищет target и не снимает время. Затем обе
//! ветви проходят recipients в signed map-порядке, до фильтра выполняют lookup
//! online-player и GameServer ID и требуют ненулевого player, ненулевой
//! GameServer и его `m_bGetFactionData`. Сообщение `0x7FE0D` всегда начинается
//! `recipient ID, operator, target ID`, после чего non-delete добавляет эту
//! per-member проекцию. Исходно игнорировавшиеся результаты `SendToMapID`
//! сохраняются в typed-отчёте и не останавливают следующий send. При
//! отсутствующем NUL уже выполненные предыдущие отправки возвращаются вместе с
//! локальным `BLOCKED_MISSING_FACT`; текущий неполный message не отправляется,
//! а чтение за массивом не воспроизводится.
//! `chrono::Local::now` заменяет Windows `GetLocalTime`: один полученный local
//! wall-clock раскладывается в те же восемь `u16`, включая Sunday-based
//! `wDayOfWeek` и миллисекунды, до первой recipient-операции.
//! Exact EXE `0x004BA8B1..0x004BA8CB` подтверждает испорченный raw stack-slot:
//! в `find` передаётся адрес входного target `param_1` по `[esp+0x98]`, а не
//! iterator-local. Проверка recipient-флага по `CPlayer+0x868` видна напрямую
//! в `0x004BA82D..0x004BA834`; после этих двух ответов reverse прекращён.
//!
//! `OnMemberExitGame` ищет signed player ID и только при найденном member
//! снимает первый local-time snapshot. Пустой регион определяется сравнением
//! единственного первого байта с `""`; он оставляет member без изменений и не
//! публикует update. Для непустого региона owner записывает NUL только в
//! `strRegion[0]`, сохраняя остальные 63 bytes, копирует первый snapshot и
//! вызывает virtual `UpdateMemberInfoToClient(player, OP_Update)`. Последний
//! по исходному контракту снимает второй local-time snapshot и сразу заменяет
//! первый до recipient-прохода; обе позиции сохранены буквально. Exact EXE
//! `0x004B64D7..0x004B64E9` подтверждает ключ `param_1` по `[esp+0x30]`,
//! `0x004B6501..0x004B6544` — порядок time-before-region-check, а
//! `0x004B6546..0x004B6575` — очистку одного byte, копирование времени и
//! virtual slot `+0x1B8` с оператором `2`. После этих ответов reverse
//! прекращён.
//!
//! `OnMemberEnterGame` сохраняет исходный порядок: online-player lookup,
//! member lookup, virtual `CShape::GetRegionID`, numeric `CGame::GetRegion`,
//! копирование имени найденного `CWorldRegion` во временный `char[256]`,
//! `strcmp` с member `strRegion[64]`, условный `strcpy` и virtual update с
//! `OP_Update`. Exact EXE `0x004C0A4B..0x004C0A57` подтверждает, что потерянный
//! raw key — адрес входного `player_id` по `[ebp+8]`. Диапазон
//! `0x004C0A69..0x004C0A7D` обнуляет ровно 256 bytes, а
//! `0x004C0A92..0x004C0AA9` оставляет их пустой C-строкой при отсутствующем
//! region key либо через найденный `pRegion` читает унаследованный
//! `CBaseObject::m_strName` по `+0x20`. Отсутствующий region поэтому означает
//! пустое имя, а не ранний выход. После этих конкретных ответов reverse
//! прекращён.
//!
//! При равенстве C-string member не мутируется и update не вызывается. При
//! различии копируются байты имени и один NUL; оставшийся хвост fixed field не
//! очищается. Затем существующий `UpdateMemberInfoToClient` снимает local-time
//! и выполняет ordered recipient-рассылку. Rust использует slice equality и
//! bounded copy только там, где старые записи доказанно помещаются. Nullable
//! `pRegion`, имя длиннее 255 bytes, отсутствующий NUL в member field и имя
//! длиннее 63 bytes являются четырьмя локальными `BLOCKED_MISSING_FACT`:
//! исходник соответственно разыменовывал null либо читал/писал за границей, а
//! безопасная реализация не назначает этим UB-путям no-op, fail-closed или
//! `unsafe`-результат.
//!
//! PDB связывает slots organizing-base `+0x13C/+0x140` с enter/exit-функциями
//! этого owner-а; оба slot-а теперь замкнуты. Заменённые project-блоки удалены;
//! tree traversal, allocator, local scratch и cleanup остаются ненаблюдаемой
//! STL/compiler/CRT-семантикой, выраженной `BTreeMap`, slices и `Drop`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::mem::{offset_of, size_of};

use chrono::{Datelike, Local, Timelike};

use super::organizing::{
    EOperator, EPurview, EPurviewOwnState, TagMemInfo, TagTimeValue,
    UnterminatedMemberField,
};
use super::organizingparam::COrganizingParam;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::{CGame, WorldRegionNameLookup};

const MEMBER_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0D;
const ENTER_REGION_BUFFER_CAPACITY: usize = 256;
const LEAVE_WORD_NAME_CAPACITY: usize = 20;
const LEAVE_WORD_CONTENT_CAPACITY: usize = 212;
const PRONOUNCE_DATA_SIZE: usize = 0x828;
const FACTION_BASE_PROPERTY_SIZE: usize = 0x38;

const ZERO_TIME: TagTimeValue = TagTimeValue {
    year: 0,
    month: 0,
    day_of_week: 0,
    day: 0,
    hour: 0,
    minute: 0,
    second: 0,
    milliseconds: 0,
};

/// Полный byte-exact блок исходного `CFaction::tagFacBaseProperty`.
///
/// Три байта выравнивания после `btCountry` сохраняются вместе с полями:
/// `CloneSaveData` копировал структуру четырнадцатью 32-битными словами.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct FactionBaseProperty {
    bytes: [u8; FACTION_BASE_PROPERTY_SIZE],
}

impl FactionBaseProperty {
    /// Принимает уже полностью восстановленные `0x38` байт без нормализации.
    pub(crate) const fn from_complete_bytes(bytes: [u8; FACTION_BASE_PROPERTY_SIZE]) -> Self {
        Self { bytes }
    }

    const fn signed_at(&self, offset: usize) -> i32 {
        i32::from_le_bytes([
            self.bytes[offset],
            self.bytes[offset + 1],
            self.bytes[offset + 2],
            self.bytes[offset + 3],
        ])
    }

    pub(crate) const fn level(&self) -> i32 {
        self.signed_at(0x00)
    }

    pub(crate) const fn experience(&self) -> i32 {
        self.signed_at(0x04)
    }

    pub(crate) const fn upgrade_experience(&self) -> i32 {
        self.signed_at(0x20)
    }

    pub(crate) const fn offense_victor_counts(&self) -> i32 {
        self.signed_at(0x08)
    }

    pub(crate) const fn defence_victor_counts(&self) -> i32 {
        self.signed_at(0x0C)
    }

    pub(crate) const fn village_war_victor_counts(&self) -> i32 {
        self.signed_at(0x10)
    }

    pub(crate) const fn member_count(&self) -> i32 {
        self.signed_at(0x14)
    }

    pub(crate) const fn union_id(&self) -> i32 {
        self.signed_at(0x18)
    }

    pub(crate) const fn permit(&self) -> bool {
        self.bytes[0x2B] != 0
    }

    pub(crate) const fn country(&self) -> u8 {
        self.bytes[0x2C]
    }

    pub(crate) const fn property_1(&self) -> i32 {
        self.signed_at(0x30)
    }

    pub(crate) const fn property_2(&self) -> i32 {
        self.signed_at(0x34)
    }

    const fn wire_bytes(&self) -> &[u8; FACTION_BASE_PROPERTY_SIZE] {
        &self.bytes
    }

    fn write_signed(&mut self, offset: usize, value: i32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn set_initial_level_permissions(&mut self, parameters: &COrganizingParam) {
        let level = self.level();
        self.bytes[0x24] = u8::from(parameters.pronounce_minimum_level() <= level);
        self.bytes[0x25] = u8::from(parameters.leave_word_minimum_level() <= level);
        self.bytes[0x26] = u8::from(parameters.endue_right_minimum_level() <= level);
        self.bytes[0x2A] = u8::from(parameters.create_union_minimum_level() <= level);
        self.bytes[0x28] = u8::from(parameters.attack_village_minimum_level() <= level);
        self.bytes[0x29] = u8::from(parameters.attack_city_minimum_level() <= level);
    }
}

/// Локальная safe-граница dirty-bit `1` для ещё узкого live-state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionCloneSaveBlock {
    MasterIdMissing,
    MissingBaseProperty,
    EstablishedTimeUnknown,
    DeleteRemainTimeAbsent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionInitialPropertyBlock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionSuperiorOrganizingBlock {
    MissingBaseProperty,
    DeleteRemainTimeAbsent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDelMemberBlock {
    MasterIdMissing,
    DeleteRemainTimeAbsent,
    MissingBaseProperty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionDelMemberReport {
    pub(crate) removed: bool,
    pub(crate) disband_countdown_started: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MemberPurviewMutation {
    InvalidPurview,
    MemberNotFound,
    Unchanged,
    Changed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionOperatorValidationBlock;

/// Результат одной исходно игнорировавшейся отправки полного property.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionPropertyDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

/// Результат одной исходно игнорировавшейся отправки enemy-set.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionEnemyDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionExperienceDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionExperienceBlock {
    MissingBaseProperty,
    MasterIdMissing,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionExperienceUpdate {
    MaximumLevel,
    Unchanged { experience: i32 },
    Updated {
        experience: i32,
        deliveries: Vec<FactionExperienceDelivery>,
    },
}

#[derive(Clone, Copy)]
enum EnemyFactionSetKind {
    Standard,
    CityWar,
}

/// Полный результат `CFaction::ReInitialPropertyByLvl` после safe-границ.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionPropertyReinitialization {
    pub(crate) level_parameters_found: bool,
    pub(crate) deliveries: Vec<FactionPropertyDelivery>,
}

/// Результат одной исходно игнорировавшейся отправки member-update.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MemberUpdateDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

/// Отчёт полного ordered recipient-прохода.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MemberUpdateReport {
    /// `None` означает delete-ветвь без target lookup.
    pub(crate) target_found: Option<bool>,
    pub(crate) deliveries: Vec<MemberUpdateDelivery>,
}

/// Безопасная граница старого чтения за фиксированным member-массивом.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MemberUpdateBuildError {
    pub(crate) field: UnterminatedMemberField,
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    /// Уже выполненные send-ы не откатываются, как и в исходном ordered цикле.
    pub(crate) completed_deliveries: Vec<MemberUpdateDelivery>,
}

impl fmt::Display for MemberUpdateBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "member-update для игрока {} не построен: {}",
            self.recipient_player_id, self.field
        )
    }
}

impl Error for MemberUpdateBuildError {}

/// Локальная safe-граница старых nullable/неограниченных C-string операций.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MemberEnterBlockedReason {
    NullRegionPointer { region_id: i32 },
    RegionNameExceedsLocalBuffer { region_id: i32, byte_len: usize },
    UnterminatedMemberRegion(UnterminatedMemberField),
    RegionNameExceedsMemberField { region_id: i32, byte_len: usize },
}

impl fmt::Display for MemberEnterBlockedReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullRegionPointer { region_id } => write!(
                formatter,
                "у tagRegion {} отсутствует исходный pRegion",
                region_id
            ),
            Self::RegionNameExceedsLocalBuffer {
                region_id,
                byte_len,
            } => write!(
                formatter,
                "имя региона {} длиной {} байт не помещается в старый char[256]",
                region_id, byte_len
            ),
            Self::UnterminatedMemberRegion(field) => field.fmt(formatter),
            Self::RegionNameExceedsMemberField {
                region_id,
                byte_len,
            } => write!(
                formatter,
                "имя региона {} длиной {} байт не помещается в старый strRegion[64]",
                region_id, byte_len
            ),
        }
    }
}

impl Error for MemberEnterBlockedReason {}

/// Итог virtual callback-а входа faction-member в игру.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MemberEnterOutcome {
    PlayerNotOnline,
    MemberNotFound,
    RegionUnchanged,
    Blocked(MemberEnterBlockedReason),
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

/// Итог virtual callback-а выхода faction-member из игры.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MemberExitOutcome {
    MemberNotFound,
    RegionAlreadyEmpty,
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

/// Ошибка безопасного C-string view одного fixed-поля `tagLeaveWord`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnterminatedLeaveWordField {
    pub(crate) field: &'static str,
}

impl fmt::Display for UnterminatedLeaveWordField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "в фиксированном поле {} отсутствует завершающий NUL",
            self.field
        )
    }
}

impl Error for UnterminatedLeaveWordField {}

/// Полное доказанное значение исходного `CFaction::tagLeaveWord`.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct TagLeaveWord {
    pub(crate) id: i32,
    pub(crate) player_id: i32,
    pub(crate) name: [u8; LEAVE_WORD_NAME_CAPACITY],
    pub(crate) time: TagTimeValue,
    pub(crate) content: [u8; LEAVE_WORD_CONTENT_CAPACITY],
}

impl TagLeaveWord {
    /// Создаёт полный 0x100-байтовый leave-word snapshot.
    pub(crate) const fn from_complete_fields(
        id: i32,
        player_id: i32,
        name: [u8; LEAVE_WORD_NAME_CAPACITY],
        time: TagTimeValue,
        content: [u8; LEAVE_WORD_CONTENT_CAPACITY],
    ) -> Self {
        Self {
            id,
            player_id,
            name,
            time,
            content,
        }
    }

    /// Возвращает `strContent` до первого NUL включительно.
    pub(crate) fn content_wire_bytes(&self) -> Result<&[u8], UnterminatedLeaveWordField> {
        let Some(terminator) = self.content.iter().position(|byte| *byte == 0) else {
            // BLOCKED_MISSING_FACT: WorldServer SaveLeaveWords RVA 0x000FACA0
            // передавал `node + 0x34` в CheckPoint без размера; тот читал бы за
            // char[212], а достижимость и результат этого пути не доказаны.
            return Err(UnterminatedLeaveWordField {
                field: "tagLeaveWord::strContent",
            });
        };
        Ok(&self.content[..=terminator])
    }
}

const _: () = {
    assert!(size_of::<FactionBaseProperty>() == FACTION_BASE_PROPERTY_SIZE);
    assert!(size_of::<TagLeaveWord>() == 0x100);
    assert!(offset_of!(TagLeaveWord, id) == 0x00);
    assert!(offset_of!(TagLeaveWord, player_id) == 0x04);
    assert!(offset_of!(TagLeaveWord, name) == 0x08);
    assert!(offset_of!(TagLeaveWord, time) == 0x1C);
    assert!(offset_of!(TagLeaveWord, content) == 0x2C);
};

/// Достигнутая member-state часть исходного `CFaction`.
pub(crate) struct CFaction {
    faction_id: i32,
    name: Vec<u8>,
    master_id: Option<i32>,
    members: BTreeMap<i32, TagMemInfo>,
    base_property: Option<FactionBaseProperty>,
    established_time: Option<TagTimeValue>,
    delete_remain_time: Option<i32>,
    owned_cities: VecDeque<i32>,
    enemy_factions: BTreeSet<i32>,
    city_war_enemy_factions: BTreeSet<i32>,
    apply_person_ids: BTreeSet<i32>,
    pronounce_data: [u8; PRONOUNCE_DATA_SIZE],
    leave_words: VecDeque<TagLeaveWord>,
    last_upload_icon_time: TagTimeValue,
    icon_data: Vec<u8>,
    change_data_type: i32,
    goods_war_count: i32,
    goods_war_last_win_time: String,
}

impl CFaction {
    /// Создаёт доказанный пустой `m_Members` с уже назначенным faction ID.
    pub(crate) const fn with_reached_member_state(faction_id: i32) -> Self {
        Self {
            faction_id,
            name: Vec::new(),
            master_id: None,
            members: BTreeMap::new(),
            base_property: None,
            established_time: None,
            delete_remain_time: None,
            owned_cities: VecDeque::new(),
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            apply_person_ids: BTreeSet::new(),
            pronounce_data: [0; PRONOUNCE_DATA_SIZE],
            leave_words: VecDeque::new(),
            last_upload_icon_time: ZERO_TIME,
            icon_data: Vec::new(),
            change_data_type: 0,
            goods_war_count: 0,
            goods_war_last_win_time: String::new(),
        }
    }

    /// Возвращает исходный signed `m_lID`.
    pub(crate) const fn faction_id(&self) -> i32 {
        self.faction_id
    }

    /// Возвращает byte-exact содержимое исходного `m_strName`.
    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    /// Возвращает достигнутый `m_lMastterID`; narrow state его не назначает.
    pub(crate) const fn master_id(&self) -> Option<i32> {
        self.master_id
    }

    /// Возвращает reached `m_Property.lLvl` без выдуманного default.
    pub(crate) const fn level(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.level()),
            None => None,
        }
    }

    /// Возвращает reached `m_Property.lExp` без выдуманного default.
    pub(crate) const fn experience(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.experience()),
            None => None,
        }
    }

    /// Возвращает полный reached `m_Property` вместе с исходным padding.
    pub(crate) const fn base_property(&self) -> Option<FactionBaseProperty> {
        self.base_property
    }

    /// Пересчитывает level-зависимый property prefix без клиентской публикации.
    pub(crate) fn initial_property_by_level(
        &mut self,
        parameters: &COrganizingParam,
    ) -> Result<bool, FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        let level = property.level();
        property.set_initial_level_permissions(parameters);

        let maximum_members = parameters.get_max_number_by_level(level);
        if property.signed_at(0x1C) != maximum_members {
            property.write_signed(0x1C, maximum_members);
        }

        let Some(level_parameters) = parameters.get_level_param(level) else {
            return Ok(false);
        };
        property.write_signed(0x20, level_parameters.experience);
        Ok(true)
    }

    /// Публикует полный `tagFacBaseProperty` всем готовым faction-members.
    pub(crate) fn update_property_to_client(
        &self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        let property = self.base_property.ok_or(FactionInitialPropertyBlock)?;
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(0x7FE0B);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add(property.wire_bytes());
            deliveries.push(FactionPropertyDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Пересчитывает property и безусловно публикует его, как старый wrapper.
    pub(crate) fn reinitialize_property_by_level(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> Result<FactionPropertyReinitialization, FactionInitialPropertyBlock> {
        let level_parameters_found = self.initial_property_by_level(parameters)?;
        let deliveries = self.update_property_to_client(game)?;
        Ok(FactionPropertyReinitialization {
            level_parameters_found,
            deliveries,
        })
    }

    fn add_victor_count(
        &mut self,
        game: &CGame,
        property_offset: usize,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        let next = property.signed_at(property_offset).wrapping_add(1);
        property.write_signed(property_offset, next);
        let deliveries = self.update_property_to_client(game)?;
        self.set_change_data(1);
        Ok(deliveries)
    }

    pub(crate) fn add_defence_victor_count(
        &mut self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x0C)
    }

    pub(crate) fn add_offense_victor_count(
        &mut self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x08)
    }

    pub(crate) fn add_village_war_victor_count(
        &mut self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x10)
    }

    /// Возвращает reached `m_EstablishedTime` без выдуманного default.
    pub(crate) const fn established_time(&self) -> Option<TagTimeValue> {
        self.established_time
    }

    /// Возвращает reached `m_lDelRemainTime` без выдуманного default.
    pub(crate) const fn delete_remain_time(&self) -> Option<i32> {
        self.delete_remain_time
    }

    /// Заменяет reached signed `m_lDelRemainTime`, если значение изменилось.
    pub(crate) fn set_delete_remain_time(&mut self, value: i32) {
        if self.delete_remain_time != Some(value) {
            self.delete_remain_time = Some(value);
        }
    }

    /// Возвращает исходный list-order `m_OwnedCities`.
    pub(crate) const fn owned_cities(&self) -> &VecDeque<i32> {
        &self.owned_cities
    }

    /// Возвращает faction ID, только если город есть в исходном list-order.
    pub(crate) fn is_owned_city(&self, region_id: i32) -> i32 {
        if self.owned_cities.contains(&region_id) {
            self.faction_id
        } else {
            0
        }
    }

    /// Возвращает faction ID только для текущего `m_lMastterID`.
    pub(crate) const fn is_master(&self, player_id: i32) -> i32 {
        match self.master_id {
            Some(master_id) if master_id == player_id => self.faction_id,
            Some(_) | None => 0,
        }
    }

    /// Возвращает копируемый `m_EnemyFactions` в signed key-order.
    pub(crate) const fn enemy_factions(&self) -> &BTreeSet<i32> {
        &self.enemy_factions
    }

    /// Сохраняет exact `set::_Mysize != 0` без зависимости от MSVC layout.
    pub(crate) fn has_enemy_faction(&self) -> bool {
        !self.enemy_factions.is_empty()
    }

    /// Возвращает копируемый `m_CityWarEnemyFactions` в signed key-order.
    pub(crate) const fn city_war_enemy_factions(&self) -> &BTreeSet<i32> {
        &self.city_war_enemy_factions
    }

    /// Сохраняет exact `set::_Mysize != 0` для city-war enemy-set.
    pub(crate) fn has_city_war_enemy_faction(&self) -> bool {
        !self.city_war_enemy_factions.is_empty()
    }

    /// Дописывает полный standard enemy-set в исходном wire-формате.
    pub(crate) fn add_enemy_factions_to_byte_array(&self, output: &mut Vec<u8>) -> bool {
        append_signed_set(output, &self.enemy_factions);
        true
    }

    /// Дописывает полный city-war enemy-set в исходном wire-формате.
    pub(crate) fn add_city_war_enemy_factions_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> bool {
        append_signed_set(output, &self.city_war_enemy_factions);
        true
    }

    fn update_enemy_set_to_client(
        &self,
        game: &CGame,
        kind: EnemyFactionSetKind,
    ) -> Vec<FactionEnemyDelivery> {
        let (message_type, enemy_factions) = match kind {
            EnemyFactionSetKind::Standard => (0x7FE11, &self.enemy_factions),
            EnemyFactionSetKind::CityWar => (0x7FE12, &self.city_war_enemy_factions),
        };
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(message_type);
            message.base_mut().add_long(recipient_player_id);
            let mut serialized_set = Vec::new();
            append_signed_set(&mut serialized_set, enemy_factions);
            message.base_mut().add(&serialized_set);
            deliveries.push(FactionEnemyDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        deliveries
    }

    pub(crate) fn update_enemy_factions_to_client(
        &self,
        game: &CGame,
    ) -> Vec<FactionEnemyDelivery> {
        self.update_enemy_set_to_client(game, EnemyFactionSetKind::Standard)
    }

    pub(crate) fn update_city_war_enemy_factions_to_client(
        &self,
        game: &CGame,
    ) -> Vec<FactionEnemyDelivery> {
        self.update_enemy_set_to_client(game, EnemyFactionSetKind::CityWar)
    }

    /// Рассылает current/upgrade exp только contributor-ам и master-у.
    pub(crate) fn update_experience_to_client(
        &self,
        game: &CGame,
    ) -> Result<Vec<FactionExperienceDelivery>, FactionExperienceBlock> {
        let property = self
            .base_property
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        let master_id = self
            .master_id
            .ok_or(FactionExperienceBlock::MasterIdMissing)?;
        let mut deliveries = Vec::new();
        for (&recipient_player_id, member) in &self.members {
            if !member.contribute && recipient_player_id != master_id {
                continue;
            }
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(0x7FE14);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(property.experience());
            message.base_mut().add_long(property.upgrade_experience());
            deliveries.push(FactionExperienceDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Clamp-ит и публикует faction experience с исходным порядком эффектов.
    pub(crate) fn set_experience(
        &mut self,
        game: &CGame,
        experience: i32,
    ) -> Result<FactionExperienceUpdate, FactionExperienceBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        if property.level() >= 12 {
            return Ok(FactionExperienceUpdate::MaximumLevel);
        }
        let experience = experience.clamp(0, 100_000_000);
        if property.experience() == experience {
            return Ok(FactionExperienceUpdate::Unchanged { experience });
        }
        property.write_signed(0x04, experience);
        self.set_change_data(1);
        let deliveries = self.update_experience_to_client(game)?;
        Ok(FactionExperienceUpdate::Updated {
            experience,
            deliveries,
        })
    }

    /// Возвращает достигнутый `m_Property.lConfederationID`.
    pub(crate) const fn superior_organizing(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.union_id()),
            None => None,
        }
    }

    /// Назначает union ID и поддерживает исходный countdown роспуска.
    pub(crate) fn set_superior_organizing(
        &mut self,
        organizing_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<(), FactionSuperiorOrganizingBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionSuperiorOrganizingBlock::MissingBaseProperty)?;
        if property.union_id() != organizing_id {
            property.write_signed(0x18, organizing_id);
        }

        if organizing_id < 1 {
            let member_count = self.members.len() as u32;
            let minimum_members = parameters.disband_faction_minimum_members() as u32;
            if member_count < minimum_members {
                let delete_remain_time = self
                    .delete_remain_time
                    .ok_or(FactionSuperiorOrganizingBlock::DeleteRemainTimeAbsent)?;
                if delete_remain_time < 0 {
                    self.delete_remain_time = Some(parameters.disband_faction_minutes());
                }
            }
        } else {
            let delete_remain_time = self
                .delete_remain_time
                .ok_or(FactionSuperiorOrganizingBlock::DeleteRemainTimeAbsent)?;
            if delete_remain_time > 0 {
                self.delete_remain_time = Some(-1);
            }
        }
        Ok(())
    }

    /// Удаляет не-master участника и при необходимости запускает роспуск.
    pub(crate) fn del_member(
        &mut self,
        player_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<Option<FactionDelMemberReport>, FactionDelMemberBlock> {
        let master_id = self.master_id.ok_or(FactionDelMemberBlock::MasterIdMissing)?;
        if player_id == master_id {
            return Ok(None);
        }

        let removed = self.members.remove(&player_id).is_some();
        let member_count = self.members.len() as u32;
        let minimum_members = parameters.disband_faction_minimum_members() as u32;
        let mut disband_countdown_started = false;
        if member_count < minimum_members {
            let delete_remain_time = self
                .delete_remain_time
                .ok_or(FactionDelMemberBlock::DeleteRemainTimeAbsent)?;
            if delete_remain_time < 0 {
                let union_id = self
                    .base_property
                    .ok_or(FactionDelMemberBlock::MissingBaseProperty)?
                    .union_id();
                if union_id < 1 {
                    self.delete_remain_time = Some(parameters.disband_faction_minutes());
                    disband_countdown_started = true;
                }
            }
        }

        Ok(Some(FactionDelMemberReport {
            removed,
            disband_countdown_started,
        }))
    }

    /// Возвращает member-title без завершающего NUL либо старую overread-границу.
    pub(crate) fn member_title(&self, player_id: i32) -> Result<Vec<u8>, UnterminatedMemberField> {
        let Some(member) = self.members.get(&player_id) else {
            return Ok(Vec::new());
        };
        let wire = member.title_wire_bytes()?;
        Ok(wire[..wire.len() - 1].to_vec())
    }

    /// Сохраняет точный missing-member результат `CFaction::IsControbute`.
    pub(crate) fn is_contribute(&self, player_id: i32) -> bool {
        self.members
            .get(&player_id)
            .is_some_and(|member| member.contribute)
    }

    /// Проверяет точное состояние `PST_Permit` одного member-права.
    pub(crate) fn is_using_purview(&self, player_id: i32, purview: i32) -> bool {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return false;
        };
        self.members
            .get(&player_id)
            .is_some_and(|member| member.purview[purview.index()] == EPurviewOwnState::Permit)
    }

    /// Переводит только `PST_No` в `PST_Permit`.
    pub(crate) fn set_member_purview(
        &mut self,
        player_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&player_id) else {
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
        player_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state == EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::No;
        MemberPurviewMutation::Changed
    }

    /// Проверяет членство и одно право requester-а.
    pub(crate) fn check_operator_validate(&self, requester_id: i32, purview: i32) -> bool {
        self.is_member(requester_id) != 0 && self.is_using_purview(requester_id, purview)
    }

    /// Проверяет операцию requester над другим faction-member.
    pub(crate) fn check_operator_validate_target(
        &self,
        requester_id: i32,
        target_id: i32,
        purview: i32,
    ) -> Result<bool, FactionOperatorValidationBlock> {
        let master_id = self.master_id.ok_or(FactionOperatorValidationBlock)?;
        if requester_id == target_id
            || target_id == master_id
            || self.is_member(requester_id) == 0
            || self.is_member(target_id) == 0
            || !self.is_using_purview(requester_id, purview)
        {
            return Ok(false);
        }
        if self.is_using_purview(target_id, purview) && requester_id != master_id {
            return Ok(false);
        }
        Ok(true)
    }

    /// Возвращает один снимок исходной signed dirty-bit mask.
    pub(crate) const fn change_data_type(&self) -> i32 {
        self.change_data_type
    }

    /// Применяет точную bit-mask семантику virtual `SetChangeData`.
    pub(crate) fn set_change_data(&mut self, change_data_type: i32) {
        if change_data_type == 0 {
            self.change_data_type = 0;
        } else if self.change_data_type & change_data_type == 0 {
            self.change_data_type |= change_data_type;
        }
    }

    /// Создаёт отдельную save-копию ровно по dirty-битам `1/2/4/8`.
    pub(crate) fn clone_save_data(&self) -> Result<Option<Self>, FactionCloneSaveBlock> {
        let change_data_type = self.change_data_type;
        if change_data_type == 0 {
            return Ok(None);
        }

        let copy_property = change_data_type & 1 != 0;
        let master_id = if copy_property {
            Some(
                self.master_id
                    .ok_or(FactionCloneSaveBlock::MasterIdMissing)?,
            )
        } else {
            None
        };
        let base_property = if copy_property {
            Some(
                self.base_property
                    .ok_or(FactionCloneSaveBlock::MissingBaseProperty)?,
            )
        } else {
            None
        };
        let established_time = if copy_property {
            Some(
                self.established_time
                    .ok_or(FactionCloneSaveBlock::EstablishedTimeUnknown)?,
            )
        } else {
            None
        };
        let delete_remain_time = if copy_property {
            Some(
                self.delete_remain_time
                    .ok_or(FactionCloneSaveBlock::DeleteRemainTimeAbsent)?,
            )
        } else {
            None
        };

        Ok(Some(Self {
            faction_id: self.faction_id,
            name: if copy_property {
                self.name.clone()
            } else {
                Vec::new()
            },
            master_id,
            members: if change_data_type & 2 != 0 {
                self.members.clone()
            } else {
                BTreeMap::new()
            },
            base_property,
            established_time,
            delete_remain_time,
            owned_cities: if change_data_type & 8 != 0 {
                self.owned_cities.clone()
            } else {
                VecDeque::new()
            },
            // CloneSaveData не копирует оба runtime enemy-set.
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            // DB ability-owner читает только ordered keys; значения
            // tagApplyPerson в достигнутой цепочке не наблюдаются.
            apply_person_ids: if change_data_type & 8 != 0 {
                self.apply_person_ids.clone()
            } else {
                BTreeSet::new()
            },
            pronounce_data: if change_data_type & 8 != 0 {
                self.pronounce_data
            } else {
                [0; PRONOUNCE_DATA_SIZE]
            },
            leave_words: if change_data_type & 4 != 0 {
                self.leave_words.clone()
            } else {
                VecDeque::new()
            },
            last_upload_icon_time: if change_data_type & 8 != 0 {
                self.last_upload_icon_time
            } else {
                ZERO_TIME
            },
            icon_data: if change_data_type & 8 != 0 {
                self.icon_data.clone()
            } else {
                Vec::new()
            },
            change_data_type,
            // Private clone-constructor не назначает эти поля; текущие
            // безопасные значения остаются только до будущей canonical
            // нормализации SaveFactionProperty.
            goods_war_count: 0,
            goods_war_last_win_time: String::new(),
        }))
    }

    /// Обновляет Goods War count/time и возвращает сохранённый signed count.
    pub(crate) fn set_goods_war_count(&mut self, goods_war_count: i32) -> i32 {
        let now = Local::now();
        self.goods_war_last_win_time = format!(
            "{}-{}-{} {}:{}:{}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        self.set_change_data(1);
        self.goods_war_count = if goods_war_count <= 0 {
            0
        } else {
            goods_war_count
        };
        self.goods_war_count
    }

    /// Возвращает текущий signed Goods War count save-копии.
    pub(crate) const fn goods_war_count(&self) -> i32 {
        self.goods_war_count
    }

    /// Возвращает текущую ASCII time-строку save-копии.
    pub(crate) fn goods_war_last_win_time(&self) -> &str {
        &self.goods_war_last_win_time
    }

    /// Заменяет только time-строку save-копии до ADO/TDS update.
    pub(crate) fn set_goods_war_last_win_time(&mut self, value: String) {
        self.goods_war_last_win_time = value;
    }

    /// Возвращает исходный const-view ordered `m_Members`.
    pub(crate) const fn get_members(&self) -> &BTreeMap<i32, TagMemInfo> {
        &self.members
    }

    /// Возвращает ordered signed keys исходного `m_ApplyPersons`.
    pub(crate) const fn get_apply_person_ids(&self) -> &BTreeSet<i32> {
        &self.apply_person_ids
    }

    /// Дописывает byte-exact `m_Pronounce` в исходный output-vector.
    pub(crate) fn get_pronounce_data(&self, output: &mut Vec<u8>) -> bool {
        output.extend_from_slice(&self.pronounce_data);
        true
    }

    /// Возвращает исходный const-view ordered `m_LeaveWords`.
    pub(crate) const fn get_leave_words(&self) -> &VecDeque<TagLeaveWord> {
        &self.leave_words
    }

    /// Возвращает дату последней загрузки faction-icon.
    pub(crate) const fn last_upload_icon_time(&self) -> TagTimeValue {
        self.last_upload_icon_time
    }

    /// Возвращает byte-exact исходный `m_IconData`.
    pub(crate) fn icon_data(&self) -> &[u8] {
        &self.icon_data
    }

    /// Возвращает младший 32-битный шаблон старого `map::_Mysize`.
    pub(crate) fn get_member_num(&self) -> i32 {
        self.members.len() as u32 as i32
    }

    /// Возвращает faction ID только для существующего member key.
    pub(crate) fn is_member(&self, player_id: i32) -> i32 {
        if self.members.contains_key(&player_id) {
            self.faction_id
        } else {
            0
        }
    }

    /// Дописывает полный ordered member snapshot в исходном byte-array формате.
    pub(crate) fn add_members_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> Result<bool, UnterminatedMemberField> {
        output.extend_from_slice(&(self.members.len() as u32).to_le_bytes());
        for member in self.members.values() {
            append_i32(output, member.id);
            append_i32(output, member.job_level);
            output.extend_from_slice(member.title_wire_bytes()?);
            output.extend_from_slice(&member.purview_wire_bytes());
            append_i32(output, member.level);
            append_i32(output, member.occupation);
            output.extend_from_slice(member.name_wire_bytes()?);
            output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
            output.extend_from_slice(member.region_wire_bytes()?);
            output.extend_from_slice(&member.last_online_wire_bytes());
        }
        Ok(true)
    }

    /// Публикует delete либо полный non-delete member-update всем готовым
    /// получателям.
    pub(crate) fn update_member_info_to_client(
        &mut self,
        game: &CGame,
        target_player_id: i32,
        operator: EOperator,
    ) -> Result<MemberUpdateReport, MemberUpdateBuildError> {
        let target_found = if operator == EOperator::Delete {
            None
        } else {
            let Some(target) = self.members.get_mut(&target_player_id) else {
                return Ok(MemberUpdateReport {
                    target_found: Some(false),
                    deliveries: Vec::new(),
                });
            };
            target.last_online_time = current_local_member_time();
            Some(true)
        };

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(MEMBER_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(target_player_id);

            if operator != EOperator::Delete {
                let target = self
                    .members
                    .get(&target_player_id)
                    .expect("non-delete target найден до recipient-прохода");
                let mut fields = Vec::new();
                if let Err(field) = append_member_update_fields(&mut fields, target) {
                    return Err(MemberUpdateBuildError {
                        field,
                        recipient_player_id,
                        game_server_id,
                        completed_deliveries: deliveries,
                    });
                }
                message.base_mut().add(&fields);
            }

            deliveries.push(MemberUpdateDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }

        Ok(MemberUpdateReport {
            target_found,
            deliveries,
        })
    }

    /// Обновляет byte-exact online-регион участника и публикует изменение.
    pub(crate) fn on_member_enter_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> MemberEnterOutcome {
        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return MemberEnterOutcome::PlayerNotOnline;
        };
        if !self.members.contains_key(&player_id) {
            return MemberEnterOutcome::MemberNotFound;
        }

        let region_id = player.get_region_id();
        let region_name = match game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound => &[][..],
            WorldRegionNameLookup::NullRegionPointer => {
                // BLOCKED_MISSING_FACT: exact EXE `0x004C0A96..0x004C0A98`
                // разыменовывает найденный tagRegion::pRegion без null-check.
                // Достижимость и наблюдаемая реакция null не доказаны.
                return MemberEnterOutcome::Blocked(MemberEnterBlockedReason::NullRegionPointer {
                    region_id,
                });
            }
            WorldRegionNameLookup::Name(name) => name,
        };

        if region_name.len() >= ENTER_REGION_BUFFER_CAPACITY {
            // BLOCKED_MISSING_FACT: старый strcpy по `0x004C0AB0..0x004C0ABC`
            // переполнял бы локальный char[256]. Safe Rust не назначает этому
            // пути новый результат и не воспроизводит запись через unsafe.
            return MemberEnterOutcome::Blocked(
                MemberEnterBlockedReason::RegionNameExceedsLocalBuffer {
                    region_id,
                    byte_len: region_name.len(),
                },
            );
        }

        let member_region_matches = {
            let member = self
                .members
                .get(&player_id)
                .expect("member проверен до region lookup");
            let member_region = match member.region_wire_bytes() {
                Ok(bytes) => &bytes[..bytes.len() - 1],
                Err(field) => {
                    // BLOCKED_MISSING_FACT: strcmp читал бы за strRegion[64].
                    return MemberEnterOutcome::Blocked(
                        MemberEnterBlockedReason::UnterminatedMemberRegion(field),
                    );
                }
            };
            member_region == region_name
        };
        if member_region_matches {
            return MemberEnterOutcome::RegionUnchanged;
        }

        let member_region_capacity = self
            .members
            .get(&player_id)
            .expect("member проверен до region lookup")
            .region
            .len();
        if region_name.len() >= member_region_capacity {
            // BLOCKED_MISSING_FACT: второй strcpy по `0x004C0B07..0x004C0B11`
            // переполнял бы strRegion[64] уже после доказанного неравенства.
            return MemberEnterOutcome::Blocked(
                MemberEnterBlockedReason::RegionNameExceedsMemberField {
                    region_id,
                    byte_len: region_name.len(),
                },
            );
        }

        let member = self
            .members
            .get_mut(&player_id)
            .expect("member проверен до region lookup");
        member.region[..region_name.len()].copy_from_slice(region_name);
        member.region[region_name.len()] = 0;

        MemberEnterOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }

    /// Очищает online-регион участника и публикует исходный update при изменении.
    pub(crate) fn on_member_exit_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> MemberExitOutcome {
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberExitOutcome::MemberNotFound;
        };

        let exit_time = current_local_member_time();
        if member.region[0] == 0 {
            return MemberExitOutcome::RegionAlreadyEmpty;
        }

        member.region[0] = 0;
        member.last_online_time = exit_time;
        MemberExitOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }
}

fn current_local_member_time() -> TagTimeValue {
    let now = Local::now();
    TagTimeValue {
        year: now.year() as u16,
        month: now.month() as u16,
        day_of_week: now.weekday().num_days_from_sunday() as u16,
        day: now.day() as u16,
        hour: now.hour() as u16,
        minute: now.minute() as u16,
        second: now.second() as u16,
        milliseconds: now.timestamp_subsec_millis() as u16,
    }
}

/// Дописывает только доказанную per-member часть non-delete update-сообщения.
fn append_member_update_fields(
    output: &mut Vec<u8>,
    member: &TagMemInfo,
) -> Result<(), UnterminatedMemberField> {
    output.extend_from_slice(member.name_wire_bytes()?);
    append_i32(output, member.level);
    append_i32(output, member.occupation);
    append_i32(output, member.job_level);
    output.extend_from_slice(member.title_wire_bytes()?);
    output.extend_from_slice(&member.purview_wire_bytes());
    output.extend_from_slice(member.region_wire_bytes()?);
    output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
    output.extend_from_slice(&member.last_online_wire_bytes());
    Ok(())
}

fn append_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn append_signed_set(output: &mut Vec<u8>, values: &BTreeSet<i32>) {
    output.extend_from_slice(&(values.len() as u32).to_le_bytes());
    for &value in values {
        append_i32(output, value);
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h

// ============================================================================
// FUNCTION: CFaction::InitialPropertyByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:155
// RVA: 0x000B4BA0
// ADDRESS: 004b4ba0
// PROTOTYPE: bool __thiscall InitialPropertyByLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CheckOperValidate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:731
// RVA: 0x000B4CE0
// ADDRESS: 004b4ce0
// PROTOTYPE: bool __thiscall CheckOperValidate(long param_1, long param_2, ePurview param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CFaction::SetDelRemainTime` RVA `0x000B4DF0` находится выше.

// ============================================================================
// FUNCTION: CFaction::GetIsPermit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2431
// RVA: 0x000B4E20
// ADDRESS: 004b4e20
// PROTOTYPE: bool __thiscall GetIsPermit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateEnemyFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2639
// RVA: 0x000B4E30
// ADDRESS: 004b4e30
// PROTOTYPE: void __thiscall UpdateEnemyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateCityWarEnemyFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2649
// RVA: 0x000B4E60
// ADDRESS: 004b4e60
// PROTOTYPE: void __thiscall UpdateCityWarEnemyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IncMaxNumber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2425
// RVA: 0x000B4E90
// ADDRESS: 004b4e90
// PROTOTYPE: bool __thiscall IncMaxNumber(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GoodsWarCheckforFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:50
// RVA: 0x000B5070
// ADDRESS: 004b5070
// PROTOTYPE: bool __cdecl GoodsWarCheckforFaction(CFaction * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsHaveEnymyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:637
// RVA: 0x000B50F0
// ADDRESS: 004b50f0
// PROTOTYPE: bool __thiscall IsHaveEnymyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsHaveCityEnemyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:644
// RVA: 0x000B5100
// ADDRESS: 004b5100
// PROTOTYPE: bool __thiscall IsHaveCityEnemyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetSuperiorOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1260
// RVA: 0x000B5110
// ADDRESS: 004b5110
// PROTOTYPE: void __thiscall SetSuperiorOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:456
// RVA: 0x000B5490
// ADDRESS: 004b5490
// PROTOTYPE: long __thiscall IsOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearOwnedCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:504
// RVA: 0x000B54C0
// ADDRESS: 004b54c0
// PROTOTYPE: bool __thiscall ClearOwnedCity(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetMemberList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:534
// RVA: 0x000B5530
// ADDRESS: 004b5530
// PROTOTYPE: void __thiscall GetMemberList(list<COrganizing*,std::allocator<COrganizing*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::RefreshOwnCityInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1288
// RVA: 0x000B5570
// ADDRESS: 004b5570
// PROTOTYPE: void __thiscall RefreshOwnCityInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateExpToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1366
// RVA: 0x000B55B0
// ADDRESS: 004b55b0
// PROTOTYPE: void __thiscall UpdateExpToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdatePronounceToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1497
// RVA: 0x000B56C0
// ADDRESS: 004b56c0
// PROTOTYPE: void __thiscall UpdatePronounceToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetControbuterNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2507
// RVA: 0x000B57E0
// ADDRESS: 004b57e0
// PROTOTYPE: long __thiscall GetControbuterNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdatePlayerFactionInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2613
// RVA: 0x000B5820
// ADDRESS: 004b5820
// PROTOTYPE: void __thiscall UpdatePlayerFactionInfo(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SendInfoToAllMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2730
// RVA: 0x000B5890
// ADDRESS: 004b5890
// PROTOTYPE: void __thiscall SendInfoToAllMember(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, long param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateOtherFacInfoToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2740
// RVA: 0x000B58F0
// ADDRESS: 004b58f0
// PROTOTYPE: void __thiscall UpdateOtherFacInfoToClient(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, eOperator param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Talk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2776
// RVA: 0x000B5A50
// ADDRESS: 004b5a50
// PROTOTYPE: void __thiscall Talk(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddApplyPersonsToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:364
// RVA: 0x000B5D30
// ADDRESS: 004b5d30
// PROTOTYPE: bool __thiscall AddApplyPersonsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddLeaveWordsToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:389
// RVA: 0x000B5DD0
// ADDRESS: 004b5dd0
// PROTOTYPE: bool __thiscall AddLeaveWordsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddEnemyFactionsToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:423
// RVA: 0x000B5E40
// ADDRESS: 004b5e40
// PROTOTYPE: bool __thiscall AddEnemyFactionsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddCityWarEnemyFactionsToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:435
// RVA: 0x000B5ED0
// ADDRESS: 004b5ed0
// PROTOTYPE: bool __thiscall AddCityWarEnemyFactionsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelOwnedCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:490
// RVA: 0x000B5F60
// ADDRESS: 004b5f60
// PROTOTYPE: bool __thiscall DelOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetEnemyList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:551
// RVA: 0x000B5FF0
// ADDRESS: 004b5ff0
// PROTOTYPE: void __thiscall GetEnemyList(list<COrganizing*,std::allocator<COrganizing*>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearEnemyFation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:675
// RVA: 0x000B6030
// ADDRESS: 004b6030
// PROTOTYPE: void __thiscall ClearEnemyFation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearCityWarEnemyFation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:683
// RVA: 0x000B6080
// ADDRESS: 004b6080
// PROTOTYPE: void __thiscall ClearCityWarEnemyFation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateAllApplyMemberToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:924
// RVA: 0x000B60D0
// ADDRESS: 004b60d0
// PROTOTYPE: void __thiscall UpdateAllApplyMemberToClient(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetExp
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1307
// RVA: 0x000B61F0
// ADDRESS: 004b61f0
// PROTOTYPE: void __thiscall SetExp(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateLeaveWordToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1453
// RVA: 0x000B6240
// ADDRESS: 004b6240
// PROTOTYPE: void __thiscall UpdateLeaveWordToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearApplyList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1520
// RVA: 0x000B6450
// ADDRESS: 004b6450
// PROTOTYPE: bool __thiscall ClearApplyList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsControbute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2457
// RVA: 0x000B64A0
// ADDRESS: 004b64a0
// PROTOTYPE: bool __thiscall IsControbute(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OnMemberLvlChange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2583
// RVA: 0x000B6590
// ADDRESS: 004b6590
// PROTOTYPE: void __thiscall OnMemberLvlChange(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsInApplyMembers
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:791
// RVA: 0x000B6760
// ADDRESS: 004b6760
// PROTOTYPE: long __thiscall IsInApplyMembers(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::EditLeaveWord
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2075
// RVA: 0x000B6790
// ADDRESS: 004b6790
// PROTOTYPE: bool __thiscall EditLeaveWord(long param_1, long param_2, eOperator param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetLWFunction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1101
// RVA: 0x000B7030
// ADDRESS: 004b7030
// PROTOTYPE: void __thiscall SetLWFunction(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetPronounceFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1117
// RVA: 0x000B7400
// ADDRESS: 004b7400
// PROTOTYPE: void __thiscall SetPronounceFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetEndueRightFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1131
// RVA: 0x000B77D0
// ADDRESS: 004b77d0
// PROTOTYPE: void __thiscall SetEndueRightFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetJoinVillageWarFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1159
// RVA: 0x000B7BA0
// ADDRESS: 004b7ba0
// PROTOTYPE: void __thiscall SetJoinVillageWarFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetJoinCityWarFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1173
// RVA: 0x000B7F70
// ADDRESS: 004b7f70
// PROTOTYPE: void __thiscall SetJoinCityWarFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetCreateUnionFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1187
// RVA: 0x000B8340
// ADDRESS: 004b8340
// PROTOTYPE: void __thiscall SetCreateUnionFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetMaxMememberNums
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1202
// RVA: 0x000B8710
// ADDRESS: 004b8710
// PROTOTYPE: void __thiscall SetMaxMememberNums(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetLvl
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1215
// RVA: 0x000B8940
// ADDRESS: 004b8940
// PROTOTYPE: void __thiscall SetLvl(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DeleteOrgaToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1647
// RVA: 0x000B8A20
// ADDRESS: 004b8a20
// PROTOTYPE: void __thiscall DeleteOrgaToClient(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Pronounce
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2000
// RVA: 0x000B8DC0
// ADDRESS: 004b8dc0
// PROTOTYPE: bool __thiscall Pronounce(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, tagTime * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UploadIcon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2395
// RVA: 0x000B8F30
// ADDRESS: 004b8f30
// PROTOTYPE: bool __thiscall UploadIcon(long param_1, tagTime * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetControbuter
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2468
// RVA: 0x000B92F0
// ADDRESS: 004b92f0
// PROTOTYPE: void __thiscall SetControbuter(long param_1, long param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOwnedCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:471
// RVA: 0x000B9DE0
// ADDRESS: 004b9de0
// PROTOTYPE: void __thiscall AddOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetMemberList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:520
// RVA: 0x000B9E60
// ADDRESS: 004b9e60
// PROTOTYPE: void __thiscall GetMemberList(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:759
// RVA: 0x000B9EF0
// ADDRESS: 004b9ef0
// PROTOTYPE: bool __thiscall DelMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::RemoveApplyMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:775
// RVA: 0x000B9F50
// ADDRESS: 004b9f50
// PROTOTYPE: long __thiscall RemoveApplyMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdatePropertyToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1343
// RVA: 0x000B9FB0
// ADDRESS: 004b9fb0
// PROTOTYPE: void __thiscall UpdatePropertyToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateEnemyFactionToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2659
// RVA: 0x000BA0F0
// ADDRESS: 004ba0f0
// PROTOTYPE: void __thiscall UpdateEnemyFactionToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateCityWarEnemyFactionToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2681
// RVA: 0x000BA210
// ADDRESS: 004ba210
// PROTOTYPE: void __thiscall UpdateCityWarEnemyFactionToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2860
// RVA: 0x000BA310
// ADDRESS: 004ba310
// PROTOTYPE: void __thiscall SetParam(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddDefenceVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2902
// RVA: 0x000BA3B0
// ADDRESS: 004ba3b0
// PROTOTYPE: void __thiscall AddDefenceVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOffenseVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2913
// RVA: 0x000BA3D0
// ADDRESS: 004ba3d0
// PROTOTYPE: void __thiscall AddOffenseVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddVillageWarVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2924
// RVA: 0x000BA3F0
// ADDRESS: 004ba3f0
// PROTOTYPE: void __thiscall AddVillageWarVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ReInitialPropertyByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:205
// RVA: 0x000BA630
// ADDRESS: 004ba630
// PROTOTYPE: bool __thiscall ReInitialPropertyByLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOwnedCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:481
// RVA: 0x000BA650
// ADDRESS: 004ba650
// PROTOTYPE: void __thiscall AddOwnedCity(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetEnemyList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:539
// RVA: 0x000BA6A0
// ADDRESS: 004ba6a0
// PROTOTYPE: set<long,std::less<long>,std::allocator<long>_> __thiscall GetEnemyList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetCityWarEnemyList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:545
// RVA: 0x000BA6D0
// ADDRESS: 004ba6d0
// PROTOTYPE: set<long,std::less<long>,std::allocator<long>_> __thiscall GetCityWarEnemyList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsUsingPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:691
// RVA: 0x000BA700
// ADDRESS: 004ba700
// PROTOTYPE: bool __thiscall IsUsingPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetMemPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:705
// RVA: 0x000BA760
// ADDRESS: 004ba760
// PROTOTYPE: void __thiscall SetMemPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Exit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1535
// RVA: 0x000BAAB0
// ADDRESS: 004baab0
// PROTOTYPE: bool __thiscall Exit(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::FireOut
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1684
// RVA: 0x000BB140
// ADDRESS: 004bb140
// PROTOTYPE: bool __thiscall FireOut(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DubAndSetJobLvl
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1759
// RVA: 0x000BB8A0
// ADDRESS: 004bb8a0
// PROTOTYPE: bool __thiscall DubAndSetJobLvl(long param_1, long param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::EndueRightToMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1835
// RVA: 0x000BBFB0
// ADDRESS: 004bbfb0
// PROTOTYPE: bool __thiscall EndueRightToMember(long param_1, long param_2, ePurview param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AbolishRightToMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1923
// RVA: 0x000BC500
// ADDRESS: 004bc500
// PROTOTYPE: bool __thiscall AbolishRightToMember(long param_1, long param_2, ePurview param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::LeaveWord
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2033
// RVA: 0x000BCA40
// ADDRESS: 004bca40
// PROTOTYPE: bool __thiscall LeaveWord(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, tagTime * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Upgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2288
// RVA: 0x000BCC60
// ADDRESS: 004bcc60
// PROTOTYPE: bool __thiscall Upgrade(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::LoadLeavewords
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2933
// RVA: 0x000BD3F0
// ADDRESS: 004bd3f0
// PROTOTYPE: void __thiscall LoadLeavewords(tagLeaveWord * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::~CFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:46
// RVA: 0x000BD590
// ADDRESS: 004bd590
// PROTOTYPE: void __thiscall ~CFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:161
// RVA: 0x000BD700
// ADDRESS: 004bd700
// PROTOTYPE: long __thiscall GetID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:162
// RVA: 0x000BD710
// ADDRESS: 004bd710
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> * __thiscall GetName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetMasterID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:163
// RVA: 0x000BD720
// ADDRESS: 004bd720
// PROTOTYPE: long __thiscall GetMasterID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetLvl
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:164
// RVA: 0x000BD730
// ADDRESS: 004bd730
// PROTOTYPE: long __thiscall GetLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:166
// RVA: 0x000BD740
// ADDRESS: 004bd740
// PROTOTYPE: long __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetPermitDemise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:168
// RVA: 0x000BD750
// ADDRESS: 004bd750
// PROTOTYPE: void __thiscall SetPermitDemise(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetCountry
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:171
// RVA: 0x000BD760
// ADDRESS: 004bd760
// PROTOTYPE: uchar __thiscall GetCountry(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetCountry
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:172
// RVA: 0x000BD770
// ADDRESS: 004bd770
// PROTOTYPE: void __thiscall SetCountry(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsSuperiorOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:192
// RVA: 0x000BD780
// ADDRESS: 004bd780
// PROTOTYPE: long __thiscall IsSuperiorOrganizing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetEstablishedTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:202
// RVA: 0x000BD790
// ADDRESS: 004bd790
// PROTOTYPE: tagTime * __thiscall GetEstablishedTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsLWFunction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:225
// RVA: 0x000BD7A0
// ADDRESS: 004bd7a0
// PROTOTYPE: bool __thiscall IsLWFunction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsCreateUnionFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:237
// RVA: 0x000BD7B0
// ADDRESS: 004bd7b0
// PROTOTYPE: bool __thiscall IsCreateUnionFun(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetOwnedCities
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:252
// RVA: 0x000BD7D0
// ADDRESS: 004bd7d0
// PROTOTYPE: list<long,std::allocator<long>_> * __thiscall GetOwnedCities(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetEneFacChanged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:269
// RVA: 0x000BD7E0
// ADDRESS: 004bd7e0
// PROTOTYPE: void __thiscall SetEneFacChanged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetCityEneFacChagned
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:270
// RVA: 0x000BD7F0
// ADDRESS: 004bd7f0
// PROTOTYPE: void __thiscall SetCityEneFacChagned(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetDefenceVictorCounts
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:288
// RVA: 0x000BD800
// ADDRESS: 004bd800
// PROTOTYPE: long __thiscall GetDefenceVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetOffenseVictorCounts
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:289
// RVA: 0x000BD810
// ADDRESS: 004bd810
// PROTOTYPE: long __thiscall GetOffenseVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetDelRemainTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:316
// RVA: 0x000BD820
// ADDRESS: 004bd820
// PROTOTYPE: long __thiscall GetDelRemainTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetTitleByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:207
// RVA: 0x000BD870
// ADDRESS: 004bd870
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __thiscall GetTitleByID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetJobLvlByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:217
// RVA: 0x000BD910
// ADDRESS: 004bd910
// PROTOTYPE: ushort __thiscall GetJobLvlByID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Initial
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:66
// RVA: 0x000BD950
// ADDRESS: 004bd950
// PROTOTYPE: bool __thiscall Initial(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOwnedCitiesToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:404
// RVA: 0x000BDD70
// ADDRESS: 004bdd70
// PROTOTYPE: bool __thiscall AddOwnedCitiesToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddEnemyOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:558
// RVA: 0x000BDE80
// ADDRESS: 004bde80
// PROTOTYPE: void __thiscall AddEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelEnemyOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:576
// RVA: 0x000BE030
// ADDRESS: 004be030
// PROTOTYPE: void __thiscall DelEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddCityWarEnemyOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:601
// RVA: 0x000BE1E0
// ADDRESS: 004be1e0
// PROTOTYPE: void __thiscall AddCityWarEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelCityWarEnemyOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:619
// RVA: 0x000BE390
// ADDRESS: 004be390
// PROTOTYPE: void __thiscall DelCityWarEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ApplyForJoin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:813
// RVA: 0x000BE520
// ADDRESS: 004be520
// PROTOTYPE: bool __thiscall ApplyForJoin(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateApplyMemberToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:891
// RVA: 0x000BEC80
// ADDRESS: 004bec80
// PROTOTYPE: void __thiscall UpdateApplyMemberToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DoJoin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:947
// RVA: 0x000BEE40
// ADDRESS: 004bee40
// PROTOTYPE: bool __thiscall DoJoin(long param_1, long param_2, long param_3, tagTime * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Disband
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1597
// RVA: 0x000BFA00
// ADDRESS: 004bfa00
// PROTOTYPE: bool __thiscall Disband(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Demise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2099
// RVA: 0x000BFDE0
// ADDRESS: 004bfde0
// PROTOTYPE: bool __thiscall Demise(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OperatorTax
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2253
// RVA: 0x000C0880
// ADDRESS: 004c0880
// PROTOTYPE: bool __thiscall OperatorTax(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OperatorCityGate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2268
// RVA: 0x000C0930
// ADDRESS: 004c0930
// PROTOTYPE: bool __thiscall OperatorCityGate(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetIsPermit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2436
// RVA: 0x000C09A0
// ADDRESS: 004c09a0
// PROTOTYPE: void __thiscall SetIsPermit(long param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OnMemberPosChange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2598
// RVA: 0x000C0B40
// ADDRESS: 004c0b40
// PROTOTYPE: void __thiscall OnMemberPosChange(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateOwnedCityToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2702
// RVA: 0x000C0BF0
// ADDRESS: 004c0bf0
// PROTOTYPE: void __thiscall UpdateOwnedCityToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetPlayerHeader
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2762
// RVA: 0x000C0D10
// ADDRESS: 004c0d10
// PROTOTYPE: long __thiscall GetPlayerHeader(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:12
// RVA: 0x000C0D50
// ADDRESS: 004c0d50
// PROTOTYPE: undefined __thiscall CFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:38
// RVA: 0x000C0EE0
// ADDRESS: 004c0ee0
// PROTOTYPE: undefined __thiscall CFaction(long param_1, long param_2, tagTime * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:293
// RVA: 0x000C10B0
// ADDRESS: 004c10b0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: CFaction::CloneSaveData RVA 0x000C1210.

// ============================================================================
// FUNCTION: CFaction::SetOwnedCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:513
// RVA: 0x000C16A0
// ADDRESS: 004c16a0
// PROTOTYPE: void __thiscall SetOwnedCity(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CheckOperValidate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:749
// RVA: 0x000C17F0
// ADDRESS: 004c17f0
// PROTOTYPE: bool __thiscall CheckOperValidate(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsEnemyFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:464
// RVA: 0x000C1830
// ADDRESS: 004c1830
// PROTOTYPE: long __thiscall IsEnemyFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:448
// RVA: 0x000C1B90
// ADDRESS: 004c1b90
// PROTOTYPE: bool __thiscall SetName(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AbolishMemPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:716
// RVA: 0x000C1DD0
// ADDRESS: 004c1dd0
// PROTOTYPE: void __thiscall AbolishMemPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsMaster
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:178
// RVA: 0x000C1EE0
// ADDRESS: 004c1ee0
// PROTOTYPE: long __thiscall IsMaster(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




































// COMPONENT_VARIANT_END: WorldServer
