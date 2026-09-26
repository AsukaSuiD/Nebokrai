//! Стационарная форма громового рассечения `CThunderSlashPhalanx`
//! (ctor `0x5F75E0`, 0xEC байт, vtable `0x660BDC`). Источник: точная пара
//! `gameserver.exe` (SHA-256 `4F5C98E0…`) + `GameServer.pdb` (RSDS match),
//! исходный владелец `appserver/skills/thunderslashphalanx.cpp/.h`. Прежний
//! переходный владелец — `src/gameserver/appserver/skills/thunderslashphalanx.rs`;
//! тела перенесены порцией T4. Живой тикер и End/публикация удаления —
//! у общего регионального runtime старого пакета; здесь снимок владельца,
//! часы, цель клетки, формула и применение попадания.
//!
//! Машинные якоря (VA = RVA + 0x400000): ctor `0x5F75E0` (12 аргументов;
//! поле [+0xCC] и [+0xD0] пишут один и тот же max-аргумент — min-аргумент
//! конструктор не читает, баг оригинала воспроизводится; [+0xE4] = уровень,
//! [+0xB8] = [+0xE8] = 0x72, [+0xC8] = 0); AI `0x5F7B40` (три чтения
//! `timeGetTime`: срок `[+0xB4]+[+0xB0] < now` unsigned, клетка
//! [+0xBC]/[+0xC0] ненулевая, период `[+0xC8]+[+0xC4] < now` строгий;
//! штамп [+0xC8] пишется ДО RTTI-разрешения региона и единственного
//! GetShape собственной клетки; первая немобильная форма не заменяется
//! следующей целью); Attack `0x5F7A00` (IsDied → пропуск; FindPlayer по
//! одному ID даже для снимка иного типа; attackable vcall+0x134;
//! OnBeenAttacked vcall+0x15C; при найденном source — IncreaseRp(1, 0));
//! Calculate `0x5F77B0`; AddToByteArray `0x5F76A0` / Decord `0x5F7730`
//! (префикс 5 dword: id 0x72, уровень [+0xE4], master type/id [+0x84]/[+0x88],
//! remained `0x5E9870`); ReplaceAffectRegion — ICF-хвост `CWeakPhalanx`
//! `0x5F54A0` (End при совпадении клетки); GetSkillID `0x5E1030`;
//! End `0x5E9DC0`; ForceMove `0x5E9AF0`.
//!
//! Машинный факт MATCH (FIX #1 порции T4, якорь Calculate `0x5F77B0`):
//!
//! ```text
//! 005f77ed: mov  edx, dword ptr [ebp + 8]    ; [this+8] — instance-id формы
//! 005f77f5: mov  dword ptr [esi], edx        ; info[+0] := instance-id
//! ```
//!
//! `info[+0]` получает живой instance-id формы (`g_lID` счётчика), а не
//! константу 0x72 прежней реконструкции. Эквивалентность потребителей
//! установлена разведкой волны (`.local/recon-t4`): читающие стороны поля
//! +0 — war-soul-предикат `is_war_soul_skill` (0x212..0x224, 530..548) и
//! wire-хвосты 0xBF60A/0xBF60B отчёта попадания; применение урона поле +0
//! не использует. Квази-особенность: при входе instance-id в окно
//! 0x212..0x224 (530..548; боевой октет 0x212..0x219 = 530..545) атака
//! призванной формы с player-мастером проходит war-soul ветки приёмника —
//! поведение оригинала сохранено буквально. По той же машинной записи
//! `info[+4]` — младший байт уровня [+0xE4], `info[+0x20]` = 0, фактор
//! `f32(query(0x4E23) * double(0.01f32))`, hit = `query(0x4E21)`; RNG
//! диапазона вызывается до RNG крита; критический множитель — FISTP-усечение
//! к нулю (`truncate_original`).
//!
//! Объявленные швы переноса (не расхождения): hub-трейты
//! `ThunderSlashPhalanxGame`/`ThunderSlashPhalanxPlayer`/
//! `ThunderSlashPhalanxContact` — переходные фасады прежнего `CGame`/
//! `CPlayer` (реализация у делегата старого пакета); имена членов сохраняют
//! исходную операцию. GetShape клетки с dyn_cast CMoveShape и query по
//! origin-name остаются у владельца региона (шов
//! `thunder_slash_region_cell_target`); клиентский снимок — общий
//! wire-конверт `summonshape`. blast/knock-back нормализация в Calc не
//! вводится и не удаляется — вопрос открыт (UNKNOWN).

use nebokrai_shared::values::CGuid;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original,
};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::CShape;

use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use super::thunderslash::THUNDER_SLASH_SKILL_ID;

const TARGET_AFFECT_DMG_FACTOR: u32 = 20_003;
const TARGET_AFFECT_HIT_MODIFIER: u32 = 20_001;

/// Снимок игрока-источника формы: только identity живой формы
/// (`FindPlayer` по одному ID; factory и factory-свойства — у владельца).
pub trait ThunderSlashPhalanxPlayer {
    fn shape(&self) -> &CShape;
}

/// Переходные фасады прежнего владельца `CGame` для тика формы и формулы;
/// имена сохраняют исходную операцию.
pub trait ThunderSlashPhalanxGame {
    type Player: ThunderSlashPhalanxPlayer;

    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    fn skill_random_below(&mut self, maximum: i32) -> i32;

    /// `CGlobeSetup::critical_rate` (fmul `0xEF3E5C` крит-ветки Calculate).
    fn thunder_slash_critical_rate(&self) -> f32;

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool;

    /// Допуск живой цели: `live_skill_target_attackable` прежнего владельца.
    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;

    /// Разрешение цели клетки к MoveShape (`resolve_state_move_shape`).
    fn thunder_slash_target_present(&self, region_id: i32, target: ShapeIdentity) -> bool;

    /// IncreaseRp(1, 0) найденного source.
    fn increase_owned_player_rp(&mut self, player_id: i32, attacking: bool, damage: u16);

    /// GetShape собственной клетки + dyn_cast CMoveShape; найденный объект
    /// разрешается живым `resolve_state_move_shape` прежнего владельца.
    fn thunder_slash_region_cell_target(
        &self,
        region_id: i32,
        x: i32,
        y: i32,
    ) -> Option<ShapeIdentity>;
}

/// Контактная стадия попадания формы: общий virtual +15C приёмника —
/// тип хода принадлежит старому main loop, а не самой форме.
pub trait ThunderSlashPhalanxContact<Runtime>: ThunderSlashPhalanxGame {
    fn apply_owned_skill_contact(
        &mut self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CThunderSlashPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    tile_x: i32,
    tile_y: i32,
    frequency_ms: u32,
    last_attack_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_attack: i32,
    dexterity: i32,
    critical_chance: i32,
    soul_attack: i32,
}

/// Цель атаки формы: единственный GetShape собственной клетки с
/// dyn_cast CMoveShape; первая немобильная форма не заменяется следующей.
pub fn thunder_slash_target<Game: ThunderSlashPhalanxGame>(
    game: &Game,
    region_id: i32,
    phalanx: &CThunderSlashPhalanx,
) -> Option<ShapeIdentity> {
    let (x, y) = phalanx.tile();
    game.thunder_slash_region_cell_target(region_id, x, y)
}

impl CThunderSlashPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимки соответствуют аргументам конструктора EXE")]
    pub fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        frequency_ms: u32,
        maximum_attack: i32,
        _minimum_attack: i32,
        element_attack: i32,
        dexterity: i32,
        critical_chance: i32,
        soul_attack: i32,
        tile_x: i32,
        tile_y: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            master,
            started_at_ms,
            lifetime_ms,
            tile_x,
            tile_y,
            frequency_ms,
            last_attack_ms: 0,
            skill_level,
            // Баг ctor 0x5F75E0: [+0xCC] и [+0xD0] пишут max-аргумент,
            // min-аргумент не читается.
            minimum_attack: maximum_attack,
            maximum_attack,
            element_attack,
            dexterity,
            critical_chance,
            soul_attack,
        }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn tile(&self) -> (i32, i32) { (self.tile_x, self.tile_y) }

    /// Срок: `[+0xB4]+[+0xB0] < now` unsigned; нулевая клетка — отказ Add.
    pub const fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
            || self.tile_x == 0 || self.tile_y == 0
    }

    /// Период: `[+0xC8]+[+0xC4] < now` строгое unsigned.
    pub const fn attack_due_at(&self, now_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(self.frequency_ms) < now_ms
    }

    pub fn mark_attack_at(&mut self, now_ms: u32) {
        self.last_attack_ms = now_ms;
    }

    /// Клиентский снимок входа 0xBF502: общий префикс skill/level/master
    /// type/id/remained, затем CShape (`AddToByteArray 0x5F76A0`).
    pub fn encode_client_snapshot(&self, now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            THUNDER_SLASH_SKILL_ID as i32,
            self.skill_level,
            self.master.master_type,
            self.master.master_id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }

    fn attack_master(&self) -> MasterInfo {
        if self.master.master_type == PLAYER_TYPE { return self.master; }
        MasterInfo {
            master_type: self.master.master_type,
            master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

/// Calculate `0x5F77B0`: `info[+0] := instance-id [this+8]` (FIX #1, см.
/// шапку), уровень из [+0xE4], три Damage-записи (1/3/4), затем RNG-крит с
/// FISTP-усечением. Отсутствие таблицы оставляет UNKNOWN/1 и пустой урон,
/// не отменяя OnBeenAttacked и IncreaseRp.
pub fn calculate_owned_thunder_slash_attack<Game: ThunderSlashPhalanxGame>(
    game: &mut Game,
    phalanx: &CThunderSlashPhalanx,
) -> AttackInformation {
    let mut attack = AttackInformation::for_master(phalanx.attack_master());
    let Some(properties) = game.skill_base_properties(THUNDER_SLASH_SKILL_ID, phalanx.skill_level) else {
        return attack;
    };
    attack.skill_id = phalanx.shape().identity().id as u32;
    attack.skill_level = phalanx.skill_level as u8;
    attack.damage_modifier = 0;
    attack.damage_factor = (f64::from(properties.query_property(TARGET_AFFECT_DMG_FACTOR))
        * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = properties.query_property(TARGET_AFFECT_HIT_MODIFIER) as i32;

    let width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let mut physical = game.skill_random_below(width).wrapping_add(phalanx.minimum_attack);
    if attack.attacker_type == PLAYER_TYPE {
        physical = physical.wrapping_add(phalanx.dexterity);
    }
    attack.damages = vec![
        AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
        AttackPower { kind: AttackPowerType::Element, hp_damage: phalanx.element_attack.max(0), mp_damage: 0 },
        AttackPower { kind: AttackPowerType::Soul, hp_damage: phalanx.soul_attack.max(0), mp_damage: 0 },
    ];
    if game.skill_random_below(100) < phalanx.critical_chance {
        attack.critical = true;
        let rate = game.thunder_slash_critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    attack
}

/// Attack `0x5F7A00`: живой target, FindPlayer по одному ID, допуск,
/// Calculate, OnBeenAttacked и IncreaseRp(1, 0) при найденном source.
pub fn apply_thunder_slash_attack<Game, Runtime>(
    game: &mut Game,
    phalanx: &CThunderSlashPhalanx,
    region_id: i32,
    target: ShapeIdentity,
    runtime: &mut Runtime,
) where
    Game: ThunderSlashPhalanxContact<Runtime>,
{
    if !game.thunder_slash_target_present(region_id, target)
        || game.base_magic_target_dead(region_id, target)
    {
        return;
    }
    // FindPlayer вызывается по одному ID даже для снимка иного типа.
    let source = game.find_player(phalanx.master.master_id)
        .map(|player| player.shape().identity());
    if let Some(source) = source
        && !game.live_skill_target_attackable(region_id, source, target)
    {
        return;
    }
    let attack = calculate_owned_thunder_slash_attack(game, phalanx);
    game.apply_owned_skill_contact(phalanx.attack_master(), target, region_id, attack, runtime);
    if let Some(source) = source {
        game.increase_owned_player_rp(source.id, true, 0);
    }
}
