//! Игрок `CPlayer` из `player.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Owner объединяет identity, region/session, контейнеры, country/faction/
//! union/JJC, DB snapshots и World wire. Load/clone/decode применяют вложенные
//! состояния в исходном порядке; поздняя ошибка сохраняет уже видимый префикс.
//! Save snapshot фиксируется до передачи worker-у и не заимствует live player.
//!
//! `UpdateFactionInfo` разрешает faction/union, меняет локальные properties и
//! публикует player/faction effects до последующих notifications. Missing owner
//! останавливает только текущую ветвь без выдуманного membership или rollback.
//!
//! Rust composition, стандартные коллекции и slices заменяют ABI, raw pointers
//! и fixed-buffer arithmetic; byte strings, GUID, signedness и wire остаются.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::sync::atomic::{AtomicI32, Ordering};

static NEXT_NET_EXCHANGE_ID: AtomicI32 = AtomicI32::new(0);

use crate::dbaccess::worlddb::dbgoods::{DbGoodsOwner, PlayerGoodsFiledSnapshot};
use crate::dbaccess::worlddb::goodslistener::{GoodsContainerTraversalSnapshot, TraversedGoods};
use crate::dbaccess::worlddb::rsjjcsys::{PlayerJjcDataSnapshot, RsJjcSysOwner};
use crate::dbaccess::worlddb::rsplayer::{
    EmbeddedFriendNameNul, LoadedPlayerScriptFlag, PlayerAbilityCreationSnapshot,
    PlayerAbilityLoadScalarSnapshot, PlayerAbilitySaveSnapshot, PlayerAbilityScalarSnapshot,
    PlayerAbilitySkill, PlayerBaseSaveSnapshot, PlayerCreationBaseSnapshot,
    PlayerCreationSnapshot, PlayerFriendName, PlayerQuestSaveEntry, PlayerQuestSaveSnapshot,
    PlayerSaveOutcome, PlayerSaveSnapshot, PlayerScriptFlagSnapshot, PlayerThing as DbPlayerThing,
    RsPlayerOwner,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::date::{TagTime, TagTimeArithmeticBlock};
use crate::public::dupliregionsetup::CDupliRegionSetup;
use nebokrai_shared::values::CGuid;
use crate::setup::globesetup::{GlobePlayerPropertyCoefficients, GlobeSetupSnapshot};
use crate::setup::leitingsetup::{CThingSetup, LeiTingDailyThing, LeiTingLocalTime};
use nebokrai_shared::resources::{
    CPlayerList, PlayerCreationPropertiesLookup, PlayerOriginEquipment,
};

use super::container::camountlimitgoodscontainer::{
    AmountContainerCodecError, CAmountLimitGoodsContainer,
};
use super::container::cbank::CBank;
use super::container::cbattlefairycontainer::CBattleFairyContainer;
use super::container::cdepot::CDepot;
use super::container::cequipmentcontainer::{CEquipmentContainer, EquipmentContainerCodecError};
use super::container::cfairycontainer::CFairyContainer;
use super::container::cjifen::CJiFen;
use super::container::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeContainerCodecError,
};
use super::container::cwallet::CWallet;
use super::container::cyuanbao::CYuanBao;
use super::country::countryparam::{CCountryParam, CountryRect};
use super::goods::cgoods::{CGoods, GoodsCodecError, GoodsDbSnapshotBlock};
use super::goods::cgoodsbaseproperties::GAP_WEAPON_LEVEL;
use super::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex, create_goods,
    query_goods_id_by_original_name_bytes,
};
use super::listener::cseekgoodslistener::CSeekGoodsListener;
use super::moveshape::CMoveShape;
use super::region::{CRegion, RegionRandomPositionBlock};
use super::shape::{ShapeDecodeError, ShapeTileCoordinateBlock};

const BASE_PROPERTY_WIRE_LEN: usize = 0x194;
const BASE_PROPERTY_LEVEL_OFFSET: usize = 0x04;
const BASE_PROPERTY_EXP_OFFSET: usize = 0x08;
const BASE_PROPERTY_HEAD_PIC_OFFSET: usize = 0x0C;
const BASE_PROPERTY_FACE_PIC_OFFSET: usize = 0x0D;
const BASE_PROPERTY_FY_ENERGY_OFFSET: usize = 0x17C;
const BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET: usize = 0x180;
const BASE_PROPERTY_LT_UP_60_COUNT_OFFSET: usize = 0x184;
const BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET: usize = 0x186;
const BASE_PROPERTY_LT_60_STAMP_OFFSET: usize = 0x188;
const BASE_PROPERTY_FOSTER_NUM_OFFSET: usize = 0x120;
const BASE_PROPERTY_HATCHER_NUM_OFFSET: usize = 0x124;
const BASE_PROPERTY_OCCUPATION_OFFSET: usize = 0x0E;
const BASE_PROPERTY_SEX_OFFSET: usize = 0x0F;
const BASE_PROPERTY_SPOUSE_ID_OFFSET: usize = 0x10;
const BASE_PROPERTY_UNION_ID_OFFSET: usize = 0x14;
const BASE_PROPERTY_PK_COUNT_OFFSET: usize = 0x1C;
const BASE_PROPERTY_KILL_COUNT_OFFSET: usize = 0x20;
const BASE_PROPERTY_HIT_TOP_LOG_OFFSET: usize = 0x24;
const BASE_PROPERTY_HOT_HIT_OFFSET: usize = 0x28;
const BASE_PROPERTY_LOAN_MAX_OFFSET: usize = 0x2C;
const BASE_PROPERTY_LOAN_OFFSET: usize = 0x30;
const BASE_PROPERTY_LOAN_TIME_OFFSET: usize = 0x34;
const BASE_PROPERTY_IS_CHARGED_OFFSET: usize = 0x38;
const BASE_PROPERTY_REMAIN_POINT_OFFSET: usize = 0x3A;
const BASE_PROPERTY_HOT_KEYS_OFFSET: usize = 0x3C;
const BASE_PROPERTY_PK_NORMAL_OFFSET: usize = 0x9C;
const BASE_PROPERTY_PK_TEAM_OFFSET: usize = 0x9D;
const BASE_PROPERTY_PK_UNION_OFFSET: usize = 0x9E;
const BASE_PROPERTY_PK_BADMAN_OFFSET: usize = 0x9F;
const BASE_PROPERTY_PK_COUNTRY_OFFSET: usize = 0xA0;
const BASE_PROPERTY_YP_OFFSET: usize = 0xA2;
const BASE_PROPERTY_HP_OFFSET: usize = 0xA4;
const BASE_PROPERTY_MP_OFFSET: usize = 0xA8;
const BASE_PROPERTY_RP_OFFSET: usize = 0xAC;
const BASE_PROPERTY_MAX_HP_OFFSET: usize = 0xB0;
const BASE_PROPERTY_MAX_MP_OFFSET: usize = 0xB4;
const BASE_PROPERTY_MAX_YP_OFFSET: usize = 0xB8;
const BASE_PROPERTY_MAX_RP_OFFSET: usize = 0xBA;
const BASE_PROPERTY_STR_OFFSET: usize = 0xBC;
const BASE_PROPERTY_DEX_OFFSET: usize = 0xC0;
const BASE_PROPERTY_CON_OFFSET: usize = 0xC4;
const BASE_PROPERTY_INT_OFFSET: usize = 0xC8;
const BASE_PROPERTY_MIN_ATK_OFFSET: usize = 0xCC;
const BASE_PROPERTY_MAX_ATK_OFFSET: usize = 0xD0;
const BASE_PROPERTY_HIT_OFFSET: usize = 0xD4;
const BASE_PROPERTY_BURDEN_OFFSET: usize = 0xD6;
const BASE_PROPERTY_CCH_OFFSET: usize = 0xD8;
const BASE_PROPERTY_DEF_OFFSET: usize = 0xDC;
const BASE_PROPERTY_DODGE_OFFSET: usize = 0xE0;
const BASE_PROPERTY_ATC_SPEED_OFFSET: usize = 0xE2;
const BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET: usize = 0xE4;
const BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET: usize = 0xE8;
const BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET: usize = 0xEA;
const BASE_PROPERTY_VIGOUR_OFFSET: usize = 0xEC;
const BASE_PROPERTY_MAX_VIGOUR_OFFSET: usize = 0xF0;
const BASE_PROPERTY_ENERGY_OFFSET: usize = 0xF4;
const BASE_PROPERTY_MAX_ENERGY_OFFSET: usize = 0xF8;
const BASE_PROPERTY_CREDIT_OFFSET: usize = 0xFC;
const BASE_PROPERTY_EXALT_OFFSET: usize = 0x100;
const BASE_PROPERTY_DISPLAY_HEAD_PIECE_OFFSET: usize = 0x104;
const BASE_PROPERTY_QUEST_TIME_BEGIN_OFFSET: usize = 0x108;
const BASE_PROPERTY_QUEST_TIME_LIMIT_OFFSET: usize = 0x10C;
const BASE_PROPERTY_QUEST_OFFSET: usize = 0x110;
const BASE_PROPERTY_EXPLOIT_OFFSET: usize = 0x114;
const BASE_PROPERTY_KUDOS_OFFSET: usize = 0x118;
const BASE_PROPERTY_FAIRY_ENABLED_OFFSET: usize = 0x11C;
const BASE_PROPERTY_BATTLE_FAIRY_ENABLED_OFFSET: usize = 0x128;
const BASE_PROPERTY_BREAK_ARMOUR_OFFSET: usize = 0x12C;
const BASE_PROPERTY_BREAK_ELEMENT_OFFSET: usize = 0x134;
const BASE_PROPERTY_BREAK_BOUND_OFFSET: usize = 0x138;
const BASE_PROPERTY_POWER_OF_GOLD_OFFSET: usize = 0x13C;
const BASE_PROPERTY_DAYS_HONOR_OFFSET: usize = 0x140;
const BASE_PROPERTY_WEEKS_HONOR_OFFSET: usize = 0x144;
const BASE_PROPERTY_MONTHS_HONOR_OFFSET: usize = 0x148;
const BASE_PROPERTY_TOTAL_HONOR_OFFSET: usize = 0x14C;
const BASE_PROPERTY_RANK_NOBILITY_OFFSET: usize = 0x150;
const BASE_PROPERTY_APPELLATION_OFFSET: usize = 0x154;
const BASE_PROPERTY_MODE_OFFSET: usize = 0x158;
const BASE_PROPERTY_FETCH_POWER_OFFSET: usize = 0x164;
const BASE_PROPERTY_MAX_FETCH_POWER_OFFSET: usize = 0x168;
const BASE_PROPERTY_AUCTION_SPACE_OFFSET: usize = 0x170;
const BASE_PROPERTY_JJC_LEVEL_OFFSET: usize = 0x174;
const BASE_PROPERTY_JJC_SCORE_OFFSET: usize = 0x178;
const BASE_PROPERTY_SZL_OFFSET: usize = 0x18C;
const BASE_PROPERTY_GODS_BATTLE_FACTION_OFFSET: usize = 0x190;

const PROPERTY_MAX_HP_OFFSET: usize = 0x00;
const PROPERTY_MAX_MP_OFFSET: usize = 0x04;
const PROPERTY_MAX_YP_OFFSET: usize = 0x08;
const PROPERTY_MAX_RP_OFFSET: usize = 0x0A;
const PROPERTY_STR_OFFSET: usize = 0x0C;
const PROPERTY_DEX_OFFSET: usize = 0x10;
const PROPERTY_CON_OFFSET: usize = 0x14;
const PROPERTY_INT_OFFSET: usize = 0x18;
const PROPERTY_MIN_ATK_OFFSET: usize = 0x1C;
const PROPERTY_MAX_ATK_OFFSET: usize = 0x20;
const PROPERTY_HIT_OFFSET: usize = 0x24;
const PROPERTY_BURDEN_OFFSET: usize = 0x26;
const PROPERTY_CCH_OFFSET: usize = 0x28;
const PROPERTY_DEF_OFFSET: usize = 0x2C;
const PROPERTY_DODGE_OFFSET: usize = 0x30;
const PROPERTY_ATC_SPEED_OFFSET: usize = 0x32;
const PROPERTY_ELEMENT_RESISTANT_OFFSET: usize = 0x34;
const PROPERTY_HP_RECOVER_SPEED_OFFSET: usize = 0x38;
const PROPERTY_MP_RECOVER_SPEED_OFFSET: usize = 0x3A;
const PROPERTY_ELEMENT_MODIFY_OFFSET: usize = 0x48;
const PROPERTY_RE_ANK_OFFSET: usize = 0x4C;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerCodecError {
    Shape(ShapeDecodeError),
    Goods(GoodsCodecError),
    AmountContainer(AmountContainerCodecError),
    VolumeContainer(VolumeContainerCodecError),
    EquipmentContainer(EquipmentContainerCodecError),
    OrganizingUpdate(PlayerOrganizingUpdateError),
    CollectionLengthOutsideLegacyRange {
        field: &'static str,
        count: usize,
    },
    BufferShorterThanDeclaredLength {
        field: &'static str,
        declared: usize,
        available: usize,
    },
    StringOutsideLegacyCapacity {
        field: &'static str,
        length: usize,
        capacity: usize,
    },
    OccupationOutsidePropertyCoefficientRange {
        occupation: u8,
    },
    UninitializedWireField {
        field: &'static str,
    },
    NegativeLength {
        field: &'static str,
        value: i32,
    },
    UnterminatedString {
        field: &'static str,
        offset: usize,
        capacity: usize,
        available: usize,
    },
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for PlayerCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(error) => error.fmt(formatter),
            Self::Goods(error) => error.fmt(formatter),
            Self::AmountContainer(error) => error.fmt(formatter),
            Self::VolumeContainer(error) => error.fmt(formatter),
            Self::EquipmentContainer(error) => error.fmt(formatter),
            Self::OrganizingUpdate(error) => error.fmt(formatter),
            Self::CollectionLengthOutsideLegacyRange { field, count } => write!(
                formatter,
                "коллекция {field} содержит {count} элементов вне 32-битного legacy-диапазона"
            ),
            Self::BufferShorterThanDeclaredLength {
                field,
                declared,
                available,
            } => write!(
                formatter,
                "буфер {field} объявляет {declared} байт, но содержит только {available}"
            ),
            Self::StringOutsideLegacyCapacity {
                field,
                length,
                capacity,
            } => write!(
                formatter,
                "строка {field} длиной {length} не помещается в legacy-буфер {capacity} байт с NUL"
            ),
            Self::OccupationOutsidePropertyCoefficientRange { occupation } => write!(
                formatter,
                "occupation {occupation} выходит за три PDB-коэффициента UpdateProperty"
            ),
            Self::UninitializedWireField { field } => write!(
                formatter,
                "constructor CPlayer не назначил wire-поле {field} до сериализации"
            ),
            Self::NegativeLength { field, value } => {
                write!(
                    formatter,
                    "поле {field} содержит отрицательную длину {value}"
                )
            }
            Self::UnterminatedString {
                field,
                offset,
                capacity,
                available,
            } => write!(
                formatter,
                "строка {field} с offset {offset} не завершена NUL в legacy-буфере {capacity} байт; доступно {available}"
            ),
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for PlayerCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Shape(error) => Some(error),
            Self::Goods(error) => Some(error),
            Self::AmountContainer(error) => Some(error),
            Self::VolumeContainer(error) => Some(error),
            Self::EquipmentContainer(error) => Some(error),
            Self::OrganizingUpdate(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ShapeDecodeError> for PlayerCodecError {
    fn from(error: ShapeDecodeError) -> Self {
        Self::Shape(error)
    }
}

impl From<GoodsCodecError> for PlayerCodecError {
    fn from(error: GoodsCodecError) -> Self {
        Self::Goods(error)
    }
}

impl From<AmountContainerCodecError> for PlayerCodecError {
    fn from(error: AmountContainerCodecError) -> Self {
        Self::AmountContainer(error)
    }
}

impl From<VolumeContainerCodecError> for PlayerCodecError {
    fn from(error: VolumeContainerCodecError) -> Self {
        Self::VolumeContainer(error)
    }
}

impl From<EquipmentContainerCodecError> for PlayerCodecError {
    fn from(error: EquipmentContainerCodecError) -> Self {
        Self::EquipmentContainer(error)
    }
}

impl From<PlayerOrganizingUpdateError> for PlayerCodecError {
    fn from(error: PlayerOrganizingUpdateError) -> Self {
        Self::OrganizingUpdate(error)
    }
}

struct PlayerBaseProperty {
    wire: [u8; BASE_PROPERTY_WIRE_LEN],
    account: Vec<u8>,
    title: Vec<u8>,
}

/// Наблюдаемый результат двух прямых мутаций `tagBaseProperty` из
/// `OnServerMessage(0x5FA06)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMurderCounterUpdate {
    pub(crate) previous_kill_count: u32,
    pub(crate) kill_count: u32,
    pub(crate) previous_pk_count: u16,
    pub(crate) pk_count: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMurderCounterReset {
    pub(crate) previous_kill_count: u32,
    pub(crate) previous_pk_count: u16,
}

pub(crate) use nebokrai_realm::characters::playerexploit::PlayerExploitUpdate;

impl PlayerBaseProperty {
    fn read_u8(&self, offset: usize) -> u8 {
        self.wire[offset]
    }

    fn write_u8(&mut self, offset: usize, value: u8) {
        self.wire[offset] = value;
    }

    fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes(
            self.wire[offset..offset + 2]
                .try_into()
                .expect("PDB-offset находится внутри tagBaseProperty wire-prefix"),
        )
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        self.wire[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.wire[offset..offset + 4]
                .try_into()
                .expect("PDB-offset находится внутри tagBaseProperty wire-prefix"),
        )
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.wire[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

struct PlayerProperty {
    wire: [u8; 0x9C],
}

impl PlayerProperty {
    fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes(
            self.wire[offset..offset + 2]
                .try_into()
                .expect("PDB-offset находится внутри tagProperty"),
        )
    }

    fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.wire[offset..offset + 4]
                .try_into()
                .expect("PDB-offset находится внутри tagProperty"),
        )
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        self.wire[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.wire[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerDefaultPropertyReport {
    pub(crate) selected_region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) direction: i32,
    pub(crate) creation_property_key: u32,
    pub(crate) creation_property_inserted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerDefaultPropertyBlock {
    RandomPosition(RegionRandomPositionBlock),
    Property(PlayerCodecError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerOriginEquipmentOutcome {
    OccupationMismatch,
    GoodsFactoryMiss { goods_id: u32 },
    Added { goods_id: u32, position: u16 },
    Rejected { goods_id: u32, position: u16 },
}

#[derive(Debug)]
pub(crate) enum PlayerOriginEquipmentBlock {
    Guid(getrandom::Error),
    Equipment(EquipmentContainerCodecError),
}

#[derive(Clone, Copy)]
struct PlayerThing {
    thing_id: u16,
    count: u16,
    max_count: u16,
    point: u16,
}

// LeiTing-типы игрока перенесены в Realm activities вместе с `CLeiTing`;
// `PlayerCodecError` остаётся здесь: цепочка containers/goods/shape,
// которую он оборачивает, ещё не перенесена в Realm.
pub(crate) use nebokrai_realm::activities::leiting::{
    PlayerLeiTingClock, PlayerLeiTingUpdateBlock, PlayerLeiTingUpdateReport,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLoadedLeiTingResetReport {
    pub(crate) reset: bool,
    pub(crate) resulting_stamp: u32,
    pub(crate) daily_thing_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorEliminateResetReport {
    pub(crate) day_reset: bool,
    pub(crate) week_reset: bool,
    pub(crate) month_reset: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLoadedGoodsInsertBlock {
    Amount(AmountContainerCodecError),
    Equipment(EquipmentContainerCodecError),
    Volume(VolumeContainerCodecError),
}

impl From<AmountContainerCodecError> for PlayerLoadedGoodsInsertBlock {
    fn from(error: AmountContainerCodecError) -> Self {
        Self::Amount(error)
    }
}

impl From<EquipmentContainerCodecError> for PlayerLoadedGoodsInsertBlock {
    fn from(error: EquipmentContainerCodecError) -> Self {
        Self::Equipment(error)
    }
}

impl From<VolumeContainerCodecError> for PlayerLoadedGoodsInsertBlock {
    fn from(error: VolumeContainerCodecError) -> Self {
        Self::Volume(error)
    }
}

#[derive(Clone, Copy)]
struct PlayerSkill {
    skill_id: u16,
    level: u16,
}

struct PlayerFriend {
    name: Vec<u8>,
    online: bool,
}

#[derive(Clone, Copy)]
struct PlayerQuest {
    quest_id: u16,
    complete: u8,
}

struct PlayerPetInformation {
    original_name: Vec<u8>,
    hp: u32,
    level: u32,
    experience: u32,
}

struct PlayerCarriageInformation {
    original_name: Vec<u8>,
    hp: u32,
    carriage_script: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(crate) struct PlayerPropertyCoefficients {
    pub(crate) str_to_max_attack: [f32; 3],
    pub(crate) str_to_burden: [f32; 3],
    pub(crate) dex_to_min_attack: [f32; 3],
    pub(crate) dex_to_stiff: [f32; 3],
    pub(crate) con_to_max_hp: [f32; 3],
    pub(crate) con_to_defense: [f32; 3],
    pub(crate) int_to_element: [f32; 3],
    pub(crate) int_to_max_mp: [f32; 3],
    pub(crate) int_to_resistant: [f32; 3],
}

impl From<GlobePlayerPropertyCoefficients> for PlayerPropertyCoefficients {
    fn from(value: GlobePlayerPropertyCoefficients) -> Self {
        Self {
            str_to_max_attack: value.str_to_max_attack,
            str_to_burden: value.str_to_burden,
            dex_to_min_attack: value.dex_to_min_attack,
            dex_to_stiff: value.dex_to_stiff,
            con_to_max_hp: value.con_to_max_hp,
            con_to_defense: value.con_to_defense,
            int_to_element: value.int_to_element,
            int_to_max_mp: value.int_to_max_mp,
            int_to_resistant: value.int_to_resistant,
        }
    }
}

pub(crate) trait PlayerLoadDataOwner {
    type Block;

    async fn load_player(&mut self, player: &mut CPlayer) -> Result<bool, Self::Block>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlayerLoadDataReport {
    pub(crate) base_max_rp: u16,
    pub(crate) player_property_key: u32,
    pub(crate) player_property_inserted: bool,
    pub(crate) upgrade_applied: bool,
    pub(crate) rp_clamped: bool,
    pub(crate) graphics_id: i32,
    pub(crate) player_speed: f32,
}

#[derive(Debug)]
pub(crate) enum PlayerLoadDataOutcome<LoadBlock> {
    ReturnedFalse,
    Loaded(PlayerLoadDataReport),
    BlockedDatabase(LoadBlock),
    BlockedProperty(PlayerCodecError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlayerOwnedRegion {
    region_id: i32,
    region_type: u16,
}

impl PlayerOwnedRegion {
    fn wire_bytes(self) -> [u8; 8] {
        let mut wire = [0; 8];
        wire[..4].copy_from_slice(&self.region_id.to_le_bytes());
        wire[4..6].copy_from_slice(&self.region_type.to_le_bytes());
        wire
    }
}

pub(crate) struct PlayerOrganizingState {
    pub(crate) faction_id: i32,
    pub(crate) faction_logo_id: i32,
    pub(crate) faction_name: Vec<u8>,
    pub(crate) faction_title: Vec<u8>,
    pub(crate) faction_master_id: i32,
    pub(crate) union_id: i32,
    pub(crate) union_master_id: i32,
    pub(crate) faction_level: u16,
    pub(crate) faction_experience: i32,
    pub(crate) force: i32,
    pub(crate) faction_contribute: bool,
    pub(crate) enemy_factions: BTreeSet<i32>,
    pub(crate) city_war_enemy_factions: BTreeSet<i32>,
    owned_regions: VecDeque<PlayerOwnedRegion>,
}

impl PlayerOrganizingState {
    pub(crate) fn clear_owned_regions(&mut self) {
        self.owned_regions.clear();
    }

    pub(crate) fn add_owned_region(&mut self, region_id: i32, region_type: u16) {
        if self
            .owned_regions
            .iter()
            .any(|owned_region| owned_region.region_id == region_id)
        {
            return;
        }
        self.owned_regions.push_back(PlayerOwnedRegion {
            region_id,
            region_type,
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerOrganizingUpdateError {
    NullFactionDuringMembershipScan {
        map_key: i32,
    },
    NullConfederationDuringMembershipScan {
        map_key: i32,
    },
    UninitializedFactionField {
        faction_id: i32,
        field: &'static str,
    },
    UnterminatedFactionMemberTitle {
        faction_id: i32,
        player_id: i32,
    },
    UninitializedRegionType {
        region_id: i32,
    },
}

impl fmt::Display for PlayerOrganizingUpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullFactionDuringMembershipScan { map_key } => write!(
                formatter,
                "faction-map содержит null по ключу {map_key} во время IsFreePlayer"
            ),
            Self::NullConfederationDuringMembershipScan { map_key } => write!(
                formatter,
                "confederation-map содержит null по ключу {map_key} во время IsFreeFaction"
            ),
            Self::UninitializedFactionField { faction_id, field } => write!(
                formatter,
                "faction {faction_id} не материализовала поле {field} до SetPlayerOrganizing"
            ),
            Self::UnterminatedFactionMemberTitle {
                faction_id,
                player_id,
            } => write!(
                formatter,
                "title игрока {player_id} во faction {faction_id} не завершён NUL"
            ),
            Self::UninitializedRegionType { region_id } => write!(
                formatter,
                "регион {region_id} не материализовал REGION_TYPE до SetPlayerOrganizing"
            ),
        }
    }
}

impl Error for PlayerOrganizingUpdateError {}

pub(crate) trait PlayerOrganizingUpdater {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError>;
}

pub(crate) trait PlayerFactionInfoContext: PlayerOrganizingUpdater {
    fn send_player_faction_info(
        &mut self,
        player_id: i32,
        message: &CMessage,
    ) -> PlayerFactionInfoDelivery;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PlayerFactionInfoDelivery {
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PlayerFactionInfoUpdateReport {
    pub(crate) player_id: i32,
    pub(crate) faction_id_after_initial_update: i32,
    pub(crate) faction_data_reset: bool,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: PlayerFactionInfoDelivery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerFactionInfoUpdateBlock {
    InitialOrganizing(PlayerOrganizingUpdateError),
    Serialization(PlayerCodecError),
}

pub(crate) struct PlayerGoodsDbProjection {
    packet: Vec<TraversedGoods>,
    equipment: Vec<TraversedGoods>,
    hand: Vec<TraversedGoods>,
    wallet: Vec<TraversedGoods>,
    yuan_bao: Vec<TraversedGoods>,
    ji_fen: Vec<TraversedGoods>,
    bank: Vec<TraversedGoods>,
    depot: Vec<TraversedGoods>,
    fairy: Vec<TraversedGoods>,
    battle_fairy: Vec<TraversedGoods>,
    auction_goods: Vec<TraversedGoods>,
    auction_wallet: Vec<TraversedGoods>,
    auction: Vec<TraversedGoods>,
    ci_qing: Vec<TraversedGoods>,
    compose_ci_qing: Vec<TraversedGoods>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PlayerDbProjectionBlock {
    Goods(GoodsDbSnapshotBlock),
    UninitializedField { field: &'static str },
    NegativeVariableDataLength { value: i32 },
    VariableDataShorterThanDeclared { declared: usize, available: usize },
    FriendNameContainsNul { friend_index: usize, offset: usize },
}

/// Узкая owned-проекция полей, которые `CRsPlayer::OpenPlayerBase*`
/// публиковал в списке персонажей.
///
/// Это не DB-save snapshot: owner списка не читал остальные containers,
/// abilities или variable-data. Отдельная проекция не превращает их
/// инициализацию в ложное условие открытия списка.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerBaseWireSnapshot {
    pub(crate) id: i32,
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

/// Одиннадцать пар ID/уровень из `CGame::GetPlayerEquipID`.
///
/// Порядок является частью World wire/DB-контракта и намеренно не совпадает с
/// числовым порядком equipment slots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentWireSnapshot {
    pub(crate) ids: [u32; 11],
    pub(crate) levels: [u8; 11],
}

impl From<GoodsDbSnapshotBlock> for PlayerDbProjectionBlock {
    fn from(block: GoodsDbSnapshotBlock) -> Self {
        Self::Goods(block)
    }
}

pub(crate) struct PlayerDbProjection<'player> {
    player: &'player CPlayer,
    country: u8,
    contribute: i32,
    goods: PlayerGoodsDbProjection,
    hot_keys: [u32; 24],
    skills: Vec<PlayerAbilitySkill>,
    friend_names: Vec<PlayerFriendName<'player>>,
    things: Vec<DbPlayerThing>,
    quests: BTreeMap<u16, PlayerQuestSaveEntry>,
    variable_data: &'player [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerCountryChangeDisposition {
    SameCountry,
    FactionMember { faction_id: i32 },
    CountryMissing,
    Changed { previous_country: Option<u8> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerCountryChangeReport {
    pub(crate) requested_country: u8,
    pub(crate) legacy_result: i32,
    pub(crate) disposition: PlayerCountryChangeDisposition,
}

impl PlayerGoodsDbProjection {
    pub(crate) fn snapshot(&self, player_id: i32) -> PlayerGoodsFiledSnapshot<'_> {
        let traversal = |objects| GoodsContainerTraversalSnapshot { objects };
        PlayerGoodsFiledSnapshot {
            player_id,
            packet: traversal(&self.packet),
            equipment: traversal(&self.equipment),
            hand: traversal(&self.hand),
            wallet: traversal(&self.wallet),
            yuan_bao: traversal(&self.yuan_bao),
            ji_fen: traversal(&self.ji_fen),
            bank: traversal(&self.bank),
            depot: traversal(&self.depot),
            fairy: traversal(&self.fairy),
            battle_fairy: traversal(&self.battle_fairy),
            auction_goods: traversal(&self.auction_goods),
            auction_wallet: traversal(&self.auction_wallet),
            auction: traversal(&self.auction),
            ci_qing: traversal(&self.ci_qing),
            compose_ci_qing: traversal(&self.compose_ci_qing),
        }
    }
}

pub(crate) struct CPlayer {
    move_shape_base: CMoveShape,
    hand: CAmountLimitGoodsContainer,
    packet: CVolumeLimitGoodsContainer,
    equipment: CEquipmentContainer,
    wallet: CWallet,
    yuan_bao: CYuanBao,
    ji_fen: CJiFen,
    bank: CBank,
    depot: CDepot,
    fairy: CFairyContainer,
    battle_fairy: CBattleFairyContainer,
    auction_goods_container: CVolumeLimitGoodsContainer,
    auction_container: CVolumeLimitGoodsContainer,
    auction_wallet: CWallet,
    ci_qing: CVolumeLimitGoodsContainer,
    compose_ci_qing: CVolumeLimitGoodsContainer,
    depot_password: Vec<u8>,
    base_property: PlayerBaseProperty,
    property: PlayerProperty,
    team_id: i32,
    ci_qing_ids: BTreeSet<u32>,
    new_skills: VecDeque<PlayerSkill>,
    friends: VecDeque<PlayerFriend>,
    daily_things: VecDeque<PlayerThing>,
    player_quests: BTreeMap<u16, PlayerQuest>,
    variable_num: i32,
    variable_data: Option<Vec<u8>>,
    variable_data_length: i32,
    silience_time: i32,
    murderer_time: u32,
    fight_state_count: i32,
    uncreated_pets: Vec<PlayerPetInformation>,
    uncreated_carriage: PlayerCarriageInformation,
    recreate_carriage: bool,
    login: bool,
    country: Option<u8>,
    contribute: Option<i32>,
    jjc_data: [u8; 0x10],
    jjc_pk_state: bool,
    session_id: Vec<u8>,
    faction_runtime: RefCell<PlayerFactionRuntimeState>,
    faction_data_received: Cell<bool>,
}

struct PlayerFactionRuntimeState {
    organizing: PlayerOrganizingState,
    city_war_died_state_time: i32,
    create_union_operator: bool,
    faction_war_operator: bool,
}

impl PlayerDbProjection<'_> {
    fn equipment_fields(&self) -> Result<([u32; 11], [i32; 11]), PlayerDbProjectionBlock> {
        self.player.base_equipment_fields()
    }

    fn jjc_snapshot(&self) -> PlayerJjcDataSnapshot {
        let word = |offset: usize| {
            u16::from_le_bytes(
                self.player.jjc_data[offset..offset + 2]
                    .try_into()
                    .expect("tagPlayerJJcData содержит восемь полных u16"),
            )
        };
        PlayerJjcDataSnapshot {
            id: self.player.get_id(),
            jjc_level: self
                .player
                .base_property
                .read_u32(BASE_PROPERTY_JJC_LEVEL_OFFSET),
            jjc_score: self
                .player
                .base_property
                .read_u32(BASE_PROPERTY_JJC_SCORE_OFFSET),
            week_join: word(0),
            week_win: word(2),
            week_lose: word(4),
            week_tie: word(6),
            season_join: word(8),
            season_win: word(10),
            season_lose: word(12),
            season_tie: word(14),
        }
    }

    fn scalar_snapshot(&self) -> PlayerAbilityScalarSnapshot<'_> {
        let base = &self.player.base_property;
        let flag = |offset| base.read_u8(offset) != 0;
        PlayerAbilityScalarSnapshot {
            id: self.player.get_id(),
            name: self.player.get_name(),
            region_id: self.player.get_region_id(),
            pos_x: self.player.move_shape_base.get_pos_x(),
            pos_y: self.player.move_shape_base.get_pos_y(),
            dir: self.player.move_shape_base.get_direction(),
            account: &base.account,
            title: &base.title,
            level: base.read_u8(BASE_PROPERTY_LEVEL_OFFSET),
            exp: base.read_u32(BASE_PROPERTY_EXP_OFFSET),
            head_pic: base.read_u8(BASE_PROPERTY_HEAD_PIC_OFFSET),
            face_pic: base.read_u8(BASE_PROPERTY_FACE_PIC_OFFSET),
            occupation: base.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET),
            sex: base.read_u8(BASE_PROPERTY_SEX_OFFSET),
            spouse_id: base.read_u32(BASE_PROPERTY_SPOUSE_ID_OFFSET),
            union_id: base.read_u32(BASE_PROPERTY_UNION_ID_OFFSET),
            murderer_time: self.player.murderer_time,
            pk_count: base.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET),
            kill_count: base.read_u32(BASE_PROPERTY_KILL_COUNT_OFFSET),
            hit_top_log: base.read_u16(BASE_PROPERTY_HIT_TOP_LOG_OFFSET),
            hot_hit: base.read_u32(BASE_PROPERTY_HOT_HIT_OFFSET),
            loan_max: base.read_u32(BASE_PROPERTY_LOAN_MAX_OFFSET),
            loan: base.read_u32(BASE_PROPERTY_LOAN_OFFSET),
            loan_time: base.read_u32(BASE_PROPERTY_LOAN_TIME_OFFSET) as i32,
            remain_point: base.read_u16(BASE_PROPERTY_REMAIN_POINT_OFFSET),
            pk_normal: flag(BASE_PROPERTY_PK_NORMAL_OFFSET),
            pk_team: flag(BASE_PROPERTY_PK_TEAM_OFFSET),
            pk_union: flag(BASE_PROPERTY_PK_UNION_OFFSET),
            pk_badman: flag(BASE_PROPERTY_PK_BADMAN_OFFSET),
            pk_country: flag(BASE_PROPERTY_PK_COUNTRY_OFFSET),
            yp: base.read_u16(BASE_PROPERTY_YP_OFFSET),
            hp: base.read_u32(BASE_PROPERTY_HP_OFFSET),
            mp: base.read_u32(BASE_PROPERTY_MP_OFFSET),
            rp: base.read_u16(BASE_PROPERTY_RP_OFFSET),
            base_max_hp: base.read_u32(BASE_PROPERTY_MAX_HP_OFFSET),
            base_max_mp: base.read_u32(BASE_PROPERTY_MAX_MP_OFFSET),
            base_max_yp: base.read_u16(BASE_PROPERTY_MAX_YP_OFFSET),
            base_max_rp: base.read_u16(BASE_PROPERTY_MAX_RP_OFFSET),
            base_str: base.read_u32(BASE_PROPERTY_STR_OFFSET),
            base_dex: base.read_u32(BASE_PROPERTY_DEX_OFFSET),
            base_con: base.read_u32(BASE_PROPERTY_CON_OFFSET),
            base_int: base.read_u32(BASE_PROPERTY_INT_OFFSET),
            base_min_atk: base.read_u32(BASE_PROPERTY_MIN_ATK_OFFSET),
            base_max_atk: base.read_u32(BASE_PROPERTY_MAX_ATK_OFFSET),
            base_hit: base.read_u16(BASE_PROPERTY_HIT_OFFSET),
            base_burden: base.read_u16(BASE_PROPERTY_BURDEN_OFFSET),
            base_cch: base.read_u16(BASE_PROPERTY_CCH_OFFSET),
            base_def: base.read_u32(BASE_PROPERTY_DEF_OFFSET),
            base_dodge: base.read_u16(BASE_PROPERTY_DODGE_OFFSET),
            base_atc_speed: base.read_u16(BASE_PROPERTY_ATC_SPEED_OFFSET),
            base_element_resistant: base.read_u32(BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET),
            base_hp_recover_speed: base.read_u16(BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET),
            base_mp_recover_speed: base.read_u16(BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET),
            base_vigour: base.read_u32(BASE_PROPERTY_VIGOUR_OFFSET),
            base_max_vigour: base.read_u32(BASE_PROPERTY_MAX_VIGOUR_OFFSET),
            base_energy: base.read_u32(BASE_PROPERTY_ENERGY_OFFSET),
            base_max_energy: base.read_u32(BASE_PROPERTY_MAX_ENERGY_OFFSET),
            base_credit: base.read_u32(BASE_PROPERTY_CREDIT_OFFSET),
            display_head_piece: base.read_u8(BASE_PROPERTY_DISPLAY_HEAD_PIECE_OFFSET),
            country: self.country,
            contribute: self.contribute,
            is_charged: flag(BASE_PROPERTY_IS_CHARGED_OFFSET),
            quest_time_begin: base.read_u32(BASE_PROPERTY_QUEST_TIME_BEGIN_OFFSET) as i32,
            quest_time_limit: base.read_u32(BASE_PROPERTY_QUEST_TIME_LIMIT_OFFSET) as i32,
            quest: flag(BASE_PROPERTY_QUEST_OFFSET),
            depot_password: &self.player.depot_password,
            exploit: base.read_u32(BASE_PROPERTY_EXPLOIT_OFFSET),
            kudos: base.read_u32(BASE_PROPERTY_KUDOS_OFFSET),
            mode: base.read_u32(BASE_PROPERTY_MODE_OFFSET),
            fairy_enabled: flag(BASE_PROPERTY_FAIRY_ENABLED_OFFSET),
            foster_num: base.read_u32(BASE_PROPERTY_FOSTER_NUM_OFFSET),
            hatcher_num: base.read_u32(BASE_PROPERTY_HATCHER_NUM_OFFSET),
            battle_fairy_enabled: flag(BASE_PROPERTY_BATTLE_FAIRY_ENABLED_OFFSET),
            fetch_power: base.read_u32(BASE_PROPERTY_FETCH_POWER_OFFSET),
            max_fetch_power: base.read_u32(BASE_PROPERTY_MAX_FETCH_POWER_OFFSET),
            auction_space: base.read_u32(BASE_PROPERTY_AUCTION_SPACE_OFFSET),
            exalt: base.read_u32(BASE_PROPERTY_EXALT_OFFSET),
            szl: base.read_u32(BASE_PROPERTY_SZL_OFFSET),
            gods_battle_faction: base.read_u32(BASE_PROPERTY_GODS_BATTLE_FACTION_OFFSET) as i32,
            base_fy_energy: base.read_u32(BASE_PROPERTY_FY_ENERGY_OFFSET),
            base_bl_fy_energy: base.read_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET),
            lt_up_60_count: base.read_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET),
            remain_jl_dan_count: base.read_u16(BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET),
            lt_60_stamp: base.read_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET),
        }
    }

    fn ability_creation_snapshot(&self) -> PlayerAbilityCreationSnapshot<'_> {
        PlayerAbilityCreationSnapshot {
            scalar: self.scalar_snapshot(),
            hot_keys: &self.hot_keys,
            skills: &self.skills,
            script_flag: PlayerScriptFlagSnapshot {
                variable_num: self.player.variable_num,
                variable_data: self.variable_data,
            },
            ex_states: self.player.move_shape_base.ex_states(),
            friend_names: &self.friend_names,
            ci_qing_ids: &self.player.ci_qing_ids,
            things: &self.things,
            jjc: self.jjc_snapshot(),
        }
    }

    pub(crate) fn creation_snapshot(
        &self,
    ) -> Result<PlayerCreationSnapshot<'_, '_>, PlayerDbProjectionBlock> {
        let base = &self.player.base_property;
        let (equipment_ids, equipment_levels) = self.equipment_fields()?;
        Ok(PlayerCreationSnapshot {
            base: PlayerCreationBaseSnapshot {
                id: self.player.get_id(),
                name: self.player.get_name().to_vec(),
                account: base.account.clone(),
                level: base.read_u8(BASE_PROPERTY_LEVEL_OFFSET),
                occupation: base.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET),
                sex: base.read_u8(BASE_PROPERTY_SEX_OFFSET),
                country: self.country,
                head: base.read_u8(BASE_PROPERTY_HEAD_PIC_OFFSET),
                equipment_ids,
                equipment_levels: equipment_levels.map(|level| level as u8),
                region_id: self.player.get_region_id(),
            },
            abilities: self.ability_creation_snapshot(),
            goods: self.goods.snapshot(self.player.get_id()),
        })
    }

    pub(crate) fn save_snapshot(
        &self,
    ) -> Result<PlayerSaveSnapshot<'_, '_, '_>, PlayerDbProjectionBlock> {
        let base = &self.player.base_property;
        let (equipment_ids, equipment_levels) = self.equipment_fields()?;
        Ok(PlayerSaveSnapshot {
            base: PlayerBaseSaveSnapshot {
                id: self.player.get_id(),
                name: self.player.get_name(),
                level: base.read_u8(BASE_PROPERTY_LEVEL_OFFSET),
                occupation: base.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET),
                sex: base.read_u8(BASE_PROPERTY_SEX_OFFSET),
                country: self.country,
                head: base.read_u8(BASE_PROPERTY_HEAD_PIC_OFFSET),
                equipment_ids,
                equipment_levels,
                region_id: self.player.get_region_id(),
            },
            abilities: PlayerAbilitySaveSnapshot {
                ability: self.ability_creation_snapshot(),
                silence_time: self.player.silience_time,
                days_honor_eliminate_num: base.read_u32(BASE_PROPERTY_DAYS_HONOR_OFFSET),
                weeks_honor_eliminate_num: base.read_u32(BASE_PROPERTY_WEEKS_HONOR_OFFSET),
                months_honor_eliminate_num: base.read_u32(BASE_PROPERTY_MONTHS_HONOR_OFFSET),
                total_honor_eliminate_num: base.read_u32(BASE_PROPERTY_TOTAL_HONOR_OFFSET),
                rank_of_nobility_id: base.read_u32(BASE_PROPERTY_RANK_NOBILITY_OFFSET),
                appellation_id: base.read_u32(BASE_PROPERTY_APPELLATION_OFFSET),
            },
            quest: PlayerQuestSaveSnapshot {
                player_id: self.player.get_id(),
                quests: &self.quests,
            },
            goods: self.goods.snapshot(self.player.get_id()),
        })
    }
}

impl CPlayer {
    pub(crate) fn reset_goods_for_db_load(&mut self) {
        self.packet.clear();
        self.packet.set_container_volume_2d(8, 0x0C);
        self.auction_goods_container.clear();
        self.auction_goods_container.set_container_volume(0x12);
        self.auction_container.clear();
        self.auction_container.set_container_volume(2);
        self.equipment.clear();
        self.hand.clear();
        self.hand.set_goods_amount_limit(1);
        self.wallet.clear();
        self.auction_wallet.clear();
        self.yuan_bao.clear();
        self.ji_fen.clear();
        self.bank.clear();
        self.depot.clear();
        self.depot.set_container_volume(0xA1);
        self.fairy.clear();
        self.fairy.set_container_volume(0x0E);
        self.battle_fairy.clear();
        self.battle_fairy.set_container_volume(0x11);
        self.ci_qing.clear();
        self.ci_qing.set_container_volume(8);
        self.compose_ci_qing.clear();
        self.compose_ci_qing.set_container_volume(3);
    }

    pub(crate) fn loaded_goods_mut(&mut self, place: i32, position: u32) -> Option<&mut CGoods> {
        match place {
            1 => self.packet.get_goods_mut(position),
            2 => self.equipment.get_goods_mut(position),
            3 => self.hand.get_goods_mut(position),
            4 => self.wallet.get_goods_mut(position),
            5 => self.yuan_bao.get_goods_mut(position),
            6 => self.ji_fen.get_goods_mut(position),
            7 => self.bank.get_goods_mut(position),
            8 => self.depot.get_goods_mut(position),
            9 => self.fairy.get_goods_mut(position),
            10 => self.battle_fairy.get_goods_mut(position),
            11 => self.auction_goods_container.get_goods_mut(position),
            12 => self.auction_wallet.get_goods_mut(position),
            13 => self.auction_container.get_goods_mut(position),
            14 => self.ci_qing.get_goods_mut(position),
            15 => self.compose_ci_qing.get_goods_mut(position),
            _ => None,
        }
    }

    pub(crate) fn insert_loaded_goods(
        &mut self,
        place: i32,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, PlayerLoadedGoodsInsertBlock> {
        match place {
            1 => Ok(self.packet.add_from_db(position, goods, registry)?),
            2 => Ok(self.equipment.add_from_db(position, goods, registry)?),
            3 => Ok(self.hand.add_from_db(position, goods, registry)?),
            4 => Ok(self.wallet.add_from_db(position, goods)),
            5 => Ok(self.yuan_bao.add_from_db(position, goods)),
            6 => Ok(self.ji_fen.add_from_db(position, goods)),
            7 => Ok(self.bank.add_from_db(position, goods)),
            8 => Ok(self.depot.add_from_db(position, goods, registry)?),
            9 => Ok(self.fairy.add_from_db(position, goods, registry)?),
            10 => Ok(self.battle_fairy.add_from_db(position, goods, registry)?),
            11 => Ok(self
                .auction_goods_container
                .add_from_db(position, goods, registry)?),
            12 => Ok(self.auction_wallet.add_from_db(position, goods)),
            13 => Ok(self
                .auction_container
                .add_from_db(position, goods, registry)?),
            14 => Ok(self.ci_qing.add_from_db(position, goods, registry)?),
            15 => Ok(self
                .compose_ci_qing
                .add_from_db(position, goods, registry)?),
            _ => Ok(Some(goods)),
        }
    }

    fn base_equipment_fields(
        &self,
    ) -> Result<([u32; 11], [i32; 11]), PlayerDbProjectionBlock> {
 // `CGame::GetPlayerEquipID`,.
        const SQL_EQUIPMENT_ORDER: [u32; 11] = [0, 1, 3, 4, 2, 9, 10, 12, 13, 14, 15];
        let mut ids = [0; 11];
        let mut levels = [0; 11];
        for (index, position) in SQL_EQUIPMENT_ORDER.into_iter().enumerate() {
            let Some(goods) = self.equipment.get_goods(position) else {
                continue;
            };
            ids[index] = goods
                .get_base_properties_index()
                .ok_or(GoodsDbSnapshotBlock::MissingBasePropertiesIndex)?;
            levels[index] = goods.get_addon_property_value(GAP_WEAPON_LEVEL, 1);
        }
        Ok((ids, levels))
    }

    pub(crate) fn equipment_wire_snapshot(
        &self,
    ) -> Result<PlayerEquipmentWireSnapshot, PlayerDbProjectionBlock> {
        let (ids, levels) = self.base_equipment_fields()?;
        Ok(PlayerEquipmentWireSnapshot {
            ids,
            levels: levels.map(|level| level as u8),
        })
    }

    pub(crate) fn player_base_wire_snapshot(
        &self,
    ) -> Result<PlayerBaseWireSnapshot, PlayerDbProjectionBlock> {
        let country = self
            .country
            .ok_or(PlayerDbProjectionBlock::UninitializedField {
                field: "m_btCountry",
            })?;
        let equipment = self.equipment_wire_snapshot()?;
        Ok(PlayerBaseWireSnapshot {
            id: self.get_id(),
            name: self.get_name().to_vec(),
            level: self.base_property.read_u8(BASE_PROPERTY_LEVEL_OFFSET),
            occupation: self
                .base_property
                .read_u8(BASE_PROPERTY_OCCUPATION_OFFSET),
            sex: self.base_property.read_u8(BASE_PROPERTY_SEX_OFFSET),
            country,
            head: self.base_property.read_u8(BASE_PROPERTY_HEAD_PIC_OFFSET),
            equipment_ids: equipment.ids,
            equipment_levels: equipment.levels,
            region_id: self.get_region_id(),
        })
    }

 /// Создаёт constructor-state, необходимый точному `CloneMapPlayer` decoder-у.
 ///
 /// Это не объявление полного старого constructor-а: ещё не действующие поля
 /// остаются в оригинал-корпусе. `m_btCountry/m_lContribute` исходник не
 /// инициализировал, поэтому до обязательного полного decode они равны
 /// `None`, а не придуманному нулю.
    pub(crate) fn with_clone_decode_constructor_state() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(400);

        let mut hand = CAmountLimitGoodsContainer::with_constructor_defaults();
        let mut packet = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let equipment = CEquipmentContainer::with_constructor_defaults();
        let wallet = CWallet::with_constructor_defaults();
        let yuan_bao = CYuanBao::with_constructor_defaults();
        let ji_fen = CJiFen::with_constructor_defaults();
        let bank = CBank::with_constructor_defaults();
        let mut depot = CDepot::with_constructor_defaults();
        let mut fairy = CFairyContainer::with_constructor_defaults();
        let mut battle_fairy = CBattleFairyContainer::with_constructor_defaults();
        let mut auction_goods_container = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let mut auction_container = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let auction_wallet = CWallet::with_constructor_defaults();
        let mut ci_qing = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let mut compose_ci_qing = CVolumeLimitGoodsContainer::with_constructor_defaults();

        let mut base_property = PlayerBaseProperty {
            wire: [0; BASE_PROPERTY_WIRE_LEN],
            account: Vec::new(),
            title: Vec::new(),
        };
        base_property.write_u32(BASE_PROPERTY_FOSTER_NUM_OFFSET, 1);
        base_property.write_u32(BASE_PROPERTY_HATCHER_NUM_OFFSET, 1);

        hand.set_goods_amount_limit(1);
        packet.set_container_volume_2d(8, 0x0C);
        auction_goods_container.set_container_volume(0x12);
        auction_container.set_container_volume(2);
        depot.set_container_volume(0xA1);
        fairy.set_container_volume(0x0E);
        battle_fairy.set_container_volume(0x11);
        ci_qing.set_container_volume(8);
        compose_ci_qing.set_container_volume(3);

        Self {
            move_shape_base,
            hand,
            packet,
            equipment,
            wallet,
            yuan_bao,
            ji_fen,
            bank,
            depot,
            fairy,
            battle_fairy,
            auction_goods_container,
            auction_container,
            auction_wallet,
            ci_qing,
            compose_ci_qing,
            depot_password: Vec::new(),
            base_property,
            property: PlayerProperty { wire: [0; 0x9C] },
            team_id: 0,
            ci_qing_ids: BTreeSet::new(),
            new_skills: VecDeque::new(),
            friends: VecDeque::new(),
            daily_things: VecDeque::new(),
            player_quests: BTreeMap::new(),
            variable_num: 0,
            variable_data: None,
            variable_data_length: 0,
            silience_time: 0,
            murderer_time: 0,
            fight_state_count: 0,
            uncreated_pets: Vec::new(),
            uncreated_carriage: PlayerCarriageInformation {
                original_name: Vec::new(),
                hp: 0,
                carriage_script: Vec::new(),
            },
            recreate_carriage: false,
            login: false,
            country: None,
            contribute: None,
            jjc_data: [0; 0x10],
            jjc_pk_state: false,
            session_id: Vec::new(),
            faction_runtime: RefCell::new(PlayerFactionRuntimeState {
                organizing: PlayerOrganizingState {
                    faction_id: 0,
                    faction_logo_id: 0,
                    faction_name: Vec::new(),
                    faction_title: Vec::new(),
                    faction_master_id: 0,
                    union_id: 0,
                    union_master_id: 0,
                    faction_level: 0,
                    faction_experience: 0,
                    force: 0,
                    faction_contribute: false,
                    enemy_factions: BTreeSet::new(),
                    city_war_enemy_factions: BTreeSet::new(),
                    owned_regions: VecDeque::new(),
                },
                city_war_died_state_time: 0,
                create_union_operator: false,
                faction_war_operator: false,
            }),
            faction_data_received: Cell::new(false),
        }
    }

 /// Выполняет create-role `CPlayer::LoadDefaultProperty` с явными
 /// заменами process-global setup/game owners и platform callbacks.
    #[allow(
        clippy::too_many_arguments,
        reason = "исходный owner достигает region, country, player-list, globe и thing setup"
    )]
    pub(crate) fn load_default_property<'region, FindRegion, Random, WeekDay, Timestamp>(
        &mut self,
        sex: u8,
        occupation: u8,
        country: u8,
        country_parameters: &mut CCountryParam,
        duplicate_regions: &CDupliRegionSetup,
        player_list: &mut CPlayerList,
        globe_setup: &GlobeSetupSnapshot,
        thing_setup: &CThingSetup,
        coefficients: &PlayerPropertyCoefficients,
        mut find_region: FindRegion,
        random: &mut Random,
        mut get_week_day: WeekDay,
        mut get_timestamp: Timestamp,
    ) -> Result<PlayerDefaultPropertyReport, PlayerDefaultPropertyBlock>
    where
        FindRegion: FnMut(i32) -> Option<&'region CRegion>,
        Random: FnMut(i32) -> i32 + ?Sized,
        WeekDay: FnMut() -> u16,
        Timestamp: FnMut() -> u32,
    {
        let start_region_id = country_parameters.start_region_or_insert(country);
        let selected_region_id =
            duplicate_regions.get_random_region(start_region_id, &mut *random);
        let mut tile_x = -1;
        let mut tile_y = -1;
        let mut direction = 0;

        if let Some(region) = find_region(selected_region_id) {
            let CountryRect {
                left,
                top,
                right,
                bottom,
            } = country_parameters.start_rect_or_insert(country);
            let width = right.checked_sub(left).ok_or(
                PlayerDefaultPropertyBlock::RandomPosition(
                    RegionRandomPositionBlock::CoordinateOverflow {
                        operation: "start rect right - left",
                    },
                ),
            )?;
            let height = bottom.checked_sub(top).ok_or(
                PlayerDefaultPropertyBlock::RandomPosition(
                    RegionRandomPositionBlock::CoordinateOverflow {
                        operation: "start rect bottom - top",
                    },
                ),
            )?;
            let position = region
                .get_random_pos_in_range(left, top, width, height, &mut *random)
                .map_err(PlayerDefaultPropertyBlock::RandomPosition)?;
            tile_x = position.x;
            tile_y = position.y;
            direction = country_parameters.start_direction_or_insert(country);
        }

        self.set_region_id(selected_region_id);
        self.set_tile_xy(tile_x, tile_y);
        let _ = self.set_direction(direction);
        self.move_shape_base.set_graphics_id(
            i32::from(sex)
                .wrapping_add(i32::from(occupation).wrapping_mul(2))
                .wrapping_add(1),
        );

        let PlayerCreationPropertiesLookup {
            key: creation_property_key,
            inserted: creation_property_inserted,
            properties,
        } = player_list.creation_properties(sex, occupation);
        self.base_property
            .write_u8(BASE_PROPERTY_OCCUPATION_OFFSET, occupation);
        self.base_property.write_u8(BASE_PROPERTY_SEX_OFFSET, sex);
        self.base_property.write_u8(BASE_PROPERTY_LEVEL_OFFSET, 1);
        self.base_property
            .write_u8(BASE_PROPERTY_IS_CHARGED_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_EXP_OFFSET, 0);
        self.base_property.title = b"temptitle".to_vec();
        self.base_property.write_u8(BASE_PROPERTY_HEAD_PIC_OFFSET, 0);
        self.base_property.write_u8(BASE_PROPERTY_FACE_PIC_OFFSET, 0);
        self.base_property
            .write_u32(BASE_PROPERTY_HOT_HIT_OFFSET, properties.hot_hit);
        self.base_property.write_u32(BASE_PROPERTY_LOAN_TIME_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_LOAN_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_LOAN_MAX_OFFSET, 0);
        self.base_property
            .write_u16(BASE_PROPERTY_REMAIN_POINT_OFFSET, properties.remain_point);
        self.base_property.write_u32(BASE_PROPERTY_SPOUSE_ID_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_UNION_ID_OFFSET, 0);
        self.base_property.write_u16(BASE_PROPERTY_PK_COUNT_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_KILL_COUNT_OFFSET, 0);
        self.base_property.write_u16(BASE_PROPERTY_HIT_TOP_LOG_OFFSET, 0);
        self.base_property.write_u8(BASE_PROPERTY_PK_NORMAL_OFFSET, 0);
        self.base_property.write_u8(BASE_PROPERTY_PK_TEAM_OFFSET, 0);
        self.base_property.write_u8(BASE_PROPERTY_PK_UNION_OFFSET, 0);
        self.base_property.write_u8(BASE_PROPERTY_PK_BADMAN_OFFSET, 1);
        self.base_property.write_u8(BASE_PROPERTY_PK_COUNTRY_OFFSET, 1);
        self.base_property.write_u16(BASE_PROPERTY_YP_OFFSET, properties.yp);
        self.base_property.write_u32(BASE_PROPERTY_HP_OFFSET, properties.hp);
        self.base_property.write_u32(BASE_PROPERTY_MP_OFFSET, properties.mp);
        self.base_property.write_u16(BASE_PROPERTY_RP_OFFSET, properties.rp);
        self.base_property
            .write_u32(BASE_PROPERTY_MAX_HP_OFFSET, properties.base_maximum_hp);
        self.base_property
            .write_u32(BASE_PROPERTY_MAX_MP_OFFSET, properties.base_maximum_mp);
        self.base_property
            .write_u16(BASE_PROPERTY_MAX_YP_OFFSET, properties.base_maximum_yp);
        self.base_property
            .write_u16(BASE_PROPERTY_MAX_RP_OFFSET, properties.base_maximum_rp);
        self.base_property
            .write_u32(BASE_PROPERTY_STR_OFFSET, properties.base_strength);
        self.base_property
            .write_u32(BASE_PROPERTY_DEX_OFFSET, properties.base_dexterity);
        self.base_property
            .write_u32(BASE_PROPERTY_CON_OFFSET, properties.base_constitution);
        self.base_property
            .write_u32(BASE_PROPERTY_INT_OFFSET, properties.base_intelligence);
        self.base_property
            .write_u32(BASE_PROPERTY_MIN_ATK_OFFSET, properties.base_minimum_attack);
        self.base_property
            .write_u32(BASE_PROPERTY_MAX_ATK_OFFSET, properties.base_maximum_attack);
        self.base_property
            .write_u16(BASE_PROPERTY_HIT_OFFSET, properties.base_hit);
        self.base_property
            .write_u16(BASE_PROPERTY_BURDEN_OFFSET, properties.base_burden);
        self.base_property
            .write_u16(BASE_PROPERTY_CCH_OFFSET, properties.base_cch);
        self.base_property
            .write_u32(BASE_PROPERTY_DEF_OFFSET, properties.base_defence);
        self.base_property
            .write_u16(BASE_PROPERTY_DODGE_OFFSET, properties.base_dodge);
        self.base_property
            .write_u16(BASE_PROPERTY_ATC_SPEED_OFFSET, properties.base_attack_speed);
        self.base_property.write_u32(
            BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET,
            properties.base_element_resistant,
        );
        self.base_property.write_u16(
            BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET,
            properties.base_hp_recover_speed,
        );
        self.base_property.write_u16(
            BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET,
            properties.base_mp_recover_speed,
        );
        self.base_property.write_u32(BASE_PROPERTY_VIGOUR_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_MAX_VIGOUR_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_ENERGY_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_MAX_ENERGY_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_CREDIT_OFFSET, 0);
        self.base_property
            .write_u8(BASE_PROPERTY_DISPLAY_HEAD_PIECE_OFFSET, 1);
        self.base_property.write_u32(BASE_PROPERTY_MODE_OFFSET, 0);
        self.base_property
            .write_u8(BASE_PROPERTY_BATTLE_FAIRY_ENABLED_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_FETCH_POWER_OFFSET, 0);
        self.base_property
            .write_u32(BASE_PROPERTY_QUEST_TIME_BEGIN_OFFSET, 0);
        self.base_property
            .write_u32(BASE_PROPERTY_QUEST_TIME_LIMIT_OFFSET, 0);
        self.base_property.write_u8(BASE_PROPERTY_QUEST_OFFSET, 0);
        self.base_property
            .write_u32(BASE_PROPERTY_BREAK_ARMOUR_OFFSET, 0);
        self.base_property
            .write_u32(BASE_PROPERTY_BREAK_BOUND_OFFSET, 0);
        self.base_property
            .write_u32(BASE_PROPERTY_BREAK_ELEMENT_OFFSET, 0);
        self.base_property
            .write_u32(BASE_PROPERTY_POWER_OF_GOLD_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_JJC_LEVEL_OFFSET, 0);
        self.base_property.write_u32(BASE_PROPERTY_JJC_SCORE_OFFSET, 0);
        self.base_property.write_u16(
            BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET,
            globe_setup.total_jing_li_dan_count(),
        );
        self.base_property
            .write_u8(BASE_PROPERTY_IS_CHARGED_OFFSET, 0);
        self.country = Some(country);
        self.contribute = Some(0);

        self.update_property(coefficients)
            .map_err(PlayerDefaultPropertyBlock::Property)?;
        self.base_property.write_u32(
            BASE_PROPERTY_MP_OFFSET,
            self.property.read_u32(PROPERTY_MAX_MP_OFFSET),
        );
        self.base_property.write_u32(
            BASE_PROPERTY_HP_OFFSET,
            self.property.read_u32(PROPERTY_MAX_HP_OFFSET),
        );
        self.move_shape_base.set_speed(globe_setup.player_speed());

        let mut daily_things = VecDeque::new();
        thing_setup.get_daily_thing_list(&mut get_week_day, &mut daily_things);
        if !daily_things.is_empty() {
            self.daily_things = daily_things
                .into_iter()
                .map(
                    |LeiTingDailyThing {
                         thing_id,
                         count,
                         max_count,
                         point,
                     }| PlayerThing {
                        thing_id,
                        count,
                        max_count,
                        point,
                    },
                )
                .collect();
        }
        self.base_property
            .write_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET, get_timestamp());

        Ok(PlayerDefaultPropertyReport {
            selected_region_id,
            tile_x,
            tile_y,
            direction,
            creation_property_key,
            creation_property_inserted,
        })
    }

 /// Применяет одну ordered запись начальной экипировки. Factory probability
 /// и modifier roll-ы остаются на общем legacy `random(bound)` callback-е.
    pub(crate) fn add_origin_equipment<Random>(
        &mut self,
        origin: &PlayerOriginEquipment,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        random: &mut Random,
    ) -> Result<PlayerOriginEquipmentOutcome, PlayerOriginEquipmentBlock>
    where
        Random: FnMut(i32) -> i32 + ?Sized,
    {
        if self
            .base_property
            .read_u8(BASE_PROPERTY_OCCUPATION_OFFSET)
            != origin.occupation
        {
            return Ok(PlayerOriginEquipmentOutcome::OccupationMismatch);
        }

        let goods_id = query_goods_id_by_original_name_bytes(
            original_name_index,
            Some(&origin.original_name),
        );
        let Some(mut goods) = create_goods(registry, goods_id, random) else {
            return Ok(PlayerOriginEquipmentOutcome::GoodsFactoryMiss { goods_id });
        };
        let guid = CGuid::create().map_err(PlayerOriginEquipmentBlock::Guid)?;
        goods.set_ex_id(&guid);
        let rejected = self
            .equipment
            .add_at(u32::from(origin.place_position), goods, registry)
            .map_err(PlayerOriginEquipmentBlock::Equipment)?
            .is_some();
        if rejected {
            Ok(PlayerOriginEquipmentOutcome::Rejected {
                goods_id,
                position: origin.place_position,
            })
        } else {
            Ok(PlayerOriginEquipmentOutcome::Added {
                goods_id,
                position: origin.place_position,
            })
        }
    }

    pub(crate) fn set_creation_identity(
        &mut self,
        name: &[u8],
        account: &[u8],
        head_picture: u8,
        face_picture: u8,
    ) {
        self.move_shape_base.set_name(name);
        self.base_property.write_u8(BASE_PROPERTY_HEAD_PIC_OFFSET, head_picture);
        self.base_property.write_u8(BASE_PROPERTY_FACE_PIC_OFFSET, face_picture);
        self.base_property.account.clear();
        self.base_property.account.extend_from_slice(account);
    }

    pub(crate) fn set_creation_service_defaults(&mut self) {
        self.base_property
            .write_u32(BASE_PROPERTY_AUCTION_SPACE_OFFSET, 5);
        self.base_property
            .write_u32(BASE_PROPERTY_JJC_LEVEL_OFFSET, 1000);
    }

    pub(crate) fn set_database_load_identity(&mut self, player_id: i32, account: &[u8]) {
        self.set_id(player_id);
        self.base_property.account.clear();
        self.base_property.account.extend_from_slice(account);
    }

    pub(crate) fn mark_largess_charged(&mut self) {
        if self.base_property.read_u8(BASE_PROPERTY_IS_CHARGED_OFFSET) == 0 {
            self.base_property
                .write_u8(BASE_PROPERTY_IS_CHARGED_OFFSET, 1);
        }
    }

    pub(crate) fn add_largess_gold_coin(
        &mut self,
        goods: Box<CGoods>,
        gold_coin_limit: u32,
    ) -> Result<bool, PlayerCodecError> {
        Ok(self
            .bank
            .add_gold_coin_of_largess(0, goods, gold_coin_limit)?
            .is_none())
    }

    pub(crate) const fn largess_depot_limit(&self) -> u32 {
        self.depot.get_goods_amount_limit()
    }

    pub(crate) fn add_largess_to_depot(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, PlayerCodecError> {
        self.depot.add_at(position, goods, registry).map_err(Into::into)
    }

    pub(crate) const fn money(&self) -> u32 {
        self.wallet.get_gold_coins_amount()
    }

    pub(crate) fn check_goods_in_packet(
        &self,
        original_name: Option<&CStr>,
        original_name_index: &GoodsOriginalNameIndex,
    ) -> i32 {
        let Some(original_name) = original_name else {
            return 0;
        };

        let mut listener = CSeekGoodsListener::new();
        listener.set_target(Some(original_name), original_name_index);
        self.packet.traversing_container(Some(&mut listener));

        listener.goods_ids().iter().fold(0i32, |amount, ex_id| {
            self.packet.find(ex_id).map_or(amount, |goods| {
                amount.wrapping_add(goods.get_amount() as i32)
            })
        })
    }

    pub(crate) fn get_account(&self) -> &[u8] {
        &self.base_property.account
    }

    pub(crate) fn increment_murder_counters(&mut self) -> PlayerMurderCounterUpdate {
        let previous_kill_count = self
            .base_property
            .read_u32(BASE_PROPERTY_KILL_COUNT_OFFSET);
        let kill_count = previous_kill_count.wrapping_add(1);
        self.base_property
            .write_u32(BASE_PROPERTY_KILL_COUNT_OFFSET, kill_count);

        let previous_pk_count = self.base_property.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET);
        let pk_count = previous_pk_count.wrapping_add(1);
        self.base_property
            .write_u16(BASE_PROPERTY_PK_COUNT_OFFSET, pk_count);

        PlayerMurderCounterUpdate {
            previous_kill_count,
            kill_count,
            previous_pk_count,
            pk_count,
        }
    }

    pub(crate) fn reset_murder_counters(&mut self) -> PlayerMurderCounterReset {
        let previous_pk_count = self.base_property.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET);
        self.base_property.write_u16(BASE_PROPERTY_PK_COUNT_OFFSET, 0);
        let previous_kill_count = self.base_property.read_u32(BASE_PROPERTY_KILL_COUNT_OFFSET);
        self.base_property.write_u32(BASE_PROPERTY_KILL_COUNT_OFFSET, 0);
        PlayerMurderCounterReset { previous_kill_count, previous_pk_count }
    }

    pub(crate) fn pk_count(&self) -> u16 {
        self.base_property.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET)
    }

    pub(crate) const fn is_god(&self) -> bool {
        self.move_shape_base.is_god()
    }

    pub(crate) fn add_exploit_wrapping(&mut self, increment: i32) -> PlayerExploitUpdate {
        let previous_exploit = self.base_property.read_u32(BASE_PROPERTY_EXPLOIT_OFFSET);
        let exploit = previous_exploit.wrapping_add(increment as u32);
        self.base_property
            .write_u32(BASE_PROPERTY_EXPLOIT_OFFSET, exploit);
        PlayerExploitUpdate {
            previous_exploit,
            increment,
            exploit,
        }
    }

    pub(crate) fn faction_data_received(&self) -> bool {
        self.faction_data_received.get()
    }

    pub(crate) fn set_faction_data_received(&self, received: bool) {
        self.faction_data_received.set(received);
    }

    pub(crate) fn clear_uncreated_pets(&mut self) {
        self.uncreated_pets.clear();
    }

    pub(crate) fn faction_war_operator(&self) -> bool {
        self.faction_runtime.borrow().faction_war_operator
    }

    pub(crate) fn set_faction_war_operator(&self, enabled: bool) {
        self.faction_runtime.borrow_mut().faction_war_operator = enabled;
    }

    pub(crate) const fn get_id(&self) -> i32 {
        self.move_shape_base.get_id()
    }

    pub(crate) fn get_net_exchange_id(&self) -> i32 {
        NEXT_NET_EXCHANGE_ID
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }

    pub(crate) const fn get_type(&self) -> i32 {
        self.move_shape_base.get_type()
    }

    pub(crate) const fn set_id(&mut self, id: i32) {
        self.move_shape_base.set_id(id);
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        self.move_shape_base.get_name()
    }

 /// Присваивает унаследованное byte- имя без global/DB проверок.
 ///
 /// Это прямой `CBaseObject::SetName`, нужный factory lifecycle; публичная
 /// игровая смена имени по-прежнему должна проходить `set_validated_name`.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.move_shape_base.set_name(name);
    }

    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.move_shape_base.set_graphics_id(graphics_id);
    }

    pub(crate) fn set_validated_name(&mut self, name: &[u8]) {
        self.move_shape_base.set_name(name);
    }

    pub(crate) fn set_fairy_container_enabled(&mut self, enabled: bool) {
        self.base_property
            .write_u8(BASE_PROPERTY_FAIRY_ENABLED_OFFSET, u8::from(enabled));
    }

    pub(crate) fn set_foster_num(&mut self, value: u32) {
        self.base_property
            .write_u32(BASE_PROPERTY_FOSTER_NUM_OFFSET, value.min(5));
    }

    pub(crate) fn set_hatcher_num(&mut self, value: u32) {
        self.base_property
            .write_u32(BASE_PROPERTY_HATCHER_NUM_OFFSET, value.min(5));
    }

    pub(crate) fn apply_loaded_ability_scalars(
        &mut self,
        loaded: &PlayerAbilityLoadScalarSnapshot<'_>,
    ) {
        let scalar = &loaded.ability;
        self.move_shape_base.set_name(scalar.name);
        self.move_shape_base.set_region_id(scalar.region_id);
        self.move_shape_base.set_pos_xy(scalar.pos_x, scalar.pos_y);
        let _ = self.move_shape_base.set_direction(scalar.dir);
        self.base_property.title = scalar.title.to_vec();
        self.depot_password = scalar.depot_password.to_vec();

        let base = &mut self.base_property;
        base.write_u8(BASE_PROPERTY_LEVEL_OFFSET, scalar.level);
        base.write_u32(BASE_PROPERTY_EXP_OFFSET, scalar.exp);
        base.write_u8(BASE_PROPERTY_HEAD_PIC_OFFSET, scalar.head_pic);
        base.write_u8(BASE_PROPERTY_FACE_PIC_OFFSET, scalar.face_pic);
        base.write_u8(BASE_PROPERTY_OCCUPATION_OFFSET, scalar.occupation);
        base.write_u8(BASE_PROPERTY_SEX_OFFSET, scalar.sex);
        base.write_u32(BASE_PROPERTY_SPOUSE_ID_OFFSET, scalar.spouse_id);
        base.write_u32(BASE_PROPERTY_UNION_ID_OFFSET, scalar.union_id);
        base.write_u16(BASE_PROPERTY_PK_COUNT_OFFSET, scalar.pk_count);
        base.write_u32(BASE_PROPERTY_KILL_COUNT_OFFSET, scalar.kill_count);
        base.write_u16(BASE_PROPERTY_HIT_TOP_LOG_OFFSET, scalar.hit_top_log);
        base.write_u32(BASE_PROPERTY_HOT_HIT_OFFSET, scalar.hot_hit);
        base.write_u32(BASE_PROPERTY_LOAN_MAX_OFFSET, scalar.loan_max);
        base.write_u32(BASE_PROPERTY_LOAN_OFFSET, scalar.loan);
        base.write_u32(BASE_PROPERTY_LOAN_TIME_OFFSET, scalar.loan_time as u32);
        base.write_u16(BASE_PROPERTY_REMAIN_POINT_OFFSET, scalar.remain_point);
        base.write_u8(BASE_PROPERTY_PK_NORMAL_OFFSET, u8::from(scalar.pk_normal));
        base.write_u8(BASE_PROPERTY_PK_TEAM_OFFSET, u8::from(scalar.pk_team));
        base.write_u8(BASE_PROPERTY_PK_UNION_OFFSET, u8::from(scalar.pk_union));
        base.write_u8(BASE_PROPERTY_PK_BADMAN_OFFSET, u8::from(scalar.pk_badman));
        base.write_u8(BASE_PROPERTY_PK_COUNTRY_OFFSET, u8::from(scalar.pk_country));
        base.write_u16(BASE_PROPERTY_YP_OFFSET, scalar.yp);
        base.write_u32(BASE_PROPERTY_HP_OFFSET, scalar.hp);
        base.write_u32(BASE_PROPERTY_MP_OFFSET, scalar.mp);
        base.write_u16(BASE_PROPERTY_RP_OFFSET, scalar.rp);
        base.write_u32(BASE_PROPERTY_MAX_HP_OFFSET, scalar.base_max_hp);
        base.write_u32(BASE_PROPERTY_MAX_MP_OFFSET, scalar.base_max_mp);
        base.write_u16(BASE_PROPERTY_MAX_YP_OFFSET, scalar.base_max_yp);
 // `LoadPlayer` не читает BaseMaxRp: его сразу вычисляет `LoadData`.
        base.write_u32(BASE_PROPERTY_STR_OFFSET, scalar.base_str);
        base.write_u32(BASE_PROPERTY_DEX_OFFSET, scalar.base_dex);
        base.write_u32(BASE_PROPERTY_CON_OFFSET, scalar.base_con);
        base.write_u32(BASE_PROPERTY_INT_OFFSET, scalar.base_int);
        base.write_u32(BASE_PROPERTY_MIN_ATK_OFFSET, scalar.base_min_atk);
        base.write_u32(BASE_PROPERTY_MAX_ATK_OFFSET, scalar.base_max_atk);
        base.write_u16(BASE_PROPERTY_HIT_OFFSET, scalar.base_hit);
        base.write_u16(BASE_PROPERTY_BURDEN_OFFSET, scalar.base_burden);
        base.write_u16(BASE_PROPERTY_CCH_OFFSET, scalar.base_cch);
        base.write_u32(BASE_PROPERTY_DEF_OFFSET, scalar.base_def);
        base.write_u16(BASE_PROPERTY_DODGE_OFFSET, scalar.base_dodge);
        base.write_u16(BASE_PROPERTY_ATC_SPEED_OFFSET, scalar.base_atc_speed);
        base.write_u32(
            BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET,
            scalar.base_element_resistant,
        );
        base.write_u16(
            BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET,
            scalar.base_hp_recover_speed,
        );
        base.write_u16(
            BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET,
            scalar.base_mp_recover_speed,
        );
        base.write_u32(BASE_PROPERTY_VIGOUR_OFFSET, scalar.base_vigour);
        base.write_u32(BASE_PROPERTY_MAX_VIGOUR_OFFSET, scalar.base_max_vigour);
        base.write_u32(BASE_PROPERTY_ENERGY_OFFSET, scalar.base_energy);
        base.write_u32(BASE_PROPERTY_MAX_ENERGY_OFFSET, scalar.base_max_energy);
        base.write_u32(BASE_PROPERTY_CREDIT_OFFSET, scalar.base_credit);
        base.write_u8(
            BASE_PROPERTY_DISPLAY_HEAD_PIECE_OFFSET,
            scalar.display_head_piece,
        );
        base.write_u8(BASE_PROPERTY_IS_CHARGED_OFFSET, u8::from(scalar.is_charged));
        base.write_u32(
            BASE_PROPERTY_QUEST_TIME_BEGIN_OFFSET,
            scalar.quest_time_begin as u32,
        );
        base.write_u32(
            BASE_PROPERTY_QUEST_TIME_LIMIT_OFFSET,
            scalar.quest_time_limit as u32,
        );
        base.write_u8(BASE_PROPERTY_QUEST_OFFSET, u8::from(scalar.quest));
        base.write_u32(BASE_PROPERTY_EXPLOIT_OFFSET, scalar.exploit);
        base.write_u32(BASE_PROPERTY_KUDOS_OFFSET, scalar.kudos);
        base.write_u32(BASE_PROPERTY_MODE_OFFSET, scalar.mode);
        base.write_u8(
            BASE_PROPERTY_FAIRY_ENABLED_OFFSET,
            u8::from(scalar.fairy_enabled),
        );
        base.write_u8(
            BASE_PROPERTY_BATTLE_FAIRY_ENABLED_OFFSET,
            u8::from(scalar.battle_fairy_enabled),
        );
        base.write_u32(BASE_PROPERTY_FETCH_POWER_OFFSET, scalar.fetch_power);
        base.write_u32(
            BASE_PROPERTY_MAX_FETCH_POWER_OFFSET,
            scalar.max_fetch_power,
        );
        base.write_u32(BASE_PROPERTY_AUCTION_SPACE_OFFSET, scalar.auction_space);
        base.write_u32(BASE_PROPERTY_EXALT_OFFSET, scalar.exalt);
        base.write_u32(BASE_PROPERTY_SZL_OFFSET, scalar.szl);
        base.write_u32(
            BASE_PROPERTY_GODS_BATTLE_FACTION_OFFSET,
            scalar.gods_battle_faction as u32,
        );
        base.write_u32(BASE_PROPERTY_FY_ENERGY_OFFSET, scalar.base_fy_energy);
        base.write_u32(
            BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET,
            scalar.base_bl_fy_energy,
        );
        base.write_u16(
            BASE_PROPERTY_LT_UP_60_COUNT_OFFSET,
            scalar.lt_up_60_count,
        );
        base.write_u16(
            BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET,
            scalar.remain_jl_dan_count,
        );
        base.write_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET, scalar.lt_60_stamp);
        base.write_u32(
            BASE_PROPERTY_DAYS_HONOR_OFFSET,
            loaded.days_honor_eliminate_num,
        );
        base.write_u32(
            BASE_PROPERTY_WEEKS_HONOR_OFFSET,
            loaded.weeks_honor_eliminate_num,
        );
        base.write_u32(
            BASE_PROPERTY_MONTHS_HONOR_OFFSET,
            loaded.months_honor_eliminate_num,
        );
        base.write_u32(
            BASE_PROPERTY_TOTAL_HONOR_OFFSET,
            loaded.total_honor_eliminate_num,
        );
        base.write_u32(
            BASE_PROPERTY_RANK_NOBILITY_OFFSET,
            loaded.rank_of_nobility_id,
        );
        base.write_u32(BASE_PROPERTY_APPELLATION_OFFSET, loaded.appellation_id);

        self.set_foster_num(scalar.foster_num);
        self.set_hatcher_num(scalar.hatcher_num);
        self.murderer_time = scalar.murderer_time;
        self.silience_time = loaded.silence_time;
        self.country = Some(scalar.country);
        self.contribute = Some(scalar.contribute);
    }

    pub(crate) fn apply_loaded_hot_keys(&mut self, hot_keys: &[u32; 24]) {
        for (index, hot_key) in hot_keys.iter().copied().enumerate() {
            self.base_property
                .write_u32(BASE_PROPERTY_HOT_KEYS_OFFSET + index * 4, hot_key);
        }
    }

    pub(crate) fn append_loaded_skills(&mut self, skills: &[PlayerAbilitySkill]) {
        self.new_skills
            .extend(skills.iter().map(|skill| PlayerSkill {
                skill_id: skill.id,
                level: skill.level,
            }));
    }

    pub(crate) fn apply_loaded_script_flag(&mut self, script: LoadedPlayerScriptFlag) {
        self.variable_num = script.variable_num;
        self.variable_data_length = i32::try_from(script.variable_data.len())
            .expect("DB-owner уже ограничил VariableList signed long");
        self.variable_data = Some(script.variable_data);
    }

    pub(crate) fn apply_loaded_ex_states(&mut self, ex_states: Vec<u8>) {
        debug_assert!(!ex_states.is_empty());
        self.move_shape_base.set_ex_states(&ex_states);
    }

    pub(crate) fn append_loaded_friends(&mut self, names: Vec<Vec<u8>>) {
        self.friends.extend(names.into_iter().map(|name| PlayerFriend {
            name,
            online: false,
        }));
    }

    pub(crate) fn extend_loaded_ci_qing(&mut self, ids: BTreeSet<u32>) {
        self.ci_qing_ids.extend(ids);
    }

    pub(crate) fn apply_loaded_things(
        &mut self,
        field_was_empty: bool,
        things: Vec<DbPlayerThing>,
        fy_energy: u32,
        thing_setup: &CThingSetup,
        mut get_week_day: impl FnMut() -> u16,
    ) {
        if !field_was_empty {
            self.daily_things = things
                .into_iter()
                .map(|thing| PlayerThing {
                    thing_id: thing.tid,
                    count: thing.count,
                    max_count: thing.max_count,
                    point: thing.point,
                })
                .collect();
            return;
        }
        if fy_energy != 0 {
            return;
        }

        let mut daily = self
            .daily_things
            .iter()
            .map(|thing| LeiTingDailyThing {
                thing_id: thing.thing_id,
                count: thing.count,
                max_count: thing.max_count,
                point: thing.point,
            })
            .collect();
        thing_setup.get_daily_thing_list(&mut get_week_day, &mut daily);
        self.daily_things = daily
            .into_iter()
            .map(|thing| PlayerThing {
                thing_id: thing.thing_id,
                count: thing.count,
                max_count: thing.max_count,
                point: thing.point,
            })
            .collect();
    }

 /// Вставляет/заменяет DB quest по unsigned key, как `map::operator[]`
 /// с последующей полной записью трёх значимых байт `tagPlayerQuest`.
    pub(crate) fn add_quest_from_db(&mut self, quest_id: u16, complete: u8) {
        self.player_quests.insert(
            quest_id,
            PlayerQuest {
                quest_id,
                complete,
            },
        );
    }

    pub(crate) fn replace_silience_time(&mut self, silience_time: i32) -> i32 {
        std::mem::replace(&mut self.silience_time, silience_time)
    }

    pub(crate) const fn get_region_id(&self) -> i32 {
        self.move_shape_base.get_region_id()
    }

    pub(crate) const fn get_pos_x(&self) -> f32 {
        self.move_shape_base.get_pos_x()
    }

    pub(crate) const fn set_pos_x(&mut self, pos_x: f32) {
        self.move_shape_base.set_pos_x(pos_x);
    }

    pub(crate) const fn get_pos_y(&self) -> f32 {
        self.move_shape_base.get_pos_y()
    }

    pub(crate) const fn set_pos_y(&mut self, pos_y: f32) {
        self.move_shape_base.set_pos_y(pos_y);
    }

    pub(crate) fn get_tile_x(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.move_shape_base.get_tile_x()
    }

    pub(crate) fn get_tile_y(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.move_shape_base.get_tile_y()
    }

    pub(crate) const fn set_pos_xy(&mut self, pos_x: f32, pos_y: f32) {
        self.move_shape_base.set_pos_xy(pos_x, pos_y);
    }

    pub(crate) const fn set_position(&mut self, position: i32) {
        self.move_shape_base.set_position(position);
    }

    pub(crate) const fn get_speed(&self) -> f32 {
        self.move_shape_base.get_speed()
    }

    pub(crate) const fn set_direction(&mut self, direction: i32) -> bool {
        self.move_shape_base.set_direction(direction)
    }

    pub(crate) const fn get_state(&self) -> u16 {
        self.move_shape_base.get_state()
    }

    pub(crate) const fn get_action(&self) -> u16 {
        self.move_shape_base.get_action()
    }

    pub(crate) const fn set_action(&mut self, action: u16) {
        self.move_shape_base.set_action(action);
    }

    pub(crate) fn set_tile_xy(&mut self, tile_x: i32, tile_y: i32) {
        self.move_shape_base.set_tile_xy(tile_x, tile_y);
    }

    pub(crate) const fn country(&self) -> Option<u8> {
        self.country
    }

    pub(crate) fn change_country(
        &mut self,
        requested_country: u8,
        country_exists: impl FnOnce(u8) -> bool,
    ) -> PlayerCountryChangeReport {
        let (legacy_result, disposition) = if self.country == Some(requested_country) {
            (-1, PlayerCountryChangeDisposition::SameCountry)
        } else if self.faction_runtime.borrow().organizing.faction_id != 0 {
            let faction_id = self.faction_runtime.borrow().organizing.faction_id;
            (
                -3,
                PlayerCountryChangeDisposition::FactionMember {
                    faction_id,
                },
            )
        } else if !country_exists(requested_country) {
            (-5, PlayerCountryChangeDisposition::CountryMissing)
        } else {
            let previous_country = self.country.replace(requested_country);
            (
                i32::from(requested_country),
                PlayerCountryChangeDisposition::Changed { previous_country },
            )
        };
        PlayerCountryChangeReport {
            requested_country,
            legacy_result,
            disposition,
        }
    }

    pub(crate) fn faction_id(&self) -> i32 {
        self.faction_runtime.borrow().organizing.faction_id
    }

    pub(crate) const fn get_team_id(&self) -> i32 {
        self.team_id
    }

    pub(crate) fn get_level(&self) -> u8 {
        self.base_property.read_u8(BASE_PROPERTY_LEVEL_OFFSET)
    }

    pub(crate) fn get_jjc_level(&self) -> u32 {
        self.base_property.read_u32(BASE_PROPERTY_JJC_LEVEL_OFFSET)
    }

    pub(crate) fn set_jjc_identity(&mut self, level: u8, jjc_level: u32) {
        self.base_property.write_u8(BASE_PROPERTY_LEVEL_OFFSET, level);
        self.base_property
            .write_u32(BASE_PROPERTY_JJC_LEVEL_OFFSET, jjc_level);
    }

    pub(crate) fn set_jjc_snapshot(
        &mut self,
        level: u8,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u8; 0x10],
    ) {
        self.set_jjc_identity(level, jjc_level);
        self.base_property
            .write_u32(BASE_PROPERTY_JJC_SCORE_OFFSET, jjc_score);
        self.jjc_data = counters;
    }

    pub(crate) fn apply_loaded_jjc_data(
        &mut self,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u16; 8],
    ) {
        self.base_property
            .write_u32(BASE_PROPERTY_JJC_LEVEL_OFFSET, jjc_level);
        self.base_property
            .write_u32(BASE_PROPERTY_JJC_SCORE_OFFSET, jjc_score);
        for (destination, counter) in self.jjc_data.chunks_exact_mut(2).zip(counters) {
            destination.copy_from_slice(&counter.to_le_bytes());
        }
    }

    pub(crate) fn credit(&self) -> u32 {
        self.base_property.read_u32(BASE_PROPERTY_CREDIT_OFFSET)
    }

    pub(crate) fn get_occupation(&self) -> u8 {
        self.base_property
            .read_u8(BASE_PROPERTY_OCCUPATION_OFFSET)
    }

    pub(crate) fn get_appellation_id(&self) -> u32 {
        self.base_property
            .read_u32(BASE_PROPERTY_APPELLATION_OFFSET)
    }

    pub(crate) fn reset_loaded_lei_ting_if_needed(
        &mut self,
        local_day: u16,
        get_reset_timestamp: impl FnOnce() -> u32,
        thing_setup: &CThingSetup,
        get_week_day: impl FnMut() -> u16,
    ) -> PlayerLoadedLeiTingResetReport {
        let up_60_count = self
            .base_property
            .read_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET);
        let reset = local_day == 1 && up_60_count > 1;
        if reset {
            let reset_timestamp = get_reset_timestamp();
            self.base_property
                .write_u32(BASE_PROPERTY_FY_ENERGY_OFFSET, 0);
            self.base_property
                .write_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET, 0);
            self.base_property
                .write_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET, 0);
            self.base_property
                .write_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET, reset_timestamp);
            self.apply_loaded_things(true, Vec::new(), 0, thing_setup, get_week_day);
        }
        PlayerLoadedLeiTingResetReport {
            reset,
            resulting_stamp: self
                .base_property
                .read_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET),
            daily_thing_count: self.daily_things.len(),
        }
    }

    pub(crate) fn reset_honor_eliminate_num(
        &mut self,
        mut save_time: TagTime,
        mut current_time: TagTime,
    ) -> Result<PlayerHonorEliminateResetReport, TagTimeArithmeticBlock> {
        current_time.hour = 0;
        current_time.minute = 0;
        current_time.second = 0;
        current_time.milliseconds = 0;

        let day_reset = save_time.legacy_lt(current_time);
        if day_reset {
            self.base_property
                .write_u32(BASE_PROPERTY_DAYS_HONOR_OFFSET, 0);
        }

        let mut week_reset = false;
        let mut month_reset = false;
        save_time.add_day(1)?;
        save_time.day_of_week = save_time.day_of_week.wrapping_add(1);
        if save_time.day_of_week == 7 {
            save_time.day_of_week = 0;
        }
        while save_time.legacy_le(current_time) {
            if !week_reset && save_time.day_of_week == 1 {
                self.base_property
                    .write_u32(BASE_PROPERTY_WEEKS_HONOR_OFFSET, 0);
                week_reset = true;
            }
            if !month_reset && save_time.day == 1 {
                self.base_property
                    .write_u32(BASE_PROPERTY_MONTHS_HONOR_OFFSET, 0);
                month_reset = true;
            }
            if week_reset && month_reset {
                break;
            }
            save_time.add_day(1)?;
            save_time.day_of_week = save_time.day_of_week.wrapping_add(1);
            if save_time.day_of_week == 7 {
                save_time.day_of_week = 0;
            }
        }

        Ok(PlayerHonorEliminateResetReport {
            day_reset,
            week_reset,
            month_reset,
        })
    }

    pub(crate) fn reset_honor_eliminate_info(&mut self, rank_mask: u32) {
        if rank_mask & 0x02 != 0 {
            self.base_property
                .write_u32(BASE_PROPERTY_WEEKS_HONOR_OFFSET, 0);
        }
        if rank_mask & 0x04 != 0 {
            self.base_property
                .write_u32(BASE_PROPERTY_MONTHS_HONOR_OFFSET, 0);
        }
        self.base_property
            .write_u32(BASE_PROPERTY_DAYS_HONOR_OFFSET, 0);
    }

    pub(crate) fn friend_count(&self) -> usize {
        self.friends.len()
    }

    pub(crate) fn friend_name(&self, index: usize) -> Option<&[u8]> {
        self.friends.get(index).map(|friend| friend.name.as_slice())
    }

    pub(crate) fn set_friend_online(&mut self, index: usize, online: bool) -> bool {
        let Some(friend) = self.friends.get_mut(index) else {
            return false;
        };
        friend.online = online;
        true
    }

    pub(crate) fn reset_selected_login_flags(&mut self) {
        self.faction_data_received.set(false);
        self.login = false;
    }

    pub(crate) fn set_player_organizing<U: PlayerOrganizingUpdater>(
        &self,
        updater: &mut U,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        let player_id = self.get_id();
        updater.set_player_organizing(
            player_id,
            &mut self.faction_runtime.borrow_mut().organizing,
        )
    }

 /// Сбрасывает faction-состояние, перечитывает organizing data, сериализует
 /// снимок и только затем публикует `0x7FE06`.
    pub(crate) fn update_faction_info<Context>(
        &self,
        context: &mut Context,
    ) -> Result<PlayerFactionInfoUpdateReport, PlayerFactionInfoUpdateBlock>
    where
        Context: PlayerFactionInfoContext,
    {
        {
            let mut runtime = self.faction_runtime.borrow_mut();
            runtime.organizing.faction_id = 0;
            runtime.organizing.faction_logo_id = 0;
            runtime.organizing.faction_name.clear();
            runtime.organizing.faction_title.clear();
            runtime.organizing.faction_master_id = 0;
            runtime.organizing.faction_level = 0;
            runtime.organizing.faction_experience = 0;
            runtime.organizing.faction_contribute = false;
            runtime.organizing.union_id = 0;
            runtime.organizing.union_master_id = 0;
            runtime.organizing.enemy_factions.clear();
            runtime.city_war_died_state_time = 0;
            runtime.organizing.city_war_enemy_factions.clear();
            runtime.create_union_operator = false;
            runtime.faction_war_operator = false;
            runtime.organizing.force = 0;
 // При faction ID `0` serializer не читает stale owned-region list;
 // очистка здесь не меняет наблюдаемый wire-контракт.
            runtime.organizing.clear_owned_regions();
        }

        self.set_player_organizing(context)
            .map_err(PlayerFactionInfoUpdateBlock::InitialOrganizing)?;
        let faction_id_after_initial_update =
            self.faction_runtime.borrow().organizing.faction_id;
        let faction_data_reset = faction_id_after_initial_update == 0;
        if faction_data_reset {
            self.faction_data_received.set(false);
        }

        let player_id = self.get_id();
        let mut message = CMessage::new(0x0007_FE06);
        message.base_mut().add_long(player_id);
        let mut organization_wire = Vec::new();
        self.add_org_sys_to_byte_array(&mut organization_wire, context)
            .map_err(PlayerFactionInfoUpdateBlock::Serialization)?;
        message.base_mut().add(&organization_wire);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_player_faction_info(player_id, &message);
        Ok(PlayerFactionInfoUpdateReport {
            player_id,
            faction_id_after_initial_update,
            faction_data_reset,
            wire,
            delivery,
        })
    }

    pub(crate) const fn set_state(&mut self, state: u16) {
        self.move_shape_base.set_state(state);
    }

    pub(crate) fn db_goods_projection(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<PlayerGoodsDbProjection, GoodsDbSnapshotBlock> {
        Ok(PlayerGoodsDbProjection {
            packet: self.packet.db_save_entries(registry)?,
            equipment: self.equipment.db_save_entries(registry)?,
            hand: self.hand.db_save_entries(registry)?,
            wallet: self.wallet.db_save_entries(registry)?,
            yuan_bao: self.yuan_bao.db_save_entries(registry)?,
            ji_fen: self.ji_fen.db_save_entries(registry)?,
            bank: self.bank.db_save_entries(registry)?,
            depot: self.depot.db_save_entries(registry)?,
            fairy: self.fairy.db_save_entries(registry)?,
            battle_fairy: self.battle_fairy.db_save_entries(registry)?,
            auction_goods: self.auction_goods_container.db_save_entries(registry)?,
            auction_wallet: self.auction_wallet.db_save_entries(registry)?,
            auction: self.auction_container.db_save_entries(registry)?,
            ci_qing: self.ci_qing.db_save_entries(registry)?,
            compose_ci_qing: self.compose_ci_qing.db_save_entries(registry)?,
        })
    }

    pub(crate) fn db_projection(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<PlayerDbProjection<'_>, PlayerDbProjectionBlock> {
        let country = self
            .country
            .ok_or(PlayerDbProjectionBlock::UninitializedField {
                field: "m_btCountry",
            })?;
        let contribute = self
            .contribute
            .ok_or(PlayerDbProjectionBlock::UninitializedField {
                field: "m_lContribute",
            })?;

        let declared = usize::try_from(self.variable_data_length).map_err(|_| {
            PlayerDbProjectionBlock::NegativeVariableDataLength {
                value: self.variable_data_length,
            }
        })?;
        let available = self.variable_data.as_deref().unwrap_or_default();
        let variable_data = available.get(..declared).ok_or(
            PlayerDbProjectionBlock::VariableDataShorterThanDeclared {
                declared,
                available: available.len(),
            },
        )?;

        let mut hot_keys = [0u32; 24];
        for (index, hot_key) in hot_keys.iter_mut().enumerate() {
            *hot_key = self
                .base_property
                .read_u32(BASE_PROPERTY_HOT_KEYS_OFFSET + index * 4);
        }
        let skills = self
            .new_skills
            .iter()
            .map(|skill| PlayerAbilitySkill {
                id: skill.skill_id,
                level: skill.level,
            })
            .collect();
        let friend_names = self
            .friends
            .iter()
            .enumerate()
            .map(|(friend_index, friend)| {
                PlayerFriendName::from_legacy_bytes(&friend.name).map_err(
                    |EmbeddedFriendNameNul { offset }| {
                        PlayerDbProjectionBlock::FriendNameContainsNul {
                            friend_index,
                            offset,
                        }
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let things = self
            .daily_things
            .iter()
            .map(|thing| DbPlayerThing {
                tid: thing.thing_id,
                count: thing.count,
                max_count: thing.max_count,
                point: thing.point,
            })
            .collect();
        let quests = self
            .player_quests
            .iter()
            .map(|(key, quest)| {
                (
                    *key,
                    PlayerQuestSaveEntry {
                        quest_id: quest.quest_id,
                        complete: quest.complete,
                    },
                )
            })
            .collect();

        Ok(PlayerDbProjection {
            player: self,
            country,
            contribute,
            goods: self.db_goods_projection(registry)?,
            hot_keys,
            skills,
            friend_names,
            things,
            quests,
            variable_data,
        })
    }

    pub(crate) const fn set_region_id(&mut self, region_id: i32) {
        self.move_shape_base.set_region_id(region_id);
    }

    pub(crate) fn add_to_byte_array<U: PlayerOrganizingUpdater>(
        &mut self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
        updater: &mut U,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self.add_to_byte_array_before_containers(destination, include_child)?;
        if include_child {
            let _ = self.add_containers_to_byte_array(destination, registry)?;
            let _ = self.add_to_byte_array_after_containers(destination, updater)?;
        }
        self.update_property(coefficients)?;
        Ok(true)
    }

    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self.decord_from_byte_array_before_containers(source, cursor, include_child)?;
        if include_child {
            let _ = self.decord_containers_from_byte_array(source, cursor, registry)?;
            let _ = self.decord_from_byte_array_after_containers(source, cursor)?;
        }
        self.update_property(coefficients)?;
        Ok(true)
    }

    pub(crate) fn update_property(
        &mut self,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<(), PlayerCodecError> {
        let strength = self.base_property.read_u32(BASE_PROPERTY_STR_OFFSET);
        let dexterity = self.base_property.read_u32(BASE_PROPERTY_DEX_OFFSET);
        let constitution = self.base_property.read_u32(BASE_PROPERTY_CON_OFFSET);
        let intelligence = self.base_property.read_u32(BASE_PROPERTY_INT_OFFSET);
        let saved_dexterity = dexterity as f32;
        let saved_intelligence = intelligence as f32;

        self.property.write_u32(PROPERTY_STR_OFFSET, strength);
        self.property.write_u32(PROPERTY_DEX_OFFSET, dexterity);
        self.property.write_u32(PROPERTY_CON_OFFSET, constitution);
        self.property.write_u32(PROPERTY_INT_OFFSET, intelligence);

        let occupation = self.base_property.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET);
        let occupation = usize::from(occupation);
        if occupation >= 3 {
 // Оригинал индексировал соседнюю static
 // память за `float[3]`; safe Rust не назначает ей коэффициент.
            return Err(
                PlayerCodecError::OccupationOutsidePropertyCoefficientRange {
                    occupation: occupation as u8,
                },
            );
        }

        let max_hp = self
            .base_property
            .read_u32(BASE_PROPERTY_MAX_HP_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                constitution,
                coefficients.con_to_max_hp[occupation],
            ));
        self.property.write_u32(PROPERTY_MAX_HP_OFFSET, max_hp);

        let max_mp = self
            .base_property
            .read_u32(BASE_PROPERTY_MAX_MP_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                intelligence,
                coefficients.int_to_max_mp[occupation],
            ));
        self.property.write_u32(PROPERTY_MAX_MP_OFFSET, max_mp);
        self.property.write_u16(
            PROPERTY_MAX_YP_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_MAX_YP_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_MAX_RP_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_MAX_RP_OFFSET),
        );

        let min_attack = self
            .base_property
            .read_u32(BASE_PROPERTY_MIN_ATK_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                dexterity,
                coefficients.dex_to_min_attack[occupation],
            ));
        self.property.write_u32(PROPERTY_MIN_ATK_OFFSET, min_attack);

        let max_attack = self
            .base_property
            .read_u32(BASE_PROPERTY_MAX_ATK_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                strength,
                coefficients.str_to_max_attack[occupation],
            ));
        self.property.write_u32(PROPERTY_MAX_ATK_OFFSET, max_attack);
        self.property.write_u16(
            PROPERTY_HIT_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_HIT_OFFSET),
        );
        let burden = self
            .base_property
            .read_u16(BASE_PROPERTY_BURDEN_OFFSET)
            .wrapping_add(legacy_x87_i64_low_u16(
                strength,
                coefficients.str_to_burden[occupation],
            ));
        self.property.write_u16(PROPERTY_BURDEN_OFFSET, burden);
        self.property.write_u16(
            PROPERTY_CCH_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_CCH_OFFSET),
        );

        let defense = self
            .base_property
            .read_u32(BASE_PROPERTY_DEF_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                constitution,
                coefficients.con_to_defense[occupation],
            ));
        self.property.write_u32(PROPERTY_DEF_OFFSET, defense);
        self.property.write_u16(
            PROPERTY_DODGE_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_DODGE_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_ATC_SPEED_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_ATC_SPEED_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_HP_RECOVER_SPEED_OFFSET,
            self.base_property
                .read_u16(BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_MP_RECOVER_SPEED_OFFSET,
            self.base_property
                .read_u16(BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET),
        );

        let element_resistant = self
            .base_property
            .read_u32(BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET)
            .wrapping_add(legacy_x87_f32_i32_bits(
                saved_intelligence,
                coefficients.int_to_resistant[occupation],
            ));
        self.property
            .write_u32(PROPERTY_ELEMENT_RESISTANT_OFFSET, element_resistant);
        self.property.write_u32(
            PROPERTY_ELEMENT_MODIFY_OFFSET,
            legacy_x87_f32_i32_bits(saved_intelligence, coefficients.int_to_element[occupation]),
        );
        self.property.write_u16(
            PROPERTY_RE_ANK_OFFSET,
            legacy_x87_f32_i64_low_u16(saved_dexterity, coefficients.dex_to_stiff[occupation]),
        );
        Ok(())
    }

    pub(crate) fn add_to_byte_array_before_containers(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self
            .move_shape_base
            .add_shape_to_byte_array(destination, include_child);
        if !include_child {
            return Ok(true);
        }

        destination.extend_from_slice(&self.base_property.wire);
        append_player_c_string(destination, &self.base_property.account);
        append_player_c_string(destination, &self.base_property.title);
        destination.extend_from_slice(&self.property.wire);
        destination.extend_from_slice(&self.team_id.to_le_bytes());
        self.add_byte_ci_qing(destination)?;

        append_player_count(destination, "m_listNewSkillID", self.new_skills.len())?;
        for skill in &self.new_skills {
            destination.extend_from_slice(&skill.skill_id.to_le_bytes());
            destination.extend_from_slice(&skill.level.to_le_bytes());
        }

        append_player_count(
            destination,
            "m_vExStates",
            self.move_shape_base.ex_states().len(),
        )?;
        destination.extend_from_slice(self.move_shape_base.ex_states());

        append_player_count(destination, "m_listFriend", self.friends.len())?;
        for friend in &self.friends {
            append_player_c_string(destination, &friend.name);
            destination.push(u8::from(friend.online));
        }
        self.add_byte_array_lei_ting(destination)?;
        Ok(true)
    }

    pub(crate) fn decord_from_byte_array_before_containers(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, PlayerCodecError> {
        self.new_skills.clear();
        self.move_shape_base.clear_ex_states();
        self.friends.clear();
        self.daily_things.clear();
        self.uncreated_pets.clear();
        self.uncreated_carriage.original_name.clear();
        self.uncreated_carriage.hp = 0;
        self.recreate_carriage = false;

        let _ = self
            .move_shape_base
            .decord_shape_from_byte_array(source, cursor, include_child)?;
        if !include_child {
            return Ok(true);
        }

        self.base_property.wire = read_player_array(source, cursor, "m_BaseProperty[0x194]")?;
        self.base_property.account =
            read_player_c_string(source, cursor, "m_BaseProperty.strAccount", 0x100)?;
        self.base_property.title =
            read_player_c_string(source, cursor, "m_BaseProperty.strTitle", 0x100)?;
        self.property.wire = read_player_array(source, cursor, "m_Property[0x9c]")?;
        self.team_id = read_player_i32(source, cursor, "m_lTeamID")?;
        self.de_byte_ci_qing(source, cursor)?;

        let skill_count = read_player_i32(source, cursor, "m_listNewSkillID count")?;
        if skill_count > 0 {
            for _ in 0..skill_count {
                let bytes = read_player_array::<4>(source, cursor, "CPlayer::tagSkill")?;
                self.new_skills.push_back(PlayerSkill {
                    skill_id: u16::from_le_bytes([bytes[0], bytes[1]]),
                    level: u16::from_le_bytes([bytes[2], bytes[3]]),
                });
            }
        }

        let ex_state_length = read_player_i32(source, cursor, "m_vExStates length")?;
        if ex_state_length < 0 {
 // Старый signed length уходил в pointer
 // arithmetic и `_AddToByteArray`; результат для high-bit wire не
 // определён согласованным оригинал.
            return Err(PlayerCodecError::NegativeLength {
                field: "m_vExStates length",
                value: ex_state_length,
            });
        }
        if ex_state_length != 0 {
            let length = ex_state_length as usize;
            let states = read_player_slice(source, cursor, "m_vExStates bytes", length)?;
            self.move_shape_base.set_ex_states(states);
        }

        let friend_count = read_player_i32(source, cursor, "m_listFriend count")?;
        if friend_count > 0 {
            for _ in 0..friend_count {
                let name = read_player_c_string(source, cursor, "tagFriend.strName", 0x94)?;
                let online = read_player_u8(source, cursor, "tagFriend.bOnline")? != 0;
                self.friends.push_back(PlayerFriend { name, online });
            }
        }
        self.decode_byte_array_lei_ting(source, cursor)?;
        Ok(true)
    }

    pub(crate) fn add_containers_to_byte_array(
        &mut self,
        destination: &mut Vec<u8>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self.hand.serialize(destination, true, registry)?;
        let _ = self.equipment.serialize(destination, true, registry)?;
        let _ = self.packet.serialize(destination, true, registry)?;
        let _ = self
            .auction_goods_container
            .serialize(destination, true, registry)?;
        let _ = self
            .auction_container
            .serialize(destination, true, registry)?;
        let _ = self.wallet.serialize(destination, true)?;
        let _ = self.auction_wallet.serialize(destination, true)?;
        let _ = self.yuan_bao.serialize(destination, true)?;
        let _ = self.ji_fen.serialize(destination, true)?;
        append_player_c_string(destination, &self.depot_password);
        let _ = self.bank.serialize(destination, true)?;
        let _ = self.depot.serialize(destination, true, registry)?;
        let _ = self.fairy.serialize(destination, true, registry)?;
        let _ = self.battle_fairy.serialize(destination, true, registry)?;
        let _ = self.ci_qing.serialize(destination, true, registry)?;
        let _ = self
            .compose_ci_qing
            .serialize(destination, true, registry)?;
        Ok(true)
    }

    pub(crate) fn decord_containers_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, PlayerCodecError> {
        self.hand.release();
        self.hand.set_goods_amount_limit(1);
        let _ = self.hand.unserialize(source, cursor, registry)?;

        self.equipment.release();
        let _ = self.equipment.unserialize(source, cursor, true, registry)?;

        self.packet.release();
        self.packet.set_container_volume_2d(8, 0x0C);
        let _ = self.packet.unserialize(source, cursor, registry)?;

        self.auction_goods_container.release();
        self.auction_goods_container.set_container_volume(0x12);
        let _ = self
            .auction_goods_container
            .unserialize(source, cursor, registry)?;

        self.auction_container.release();
        self.auction_container.set_container_volume(2);
        let _ = self
            .auction_container
            .unserialize(source, cursor, registry)?;

        self.wallet.release();
        let _ = self.wallet.unserialize(source, cursor, true, registry)?;

        self.auction_wallet.release();
        let _ = self
            .auction_wallet
            .unserialize(source, cursor, true, registry)?;

        self.yuan_bao.release();
        let _ = self.yuan_bao.unserialize(source, cursor, true, registry)?;

        self.ji_fen.release();
        let _ = self.ji_fen.unserialize(source, cursor, true, registry)?;

        self.depot_password = read_player_c_string(source, cursor, "m_strDepotPassword", 0x6C)?;

        self.bank.release();
        let _ = self.bank.unserialize(source, cursor, true, registry)?;

        self.depot.release();
        self.depot.set_container_volume(0xA1);
        let _ = self.depot.unserialize(source, cursor, true, registry)?;

        self.fairy.release();
        self.fairy.set_container_volume(0x0E);
        let _ = self.fairy.unserialize(source, cursor, true, registry)?;

        self.battle_fairy.release();
        self.battle_fairy.set_container_volume(0x11);
        let _ = self
            .battle_fairy
            .unserialize(source, cursor, true, registry)?;

        self.ci_qing.release();
        self.ci_qing.set_container_volume(8);
        let _ = self.ci_qing.unserialize(source, cursor, registry)?;

        self.compose_ci_qing.release();
        self.compose_ci_qing.set_container_volume(3);
        let _ = self.compose_ci_qing.unserialize(source, cursor, registry)?;
        Ok(true)
    }

    pub(crate) fn add_to_byte_array_after_containers<U: PlayerOrganizingUpdater>(
        &mut self,
        destination: &mut Vec<u8>,
        updater: &mut U,
    ) -> Result<bool, PlayerCodecError> {
        destination.extend_from_slice(&self.variable_num.to_le_bytes());
        destination.extend_from_slice(&self.variable_data_length.to_le_bytes());
        if let Some(variable_data) = &self.variable_data {
            if self.variable_data_length < 0 {
                return Err(PlayerCodecError::NegativeLength {
                    field: "m_lVariableDataLength",
                    value: self.variable_data_length,
                });
            }
            let declared = self.variable_data_length as usize;
            if variable_data.len() < declared {
                return Err(PlayerCodecError::BufferShorterThanDeclaredLength {
                    field: "m_pVariableData",
                    declared,
                    available: variable_data.len(),
                });
            }
            destination.extend_from_slice(&variable_data[..declared]);
        }

        destination.extend_from_slice(&self.silience_time.to_le_bytes());
        destination.push(u8::from(self.move_shape_base.is_god()));
        destination.extend_from_slice(&self.murderer_time.to_le_bytes());
        destination.extend_from_slice(&self.fight_state_count.to_le_bytes());

        append_player_count(destination, "m_vUncreatedPets", self.uncreated_pets.len())?;
        for pet in &self.uncreated_pets {
            append_player_c_string(destination, &pet.original_name);
            destination.extend_from_slice(&pet.hp.to_le_bytes());
            destination.extend_from_slice(&pet.level.to_le_bytes());
            destination.extend_from_slice(&pet.experience.to_le_bytes());
        }

        append_player_c_string(destination, &self.uncreated_carriage.original_name);
        append_player_c_string(destination, &self.uncreated_carriage.carriage_script);
        destination.extend_from_slice(&self.uncreated_carriage.hp.to_le_bytes());
        destination.push(u8::from(self.recreate_carriage));
        destination.push(u8::from(self.login));
        destination.extend_from_slice(
            &self
                .faction_runtime
                .borrow()
                .city_war_died_state_time
                .to_le_bytes(),
        );
        let _ = self.add_quest_data_to_byte_array(destination)?;
        let country = self
            .country
            .ok_or(PlayerCodecError::UninitializedWireField {
                field: "m_btCountry",
            })?;
        destination.push(country);
        let contribute = self
            .contribute
            .ok_or(PlayerCodecError::UninitializedWireField {
                field: "m_lContribute",
            })?;
        destination.extend_from_slice(&contribute.to_le_bytes());
        destination.extend_from_slice(&self.jjc_data);
        destination.push(u8::from(self.jjc_pk_state));
        let _ = self.add_org_sys_to_byte_array(destination, updater)?;
        append_player_fixed_c_string(destination, "m_strSessionID", &self.session_id, 0x40)?;
        Ok(true)
    }

    pub(crate) fn decord_from_byte_array_after_containers(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        self.variable_num = read_player_i32(source, cursor, "m_lVariableNum")?;
        self.variable_data_length = read_player_i32(source, cursor, "m_lVariableDataLength")?;
        if self.variable_data_length < 0 {
 // Оригинал передавал high-bit length в
 // `operator new` и безразмерный `_GetBufferFromByteArray`.
            return Err(PlayerCodecError::NegativeLength {
                field: "m_lVariableDataLength",
                value: self.variable_data_length,
            });
        }
        let variable_data = read_player_slice(
            source,
            cursor,
            "m_pVariableData",
            self.variable_data_length as usize,
        )?;
        self.variable_data = Some(variable_data.to_vec());

        self.silience_time = read_player_i32(source, cursor, "m_lSilienceTime")?;
        let is_god = read_player_u8(source, cursor, "m_bIsGod")? != 0;
        self.move_shape_base.set_is_god(is_god);
        self.murderer_time = read_player_u32(source, cursor, "m_dwMurdererTime")?;
        let _wire_fight_state_count = read_player_i32(source, cursor, "m_lFightStateCount wire")?;
        self.fight_state_count = 2;

        let pet_count = read_player_i32(source, cursor, "m_vUncreatedPets count")?;
        if pet_count < 0 {
 // Исходный `for (count; count != 0; --count)`
 // для отрицательного значения уходит в signed overflow/overread.
            return Err(PlayerCodecError::NegativeLength {
                field: "m_vUncreatedPets count",
                value: pet_count,
            });
        }
        for _ in 0..pet_count {
            self.uncreated_pets.push(PlayerPetInformation {
                original_name: read_player_c_string(
                    source,
                    cursor,
                    "tagPetInformation.strOriginalName",
                    0x94,
                )?,
                hp: read_player_u32(source, cursor, "tagPetInformation.dwHp")?,
                level: read_player_u32(source, cursor, "tagPetInformation.dwLevel")?,
                experience: read_player_u32(source, cursor, "tagPetInformation.dwExperience")?,
            });
        }

        self.uncreated_carriage.original_name =
            read_player_c_string(source, cursor, "tagCarriageInfo.strOriginalName", 0x94)?;
        self.uncreated_carriage.carriage_script =
            read_player_c_string(source, cursor, "tagCarriageInfo.strCarriageScript", 0x94)?;
        self.uncreated_carriage.hp = read_player_u32(source, cursor, "tagCarriageInfo.dwHp")?;
        self.recreate_carriage = read_player_u8(source, cursor, "m_bReCreateCarriage")? != 0;
        self.login = read_player_u8(source, cursor, "m_bLogin")? != 0;
        self.faction_runtime.borrow_mut().city_war_died_state_time =
            read_player_i32(source, cursor, "m_lCityWarDiedStateTime")?;
        let _ = self.decord_quest_data_from_byte_array(source, cursor)?;
        self.country = Some(read_player_u8(source, cursor, "m_btCountry")?);
        self.contribute = Some(read_player_i32(source, cursor, "m_lContribute")?);
        self.jjc_data = read_player_array(source, cursor, "m_jjcData[0x10]")?;
        self.jjc_pk_state = read_player_u8(source, cursor, "bJJcPkState")? != 0;
        self.session_id = read_player_c_string(source, cursor, "m_strSessionID", 0x40)?;
        Ok(true)
    }

    pub(crate) fn add_org_sys_to_byte_array<U: PlayerOrganizingUpdater>(
        &self,
        destination: &mut Vec<u8>,
        updater: &mut U,
    ) -> Result<bool, PlayerCodecError> {
        let mut runtime = self.faction_runtime.borrow_mut();
        updater.set_player_organizing(self.get_id(), &mut runtime.organizing)?;

        let organizing = &runtime.organizing;
        destination.extend_from_slice(&organizing.faction_id.to_le_bytes());
        if organizing.faction_id <= 0 {
            return Ok(true);
        }

        destination.extend_from_slice(&organizing.faction_logo_id.to_le_bytes());
        destination.extend_from_slice(&organizing.faction_level.to_le_bytes());
        destination.extend_from_slice(&organizing.faction_experience.to_le_bytes());
        destination.extend_from_slice(&organizing.force.to_le_bytes());
        destination.extend_from_slice(&u32::from(organizing.faction_contribute).to_le_bytes());
        append_player_c_string(destination, &organizing.faction_name);
        append_player_c_string(destination, &organizing.faction_title);
        destination.extend_from_slice(&organizing.faction_master_id.to_le_bytes());
        destination.extend_from_slice(&organizing.union_id.to_le_bytes());
        destination.extend_from_slice(&organizing.union_master_id.to_le_bytes());

        append_player_count(
            destination,
            "m_EnemyFactions",
            organizing.enemy_factions.len(),
        )?;
        for &faction_id in &organizing.enemy_factions {
            destination.extend_from_slice(&faction_id.to_le_bytes());
        }

        append_player_count(
            destination,
            "m_CityWarEnemyFactions",
            organizing.city_war_enemy_factions.len(),
        )?;
        for &faction_id in &organizing.city_war_enemy_factions {
            destination.extend_from_slice(&faction_id.to_le_bytes());
        }

        append_player_count(
            destination,
            "m_OwnedRegions",
            organizing.owned_regions.len(),
        )?;
        for &owned_region in &organizing.owned_regions {
            destination.extend_from_slice(&owned_region.wire_bytes());
        }
        Ok(true)
    }

    pub(crate) fn add_byte_ci_qing(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerCodecError> {
        append_player_count(destination, "m_setCiQingList", self.ci_qing_ids.len())?;
        for &id in &self.ci_qing_ids {
            destination.extend_from_slice(&id.to_le_bytes());
        }
        Ok(())
    }

    pub(crate) fn de_byte_ci_qing(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerCodecError> {
        self.ci_qing_ids.clear();
        let count = read_player_u32(source, cursor, "m_setCiQingList count")?;
        for _ in 0..count {
            let id = read_player_u32(source, cursor, "m_setCiQingList value")?;
            self.ci_qing_ids.insert(id);
        }
        Ok(())
    }

    pub(crate) fn add_byte_array_lei_ting(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerCodecError> {
        destination.extend_from_slice(
            &self
                .base_property
                .read_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u32(BASE_PROPERTY_FY_ENERGY_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u16(BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET)
                .to_le_bytes(),
        );
        append_player_count(destination, "m_listThing", self.daily_things.len())?;
        for thing in &self.daily_things {
            destination.extend_from_slice(&thing.thing_id.to_le_bytes());
            destination.extend_from_slice(&thing.count.to_le_bytes());
            destination.extend_from_slice(&thing.max_count.to_le_bytes());
            destination.extend_from_slice(&thing.point.to_le_bytes());
        }
        Ok(())
    }

    pub(crate) fn update_lei_ting<Clock: PlayerLeiTingClock>(
        &mut self,
        update_kind: u32,
        stamp: &mut LeiTingLocalTime,
        total_jing_li_dan_count: u16,
        thing_setup: &CThingSetup,
        clock: &mut Clock,
    ) -> Result<PlayerLeiTingUpdateReport, PlayerLeiTingUpdateBlock<Clock::Block>> {
        let previous_stamp = self
            .base_property
            .read_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET);
        self.base_property
            .write_u32(BASE_PROPERTY_FY_ENERGY_OFFSET, 0);
        self.base_property.write_u16(
            BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET,
            total_jing_li_dan_count,
        );

        let previous_time = clock
            .local_time_from_timestamp(previous_stamp)
            .map_err(PlayerLeiTingUpdateBlock::PreviousLocalTime)?;
        let stamp_replaced = previous_time.year_since_1900 != stamp.year_since_1900
            || previous_time.year_day != stamp.year_day;
        if stamp_replaced {
            let normalized_stamp = clock
                .mktime(stamp)
                .map_err(PlayerLeiTingUpdateBlock::Stamp)?;
            self.base_property.write_u32(
                BASE_PROPERTY_LT_60_STAMP_OFFSET,
                normalized_stamp as u32,
            );
        }

        if update_kind == 1 {
            let flags = self
                .base_property
                .read_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET);
            self.base_property
                .write_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET, flags & 0xFFFF_FFF0);
        } else if update_kind == 2 {
            self.base_property
                .write_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET, 0);
            self.base_property
                .write_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET, 0);
        }

        let mut daily_things = VecDeque::new();
        thing_setup.get_daily_thing_list(|| clock.current_week_day(), &mut daily_things);
        let daily_list_replaced = !daily_things.is_empty();
        if daily_list_replaced {
            self.daily_things = daily_things
                .into_iter()
                .map(
                    |LeiTingDailyThing {
                         thing_id,
                         count,
                         max_count,
                         point,
                     }| PlayerThing {
                        thing_id,
                        count,
                        max_count,
                        point,
                    },
                )
                .collect();
        }

        Ok(PlayerLeiTingUpdateReport {
            player_id: self.get_id(),
            update_kind,
            previous_stamp,
            resulting_stamp: self
                .base_property
                .read_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET),
            stamp_replaced,
            daily_list_replaced,
            daily_thing_count: self.daily_things.len(),
        })
    }

    pub(crate) fn decode_byte_array_lei_ting(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerCodecError> {
        let enable_flags = read_player_u32(source, cursor, "dwfyenFlag")?;
        self.base_property
            .write_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET, enable_flags);
        let energy = read_player_u32(source, cursor, "dwfyEnergy")?;
        self.base_property
            .write_u32(BASE_PROPERTY_FY_ENERGY_OFFSET, energy);
        let stamp = read_player_u32(source, cursor, "dwLT60Stamp")?;
        self.base_property
            .write_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET, stamp);
        let up_60_count = read_player_u16(source, cursor, "wLTUp60Cnt")?;
        self.base_property
            .write_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET, up_60_count);
        let remaining = read_player_u16(source, cursor, "wRemainJingLiDanCnt")?;
        self.base_property
            .write_u16(BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET, remaining);

        self.daily_things.clear();
        let count = read_player_u32(source, cursor, "m_listThing count")?;
        for _ in 0..count {
            self.daily_things.push_back(PlayerThing {
                thing_id: read_player_u16(source, cursor, "tagThing.wTID")?,
                count: read_player_u16(source, cursor, "tagThing.wCnt")?,
                max_count: read_player_u16(source, cursor, "tagThing.wMaxCnt")?,
                point: read_player_u16(source, cursor, "tagThing.wPoint")?,
            });
        }
        Ok(())
    }

    pub(crate) fn add_quest_data_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<bool, PlayerCodecError> {
        append_player_count(destination, "m_PlayerQuests", self.player_quests.len())?;
        for quest in self.player_quests.values() {
            destination.extend_from_slice(&quest.quest_id.to_le_bytes());
            destination.push(quest.complete);
        }
        Ok(true)
    }

    pub(crate) fn decord_quest_data_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        self.player_quests.clear();
        let count = read_player_i32(source, cursor, "m_PlayerQuests count")?;
        if count > 0 {
            for _ in 0..count {
                let quest_id = read_player_u16(source, cursor, "tagPlayerQuest.wQuestID")?;
                let complete = read_player_u8(source, cursor, "tagPlayerQuest.byComplete")?;
                self.player_quests
                    .insert(quest_id, PlayerQuest { quest_id, complete });
            }
        }
        Ok(true)
    }

    pub(crate) async fn load_data<L: PlayerLoadDataOwner>(
        &mut self,
        loader: &mut L,
        player_list: &mut CPlayerList,
        globe_setup: &GlobeSetupSnapshot,
        coefficients: &PlayerPropertyCoefficients,
    ) -> PlayerLoadDataOutcome<L::Block> {
        match loader.load_player(self).await {
            Ok(false) => return PlayerLoadDataOutcome::ReturnedFalse,
            Err(block) => return PlayerLoadDataOutcome::BlockedDatabase(block),
            Ok(true) => {}
        }

        let occupation = self.base_property.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET);
        let level = self.base_property.read_u8(BASE_PROPERTY_LEVEL_OFFSET);
        let sex = self.base_property.read_u8(BASE_PROPERTY_SEX_OFFSET);

        let base_max_rp = globe_setup.base_max_rp(occupation, level);
        self.base_property
            .write_u16(BASE_PROPERTY_MAX_RP_OFFSET, base_max_rp);

        let PlayerCreationPropertiesLookup {
            key: player_property_key,
            inserted: player_property_inserted,
            properties,
        } = player_list.creation_properties(sex, occupation);
        self.base_property.write_u16(
            BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET,
            properties.base_hp_recover_speed,
        );
        self.base_property.write_u16(
            BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET,
            properties.base_mp_recover_speed,
        );

        let upgrade = player_list.properties_upgrade(occupation, level);
        let upgrade_applied = upgrade.is_some();
        if let Some(upgrade) = upgrade {
            self.base_property
                .write_u32(BASE_PROPERTY_MAX_HP_OFFSET, upgrade.base_maximum_hp);
            self.base_property
                .write_u32(BASE_PROPERTY_MAX_MP_OFFSET, upgrade.base_maximum_mp);
            self.base_property
                .write_u32(BASE_PROPERTY_STR_OFFSET, upgrade.base_strength);
            self.base_property
                .write_u32(BASE_PROPERTY_DEX_OFFSET, upgrade.base_dexterity);
            self.base_property
                .write_u32(BASE_PROPERTY_CON_OFFSET, upgrade.base_constitution);
            self.base_property
                .write_u32(BASE_PROPERTY_INT_OFFSET, upgrade.base_intelligence);
            self.base_property
                .write_u16(BASE_PROPERTY_BURDEN_OFFSET, upgrade.base_burden);
        }

        if let Err(block) = self.update_property(coefficients) {
            return PlayerLoadDataOutcome::BlockedProperty(block);
        }

        let maximum_rp = self.property.read_u16(PROPERTY_MAX_RP_OFFSET);
        let current_rp = self.base_property.read_u16(BASE_PROPERTY_RP_OFFSET);
        let rp_clamped = maximum_rp < current_rp;
        if rp_clamped {
            self.base_property
                .write_u16(BASE_PROPERTY_RP_OFFSET, maximum_rp);
        }

        let graphics_id = i32::from(sex) + 1 + i32::from(occupation) * 2;
        self.move_shape_base.set_graphics_id(graphics_id);
        let player_speed = globe_setup.player_speed();
        self.move_shape_base.set_speed(player_speed);

        PlayerLoadDataOutcome::Loaded(PlayerLoadDataReport {
            base_max_rp,
            player_property_key,
            player_property_inserted,
            upgrade_applied,
            rp_clamped,
            graphics_id,
            player_speed,
        })
    }

 /// Передаёт полный снимок игрока DB-owner-у и возвращает его точный итог.
 ///
 /// `snapshot` должен быть создан из этой же player-сущности.
    pub(crate) async fn save_data<P, J, G>(
        &self,
        snapshot: &PlayerSaveSnapshot<'_, '_, '_>,
        connection: Option<&mut WorldTdsClient>,
        player_database: &mut P,
        jjc_database: &mut J,
        goods_database: &mut G,
    ) -> PlayerSaveOutcome
    where
        P: RsPlayerOwner<CPlayer>,
        J: RsJjcSysOwner,
        G: DbGoodsOwner<CPlayer>,
    {
        player_database
            .save_player(Some(snapshot), connection, jjc_database, goods_database)
            .await
    }
}

impl nebokrai_realm::activities::rsjjcsys::RsJjcSysPlayerTarget for CPlayer {
    fn jjc_target_player_id(&self) -> i32 {
        self.get_id()
    }

    fn apply_loaded_jjc_snapshot(&mut self, jjc_level: u32, jjc_score: u32, counters: [u16; 8]) {
        CPlayer::apply_loaded_jjc_data(self, jjc_level, jjc_score, counters);
    }
}

impl nebokrai_realm::characters::honorranks::HonorRankPlayerView for CPlayer {
    fn get_id(&self) -> i32 {
        self.get_id()
    }

    fn get_level(&self) -> u8 {
        self.get_level()
    }

    fn get_occupation(&self) -> u8 {
        self.get_occupation()
    }

    fn get_appellation_id(&self) -> u32 {
        self.get_appellation_id()
    }

    fn get_name(&self) -> &[u8] {
        self.get_name()
    }

    fn country(&self) -> Option<u8> {
        self.country()
    }
}

impl nebokrai_realm::characters::playerdataqueue::PlayerDataResetHonorEliminate for CPlayer {
    fn reset_honor_eliminate_info(&mut self, rank_mask: u32) {
        CPlayer::reset_honor_eliminate_info(self, rank_mask);
    }
}

impl nebokrai_realm::activities::leiting::LeiTingPlayerCodec for CPlayer {
    type Block = PlayerCodecError;

    fn add_byte_array_lei_ting(&self, destination: &mut Vec<u8>) -> Result<(), Self::Block> {
        CPlayer::add_byte_array_lei_ting(self, destination)
    }
}

impl nebokrai_realm::activities::jjcsystem::JjcPlayerView for CPlayer {
    fn get_level(&self) -> u8 {
        self.get_level()
    }

    fn get_jjc_level(&self) -> u32 {
        self.get_jjc_level()
    }
}

impl nebokrai_realm::characters::playerloadworker::WorldPlayerLoadFactory for CPlayer {
    fn new_database_load_player() -> Self {
        CPlayer::with_clone_decode_constructor_state()
    }

    fn set_database_load_identity(&mut self, player_id: i32, account: &[u8]) {
        CPlayer::set_database_load_identity(self, player_id, account);
    }
}

fn append_player_count(
    destination: &mut Vec<u8>,
    field: &'static str,
    count: usize,
) -> Result<(), PlayerCodecError> {
    let count = u32::try_from(count)
        .map_err(|_| PlayerCodecError::CollectionLengthOutsideLegacyRange { field, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_player_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let visible = value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(value.len());
    destination.extend_from_slice(&value[..visible]);
    destination.push(0);
}

fn append_player_fixed_c_string(
    destination: &mut Vec<u8>,
    field: &'static str,
    value: &[u8],
    capacity: usize,
) -> Result<(), PlayerCodecError> {
    let length = value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(value.len());
    if length >= capacity {
        return Err(PlayerCodecError::StringOutsideLegacyCapacity {
            field,
            length,
            capacity,
        });
    }
    append_player_c_string(destination, value);
    Ok(())
}

/// Возвращает точный усечённый модуль `u32 * binary32` для x87 integer-convert.
///
/// Safe `softfloat-wrapper` не предоставляет `extFloat80`, а найденные
/// extFloat80 bindings требуют широкого `unsafe` FFI. Здесь общий soft-float
/// runtime не нужен: произведение 32-битного integer и 24-битной significand
/// содержит не более 56 значащих бит и потому точно помещается в 64-битную
/// x87 significand. Разбор IEEE-754 bits сохраняет это единственное действующее
/// действие без округления через недостаточный `f64`.
fn legacy_x87_truncated_magnitude(
    value: u32,
    coefficient: f32,
    positive_limit: u128,
    negative_limit: u128,
) -> Option<(bool, u128)> {
    let bits = coefficient.to_bits();
    let negative = bits >> 31 != 0;
    let exponent_bits = (bits >> 23) & 0xFF;
    let fraction = bits & 0x7F_FFFF;
    if exponent_bits == 0xFF {
        return None;
    }

    let (significand, exponent) = if exponent_bits == 0 {
        (u128::from(fraction), -149)
    } else {
        (
            u128::from((1 << 23) | fraction),
            exponent_bits as i32 - 127 - 23,
        )
    };
    let product = u128::from(value) * significand;
    legacy_x87_truncated_parts(negative, product, exponent, positive_limit, negative_limit)
}

fn legacy_binary32_parts(value: f32) -> Option<(bool, u128, i32)> {
    let bits = value.to_bits();
    let negative = bits >> 31 != 0;
    let exponent_bits = (bits >> 23) & 0xFF;
    let fraction = bits & 0x7F_FFFF;
    if exponent_bits == 0xFF {
        return None;
    }
    if exponent_bits == 0 {
        Some((negative, u128::from(fraction), -149))
    } else {
        Some((
            negative,
            u128::from((1 << 23) | fraction),
            exponent_bits as i32 - 127 - 23,
        ))
    }
}

fn legacy_x87_truncated_f32_magnitude(
    value: f32,
    coefficient: f32,
    positive_limit: u128,
    negative_limit: u128,
) -> Option<(bool, u128)> {
    let (value_negative, value_significand, value_exponent) = legacy_binary32_parts(value)?;
    let (coefficient_negative, coefficient_significand, coefficient_exponent) =
        legacy_binary32_parts(coefficient)?;
    legacy_x87_truncated_parts(
        value_negative != coefficient_negative,
        value_significand * coefficient_significand,
        value_exponent + coefficient_exponent,
        positive_limit,
        negative_limit,
    )
}

fn legacy_x87_truncated_parts(
    negative: bool,
    product: u128,
    exponent: i32,
    positive_limit: u128,
    negative_limit: u128,
) -> Option<(bool, u128)> {
    if product == 0 {
        return Some((negative, 0));
    }
    let limit = if negative {
        negative_limit
    } else {
        positive_limit
    };

    let magnitude = if exponent >= 0 {
        let shift = exponent as u32;
        if shift >= u128::BITS || product > (limit >> shift) {
            return None;
        }
        product << shift
    } else {
        let shift = exponent.unsigned_abs();
        if shift >= u128::BITS {
            0
        } else {
            product >> shift
        }
    };
    (magnitude <= limit).then_some((negative, magnitude))
}

fn legacy_x87_i32_bits(value: u32, coefficient: f32) -> u32 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_magnitude(value, coefficient, i32::MAX as u128, 1_u128 << 31)
    else {
        return i32::MIN as u32;
    };
    if negative {
        0_u32.wrapping_sub(magnitude as u32)
    } else {
        magnitude as u32
    }
}

fn legacy_x87_i64_low_u16(value: u32, coefficient: f32) -> u16 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_magnitude(value, coefficient, i64::MAX as u128, 1_u128 << 63)
    else {
        return 0;
    };
    let bits = if negative {
        0_u64.wrapping_sub(magnitude as u64)
    } else {
        magnitude as u64
    };
    bits as u16
}

fn legacy_x87_f32_i32_bits(value: f32, coefficient: f32) -> u32 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_f32_magnitude(value, coefficient, i32::MAX as u128, 1_u128 << 31)
    else {
        return i32::MIN as u32;
    };
    if negative {
        0_u32.wrapping_sub(magnitude as u32)
    } else {
        magnitude as u32
    }
}

fn legacy_x87_f32_i64_low_u16(value: f32, coefficient: f32) -> u16 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_f32_magnitude(value, coefficient, i64::MAX as u128, 1_u128 << 63)
    else {
        return 0;
    };
    let bits = if negative {
        0_u64.wrapping_sub(magnitude as u64)
    } else {
        magnitude as u64
    };
    bits as u16
}

fn read_player_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, PlayerCodecError> {
    Ok(i32::from_le_bytes(read_player_array(
        source, cursor, field,
    )?))
}

fn read_player_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, PlayerCodecError> {
    Ok(u32::from_le_bytes(read_player_array(
        source, cursor, field,
    )?))
}

fn read_player_u16(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u16, PlayerCodecError> {
    Ok(u16::from_le_bytes(read_player_array(
        source, cursor, field,
    )?))
}

fn read_player_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, PlayerCodecError> {
    Ok(read_player_array::<1>(source, cursor, field)?[0])
}

fn read_player_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    capacity: usize,
) -> Result<Vec<u8>, PlayerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let searchable = available.min(capacity);
    let Some(bytes) = source.get(offset..offset.saturating_add(searchable)) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 1,
            available,
        });
    };
    if let Some(length) = bytes.iter().position(|&byte| byte == 0) {
        *cursor = offset + length + 1;
        return Ok(bytes[..length].to_vec());
    }

 // Старый вспомогательный код писал до NUL в фиксированный стековый буфер.
 // Rust не воспроизводит ни переполнение, ни чтение за источником.
    Err(PlayerCodecError::UnterminatedString {
        field,
        offset,
        capacity,
        available,
    })
}

fn read_player_slice<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    field: &'static str,
    length: usize,
) -> Result<&'a [u8], PlayerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(length) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: length,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: length,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

fn read_player_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], PlayerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(N) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
 // Старый вспомогательный код не получал длину источника и продолжал
 // чтение. Rust останавливает только эту локальную операцию.
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor = end;
    Ok(bytes
        .try_into()
        .expect("slice содержит ровно запрошенное число байт"))
}
