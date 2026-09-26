//! Экипировка, свойства, улучшение и сброс боевой феи игрока (`CPlayer`-ядро
//! war-soul goods): property-pass `BFPropertyAdd(±1)` с формулами ±gear и
//! double-apply quirk, позиционный/безпозиционный Add/Remove/Take экипировки
//! контейнера боевой феи, полный player-side opcode `0x8FC2A` распределения
//! потенциала (масштаб ×10000, коэффициент 1.5), полный путь улучшения с
//! аудитом `0x60202/0x60203`, сброс потенциала предметом `ZHQLS01` и сброс
//! навыка предметами `ZHJNS01/02` поверх правил `battlefairy.rs`.
//!
//! Источник: точная пара `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Прежний переходный владелец — `src/gameserver/appserver/player.rs`
//! (`appserver/player.cpp/.h`). Тела перенесены буквально: построчная сверка
//! со старым файлом MATCH после нормализации `pub(crate)`→`pub`, путей типов
//! швов и представления ячейки контейнера позицией (объявленные швы ниже).
//! Доказательная база — разведка BF-семьи `player.rs` (порция №7, MATCH по
//! представителям); дизассембл-спотчеки этой порцией выполнены на том же
//! образе (capstone по `.text` точной пары):
//!
//! | представитель | RVA | статус |
//! |---|---|---|
//! | `AllocatePotential` | `0xFF480` | VERIFIED_DISASSEMBLY (спотчек порции): jump-table VA `0x50000C` по ключам `0x9B..0xA1`; ATTACK/SPRITE — f64 `1.5` (`0x653008`); добавки `0xB9`/`0xBA` (MAX_HP/MAX_MP) у ветвей `0xA1`/`0xA0`; четыре player-ветви ×(−1e-5, `0x653C68`)+`sub` ≡ `+0.00001`; per-call `PropertiesChanged` (virtual `+0x9C`) и хвост `0xBF918`; отключённая feature (байт `0xEF46C4`) — текст «8» (`ZHGS0008`) внутри каждой записи |
//! | обработчик `0x8FC2A` | `0x95F00..0x962C0` | VERIFIED_DISASSEMBLY (спотчек порции): `imul eax, eax, 0x2710` — клиентские очки масштабируются ×10000 до map-insert (first-wins), агрегат остаётся немасштабированным (signed `jg` по потенциалу `0xA3`); итерация std::map вызывает `AllocatePotential` по каждой записи; внешний `0xBF918` после цикла, при insufficient — пропуск |
//! | `BFPropertyAdd` | `0x1020E0` | VERIFIED_DISASSEMBLY (спотчек порции): ранний reject только ячейки `0xC`; десять addon-чтений (`0xCA/0xCB/0xCF/0xCC/0xCD/0xCE/0xD0/0xE3/0xC8/0xC9`) × delta; записи головного `0x9B/0x9C/0xA1/0x9E/0x9F/0xA0/0x9D/0xDA`, `0xB9` (+`0xCF`+`0xC8`), `0xBA` (+`0xCE`+`0xC9`); клампы current 0x99/0x9A по maximum 0xB9/0xBA; player-ветви с occupation-таблицами BSS (GlobeSetup); double-apply quirk подтверждён машинно: второй раунд SetMaxHp/SetStrength/SetIntelligence/SetDexterity (`0x42ACF0/0x42AD30/0x42AD90/0x42AD50`) после производных коэффициентов (`0x1029A7–0x102AEA`); хвост `PropertiesChanged`+`0xBF918` |
//! | reset reconcile tail | `0xFF031` | VERIFIED_DISASSEMBLY (спотчек порции): семь пар tracked→0 / property−=tracked в порядке `0xBB..0xC1`; recovered ((sprite+attack)·2/3 + blast+brave+agility+spiritualism+strength) — f64 `2/3` (`0x653C40`); потенциал `0xA3` += trunc; четыре вычитания из свойств игрока ×(+1e-5, `0x653C38`) ≡ `−0.00001` |
//! | upgrade player-tail (аудит) | `0x100530` | PARTIAL: якоря кадров `0x60203` событий 1/2 (сериализация target+4 gem: name/price/amount, затем money-поля игрока) и их event-гейты подтверждены спотчеком; полный построчный вывер тела улучшения этой порцией не переоткрывался — опора на разведку порции №7 (MATCH по представителям) |
//! | container queries | `0xFC9A0/0xFCAB0/0xFCBF0` | MATCH разведки порции №7 (typed-шов, владелец — контейнерная порция): `GetFailResult` ≡ filter(!=0).fold(4,min), `GetProbability` порядок 13,14,15,16,12 signed→clamp 0..=100, `GetUpgradePrice` запись поля только при наличии предмета ячейки 12 |
//!
//! Константная спот-сверка порции (та же пара образов): `1.5` подтверждена
//! (f64 `0x653008`, ветви ATTACK/SPRITE `AllocatePotential`); `0.00001`
//! подтверждена как отрицательная константа `-1e-5` с последующим `sub`
//! (`0x653C68`, оборот property/addon-путей) и положительная `1e-5` с `sub` у
//! reset-хвоста (`0x653C38`) — обе ≡ `f64::from(amount) * 0.00001` на
//! достижимых входах; double-apply quirk подтверждён машинно (строка выше).
//! Масштаб ×10000 подтверждён (`imul 0x2710` обработчика `0x8FC2A`).
//!
//! Честные оговорки (не переоткрываются порцией): (1) машинный guard
//! `BFPropertyAdd` отклоняет ровно ячейку `12`, zone-предикат сужает допуск
//! до восьми gear-позиций `0..=7` — тождественно на достижимых caller-ах:
//! container `validate_add_at` порождает `property_effect` только для gear
//! (снято в контейнерной модели порции №7); (2) машина усекает сумму
//! `current + amount*1.5` одним FISTP, zone — прибавку до сложения:
//! тождественно на домене opcode-пути (amount = очки×10000 чётное и
//! неотрицательное за gate потенциала); отдельные (не opcode) caller-ы
//! native `AllocatePotential` с иными amount этой порцией не исследованы;
//! (3) ulp-порядок FP-множителей `(amount×coeff)×k` против `(amount×k)×coeff`
//! в player-формулах не переустанавливался — значения сходятся на целых
//! прибавках домена; (4) полный вывер upgrade-тела — по разведке порции №7,
//! спотчек этой порции покрывает только аудит-якоря.
//!
//! Объявленные швы переноса (не расхождения). Все обращения к живым полям
//! прежнего `CPlayer` (equipment ячейка 10, `battle_fairy_container` + base,
//! packet-рюкзак, wallet, `CMoveShape` навыков, ID/регион/IP) типизированы
//! методами hub-трейта [`BattleFairyGearHost`], реализация — адаптер
//! `BattleFairyGearPlayerAdapter` в прежнем `appserver/player.rs`; имена
//! членов сохраняют исходную операцию, no-op/`None` при отсутствующем живом
//! предмете повторяют исходные `Option`-цепи (`expect`-узлы исходного тела
//! структурно недостижимы за его же гейтами). Ячейка контейнера представлена
//! позицией `u32`; валидация и enum `BattleFairyCell` остаются у владельца
//! контейнерной порции. `CGoods`, фабрика, сериализация `0xBF918` и записи
//! исходов контейнера/wallet — opaque associated-типы шва; zone-перечисления
//! эффектов параметризованы ими, старый пакет подставляет прежние типы
//! alias-ами без изменения имён/полей (потребители без правок). RNG
//! (`&mut dyn FnMut(i32) -> i32`) и setup-коэффициенты
//! (`GlobePlayerPropertyCoefficients`, Shared) передаются дословно.
//! Report-структуры с полем прежнего `GameEffectJournal`, конверт эффектов в
//! `GameEffect` и упорядоченная доставка остаются у прежнего владельца;
//! отправка `0xBF918` — существующий sender-шов порции №6b/транспортной
//! волны, здесь не дублируется.
//!
//! Зеркальные правила-мелочи: скалярный clamp `i32::MAX` боевых полей
//! дублирован приватной копией (canonical hub-копия — `clamp_combat_scalar`
//! прежнего `CPlayer` до переноса setter-семьи; goods-touching варианты
//! `add_battle_fairy_addon`/`clamp_battle_fairy_current` остаются у прежнего
//! владельца, их разделяет соседний блок refresh-экипировки и адаптер шва).

use std::collections::BTreeMap;

use nebokrai_shared::resources::GlobePlayerPropertyCoefficients;
use nebokrai_shared::values::CGuid;

use crate::combat::PlayerCombatProperties;
use crate::content::goods::{
    GAP_BF_AGILITY, GAP_BF_AGILITY_POTENTIAL, GAP_BF_ALL_SKILL, GAP_BF_ATTACK,
    GAP_BF_ATTACK_POTENTIAL, GAP_BF_BATTLE_FAIRY, GAP_BF_BLAST, GAP_BF_BLAST_POTENTIAL,
    GAP_BF_BRAVE, GAP_BF_BRAVE_POTENTIAL, GAP_BF_CUT_HURT_SCALE, GAP_BF_EARTH, GAP_BF_EARTH_SKILL,
    GAP_BF_HP, GAP_BF_HUOXIESHU_SKILL, GAP_BF_LINGZHISHU_SKILL, GAP_BF_MAN, GAP_BF_MAN_SKILL,
    GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP, GAP_BF_POTENTIAL, GAP_BF_SKY, GAP_BF_SKY_SKILL,
    GAP_BF_SPRITE, GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_POTENTIAL,
    GAP_BF_STRENGH, GAP_BF_STRENGH_POTENTIAL, GAP_BF_WEAPON_LEVEL, GAP_GEM_LEVEL,
};
use crate::regions::ShapeIdentity;

use super::battlefairy::{
    BattleFairyResetItemChange, BattleFairyResetItemLookup, BattleFairyResetPreflight,
    BattleFairySkillProperty, EQUIPPED_SKILL_PROPERTIES, battle_fairy_reset_item,
    battle_fairy_reset_item_change, battle_fairy_reset_preflight, battle_fairy_reset_slot,
    battle_fairy_skill_entry, battle_fairy_skill_id, select_battle_fairy_reset_skill,
    write_battle_fairy_reset_skill,
};

/// Позиция ячейки боевого оружия контейнера (enum владельца контейнерной
/// порции, `repr(u32)` Equipment).
const BATTLE_FAIRY_CELL_EQUIPMENT: u32 = 12;
/// Позиции gem-ячеек улучшения (`GemBase/GemOne/GemTwo/GemThree`).
const BATTLE_FAIRY_CELL_GEM_BASE: u32 = 13;
const BATTLE_FAIRY_CELL_GEM_ONE: u32 = 14;
const BATTLE_FAIRY_CELL_GEM_TWO: u32 = 15;
const BATTLE_FAIRY_CELL_GEM_THREE: u32 = 16;

/// Сетевой кадр установленного навыка боевой феи.
pub const BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE: u32 = 0x0b_f71d;
/// Сетевой кадр снятого навыка боевой феи.
pub const BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE: u32 = 0x0b_f71e;
const BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING: &str = "ZHGS0022";

/// Дублированный clamp боевых полей `i32::MAX` (см. шапку, зеркальные правила).
const LEGACY_COMBAT_MAXIMUM: u32 = i32::MAX as u32;

/// Addon-значения размещаемого предмета, читаемые до property-pass.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BattleFairyGearAddons {
    pub attack: i32,
    pub sprite: i32,
    pub strength: i32,
    pub brave: i32,
    pub agility: i32,
    pub spiritualism: i32,
    pub blast: i32,
    pub cut_hurt: i32,
    pub life: i32,
    pub mana: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairyEquipmentMutationOutcome<AddOutcome, Removal> {
    Added(AddOutcome),
    Removed(Removal),
    MissingGoods,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairyEquipmentMutationEffect<Update> {
    PropertiesChanged { player_id: i32 },
    BattleFairyUpdated(Update),
}

/// Результат player-tail экипировки до конверта в прежний журнал эффектов.
#[must_use = "resolution сохраняет ранние property effects и исход контейнера"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyEquipmentMutationResolution<Host: BattleFairyGearHost + ?Sized> {
    pub cell_position: Option<u32>,
    pub property_applied: bool,
    pub outcome: BattleFairyEquipmentMutationOutcome<Host::AddOutcome, Host::Removal>,
    pub effects: Vec<BattleFairyEquipmentMutationEffect<Host::GoodsUpdate>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyPotentialAllocationOutcome {
    MissingHeadgear,
    InvalidHeadgear,
    AggregateInsufficient,
    Processed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairyPotentialAllocationEffect<Update> {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(Update),
}

/// Результат распределения потенциала до конверта в прежний журнал.
#[must_use = "allocation resolution сохраняет ordered player и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyPotentialAllocationResolution<Host: BattleFairyGearHost + ?Sized> {
    pub outcome: BattleFairyPotentialAllocationOutcome,
    pub effects: Vec<BattleFairyPotentialAllocationEffect<Host::GoodsUpdate>>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BattleFairyUpgradeLogGates {
    pub success: bool,
    pub failure: bool,
    pub lost_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyUpgradeOutcome {
    MissingRegion,
    InsufficientMoney,
    InvalidEquipment,
    MissingBaseGem,
    GemLevelMismatch,
    MaximumLevel,
    Succeeded,
    FailedKept,
    FailedDowngraded,
    FailedReset,
    FailedDestroyed,
    ConsumptionStopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyUpgradeGoodsSnapshot {
    pub identity: ShapeIdentity,
    pub name: Vec<u8>,
    pub price: u32,
    pub amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairyUpgradePlayerSnapshot {
    pub pk_count: u16,
    pub money: u32,
    pub depot_money: u32,
    pub region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub client_ip: u32,
}

/// Списание валюты кошелька игрока (`PlayerMoneyDecrease` прежнего владельца).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyMoneyChange<Outcome> {
    pub previous: u32,
    pub current: u32,
    pub outcome: Outcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairyUpgradeEffect<Update, CurrencyOutcome, ConsumedGem, Removal> {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        format_value: Option<u32>,
    },
    MoneyChanged {
        player_id: i32,
        previous: u32,
        current: u32,
        outcome: CurrencyOutcome,
    },
    GoodsUpdated(Update),
    GemConsumed {
        player_id: i32,
        consumed: ConsumedGem,
    },
    TargetDeleted {
        player_id: i32,
        goods: BattleFairyUpgradeGoodsSnapshot,
        position: u32,
        removal: Removal,
    },
    Audit {
        message_type: u32,
        event: u8,
        player_id: i32,
        player: BattleFairyUpgradePlayerSnapshot,
        target: BattleFairyUpgradeGoodsSnapshot,
        gems: [Option<BattleFairyUpgradeGoodsSnapshot>; 4],
    },
}

/// Результат улучшения до конверта в прежний журнал.
#[must_use = "upgrade resolution содержит wallet, ownership и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyUpgradeResolution<Host: BattleFairyGearHost + ?Sized> {
    pub outcome: BattleFairyUpgradeOutcome,
    pub effects: Vec<
        BattleFairyUpgradeEffect<
            Host::GoodsUpdate,
            Host::CurrencyOutcome,
            Host::ConsumedGem,
            Host::Removal,
        >,
    >,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyPotentialResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairyPotentialResetEffect<Update, Removal> {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: ShapeIdentity,
        position: Option<u32>,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<Removal>,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(Update),
}

/// Результат сброса потенциала до конверта в прежний журнал.
#[must_use = "reset resolution содержит packet ownership и player/network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyPotentialResetResolution<Host: BattleFairyGearHost + ?Sized> {
    pub outcome: BattleFairyPotentialResetOutcome,
    pub effects: Vec<BattleFairyPotentialResetEffect<Host::GoodsUpdate, Host::Removal>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairySkillResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    InvalidPosition,
    SelectedSkillUnavailable,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairySkillRemoved {
    pub message_type: u32,
    pub player_id: i32,
    pub skill_id: u32,
    pub skill_name: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairySkillAdded {
    pub message_type: u32,
    pub player_id: i32,
    pub skill_id: u32,
    pub skill_level: i32,
    pub skill_type: u32,
    pub skill_name: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairySkillResetEffect<Update, Removal> {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: ShapeIdentity,
        position: Option<u32>,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<Removal>,
    },
    SkillRemoved(BattleFairySkillRemoved),
    SkillAdded(BattleFairySkillAdded),
    SelectedSkillLearned(BattleFairySkillAdded),
    GoodsUpdated(Update),
}

/// Результат сброса навыка до конверта в прежний журнал.
#[must_use = "skill reset resolution содержит packet, skill-state и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairySkillResetResolution<Host: BattleFairyGearHost + ?Sized> {
    pub outcome: BattleFairySkillResetOutcome,
    pub effects: Vec<BattleFairySkillResetEffect<Host::GoodsUpdate, Host::Removal>>,
}

/// Переходные фасады прежнего владельца `CPlayer`, открывающие перенесённым
/// телам только прежние обращения; имена членов сохраняют исходную операцию.
/// Реализация — адаптер `BattleFairyGearPlayerAdapter` в `appserver/player.rs`.
pub trait BattleFairyGearHost {
    /// Живой предмет прежнего пакета (`CGoods`).
    type Goods;

    /// Запись обновления предмета, собранная прежним владельцем
    /// (`BattleFairyDefaultGoodsUpdate`; кадр `0xBF918` собирается им же).
    type GoodsUpdate;

    /// Исход positional/automatic add контейнера боевой феи.
    type AddOutcome;

    /// Исход отделения предмета volume-контейнера.
    type Removal;

    /// Исход уменьшения валюты кошелька игрока.
    type CurrencyOutcome;

    /// Запись о расходе камня улучшения контейнером боевой феи.
    type ConsumedGem: Clone;

    // --- Головной предмет (equipment ячейка 10 прежнего CPlayer). ---

    /// Чтение addon-значения; `None` повторяет отсутствующий предмет.
    fn headgear_addon(&self, property: i32, index: u32) -> Option<i32>;

    /// `add_battle_fairy_addon`: wrapping-сложение с хранимым значением.
    fn add_headgear_addon(&mut self, property: i32, delta: i32);

    /// Ядро записи addon-значения; stored-liveness отбрасывается, как в
    /// исходных `let _stored =`/`let _ =` перенесённых тел.
    fn set_headgear_addon(&mut self, property: i32, index: u32, value: i32);

    /// `clamp_battle_fairy_current`: current не превышает maximum.
    fn clamp_headgear_current(&mut self, current_property: i32, maximum_property: i32);

    /// Identity и old-client payload головного предмета одним доступом;
    /// wire-сборка `0xBF918` — шов прежнего владельца.
    fn headgear_goods_update(&mut self) -> Option<Self::GoodsUpdate>;

    // --- Боевые свойства игрока. ---

    fn combat_properties(&mut self) -> &mut PlayerCombatProperties;

    /// Регионное присутствие (`server_region_id.is_some()` прежнего тела).
    fn server_region_present(&self) -> bool;

    // --- Контейнер боевой феи и его base-хранилище (typed-чтения). ---

    /// `BattleFairyGearAddons::read` размещаемого предмета.
    fn gear_addons(&self, goods: &Self::Goods) -> BattleFairyGearAddons;

    /// Catalog BF equip-place безпозиционного add: валидная позиционная
    /// ячейка либо `None` (цепочка `from_position` прежнего владельца).
    fn battle_fairy_equip_cell(&self, goods: &Self::Goods) -> Option<u32>;

    /// Ранний `BFPropertyAdd(+1)` до base-модификации: (позиция, delta).
    fn property_effect_before_add(
        &self,
        cell_position: u32,
        goods: &Self::Goods,
    ) -> Option<(u32, i32)>;

    fn battle_fairy_add_at(
        &mut self,
        cell_position: u32,
        incoming: &mut Option<Self::Goods>,
        owner_progress_allows: bool,
    ) -> Self::AddOutcome;

    fn battle_fairy_add(
        &mut self,
        incoming: &mut Option<Self::Goods>,
        owner_progress_allows: bool,
    ) -> Self::AddOutcome;

    /// Позиция предмета в base-хранилище по GUID.
    fn battle_fairy_goods_position(&self, ex_id: CGuid) -> Option<u32>;

    /// Addon-значения предмета ячейки до отделения (`read` на base-предмете).
    fn battle_fairy_cell_addons(&self, position: u32) -> Option<BattleFairyGearAddons>;

    fn battle_fairy_remove_goods(&mut self, ex_id: CGuid) -> Option<Self::Removal>;

    /// Количество предмета ячейки; `None` повторяет пустую ячейку.
    fn battle_fairy_cell_amount(&self, position: u32) -> Option<u32>;

    /// GUID предмета ячейки; `None` повторяет пустую ячейку.
    fn battle_fairy_cell_ex_id(&self, position: u32) -> Option<CGuid>;

    fn battle_fairy_take_goods(
        &mut self,
        position: u32,
        amount: u32,
        create_goods: &mut dyn FnMut(u32) -> Option<Self::Goods>,
    ) -> Option<Self::Removal>;

    /// Identity и old-client payload предмета ячейки одним доступом.
    fn battle_fairy_cell_goods_update(&mut self, position: u32) -> Option<Self::GoodsUpdate>;

    // --- Улучшение экипировки боевой феи. ---

    fn battle_fairy_upgrade_price(&mut self) -> u32;

    fn currency_amount(&self) -> u32;

    fn battle_fairy_cell_present(&self, position: u32) -> bool;

    /// `can_battle_fairy_equipment_upgrade` предмета ячейки 12; вызывается
    /// ветвью только после подтверждённого presence.
    fn battle_fairy_equipment_can_upgrade(&self) -> bool;

    fn battle_fairy_cell_addon(&self, position: u32, property: i32, index: u32) -> Option<i32>;

    /// `BattleFairyUpgradeGoodsSnapshot::capture` предмета ячейки.
    fn battle_fairy_cell_snapshot(&self, position: u32) -> Option<BattleFairyUpgradeGoodsSnapshot>;

    fn battle_fairy_probability(&self) -> u32;

    /// `CPlayer::DecreaseMoney` кошелька: previous/current после списания и
    /// wallet outcome.
    fn decrease_money(&mut self, amount: u32) -> BattleFairyMoneyChange<Self::CurrencyOutcome>;

    /// Аудит-снимок игрока после списания (pk/money/depot/region/tile/IP).
    fn audit_player_snapshot(&self) -> BattleFairyUpgradePlayerSnapshot;

    fn battle_fairy_success_result(&self, random: &mut dyn FnMut(i32) -> i32) -> u32;

    /// Запись уровня предмета ячейки через фабрику (`Upgrade` ядро);
    /// upgraded-liveness отбрасывается, при пустой ячейке — no-op.
    fn set_battle_fairy_cell_goods_level(&mut self, position: u32, level: i32);

    fn battle_fairy_fail_result(&self) -> u32;

    /// Отделение экипировки улучшения (ячейка 12) при разрушении.
    fn battle_fairy_delete_upgrade_target(&mut self) -> Option<Self::Removal>;

    fn battle_fairy_consume_upgrade_gem(&mut self, cell_position: u32)
    -> Option<Self::ConsumedGem>;

    /// Проекция полей записи расхода (removed, previous_amount, remaining_amount).
    fn consumed_gem_facts(gem: &Self::ConsumedGem) -> (bool, u32, u32);

    // --- Пакет игрока (рюкзак) для предметов сброса. ---

    /// Native lookup по original name: factory ID и обход до первого
    /// совпадения, identity и количество.
    fn find_reset_goods(&self, name: &[u8]) -> Option<(ShapeIdentity, u32)>;

    fn packet_goods_position(&self, ex_id: CGuid) -> Option<u32>;

    fn packet_remove_goods(&mut self, ex_id: CGuid) -> Option<Self::Removal>;

    /// Установка остатка стека; `false` повторяет отказ живого доступа.
    fn packet_set_goods_amount(&mut self, position: u32, remaining: u32) -> bool;

    // --- Навыки боевой феи игрока (`CMoveShape` прежнего владельца). ---

    /// `DelWarSoulSkillInPlayer`-ядро `delete_skill`; deleted-liveness
    /// отбрасывается, как в исходных `let _deleted =`.
    fn delete_war_soul_skill(&mut self, skill_id: u32);

    /// Имя пережившего удаление навыка для кадра снятия: внешний `None`
    /// повторяет отсутствующий после отказа category lookup навык.
    fn war_soul_removed_skill_name(&self, skill_id: u32) -> Option<Option<Vec<u8>>>;

    /// `AddWarSoulSkillToPalyer`-ядро `add_skill`; added-liveness
    /// отбрасывается, как в исходных `let _added =`.
    fn add_war_soul_skill(&mut self, skill_id: u32, level: i32);

    /// Поля снимка установленного навыка (level, type, name); внешний `None`
    /// повторяет отсутствующий навык.
    fn war_soul_added_skill_facts(&self, skill_id: u32) -> Option<(i32, u32, Option<Vec<u8>>)>;
}

/// Числовой ключ addon-свойства навыковой позиции головного предмета.
pub const fn battle_fairy_skill_property_key(property: BattleFairySkillProperty) -> i32 {
    match property {
        BattleFairySkillProperty::RequestedOffset(offset) => GAP_BF_MAN.wrapping_add(offset),
        BattleFairySkillProperty::Sky => GAP_BF_SKY,
        BattleFairySkillProperty::Earth => GAP_BF_EARTH,
        BattleFairySkillProperty::Man => GAP_BF_MAN,
        BattleFairySkillProperty::SkySkill => GAP_BF_SKY_SKILL,
        BattleFairySkillProperty::EarthSkill => GAP_BF_EARTH_SKILL,
        BattleFairySkillProperty::ManSkill => GAP_BF_MAN_SKILL,
        BattleFairySkillProperty::AllSkill => GAP_BF_ALL_SKILL,
        BattleFairySkillProperty::Huoxieshu => GAP_BF_HUOXIESHU_SKILL,
        BattleFairySkillProperty::Lingzhishu => GAP_BF_LINGZHISHU_SKILL,
    }
}

/// Ровно восемь gear-позиций (`0..=7`) запускают property-pass; машинный
/// guard отклоняет только ячейку `12` (шапка, оговорка 1).
const fn is_battle_fairy_property_cell(cell_position: u32) -> bool {
    cell_position < 8
}

const fn clamp_combat_scalar(value: u32) -> u32 {
    if LEGACY_COMBAT_MAXIMUM < value {
        LEGACY_COMBAT_MAXIMUM
    } else {
        value
    }
}

/// `BFPropertyAdd/AllocatePotential` сначала FISTP-усекают delta, затем
/// выполняют целочисленное сложение/вычитание с текущим свойством.
fn add_battle_fairy_u32(current: u32, delta: f64) -> u32 {
    let next = i64::from(current) + delta.trunc() as i64;
    if next < 0 {
        0
    } else {
        clamp_combat_scalar(next as u32)
    }
}

fn add_battle_fairy_u16(current: u16, delta: f64) -> u16 {
    let next = i64::from(current) + delta.trunc() as i64;
    if next < 0 { 0 } else { next as u16 }
}

fn add_battle_fairy_i32(current: i32, delta: f64) -> i32 {
    let next = i64::from(current) + delta.trunc() as i64;
    if next < 0 { 0 } else { next as i32 }
}

/// Чтение addon-свойства головного предмета через шов; отсутствующий предмет
/// структурно недостижим за гейтами тела (исходный `expect`/`?` покрыт выше).
fn headgear_property_or_default<Host: BattleFairyGearHost + ?Sized>(
    host: &Host,
    property: i32,
    index: u32,
) -> i32 {
    host.headgear_addon(property, index).unwrap_or_default()
}

/// Чтение навыкового свойства головного предмета по enum-позиции правил
/// `battlefairy.rs` (числовой ключ вычисляется здесь, живое чтение — у шва).
fn headgear_skill_property_or_default<Host: BattleFairyGearHost + ?Sized>(
    host: &Host,
    property: BattleFairySkillProperty,
    index: u32,
) -> i32 {
    headgear_property_or_default(host, battle_fairy_skill_property_key(property), index)
}

/// Обновление боевого духа `GetWarSoulGoods`: только головной предмет с
/// единичным флагом боевой феи.
fn war_soul_goods_update<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
) -> Option<Host::GoodsUpdate> {
    if host.headgear_addon(GAP_BF_BATTLE_FAIRY, 1) != Some(1) {
        return None;
    }
    host.headgear_goods_update()
}

/// Property-pass `BFPropertyAdd(±delta)`: addon-перенос в головной предмет и
/// боевые свойства игрока включая double-apply quirk четырёх основных
/// значений после производных коэффициентов.
pub fn apply_battle_fairy_property<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    cell_position: u32,
    addons: BattleFairyGearAddons,
    delta: i32,
    occupation: u8,
    coefficients: GlobePlayerPropertyCoefficients,
) -> Option<Host::GoodsUpdate> {
    if delta == 0 || !is_battle_fairy_property_cell(cell_position) {
        return None;
    }
    let occupation = usize::from(occupation).min(2);
    if host.headgear_addon(GAP_BF_BATTLE_FAIRY, 1) != Some(1) {
        return None;
    }

    for (source, target) in [
        (addons.attack, GAP_BF_ATTACK),
        (addons.sprite, GAP_BF_SPRITE),
        (addons.strength, GAP_BF_STRENGH),
        (addons.brave, GAP_BF_BRAVE),
        (addons.agility, GAP_BF_AGILITY),
        (addons.spiritualism, GAP_BF_SPRITUALISM),
        (addons.blast, GAP_BF_BLAST),
        (addons.cut_hurt, GAP_BF_CUT_HURT_SCALE),
    ] {
        host.add_headgear_addon(target, source.wrapping_mul(delta));
    }
    host.add_headgear_addon(
        GAP_BF_MAX_HP,
        addons
            .strength
            .wrapping_add(addons.life)
            .wrapping_mul(delta),
    );
    host.add_headgear_addon(
        GAP_BF_MAX_MP,
        addons
            .spiritualism
            .wrapping_add(addons.mana)
            .wrapping_mul(delta),
    );
    host.clamp_headgear_current(GAP_BF_HP, GAP_BF_MAX_HP);
    host.clamp_headgear_current(GAP_BF_MP, GAP_BF_MAX_MP);

    let strength = f64::from(addons.strength) * f64::from(delta) * 0.00001;
    let brave = f64::from(addons.brave) * f64::from(delta) * 0.00001;
    let agility = f64::from(addons.agility) * f64::from(delta) * 0.00001;
    let spiritualism = f64::from(addons.spiritualism) * f64::from(delta) * 0.00001;
    let combat = host.combat_properties();
    combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
    combat.strength = add_battle_fairy_u32(combat.strength, brave);
    combat.maximum_attack = add_battle_fairy_u32(
        combat.maximum_attack,
        brave * f64::from(coefficients.str_to_max_attack[occupation]),
    );
    combat.burden = add_battle_fairy_u16(
        combat.burden,
        brave * f64::from(coefficients.str_to_burden[occupation]),
    );
    combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);
    combat.minimum_attack = add_battle_fairy_u32(
        combat.minimum_attack,
        agility * f64::from(coefficients.dex_to_min_attack[occupation]),
    );
    combat.reank = add_battle_fairy_u16(
        combat.reank,
        agility * f64::from(coefficients.dex_to_stiff[occupation]),
    );
    combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
    combat.element_modify = add_battle_fairy_i32(
        combat.element_modify,
        spiritualism * f64::from(coefficients.int_to_element[occupation]),
    );
    combat.maximum_mp = add_battle_fairy_u32(
        combat.maximum_mp,
        spiritualism * f64::from(coefficients.int_to_max_mp[occupation]),
    );
    combat.element_resistance = add_battle_fairy_u32(
        combat.element_resistance,
        spiritualism * f64::from(coefficients.int_to_resistant[occupation]),
    );

    // Подтверждённый RU quirk: четыре основных значения применяются
    // повторно после производных коэффициентов.
    combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
    combat.strength = add_battle_fairy_u32(combat.strength, brave);
    combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
    combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);

    host.headgear_goods_update()
}

/// Exact positional `CBattleFairyContainer::Add`: для gear-ячеек
/// `BFPropertyAdd(+1)` является ранним partial effect и сохраняется даже
/// если base storage затем отвергнет товар.
pub fn add_battle_fairy_goods<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    cell_position: u32,
    incoming: &mut Option<Host::Goods>,
    occupation: u8,
    coefficients: GlobePlayerPropertyCoefficients,
    owner_progress_allows: bool,
) -> BattleFairyEquipmentMutationResolution<Host> {
    let early_property = incoming.as_ref().and_then(|goods| {
        host.property_effect_before_add(cell_position, goods)
            .map(|(cell, delta)| (cell, delta, host.gear_addons(goods)))
    });
    let mut property_applied = false;
    let mut effects = Vec::new();
    if let Some((cell, delta, addons)) = early_property
        && let Some(update) =
            apply_battle_fairy_property(host, cell, addons, delta, occupation, coefficients)
    {
        property_applied = true;
        effects.push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
        effects.push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
            update,
        ));
    }
    let outcome = host.battle_fairy_add_at(cell_position, incoming, owner_progress_allows);
    BattleFairyEquipmentMutationResolution {
        cell_position: Some(cell_position),
        property_applied,
        outcome: BattleFairyEquipmentMutationOutcome::Added(outcome),
        effects,
    }
}

/// Безпозиционный overload сначала читает catalog BF equip-place. Только
/// валидная колонка достигает player property-tail; все typed reject-и
/// остаются у container owner-а без выдуманного размещения.
pub fn add_battle_fairy_goods_auto<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    incoming: &mut Option<Host::Goods>,
    occupation: u8,
    coefficients: GlobePlayerPropertyCoefficients,
    owner_progress_allows: bool,
) -> BattleFairyEquipmentMutationResolution<Host> {
    let cell = incoming
        .as_ref()
        .and_then(|goods| host.battle_fairy_equip_cell(goods));
    if let Some(cell) = cell {
        return add_battle_fairy_goods(
            host,
            player_id,
            cell,
            incoming,
            occupation,
            coefficients,
            owner_progress_allows,
        );
    }
    let outcome = host.battle_fairy_add(incoming, owner_progress_allows);
    BattleFairyEquipmentMutationResolution {
        cell_position: None,
        property_applied: false,
        outcome: BattleFairyEquipmentMutationOutcome::Added(outcome),
        effects: Vec::new(),
    }
}

/// Exact `Remove`: base container отделяет goods до `BFPropertyAdd(-1)`;
/// успешный property path сериализует battle fairy дважды — один раз в
/// `BFPropertyAdd`, затем ещё раз в override `Remove`.
pub fn remove_battle_fairy_goods<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    ex_id: CGuid,
    occupation: u8,
    coefficients: GlobePlayerPropertyCoefficients,
) -> BattleFairyEquipmentMutationResolution<Host> {
    let position = host.battle_fairy_goods_position(ex_id);
    let addons = position.and_then(|position| host.battle_fairy_cell_addons(position));
    let Some(outcome) = host.battle_fairy_remove_goods(ex_id) else {
        return BattleFairyEquipmentMutationResolution {
            cell_position: position,
            property_applied: false,
            outcome: BattleFairyEquipmentMutationOutcome::MissingGoods,
            effects: Vec::new(),
        };
    };
    let mut resolution = BattleFairyEquipmentMutationResolution {
        cell_position: position,
        property_applied: false,
        outcome: BattleFairyEquipmentMutationOutcome::Removed(outcome),
        effects: Vec::new(),
    };
    if let (Some(cell), Some(addons)) = (position, addons)
        && let Some(first_update) =
            apply_battle_fairy_property(host, cell, addons, -1, occupation, coefficients)
    {
        resolution.property_applied = true;
        resolution
            .effects
            .push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
        resolution
            .effects
            .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                first_update,
            ));
        if let Some(battle_fairy) = war_soul_goods_update(host) {
            resolution
                .effects
                .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                    battle_fairy,
                ));
        }
    }
    resolution
}

/// Positional `Remove(position, amount)` использует полный player-tail
/// для whole goods. Partial stack remove относится к material/gem cells и
/// не запускает `BFPropertyAdd(-1)`, пока исходный slot остаётся занят.
pub fn take_battle_fairy_goods<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    cell_position: u32,
    amount: u32,
    occupation: u8,
    coefficients: GlobePlayerPropertyCoefficients,
    create_goods: &mut dyn FnMut(u32) -> Option<Host::Goods>,
) -> BattleFairyEquipmentMutationResolution<Host> {
    let Some(current_amount) = host.battle_fairy_cell_amount(cell_position) else {
        return BattleFairyEquipmentMutationResolution {
            cell_position: Some(cell_position),
            property_applied: false,
            outcome: BattleFairyEquipmentMutationOutcome::MissingGoods,
            effects: Vec::new(),
        };
    };
    if current_amount == amount {
        let Some(ex_id) = host.battle_fairy_cell_ex_id(cell_position) else {
            return BattleFairyEquipmentMutationResolution {
                cell_position: Some(cell_position),
                property_applied: false,
                outcome: BattleFairyEquipmentMutationOutcome::MissingGoods,
                effects: Vec::new(),
            };
        };
        return remove_battle_fairy_goods(host, player_id, ex_id, occupation, coefficients);
    }
    let outcome = host.battle_fairy_take_goods(cell_position, amount, create_goods);
    BattleFairyEquipmentMutationResolution {
        cell_position: Some(cell_position),
        property_applied: false,
        outcome: outcome.map_or(
            BattleFairyEquipmentMutationOutcome::MissingGoods,
            BattleFairyEquipmentMutationOutcome::Removed,
        ),
        effects: Vec::new(),
    }
}

/// Полный player-side opcode `0x8FC2A`. `allocations` содержат пары
/// property/client-points прямо из packet-а: legacy outer caller суммирует
/// unscaled points, но передаёт каждому `AllocatePotential` wrapping
/// `points * 10000`. `std::map::insert` сохраняет первую запись ключа.
pub fn allocate_battle_fairy_potential<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    battle_fairy_enabled: bool,
    allocations: &[(i32, i32)],
    occupation: u8,
    coefficients: GlobePlayerPropertyCoefficients,
) -> BattleFairyPotentialAllocationResolution<Host> {
    let aggregate_client_points = allocations
        .iter()
        .fold(0i32, |total, (_, points)| total.wrapping_add(*points));
    let mut resolution = BattleFairyPotentialAllocationResolution {
        outcome: BattleFairyPotentialAllocationOutcome::MissingHeadgear,
        effects: Vec::new(),
    };
    let Some(battle_fairy_flag) = host.headgear_addon(GAP_BF_BATTLE_FAIRY, 1) else {
        return resolution;
    };
    if battle_fairy_flag != 1 {
        resolution.outcome = BattleFairyPotentialAllocationOutcome::InvalidHeadgear;
        resolution
            .effects
            .push(BattleFairyPotentialAllocationEffect::Notification {
                player_id,
                string_id: "ZHGS0009",
                color: 0xffff_ffff,
            });
        return resolution;
    }
    if headgear_property_or_default(host, GAP_BF_POTENTIAL, 1).wrapping_sub(aggregate_client_points)
        < 0
    {
        resolution.outcome = BattleFairyPotentialAllocationOutcome::AggregateInsufficient;
        return resolution;
    }

    let mut ordered = BTreeMap::new();
    for &(property, points) in allocations {
        ordered.entry(property).or_insert(points);
    }
    for (property, points) in ordered {
        if !battle_fairy_enabled {
            resolution
                .effects
                .push(BattleFairyPotentialAllocationEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            continue;
        }
        let amount = points.wrapping_mul(10_000);
        allocate_one_battle_fairy_potential(host, occupation, property, amount, coefficients);
        tracing::trace!(
            player_id,
            property,
            points,
            "свойство потенциала боевой феи обработано"
        );
        resolution
            .effects
            .push(BattleFairyPotentialAllocationEffect::PropertiesChanged { player_id });
        if let Some(goods) = war_soul_goods_update(host) {
            resolution
                .effects
                .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(goods));
        }
    }

    // Outer goods-message сериализует headgear ещё раз независимо от
    // feature-disabled/unknown-property результата внутренних вызовов.
    if let Some(goods) = war_soul_goods_update(host) {
        resolution
            .effects
            .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(goods));
    }
    resolution.outcome = BattleFairyPotentialAllocationOutcome::Processed;
    resolution
}

fn allocate_one_battle_fairy_potential<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    occupation: u8,
    property: i32,
    amount: i32,
    coefficients: GlobePlayerPropertyCoefficients,
) {
    let occupation = usize::from(occupation).min(2);
    let mut player_delta = None;
    {
        let Some(potential) = host.headgear_addon(GAP_BF_POTENTIAL, 1) else {
            return;
        };
        if potential.wrapping_sub(amount) < 0 {
            return;
        }
        let (tracked_property, applied_amount) = match property {
            GAP_BF_ATTACK => (
                GAP_BF_ATTACK_POTENTIAL,
                (f64::from(amount) * 1.5).trunc() as i32,
            ),
            GAP_BF_SPRITE => (
                GAP_BF_SPRITE_POTENTIAL,
                (f64::from(amount) * 1.5).trunc() as i32,
            ),
            GAP_BF_BLAST => (GAP_BF_BLAST_POTENTIAL, amount),
            GAP_BF_BRAVE => (GAP_BF_BRAVE_POTENTIAL, amount),
            GAP_BF_AGILITY => (GAP_BF_AGILITY_POTENTIAL, amount),
            GAP_BF_SPRITUALISM => (GAP_BF_SPRITUALISM_POTENTIAL, amount),
            GAP_BF_STRENGH => (GAP_BF_STRENGH_POTENTIAL, amount),
            _ => return,
        };
        host.add_headgear_addon(property, applied_amount);
        host.add_headgear_addon(tracked_property, applied_amount);
        host.set_headgear_addon(GAP_BF_POTENTIAL, 1, potential.wrapping_sub(amount));
        if property == GAP_BF_SPRITUALISM {
            host.add_headgear_addon(GAP_BF_MAX_MP, amount);
        } else if property == GAP_BF_STRENGH {
            host.add_headgear_addon(GAP_BF_MAX_HP, amount);
        }
        if matches!(
            property,
            GAP_BF_BRAVE | GAP_BF_AGILITY | GAP_BF_SPRITUALISM | GAP_BF_STRENGH
        ) {
            player_delta = Some((property, f64::from(amount) * 0.00001));
        }
    }

    let Some((property, delta)) = player_delta else {
        return;
    };
    let combat = host.combat_properties();
    match property {
        GAP_BF_BRAVE => {
            combat.strength = add_battle_fairy_u32(combat.strength, delta);
            combat.maximum_attack = add_battle_fairy_u32(
                combat.maximum_attack,
                delta * f64::from(coefficients.str_to_max_attack[occupation]),
            );
            combat.burden = add_battle_fairy_u16(
                combat.burden,
                delta * f64::from(coefficients.str_to_burden[occupation]),
            );
        }
        GAP_BF_AGILITY => {
            combat.dexterity = add_battle_fairy_u32(combat.dexterity, delta);
            combat.minimum_attack = add_battle_fairy_u32(
                combat.minimum_attack,
                delta * f64::from(coefficients.dex_to_min_attack[occupation]),
            );
            combat.reank = add_battle_fairy_u16(
                combat.reank,
                delta * f64::from(coefficients.dex_to_stiff[occupation]),
            );
        }
        GAP_BF_SPRITUALISM => {
            combat.intelligence = add_battle_fairy_u32(combat.intelligence, delta);
            combat.element_modify = add_battle_fairy_i32(
                combat.element_modify,
                delta * f64::from(coefficients.int_to_element[occupation]),
            );
            combat.maximum_mp = add_battle_fairy_u32(
                combat.maximum_mp,
                delta * f64::from(coefficients.int_to_max_mp[occupation]),
            );
            combat.element_resistance = add_battle_fairy_u32(
                combat.element_resistance,
                delta * f64::from(coefficients.int_to_resistant[occupation]),
            );
        }
        GAP_BF_STRENGH => {
            combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, delta);
        }
        _ => {}
    }
}

fn push_battle_fairy_upgrade_notification<Update, CurrencyOutcome, ConsumedGem, Removal>(
    effects: &mut Vec<BattleFairyUpgradeEffect<Update, CurrencyOutcome, ConsumedGem, Removal>>,
    player_id: i32,
    string_id: &'static str,
    format_value: Option<u32>,
) {
    effects.push(BattleFairyUpgradeEffect::Notification {
        player_id,
        string_id,
        color: 0xffff_ffff,
        format_value,
    });
}

pub fn upgrade_battle_fairy_equipment<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    log_gates: BattleFairyUpgradeLogGates,
    random: &mut dyn FnMut(i32) -> i32,
) -> BattleFairyUpgradeResolution<Host> {
    let price = host.battle_fairy_upgrade_price();
    let mut resolution = BattleFairyUpgradeResolution {
        outcome: BattleFairyUpgradeOutcome::MissingRegion,
        effects: Vec::new(),
    };
    if !host.server_region_present() {
        return resolution;
    }
    if host.currency_amount() < price {
        resolution.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0015",
            Some(price),
        );
        return resolution;
    }
    if !host.battle_fairy_cell_present(BATTLE_FAIRY_CELL_EQUIPMENT) {
        resolution.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0014",
            None,
        );
        return resolution;
    }
    if !host.battle_fairy_equipment_can_upgrade() {
        resolution.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0014",
            None,
        );
        return resolution;
    }
    let current_level = headgear_cell_property_or_default(
        host,
        BATTLE_FAIRY_CELL_EQUIPMENT,
        GAP_BF_WEAPON_LEVEL,
        1,
    );
    let target = host
        .battle_fairy_cell_snapshot(BATTLE_FAIRY_CELL_EQUIPMENT)
        .expect("equipment cell presence подтверждена гейтами выше");
    if !host.battle_fairy_cell_present(BATTLE_FAIRY_CELL_GEM_BASE) {
        resolution.outcome = BattleFairyUpgradeOutcome::MissingBaseGem;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0013",
            None,
        );
        return resolution;
    }
    let minimum =
        headgear_cell_property_or_default(host, BATTLE_FAIRY_CELL_GEM_BASE, GAP_GEM_LEVEL, 1);
    let maximum =
        headgear_cell_property_or_default(host, BATTLE_FAIRY_CELL_GEM_BASE, GAP_GEM_LEVEL, 2)
            .max(minimum);
    if current_level < minimum || maximum < current_level {
        resolution.outcome = BattleFairyUpgradeOutcome::GemLevelMismatch;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0012",
            None,
        );
        return resolution;
    }
    if 98 < current_level as u32 {
        resolution.outcome = BattleFairyUpgradeOutcome::MaximumLevel;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0021",
            None,
        );
        return resolution;
    }
    let probability = host.battle_fairy_probability();
    tracing::trace!(
        player_id,
        price,
        probability,
        current_level,
        "параметры улучшения боевой феи рассчитаны"
    );
    if host.currency_amount() < price {
        resolution.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0020",
            None,
        );
        return resolution;
    }
    let gems = [
        BATTLE_FAIRY_CELL_GEM_BASE,
        BATTLE_FAIRY_CELL_GEM_ONE,
        BATTLE_FAIRY_CELL_GEM_TWO,
        BATTLE_FAIRY_CELL_GEM_THREE,
    ]
    .map(|position| host.battle_fairy_cell_snapshot(position));
    let money = host.decrease_money(price);
    resolution
        .effects
        .push(BattleFairyUpgradeEffect::MoneyChanged {
            player_id,
            previous: money.previous,
            current: money.current,
            outcome: money.outcome,
        });

    let audit_player = host.audit_player_snapshot();

    let success = (random(100) as u32).wrapping_add(1) <= probability;
    let mut target_present = true;
    if success {
        let increase = host.battle_fairy_success_result(random);
        let target_level = (current_level as u32).wrapping_add(increase).min(99) as i32;
        host.set_battle_fairy_cell_goods_level(BATTLE_FAIRY_CELL_EQUIPMENT, target_level);
        resolution.outcome = BattleFairyUpgradeOutcome::Succeeded;
        push_battle_fairy_upgrade_notification(
            &mut resolution.effects,
            player_id,
            "ZHGS0002",
            None,
        );
        if log_gates.success {
            resolution.effects.push(BattleFairyUpgradeEffect::Audit {
                message_type: 0x0006_0203,
                event: 1,
                player_id,
                player: audit_player,
                target: target.clone(),
                gems: gems.clone(),
            });
        }
    } else {
        if log_gates.failure {
            resolution.effects.push(BattleFairyUpgradeEffect::Audit {
                message_type: 0x0006_0203,
                event: 2,
                player_id,
                player: audit_player,
                target: target.clone(),
                gems: gems.clone(),
            });
        }
        match host.battle_fairy_fail_result() {
            1 => {
                resolution.outcome = BattleFairyUpgradeOutcome::FailedKept;
                push_battle_fairy_upgrade_notification(
                    &mut resolution.effects,
                    player_id,
                    "ZHGS0016",
                    None,
                );
            }
            2 => {
                resolution.outcome = BattleFairyUpgradeOutcome::FailedDowngraded;
                push_battle_fairy_upgrade_notification(
                    &mut resolution.effects,
                    player_id,
                    "ZHGS0017",
                    None,
                );
                if current_level != 0 {
                    host.set_battle_fairy_cell_goods_level(
                        BATTLE_FAIRY_CELL_EQUIPMENT,
                        current_level.wrapping_sub(1),
                    );
                }
            }
            3 => {
                resolution.outcome = BattleFairyUpgradeOutcome::FailedReset;
                push_battle_fairy_upgrade_notification(
                    &mut resolution.effects,
                    player_id,
                    "ZHGS0018",
                    None,
                );
                host.set_battle_fairy_cell_goods_level(BATTLE_FAIRY_CELL_EQUIPMENT, 0);
            }
            4 => {
                resolution.outcome = BattleFairyUpgradeOutcome::FailedDestroyed;
                push_battle_fairy_upgrade_notification(
                    &mut resolution.effects,
                    player_id,
                    "ZHGS0019",
                    None,
                );
                if log_gates.lost_target {
                    resolution.effects.push(BattleFairyUpgradeEffect::Audit {
                        message_type: 0x0006_0202,
                        event: 5,
                        player_id,
                        player: audit_player,
                        target: target.clone(),
                        gems: gems.clone(),
                    });
                }
                if let Some(removal) = host.battle_fairy_delete_upgrade_target() {
                    target_present = false;
                    resolution
                        .effects
                        .push(BattleFairyUpgradeEffect::TargetDeleted {
                            player_id,
                            goods: target.clone(),
                            position: BATTLE_FAIRY_CELL_EQUIPMENT,
                            removal,
                        });
                }
            }
            _ => {
                resolution.outcome = BattleFairyUpgradeOutcome::FailedKept;
            }
        }
    }
    if target_present
        && let Some(goods) = host.battle_fairy_cell_goods_update(BATTLE_FAIRY_CELL_EQUIPMENT)
    {
        resolution
            .effects
            .push(BattleFairyUpgradeEffect::GoodsUpdated(goods));
    }

    for position in [
        BATTLE_FAIRY_CELL_GEM_BASE,
        BATTLE_FAIRY_CELL_GEM_ONE,
        BATTLE_FAIRY_CELL_GEM_TWO,
        BATTLE_FAIRY_CELL_GEM_THREE,
    ] {
        let was_present = host.battle_fairy_cell_present(position);
        let Some(consumed) = host.battle_fairy_consume_upgrade_gem(position) else {
            if was_present || position == BATTLE_FAIRY_CELL_GEM_BASE {
                resolution.outcome = BattleFairyUpgradeOutcome::ConsumptionStopped;
                break;
            }
            continue;
        };
        let (removed, previous_amount, remaining_amount) = Host::consumed_gem_facts(&consumed);
        tracing::trace!(
            player_id,
            cell_position = position,
            removed,
            previous_amount,
            remaining_amount,
            "камень улучшения боевой феи израсходован"
        );
        resolution
            .effects
            .push(BattleFairyUpgradeEffect::GemConsumed {
                player_id,
                consumed: consumed.clone(),
            });
        if !removed && let Some(goods) = host.battle_fairy_cell_goods_update(position) {
            resolution
                .effects
                .push(BattleFairyUpgradeEffect::GoodsUpdated(goods));
        }
    }
    resolution
}

/// Чтение addon-значения предмета ячейки контейнера; presence же подтверждён
/// выше по потоку (исходные `get_goods`+unwrap-цепи).
fn headgear_cell_property_or_default<Host: BattleFairyGearHost + ?Sized>(
    host: &Host,
    position: u32,
    property: i32,
    index: u32,
) -> i32 {
    host.battle_fairy_cell_addon(position, property, index)
        .unwrap_or_default()
}

/// Списание move-out пары tracked/property потенциала головного предмета.
fn take_battle_fairy_potential_value<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    tracked: i32,
    property: i32,
) -> i32 {
    let value = headgear_property_or_default(host, tracked, 1);
    host.set_headgear_addon(tracked, 1, 0);
    let current = headgear_property_or_default(host, property, 1);
    host.set_headgear_addon(property, 1, current.wrapping_sub(value));
    value
}

pub fn reset_battle_fairy_potential<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    battle_fairy_enabled: bool,
) -> BattleFairyPotentialResetResolution<Host> {
    let mut resolution = BattleFairyPotentialResetResolution {
        outcome: BattleFairyPotentialResetOutcome::MissingHeadgear,
        effects: Vec::new(),
    };
    if !battle_fairy_enabled {
        resolution.outcome = BattleFairyPotentialResetOutcome::FeatureDisabled;
        resolution
            .effects
            .push(BattleFairyPotentialResetEffect::Notification {
                player_id,
                string_id: "ZHGS0008",
                color: 0xffff_0000,
            });
        return resolution;
    }
    let Some(battle_fairy_flag) = host.headgear_addon(GAP_BF_BATTLE_FAIRY, 1) else {
        return resolution;
    };
    if battle_fairy_flag != 1 {
        resolution.outcome = BattleFairyPotentialResetOutcome::InvalidHeadgear;
        resolution
            .effects
            .push(BattleFairyPotentialResetEffect::Notification {
                player_id,
                string_id: "ZHGS0009",
                color: 0xffff_ffff,
            });
        return resolution;
    }

    let reset_item = host.find_reset_goods(b"ZHQLS01");
    let Some((reset_identity, reset_amount)) = reset_item else {
        resolution.outcome = BattleFairyPotentialResetOutcome::MissingResetItem;
        resolution
            .effects
            .push(BattleFairyPotentialResetEffect::Notification {
                player_id,
                string_id: "ZHGS0010",
                color: 0xffff_ffff,
            });
        return resolution;
    };
    let reset_position = host.packet_goods_position(reset_identity.ex_id);
    let (remaining_amount, consumed, removal) = if reset_amount == 0 {
        (0, false, None)
    } else if reset_amount == 1 {
        let removal = host.packet_remove_goods(reset_identity.ex_id);
        (
            if removal.is_some() { 0 } else { reset_amount },
            removal.is_some(),
            removal,
        )
    } else {
        let remaining = reset_amount.wrapping_sub(1);
        let mut consumed = false;
        if let Some(position) = reset_position
            && host.packet_set_goods_amount(position, remaining)
        {
            consumed = true;
        }
        (
            if consumed { remaining } else { reset_amount },
            consumed,
            None,
        )
    };
    resolution
        .effects
        .push(BattleFairyPotentialResetEffect::PacketItemConsumed {
            player_id,
            goods: reset_identity,
            position: reset_position,
            previous_amount: reset_amount,
            remaining_amount,
            consumed,
            removal,
        });

    let recovered = {
        let attack =
            take_battle_fairy_potential_value(host, GAP_BF_ATTACK_POTENTIAL, GAP_BF_ATTACK);
        let sprite =
            take_battle_fairy_potential_value(host, GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITE);
        let blast = take_battle_fairy_potential_value(host, GAP_BF_BLAST_POTENTIAL, GAP_BF_BLAST);
        let brave = take_battle_fairy_potential_value(host, GAP_BF_BRAVE_POTENTIAL, GAP_BF_BRAVE);
        let agility =
            take_battle_fairy_potential_value(host, GAP_BF_AGILITY_POTENTIAL, GAP_BF_AGILITY);
        let spiritualism = take_battle_fairy_potential_value(
            host,
            GAP_BF_SPRITUALISM_POTENTIAL,
            GAP_BF_SPRITUALISM,
        );
        let strength =
            take_battle_fairy_potential_value(host, GAP_BF_STRENGH_POTENTIAL, GAP_BF_STRENGH);
        let recovered = ((f64::from(sprite) + f64::from(attack)) * (2.0 / 3.0)
            + f64::from(blast)
            + f64::from(brave)
            + f64::from(agility)
            + f64::from(spiritualism)
            + f64::from(strength))
        .trunc() as i32;
        let potential = headgear_property_or_default(host, GAP_BF_POTENTIAL, 1);
        host.set_headgear_addon(GAP_BF_POTENTIAL, 1, potential.wrapping_add(recovered));
        (recovered, brave, agility, spiritualism, strength)
    };
    tracing::trace!(
        player_id,
        recovered_potential = recovered.0,
        "потенциал боевой феи восстановлен"
    );
    {
        let combat = host.combat_properties();
        combat.strength = clamp_combat_scalar(
            combat
                .strength
                .wrapping_sub((f64::from(recovered.1) * 0.00001).trunc() as u32),
        );
        combat.dexterity = clamp_combat_scalar(
            combat
                .dexterity
                .wrapping_sub((f64::from(recovered.2) * 0.00001).trunc() as u32),
        );
        combat.maximum_hp = clamp_combat_scalar(
            combat
                .maximum_hp
                .wrapping_sub((f64::from(recovered.4) * 0.00001).trunc() as u32),
        );
        combat.intelligence = clamp_combat_scalar(
            combat
                .intelligence
                .wrapping_sub((f64::from(recovered.3) * 0.00001).trunc() as u32),
        );
    }
    resolution
        .effects
        .push(BattleFairyPotentialResetEffect::PropertiesChanged { player_id });
    let headgear = host
        .headgear_goods_update()
        .expect("reset не отделяет equipped headgear");
    resolution
        .effects
        .push(BattleFairyPotentialResetEffect::GoodsUpdated(headgear));
    resolution.outcome = BattleFairyPotentialResetOutcome::Reset;
    resolution
}

fn battle_fairy_skill_snapshot(
    player_id: i32,
    skill_id: u32,
    skill_level: i32,
    skill_type: u32,
    skill_name: Option<Vec<u8>>,
) -> BattleFairySkillAdded {
    BattleFairySkillAdded {
        message_type: BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE,
        player_id,
        skill_id,
        skill_level,
        skill_type,
        skill_name,
    }
}

/// Полный player-side `CBattleFairyContainer::ResetSkill`. `consume_item`
/// соответствует третьему native аргументу: script allocation передаёт
/// ноль, прямой gameplay caller может потребовать `ZHJNS01/02`.
pub fn reset_battle_fairy_skill<Host: BattleFairyGearHost + ?Sized>(
    host: &mut Host,
    player_id: i32,
    battle_fairy_enabled: bool,
    position: i32,
    consume_item: bool,
    random: &mut dyn FnMut(i32) -> i32,
) -> BattleFairySkillResetResolution<Host> {
    let mut resolution = BattleFairySkillResetResolution {
        outcome: BattleFairySkillResetOutcome::MissingHeadgear,
        effects: Vec::new(),
    };
    match battle_fairy_reset_preflight(battle_fairy_enabled, || {
        host.headgear_addon(GAP_BF_BATTLE_FAIRY, 1)
    }) {
        BattleFairyResetPreflight::FeatureDisabled => {
            resolution.outcome = BattleFairySkillResetOutcome::FeatureDisabled;
            resolution
                .effects
                .push(BattleFairySkillResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            return resolution;
        }
        BattleFairyResetPreflight::MissingHeadgear => return resolution,
        BattleFairyResetPreflight::InvalidHeadgear => {
            resolution.outcome = BattleFairySkillResetOutcome::InvalidHeadgear;
            return resolution;
        }
        BattleFairyResetPreflight::Ready => {}
    }

    if consume_item {
        let reset_item = battle_fairy_reset_item(position, |name| host.find_reset_goods(name));
        let reset_item = match reset_item {
            BattleFairyResetItemLookup::InvalidPosition => None,
            BattleFairyResetItemLookup::MissingItem => {
                resolution.outcome = BattleFairySkillResetOutcome::MissingResetItem;
                resolution
                    .effects
                    .push(BattleFairySkillResetEffect::Notification {
                        player_id,
                        string_id: BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING,
                        color: 0xffff_ffff,
                    });
                return resolution;
            }
            BattleFairyResetItemLookup::Found(item) => Some(item),
        };
        if let Some((reset_identity, reset_amount)) = reset_item {
            let reset_position = host.packet_goods_position(reset_identity.ex_id);
            let change = battle_fairy_reset_item_change(reset_amount);
            let (remaining_amount, consumed, removal) = match change {
                BattleFairyResetItemChange::Remove => {
                    let removal = host.packet_remove_goods(reset_identity.ex_id);
                    (
                        if removal.is_some() { 0 } else { reset_amount },
                        removal.is_some(),
                        removal,
                    )
                }
                BattleFairyResetItemChange::SetAmount(remaining) => {
                    let mut consumed = false;
                    if let Some(reset_position) = reset_position
                        && host.packet_set_goods_amount(reset_position, remaining)
                    {
                        consumed = true;
                    }
                    (
                        if consumed { remaining } else { reset_amount },
                        consumed,
                        None,
                    )
                }
            };
            resolution
                .effects
                .push(BattleFairySkillResetEffect::PacketItemConsumed {
                    player_id,
                    goods: reset_identity,
                    position: reset_position,
                    previous_amount: reset_amount,
                    remaining_amount,
                    consumed,
                    removal,
                });
        }
    }

    let reset_slot = battle_fairy_reset_slot(position, |property, index| {
        headgear_skill_property_or_default(host, property, index)
    });
    let Some(reset_slot) = reset_slot else {
        resolution.outcome = BattleFairySkillResetOutcome::InvalidPosition;
        return resolution;
    };
    let previous_skill = reset_slot.previous_skill();
    let property = battle_fairy_skill_property_key(reset_slot.property);
    tracing::trace!(
        player_id,
        position,
        previous_skill,
        "прежний навык боевой феи выбран для сброса"
    );

    // В каждом native switch-case полный detach расположен перед первым
    // random(), а не только перед addon mutation.
    for equipped_property in EQUIPPED_SKILL_PROPERTIES {
        let skill_id = battle_fairy_skill_id(equipped_property, |property, index| {
            headgear_skill_property_or_default(host, property, index)
        });
        if skill_id == 0 {
            continue;
        }
        host.delete_war_soul_skill(skill_id);
        tracing::trace!(
            player_id,
            skill_id,
            "навык боевой феи отсоединён при сбросе"
        );
        // Native `DelWarSoulSkillInPlayer` вызывает TellClient после
        // DelSkill. Поэтому packet удаления существует лишь если skill
        // пережил отказ category lookup.
        if let Some(skill_name) = host.war_soul_removed_skill_name(skill_id) {
            resolution
                .effects
                .push(BattleFairySkillResetEffect::SkillRemoved(
                    BattleFairySkillRemoved {
                        message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                        player_id,
                        skill_id,
                        skill_name,
                    },
                ));
        }
    }

    let selected_skill = select_battle_fairy_reset_skill(
        reset_slot.replaced,
        reset_slot.current_skills,
        reset_slot.current_all_skill,
        random,
    );
    tracing::trace!(
        player_id,
        selected_skill,
        "новый навык боевой феи выбран при сбросе"
    );

    write_battle_fairy_reset_skill(selected_skill, |index, value| {
        host.set_headgear_addon(property, index, value);
    });

    for equipped_property in EQUIPPED_SKILL_PROPERTIES {
        let (skill_id, level) = battle_fairy_skill_entry(equipped_property, |property, index| {
            headgear_skill_property_or_default(host, property, index)
        });
        host.add_war_soul_skill(skill_id, level);
        if let Some((skill_level, skill_type, skill_name)) =
            host.war_soul_added_skill_facts(skill_id)
        {
            tracing::trace!(
                player_id,
                skill_id,
                "навык боевой феи присоединён после сброса"
            );
            resolution
                .effects
                .push(BattleFairySkillResetEffect::SkillAdded(
                    battle_fairy_skill_snapshot(
                        player_id,
                        skill_id,
                        skill_level,
                        skill_type,
                        skill_name,
                    ),
                ));
        }
    }

    let Some((skill_level, skill_type, skill_name)) =
        host.war_soul_added_skill_facts(selected_skill)
    else {
        resolution.outcome = BattleFairySkillResetOutcome::SelectedSkillUnavailable;
        return resolution;
    };
    resolution
        .effects
        .push(BattleFairySkillResetEffect::SelectedSkillLearned(
            battle_fairy_skill_snapshot(
                player_id,
                selected_skill,
                skill_level,
                skill_type,
                skill_name,
            ),
        ));
    let headgear = host
        .headgear_goods_update()
        .expect("ResetSkill не отделяет equipped headgear");
    resolution
        .effects
        .push(BattleFairySkillResetEffect::GoodsUpdated(headgear));
    resolution.outcome = BattleFairySkillResetOutcome::Reset;
    resolution
}
