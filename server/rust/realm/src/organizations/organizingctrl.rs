//! Блоки и lookup-типы `COrganizingCtrl`, вынесенные сюда заранее: сам
//! контроллер остаётся в старом пакете до шага переноса
//! organizing-области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use crate::app::world_game_view::WorldGameView;
use crate::organizations::faction::{
    FactionDelMemberBlock, FactionDelMemberReport, FactionInitialPropertyBlock,
    FactionPropertyDelivery, FactionSuperiorOrganizingBlock,
};
use crate::organizations::organizingparam::COrganizingParam;

/// Результат `COrganizingCtrl::is_free_player`: свободный игрок, член
/// фракции либо ячейка с null-указателем фракции.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreePlayerLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnionMemberDetachOutcome {
    NonPositiveFactionId,
    FactionEntryMissing,
    NullFactionPointer,
    Detached {
        deliveries: Vec<FactionPropertyDelivery>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnionMemberDetachBlockSource {
    SuperiorOrganizing(FactionSuperiorOrganizingBlock),
    Property(FactionInitialPropertyBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnionMemberDetachBlock {
    pub faction_id: i32,
    pub source: UnionMemberDetachBlockSource,
}

/// Наблюдаемый результат `COrganizingCtrl::OnDeleteRole` до wire-ответа
/// LoginServer. Числа совпадают с switch `0..=4`.
#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingDeleteRoleOutcome {
    AllowedNoFaction,
    CountryJob,
    MemberRemoved {
        faction_id: i32,
        removal: Option<FactionDelMemberReport>,
    },
    FactionMaster {
        faction_id: i32,
    },
    UnionMissing {
        faction_id: i32,
        union_id: i32,
    },
    UnionDetached {
        faction_id: i32,
        union_id: i32,
        detach: UnionMemberDetachOutcome,
    },
}

impl OrganizingDeleteRoleOutcome {
    pub const fn legacy_code(&self) -> i32 {
        match self {
            Self::AllowedNoFaction | Self::MemberRemoved { .. } | Self::UnionMissing { .. } => 0,
            Self::FactionMaster { .. } => 1,
            Self::UnionDetached { .. } => 3,
            Self::CountryJob => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingDeleteRoleBlock {
    NullFactionDuringMembershipScan { map_key: i32 },
    MemberRemoval {
        faction_id: i32,
        source: FactionDelMemberBlock,
    },
    MissingFactionProperty { faction_id: i32 },
    UnionDetach(UnionMemberDetachBlock),
}

/// Узкий dyn-шов ветви удаления роли к organizing-контроллеру. Inherent
/// `COrganizingCtrl::on_delete_role` дополнительно получает игру как
/// `&dyn WorldGameView` (его единственный игровой путь — refresh свойств
/// фракции при отвязке от союза), поэтому gate реализуется на контроллере
/// у владельца организаций и делегирует inherent-методу; игру в шов
/// передаёт сам обработчик коротким перезаймом по форме handler-волны.
pub trait WorldDeleteRoleOrganizingGate {
    fn apply_delete_role(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        player_id: i32,
        country_has_job: bool,
    ) -> Result<OrganizingDeleteRoleOutcome, OrganizingDeleteRoleBlock>;
}
