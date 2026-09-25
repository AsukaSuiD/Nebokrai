//! DB-владелец `CRsPlayer` WorldServer из `rsplayer.cpp`; трейт `RsPlayerOwner` и
//! его data-семья перенесены в Realm `persistence/rsplayer`, здесь остаётся
//! Tiberius-реализация и их реэкспорт для переходных потребителей.
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
//!
//! Owner охватывает create/open/load/save игрока, отдельные field codecs,
//! honor ranks, JJC/LeiTing maintenance и lookup-операции account/name/country.
//! Equipment snapshot сохраняет порядок HELM, BODY, GLOV, BOOT, WEAPON, BACK,
//! HEADGEAR, FROCK, WING, MANTEAU, FAIRY и исходную 32-битную арифметику.
//!
//! `GetPlayerDeletionDate` различает `0` и catch-result `-1`; caller считает
//! `-1` ненулевым timestamp. Очередность SQL и уже выполненные partial effects
//! не откатываются автоматически. Параметризованный Tiberius, owned byte
//! strings и typed snapshots заменяют ADO/COM, globals и fixed buffers, не
//! меняя схемы, provider-order, значения отказа или wire-контракты caller-а.
//! Realm-трейт параметризован игроком; локальная реализация для
//! `TiberiusRsPlayer` разрешена orphan-правилом и связывает generic-параметр с
//! `CPlayer`, а block-типы загрузки — associated types goods-владельца.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::convert::Infallible;
use std::mem::size_of;

use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike};
use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::{Query, Row};

use crate::dbaccess::row::{get_integer as read_ado_integer, get_value};
use crate::dbaccess::worlddb::dbgoods::{DbGoodsOwner, GoodsFiledSaveOutcome};
use crate::dbaccess::worlddb::rsjjcsys::{PlayerJjcLoadOutcome, RsJjcSysOwner};
use crate::dbaccess::worlddb::rssetup::{WorldDatabaseSettings, WorldTdsClient};
use crate::public::date::TagTime;
use crate::setup::leitingsetup::CThingSetup;
use crate::worldserver::appworld::goods::cgoods::GoodsLoadedAddonBlock;
use crate::worldserver::appworld::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use crate::worldserver::appworld::player::{
    CPlayer, PlayerLoadDataOwner, PlayerLoadedGoodsInsertBlock,
};
use crate::worldserver::worldserver::playerranks::CPlayerRanks;
use nebokrai_realm::characters::playerranks::PlayerRankOrganizingLookup;
use nebokrai_realm::content::dbgoods::GoodsLoadOutcome as GenericGoodsLoadOutcome;
use nebokrai_realm::persistence::rsplayer::{
    PlayerLoadBlock as GenericPlayerLoadBlock, PlayerLoadOutcome as GenericPlayerLoadOutcome,
};

pub(crate) use nebokrai_realm::persistence::rsplayer::{
    CollectedHonorRanksFields, HonorRanksBlobBlock,
    HonorRanksByTypeSaveOutcome, HonorRanksFieldSink, HonorRanksSaveBlock, HonorRanksSaveOutcome,
    LoadedPlayerScriptFlag, PlayerAbilityBinaryField, PlayerAbilityBlobDecodeBlock,
    PlayerAbilityCreationSnapshot, PlayerAbilityLoadScalarSnapshot,
    PlayerAbilityQueryLoadFailure, PlayerAbilityQueryLoadOutcome, PlayerAbilityRowLoadFailure,
    PlayerAbilitySaveSnapshot, PlayerAbilityScalarSnapshot, PlayerAbilitySkill,
    PlayerBaseCreateOutcome, PlayerBaseDatabaseRow, PlayerBaseLoadFailure, PlayerBaseSaveSnapshot,
    PlayerCreateBlock, PlayerCreateOutcome, PlayerCreationBaseSnapshot, PlayerCreationSnapshot,
    PlayerDeleteOutcome, PlayerDeleteTimeBlock, PlayerFriendName, PlayerLoadFailure,
    PlayerQuestQueryLoadFailure, PlayerQuestQueryLoadOutcome, PlayerQuestSaveEntry,
    PlayerQuestSaveSnapshot, PlayerRanksStatBlock, PlayerRanksStatFailure, PlayerRanksStatOutcome,
    PlayerSaveBlock, PlayerSaveOutcome, PlayerSaveSnapshot, PlayerScriptFlagSnapshot, PlayerThing,
    RsPlayerNotice, RsPlayerOperation, RsPlayerOwner, RsPlayerSaveError,
};

pub(crate) type PlayerLoadBlock =
    GenericPlayerLoadBlock<GoodsLoadedAddonBlock, PlayerLoadedGoodsInsertBlock>;
pub(crate) type PlayerLoadOutcome =
    GenericPlayerLoadOutcome<GoodsLoadedAddonBlock, PlayerLoadedGoodsInsertBlock>;

const CREATE_PLAYER_BASE_PREFIX: &[u8] = b"INSERT INTO CSL_PLAYER_BASE (id,name,Account,levels,occupation,sex,Country,HEAD,\t\t\t\t\t HELM,BODY,GLOV,BOOT,WEAPON,BACK,\t\t\t\t\t HEADGEAR,FROCK,WING,MANTEAU,FAIRY,\t\t\t\t\t HelmLevel,BodyLevel,GlovLevel,BootLevel,WeaponLevel,BackLevel,\t\t\t\t\t HEADGEARLevel,FROCKLevel,WINGLevel,MANTEAULevel,FAIRYLevel,\t\t\t\t\t Region) \t\t\t\t VALUES (";
const SAVE_PLAYER_BASE_SQL: &str = "IF EXISTS (SELECT TOP 1 id FROM CSL_PLAYER_BASE WHERE id = @P30) BEGIN UPDATE TOP (1) CSL_PLAYER_BASE SET [Name] = @P1, [Levels] = @P2, [Occupation] = @P3, [Sex] = @P4, [Country] = @P5, [HEAD] = @P6, [HELM] = @P7, [BODY] = @P8, [GLOV] = @P9, [BOOT] = @P10, [WEAPON] = @P11, [BACK] = @P12, [HEADGEAR] = @P13, [FROCK] = @P14, [WING] = @P15, [MANTEAU] = @P16, [FAIRY] = @P17, [HelmLevel] = @P18, [BodyLevel] = @P19, [GlovLevel] = @P20, [BootLevel] = @P21, [WeaponLevel] = @P22, [BackLevel] = @P23, [HEADGEARLevel] = @P24, [FROCKLevel] = @P25, [WINGLevel] = @P26, [MANTEAULevel] = @P27, [FAIRYLevel] = @P28, [Region] = @P29 WHERE id = @P30; SELECT CAST(@@ROWCOUNT AS int) AS UpdatedRows END ELSE SELECT CAST(0 AS int) AS UpdatedRows";
const VALUE_GROUP_BREAK: &[u8] = b",\t\t\t\t\t ";

const HONOR_RANKS_SELECT_PREFIX: &str = "select * from CSL_HonorRanks where SortDate = '";
const HONOR_RANKS_INSERT_PREFIX: &str = "insert into CSL_HonorRanks(SortDate) values('";
const HONOR_RANKS_UPDATE_PREFIX: &str = "UPDATE TOP (1) CSL_HonorRanks SET ";
pub(crate) use nebokrai_realm::characters::honordb::{
    HONOR_RANK_BLOB_HEADER_SIZE, HONOR_RANK_CATEGORY_COUNT, HONOR_RANK_ENTRY_SIZE,
    HONOR_RANK_TYPE_COUNT, HonorRankDbEntry, HonorRanksBlobDecodeBlock,
    HonorRanksCopyTimeSnapshot, HonorRanksDbDataSnapshot, HonorRanksLoadBlock,
    HonorRanksLoadFailure, HonorRanksLoadOutcome, HonorRanksLoadSink, HonorRanksSavePeriod,
    HonorRanksType,
};

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

pub(crate) struct TiberiusRsPlayer {
    settings: WorldDatabaseSettings,
    notices: VecDeque<RsPlayerNotice>,
}

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
    G: DbGoodsOwner<
        CPlayer,
        AddonBlock = GoodsLoadedAddonBlock,
        InsertBlock = PlayerLoadedGoodsInsertBlock,
    >,
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
            PlayerLoadOutcome::ReturnedFalse(source) => {
                // Ошибки Tiberius могут содержать значение поля. В журнал выводим
                // этап, а исходный результат сохраняем в bool-контракте LoadData.
                let stage = match &source {
                    PlayerLoadFailure::Connection(_) => "connection",
                    PlayerLoadFailure::Ability(_) => "ability-query",
                    PlayerLoadFailure::Quest(_) => "quest",
                    PlayerLoadFailure::Goods(_) => "goods",
                    PlayerLoadFailure::Jjc(_) => "jjc",
                };
                tracing::warn!(player_id = player.get_id(), stage, "World: загрузка персонажа из БД отклонена");
                Ok(false)
            }
            PlayerLoadOutcome::BlockedMissingFact(source) => {
                match &source {
                    PlayerLoadBlock::Ability(error) => match error {
                        PlayerAbilityRowLoadFailure::Database { column, .. }
                        | PlayerAbilityRowLoadFailure::MissingRequiredValue { column }
                        | PlayerAbilityRowLoadFailure::NumericOutsideLegacyRange { column, .. } => {
                            let reason = match error {
                                PlayerAbilityRowLoadFailure::Database { .. } => "column-or-type",
                                PlayerAbilityRowLoadFailure::MissingRequiredValue { .. } => "null",
                                _ => "numeric-range",
                            };
                            tracing::warn!(player_id = player.get_id(), stage = "ability-scalar", column, reason, "World: не прочитано поле персонажа");
                        }
                        PlayerAbilityRowLoadFailure::Blob { field, source } => {
                            tracing::warn!(player_id = player.get_id(), stage = "ability-blob", ?field, error = ?source, "World: не разобрано составное поле персонажа");
                        }
                        PlayerAbilityRowLoadFailure::HonorTime(source) => {
                            tracing::warn!(player_id = player.get_id(), stage = "honor-time", error = ?source, "World: ошибка времени персонажа");
                        }
                    },
                    PlayerLoadBlock::Goods(source) => {
                        tracing::warn!(player_id = player.get_id(), stage = "goods", error = ?source, "World: не загружены предметы персонажа");
                    }
                }
                Err(TiberiusPlayerLoadDataBlock::Reconstruction(source))
            }
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PlayerAbilityScalarValue<'a> {
    I4(i32),
    Ui4(u32),
    Ui2(u16),
    Ui1(u8),
    VariantBool(bool),
    R4(f32),
    Int(i32),
    BStr(&'a [u8]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlayerAbilityScalarAssignment<'a> {
    pub(crate) field: PlayerAbilityScalarField,
    pub(crate) value: PlayerAbilityScalarValue<'a>,
}

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

pub(crate) fn encode_player_quest_data(snapshot: &PlayerQuestSaveSnapshot<'_>) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(snapshot.quests.len() * 3);
    for quest in snapshot.quests.values() {
        encoded.extend_from_slice(&quest.quest_id.to_le_bytes());
        encoded.push(quest.complete);
    }
    encoded
}

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

pub(crate) trait PlayerAbilityFieldSink {
    type Error;

    fn append_binary_field(
        &mut self,
        field: PlayerAbilityBinaryField,
        bytes: &[u8],
    ) -> Result<(), Self::Error>;
}

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

pub(crate) fn save_script_flag<S: PlayerAbilityFieldSink>(
    snapshot: &PlayerScriptFlagSnapshot<'_>,
    sink: &mut S,
) -> Result<(), S::Error> {
    let mut bytes = Vec::with_capacity(4);
    bytes.extend_from_slice(&snapshot.variable_num.to_le_bytes());
    bytes.extend_from_slice(snapshot.variable_data);
    sink.append_binary_field(PlayerAbilityBinaryField::ScriptFlag, &bytes)
}

pub(crate) fn save_state_field<S: PlayerAbilityFieldSink>(
    ex_states: &[u8],
    sink: &mut S,
) -> Result<(), S::Error> {
    sink.append_binary_field(PlayerAbilityBinaryField::State, ex_states)
}

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

pub(crate) fn load_skill_field(blob: &[u8]) -> Vec<PlayerAbilitySkill> {
    blob.chunks_exact(4)
        .map(|entry| PlayerAbilitySkill {
            id: u16::from_le_bytes(entry[0..2].try_into().expect("полный tagSkill")),
            level: u16::from_le_bytes(entry[2..4].try_into().expect("полный tagSkill")),
        })
        .collect()
}

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

pub(crate) fn load_state_field(blob: &[u8]) -> Option<Vec<u8>> {
    (!blob.is_empty()).then(|| blob.to_vec())
}

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

pub(crate) fn load_ci_qing_field(blob: &[u8]) -> BTreeSet<u32> {
    blob.chunks_exact(4)
        .map(|entry| u32::from_le_bytes(entry.try_into().expect("полный tattoo ID")))
        .collect()
}

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

pub(crate) fn load_quest_data(blob: &[u8]) -> Vec<PlayerQuestSaveEntry> {
    blob.chunks_exact(3)
        .map(|entry| PlayerQuestSaveEntry {
            quest_id: u16::from_le_bytes(entry[0..2].try_into().expect("полный quest value")),
            complete: entry[2],
        })
        .collect()
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
    let first_error = match get_value::<bool>(row, column) {
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
    // SQL float(53) хранит f64; игровая координата и wire-поле — f32.
    // SQL real уже хранит f32. Tiberius не выполняет это сужение сам.
    let value = get_value::<f64>(row, column)
        .map(|value| value.map(|value| value as f32))
        .or_else(|_| get_value::<f32>(row, column));
    match value {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(PlayerAbilityRowLoadFailure::MissingRequiredValue { column }),
        Err(source) => Err(PlayerAbilityRowLoadFailure::Database { column, source }),
    }
}

fn required_ability_ansi(
    row: &Row,
    column: &'static str,
) -> Result<Vec<u8>, PlayerAbilityRowLoadFailure> {
    match get_value::<&str>(row, column) {
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
    match get_value::<&[u8]>(row, column) {
        Ok(Some(value)) => Ok(value.to_vec()),
        Ok(None) => Ok(Vec::new()),
        Err(source) => Err(PlayerAbilityRowLoadFailure::Database { column, source }),
    }
}

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
    pub(crate) fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            settings: settings.clone(),
            notices: VecDeque::new(),
        }
    }

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
        let save_time = match get_value::<NaiveDateTime>(&row, "SaveTime") {
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
        let blob = match get_value::<&[u8]>(&row, "QuestData") {
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

impl nebokrai_realm::app::world_game_view::WorldRenameDbView for TiberiusRsPlayer {
    fn is_name_exist<'a>(
        &'a mut self,
        player_name: &'a [u8],
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + 'a>> {
        Box::pin(async move {
            RsPlayerOwner::<CPlayer>::is_name_exist(self, player_name, active_transaction).await
        })
    }
}

impl nebokrai_realm::app::world_game_view::WorldDeleteRoleDbView for TiberiusRsPlayer {
    fn get_player_country_by_id<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = u8> + 'a>> {
        Box::pin(async move {
            RsPlayerOwner::<CPlayer>::get_player_country_by_id(self, player_id, active_transaction)
                .await
        })
    }

    fn get_player_deletion_date<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = i32> + 'a>> {
        Box::pin(async move {
            RsPlayerOwner::<CPlayer>::get_player_deletion_date(self, player_id, active_transaction)
                .await
        })
    }

    fn get_player_name_by_id<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + 'a>> {
        Box::pin(async move {
            RsPlayerOwner::<CPlayer>::get_player_name_by_id(self, player_id, active_transaction)
                .await
        })
    }
}

impl nebokrai_realm::app::world_game_view::WorldCreateRoleDbView for TiberiusRsPlayer {
    fn get_player_count_in_cdkey<'a>(
        &'a mut self,
        account: &'a [u8],
        creation_count: u8,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<u8>> + 'a>> {
        Box::pin(async move {
            RsPlayerOwner::<CPlayer>::get_player_count_in_cdkey(
                self,
                account,
                creation_count,
                active_transaction,
            )
            .await
        })
    }

    fn is_name_exist<'a>(
        &'a mut self,
        player_name: &'a [u8],
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + 'a>> {
        Box::pin(async move {
            RsPlayerOwner::<CPlayer>::is_name_exist(self, player_name, active_transaction).await
        })
    }
}

impl RsPlayerOwner<CPlayer> for TiberiusRsPlayer {
    fn load_player<J, G, WeekDay>(
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
    ) -> impl std::future::Future<Output = GenericPlayerLoadOutcome<G::AddonBlock, G::InsertBlock>>
    where
        J: RsJjcSysOwner,
        G: DbGoodsOwner<CPlayer>,
        WeekDay: FnMut() -> u16,
    {
        async move {
        let mut standalone_connection;
        let active_transaction = match active_transaction {
            Some(active_transaction) => active_transaction,
            None => {
                standalone_connection = match self.settings.connect().await {
                    Ok(connection) => connection,
                    Err(source) => {
                        return GenericPlayerLoadOutcome::ReturnedFalse(
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
                return GenericPlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Ability(source));
            }
            PlayerAbilityQueryLoadOutcome::BlockedMalformed(source) => {
                return GenericPlayerLoadOutcome::BlockedMissingFact(GenericPlayerLoadBlock::Ability(source));
            }
        }

        match self
            .load_player_quest_data(player, Some(&mut *active_transaction))
            .await
        {
            PlayerQuestQueryLoadOutcome::ReturnedTrue { .. } => {}
            PlayerQuestQueryLoadOutcome::ReturnedFalse(source) => {
                return GenericPlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Quest(source));
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
            GenericGoodsLoadOutcome::ReturnedTrue { .. } => {}
            GenericGoodsLoadOutcome::ReturnedFalse(source) => {
                return GenericPlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Goods(source));
            }
            GenericGoodsLoadOutcome::BlockedMissingFact(source) => {
                return GenericPlayerLoadOutcome::BlockedMissingFact(GenericPlayerLoadBlock::Goods(source));
            }
        }

        match jjc_owner
            .load_jjc_data(player, Some(&mut *active_transaction))
            .await
        {
            PlayerJjcLoadOutcome::ReturnedTrue { .. } => GenericPlayerLoadOutcome::ReturnedTrue,
            PlayerJjcLoadOutcome::ReturnedFalse(source) => {
                GenericPlayerLoadOutcome::ReturnedFalse(PlayerLoadFailure::Jjc(source))
            }
        }
        }
    }

    fn get_player_count_in_db_by_cdkey(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Option<u8>> + Send {
        async move {
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
    }

    fn open_player_base_in_db(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Result<Vec<PlayerBaseDatabaseRow>, PlayerBaseLoadFailure>> + Send {
        async move {
        const EQUIPMENT_ID_FIELDS: [&str; 11] = [
            "HELM", "BODY", "GLOV", "BOOT", "WEAPON", "BACK", "Headgear", "Frock",
            "Wing", "Manteau", "Fairy",
        ];
        const EQUIPMENT_LEVEL_FIELDS: [&str; 11] = [
            "HelmLevel",
            "BodyLevel",
            "GlovLevel",
            "BootLevel",
            "WeaponLevel",
            "BackLevel",
            "HeadgearLevel",
            "FrockLevel",
            "WingLevel",
            "ManteauLevel",
            "FairyLevel",
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
            let name = match get_value::<&str>(&row, "Name") {
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
    }

    fn get_player_deletion_date(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = i32> + Send {
        async move {
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
        let deletion_date = match get_value::<NaiveDateTime>(&row, "DelDate") {
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
 // Оригинал `_mktime == -1` нормализовался в ноль до возврата.
        i32::try_from(local_midnight.timestamp()).unwrap_or(0)
        }
    }

    fn get_player_country_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = u8> + Send {
        async move {
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
    }

    fn get_player_name_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Vec<u8>> + Send {
        async move {
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
        match get_value::<&str>(&row, "Name") {
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
    }

    fn validate_player_id_in_cdkey(
        &mut self,
        account: &[u8],
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
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
        let mut query = Query::new("SELECT ID FROM csl_player_base WHERE Account=@P1");
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
    }

    fn is_name_exist(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
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
    }

    fn get_cd_key(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Vec<u8>> + Send {
        async move {
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
        match get_value::<&str>(&account_row, "Account") {
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
    }

    fn stat_ranks<Organizing: PlayerRankOrganizingLookup>(
        &mut self,
        ranks: &mut CPlayerRanks,
        organizing: &Organizing,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = PlayerRanksStatOutcome> {
        async move {
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
                    match get_value::<$type>(&row, $column) {
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
    }

    fn create_player<J: RsJjcSysOwner, G: DbGoodsOwner<CPlayer>>(
        &mut self,
        snapshot: Option<&PlayerCreationSnapshot<'_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> impl std::future::Future<Output = PlayerCreateOutcome> {
        async move {
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
    }

    fn save_player<J: RsJjcSysOwner, G: DbGoodsOwner<CPlayer>>(
        &mut self,
        snapshot: Option<&PlayerSaveSnapshot<'_, '_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> impl std::future::Future<Output = PlayerSaveOutcome> {
        async move {
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
    }

    fn create_player_base(
        &mut self,
        snapshot: &PlayerCreationBaseSnapshot,
        active_transaction: &mut WorldTdsClient,
    ) -> impl std::future::Future<Output = PlayerBaseCreateOutcome> + Send {
        async move {
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
    }

    fn save_player_base(
        &mut self,
        snapshot: Option<&PlayerBaseSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
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
    }

    fn create_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: &PlayerAbilityCreationSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
        jjc_owner: &mut J,
    ) -> impl std::future::Future<Output = bool> {
        async move {
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
    }

    fn save_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: Option<&PlayerAbilitySaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
    ) -> impl std::future::Future<Output = bool> {
        async move {
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
    }

    fn save_quest_data(
        &mut self,
        snapshot: Option<&PlayerQuestSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
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
    }

    fn restore_player(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
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
    }

    fn delete_player(
        &mut self,
        player_id: u32,
        deletion_time: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = PlayerDeleteOutcome> + Send {
        async move {
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
    }

    fn load_honor_ranks<S: HonorRanksLoadSink>(
        &mut self,
        sink: &mut S,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = HonorRanksLoadOutcome> {
        async move {
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
                let blob = match get_value::<&[u8]>(&row, rank_type.column_name()) {
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
    }

    fn insert_honor_ranks(
        &mut self,
        snapshot: &HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
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

    fn save_honor_ranks(
        &mut self,
        snapshot: &mut HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = HonorRanksSaveOutcome> + Send {
        async move {
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
    }

    fn pop_notice(&mut self) -> Option<RsPlayerNotice> {
        self.notices.pop_front()
    }
}

impl nebokrai_realm::characters::honorranks::HonorRanksDbOwner for TiberiusRsPlayer {
    fn load_honor_ranks<S: HonorRanksLoadSink>(
        &mut self,
        sink: &mut S,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = HonorRanksLoadOutcome> {
        RsPlayerOwner::load_honor_ranks(self, sink, active_transaction)
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
