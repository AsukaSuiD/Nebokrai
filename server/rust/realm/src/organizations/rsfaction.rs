//! World DB-владелец `CRsFaction` из `rsfaction.cpp`, подтверждённый
//! `Nworldserver.exe` и `WorldServer.pdb`. Трейт `RsFactionOwner` и его
//! data-семья перенесены в Realm `organizations/`; Tiberius-реализация
//! остаётся в старом пакете, потому что save-ветка leave word ещё вызывает
//! `CGame::check_point` владельца игры.
//!
//! Сохраняются раздельные операции faction property, members, applications,
//! leave words, icon, pronounce и ability, их SQL-порядок и исходные bool
//! результаты. Load строит записи в порядке провайдера; save не добавляет
//! транзакцию или rollback поверх уже выполненных команд. Tiberius и owned
//! values заменяют ADO/COM и MSVC containers без изменения DB-семантики.
//!
//! `&dyn WorldGameView` заменяет прямую ссылку на игру: owner потребляется
//! generic-связкой `F: RsFactionOwner` и associated type старого `CGame`,
//! поэтому RPITIT-форма без dyn-совместимости сохраняется, как у
//! `persistence::rsplayer`. Будущие `load_all_factions`/`load_faction_property`
//! захватывают `&dyn WorldGameView` без Sync-ограничения, поэтому `Send` на
//! них не добавлен; save/delete-методы захватывают только `&mut self`,
//! `FactionSaveSnapshot`/`CFaction`-данные и `&mut WorldTdsClient` и помечены
//! `+ Send`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::app::world_game_view::WorldGameView;
use crate::content::organizing::UnterminatedMemberField;
use crate::organizations::faction::{
    CFaction, FactionBaseProperty, FactionInitialBlock, UnterminatedLeaveWordField,
};
use crate::organizations::organizingparam::COrganizingParam;
use crate::persistence::rssetup::{WorldDatabaseConnectionError, WorldTdsClient};

/// Безопасный prefix concrete `CFaction`, сформированный private owner-ом
/// `LoadFactionProperty` до следующих четырёх DB-этапов `LoadAllFaction`.
pub type FactionPropertyLoadStaging = BTreeMap<i32, CFaction>;

/// Наблюдаемый исход private owner-а `LoadFactionProperty`.
///
/// `factions` сохраняет уже вставленный map-prefix. Повторный ID заменяет
/// прежний объект, как `std::map::operator[]`; старая утечка pointer-а не
/// переносится, поскольку не меняла опубликованное последнее значение.
pub enum FactionPropertyLoadOutcome {
    ReturnedTrue { factions: FactionPropertyLoadStaging },
    ReturnedFalse { factions: FactionPropertyLoadStaging },
    BlockedMissingFact {
        factions: FactionPropertyLoadStaging,
        block: FactionInitialBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionMemberLoadBlock {
    pub visible_len: usize,
    pub capacity: usize,
}

pub enum FactionMembersLoadOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(FactionMemberLoadBlock),
}

/// Безопасная граница короткого binary `Pronounce`: оригинал memcpy читал бы за
/// полученный DB chunk, а внешняя реакция повреждённой строки не определена.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionPronounceLoadBlock {
    pub actual_size: usize,
}

pub enum FactionAbilityLoadOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(FactionPronounceLoadBlock),
}

/// Итог верхнего `LoadAllFaction` до передачи готовой map парному controller-owner.
/// PDB задаёт `int` return; для raw temporary `std::map` это его число элементов
/// до cleanup. Неуспешная ветвь удаляет map, но возвращаемое в `Initialize` log
/// число относится к сохранённому до удаления staging-prefix.
pub enum FactionLoadOutcome {
    ReturnedTrue {
        factions: FactionPropertyLoadStaging,
        reported_count: i32,
    },
    ReturnedFalse {
        factions: FactionPropertyLoadStaging,
        reported_count: i32,
    },
    BlockedInitial {
        factions: FactionPropertyLoadStaging,
        block: FactionInitialBlock,
        reported_count: i32,
    },
    BlockedMember {
        factions: FactionPropertyLoadStaging,
        block: FactionMemberLoadBlock,
        reported_count: i32,
    },
    BlockedPronounce {
        factions: FactionPropertyLoadStaging,
        block: FactionPronounceLoadBlock,
        reported_count: i32,
    },
}

pub fn reported_faction_load_count(factions: &FactionPropertyLoadStaging) -> i32 {
    i32::try_from(factions.len()).unwrap_or(i32::MAX)
}

pub struct FactionPropertySaveSnapshot {
    pub master_id: i32,
    pub base_property: FactionBaseProperty,
    pub delete_remain_time: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionSaveProjectionBlock {
    MasterId,
    BaseProperty,
    DeleteRemainTime,
}

pub struct FactionSaveSnapshot<'faction> {
    pub faction: &'faction mut CFaction,
    pub canonical_goods_war_count: Option<i32>,
    pub property: Option<FactionPropertySaveSnapshot>,
}

impl<'faction> FactionSaveSnapshot<'faction> {
    pub fn from_faction(
        faction: &'faction mut CFaction,
        canonical_goods_war_count: Option<i32>,
    ) -> Result<Self, FactionSaveProjectionBlock> {
        let property = if faction.change_data_type() & 1 != 0 {
            Some(FactionPropertySaveSnapshot {
                master_id: faction
                    .master_id()
                    .ok_or(FactionSaveProjectionBlock::MasterId)?,
                base_property: faction
                    .base_property()
                    .ok_or(FactionSaveProjectionBlock::BaseProperty)?,
                delete_remain_time: faction
                    .delete_remain_time()
                    .ok_or(FactionSaveProjectionBlock::DeleteRemainTime)?,
            })
        } else {
            None
        };

        Ok(Self {
            faction,
            canonical_goods_war_count,
            property,
        })
    }
}

/// Исходный bool-результат `SaveFactionMembers` либо безопасная граница старого
/// чтения `strTitle` за фиксированным массивом.
#[derive(Debug)]
pub enum FactionMembersSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(UnterminatedMemberField),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionLeaveWordSaveBlock {
    UnterminatedContent(UnterminatedLeaveWordField),
}

impl fmt::Display for FactionLeaveWordSaveBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedContent(block) => block.fmt(formatter),
        }
    }
}

impl Error for FactionLeaveWordSaveBlock {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnterminatedContent(block) => Some(block),
        }
    }
}

impl From<UnterminatedLeaveWordField> for FactionLeaveWordSaveBlock {
    fn from(block: UnterminatedLeaveWordField) -> Self {
        Self::UnterminatedContent(block)
    }
}

#[derive(Debug)]
pub enum FactionLeaveWordsSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(FactionLeaveWordSaveBlock),
}

#[derive(Debug)]
pub enum FactionSaveBlock {
    Member(UnterminatedMemberField),
    LeaveWord(FactionLeaveWordSaveBlock),
}

#[derive(Debug)]
pub enum FactionSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(FactionSaveBlock),
}

#[derive(Debug)]
pub struct RsFactionNotice {
    pub operation: RsFactionOperation,
    pub error: RsFactionSaveError,
}

#[derive(Clone, Copy, Debug)]
pub enum RsFactionOperation {
    LoadFactionProperty,
    LoadFactionMembers,
    LoadLeaveWords,
    LoadAbility,
    LoadFactionApplyPersons,
    SaveFaction,
    DeleteFaction,
    SaveFactionProperty,
    DeleteFactionMembers,
    SaveFactionMember,
    DeleteFactionLeaveWords,
    SaveFactionLeaveWord,
    SaveFactionAbility,
    DeleteFactionApplyPersons,
    SaveFactionApplyPerson,
}

#[derive(Debug)]
pub enum RsFactionSaveError {
    Database(RsFactionDatabaseError),
    MissingConnection,
    MissingPropertyRow,
    MissingSettings,
    Connection(WorldDatabaseConnectionError),
    MissingRequiredValue(&'static str),
    MissingFaction { faction_id: i32 },
}

impl fmt::Display for RsFactionSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => {
                formatter.write_str("не передано соединение World faction DB")
            }
            Self::MissingPropertyRow => formatter
                .write_str("World faction DB не содержит base-property строку для обновления"),
            Self::MissingSettings => formatter.write_str("не заданы параметры World faction DB"),
            Self::Connection(error) => error.fmt(formatter),
            Self::MissingRequiredValue(column) => {
                write!(formatter, "в faction DB отсутствует обязательное поле {column}")
            }
            Self::MissingFaction { faction_id } => {
                write!(formatter, "не найдена faction {faction_id} для member DB-строки")
            }
        }
    }
}

impl Error for RsFactionSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Connection(error) => Some(error),
            Self::MissingConnection
            | Self::MissingPropertyRow
            | Self::MissingSettings
            | Self::MissingRequiredValue(_)
            | Self::MissingFaction { .. } => None,
        }
    }
}

#[derive(Debug)]
pub struct RsFactionDatabaseError(tiberius::error::Error);

impl fmt::Display for RsFactionDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World faction DB: {}", self.0)
    }
}

impl Error for RsFactionDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsFactionDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

/// Ошибка одного field/query шага `LoadFactionProperty` до преобразования в
/// диагностическое сообщение. Runtime SQL и DB-значения намеренно не выдаются.
#[derive(Debug)]
pub enum FactionLoadReadError {
    Database(tiberius::error::Error),
    MissingRequiredValue(&'static str),
    MissingFaction { faction_id: i32 },
}

impl From<tiberius::error::Error> for FactionLoadReadError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Database(error)
    }
}

impl From<FactionLoadReadError> for RsFactionSaveError {
    fn from(error: FactionLoadReadError) -> Self {
        match error {
            FactionLoadReadError::Database(error) => Self::Database(error.into()),
            FactionLoadReadError::MissingRequiredValue(column) => Self::MissingRequiredValue(column),
            FactionLoadReadError::MissingFaction { faction_id } => Self::MissingFaction { faction_id },
        }
    }
}

pub trait RsFactionOwner {
 /// Выполняет полный оригинал порядок пяти load-owner-ов на одном World DB
 /// connection и возвращает ready-to-publish faction staging map.
    fn load_all_factions(
        &mut self,
        master_title: &[u8],
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
    ) -> impl std::future::Future<Output = FactionLoadOutcome>;

 /// Открывает отдельное World DB соединение и materialize-ит только
 /// `LoadFactionProperty`; оставшиеся load-owner-ы пока не смешиваются с
 /// этим staging map.
    fn load_faction_property(
        &mut self,
        master_title: &[u8],
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
    ) -> impl std::future::Future<Output = FactionPropertyLoadOutcome>;

    fn save_faction(
        &mut self,
        snapshot: Option<&mut FactionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = FactionSaveOutcome> + Send;

    fn del_faction(
        &mut self,
        faction_id: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_faction_property(
        &mut self,
        snapshot: Option<&mut FactionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

 /// Перезаписывает ordered member-снимок для dirty-bit `2` внутри
 /// caller-транзакции.
    fn save_faction_members(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = FactionMembersSaveOutcome> + Send;

 /// Перезаписывает ordered leave-word snapshot для dirty-bit `4` внутри
 /// caller-транзакции.
    fn save_leave_words(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = FactionLeaveWordsSaveOutcome> + Send;

 /// Сохраняет ability-row, затем ordered apply-person snapshot внутри
 /// caller-транзакции.
    fn save_ability(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn pop_notice(&mut self) -> Option<RsFactionNotice>;
}
