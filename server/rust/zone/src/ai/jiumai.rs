//! ИИ близнецов `CJiuMai` (AI101): создание и связывание пары, сближение
//! близнецов, выбор цели с минимальным текущим HP и hurt-поведение с отходом
//! или приёмом атакующего.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\jiumai.cpp`.
//! Машинная сверка: разобраны все шесть функций класса, кроме ctor
//! `0x0060A5D0` — построчно не читан, а его известный эффект (нулевые
//! lifecycle-поля) зафиксирован владельцем состояния:
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | `OnIdle`: призванный близнец берёт `m_lTwinsID` из `GetMasterInfo()->lMasterID`; обычный монстр один раз ставит `tagMasterInfo{lMasterType=[owner+4], lMasterID=[owner+8]}`, `GetRandomPosInRange(x−5, y−5, 10, 10)`, `AddSummonedCreature(…, −1, 0xFFFFFFFF)` и пишет ID близнеца, при отказе spawn — `−1`; любой исход завершается базовым `CMonsterAI::OnIdle` (`0x005DCC60`) | VA `0x0060A5F0` | [`ensure_jiumai_twin`] | `MATCH`, кроме ветки отказа `GetRandomPosInRange`: машинный код спавнит с уже записанным выходом функции позиции и ставит `−1` только при отказе `AddSummonedCreature`/cast; hub-форма `position.ok().and_then(spawn)` даёт `−1` и при отказе позиции — `PARTIAL` |
//! | `OnSchedule`: только живой владелец с пустыми очередями `[+0x14]`/`[+0x28]`; живой близнец (region `FindObject(600, m_lTwinsID=[this+0x7C])`, dynamic_cast) дальше пяти клеток при текущей цели не ближе к владельцу получает `GetRandomPosInRange(twin.x−5, twin.y−5, 10, 10)` и `CMoveShape::ForceMove(run=0)`; затем общий боевой хвост без `GetAtcInterval`-гейта (`HasTarget` → цель жива и `IsAttackAble` → навык/`OnChangeSkill` → vt `+0x4C` → Begin → `ASA_ATTACK`, отказ — `OnLoseTarget + AddAIEvent(5)`) | VA `0x0060AB10` (Run-вход `0x0060AB16` — пустой общий hook `0x00485540`) | [`maintain_jiumai_twin`] префикс; боевой хвост — общий диспетчер `ai/monsterai.rs` | `MATCH` |
//! | `OnSearchEnemy`: игроки перед питомцами, минимальный текущий HP (vt `+0xD0`) внутри `GetGuardRange` (vt `+0x138`), равный HP сохраняет первую запись, живость — фильтром | VA `0x0060AD10` | [`select_jiumai_enemy`] | `MATCH` |
//! | `SetTarget`: базовый `CMonsterAI::SetTarget` (`0x005DCC40`); живой близнец без собственной цели (`GetAI` cast, `HasTarget == 0`) получает virtual `SetTarget` той же пары | VA `0x0060AA50` | [`assign_jiumai_target`] | `MATCH` |
//! | `OnLoseTarget`: базовый `CMonsterAI::OnLoseTarget` (`0x005DCC30`); при успехе — живой близнец с `HasTarget` получает свой virtual `OnLoseTarget`; возврат 1/0 по базовому переходу | VA `0x0060A990` | [`release_jiumai_target`], [`synchronize_jiumai_target_loss`] | `MATCH` |
//! | `WhenBeenHurted`: базовый hurt всегда; тип 400 вне боя — существующий игрок региона принимается целью; иначе отход от ближайшего игрока, шаг к ближайшему монстру (иначе), отсутствие ориентира — шаг по текущему направлению; тип 600 вне боя — приручённое существо (`0x004E6460`) или повозка (`0x004E6D30`) принимается целью | VA `0x0060A750` | [`retarget_jiumai_after_hurt`] | `MATCH` |
//! | hurt-отход: найденный ориентир задаёт направление формы `CShape::SetDir` (owner vtable `+0x60`, `0x0044A1C0`) перед `GetDirPos` и общим координатным `CBaseAI::MoveTo(run=0)` (AI vtable `+0x58`, `0x004C9020`); отсутствие ориентира сохраняет текущее `GetDir` (vtable `+0x5C`) | RVA-место `0x0060A89F`/`0x0060A8E7`; vtable `CShape` `0x0064EA04` | [`retarget_jiumai_after_hurt`] (запись направления) | `MATCH`; прежний hub эту запись не выполнял (расхождение устранено) |
//!
//! `GetDirPos` (`0x0045B330`) безотказен для восьми направлений (табличное
//! сложение дельт) — `Result`-форма `CShape::get_direction_position` в
//! валидном потоке не даёт иного исхода. Входной state-вопрос `0x0047B150`
//! собственных `OnSearchEnemy` и пустой hook `0x00485540` собственных
//! `OnSchedule` — `RET1`-эквиваленты, наблюдаемого эффекта не имеют.
//!
//! Остаются hub-владением: общий monster tick hub — `Run` (`0x0060E250` →
//! `CBaseAI::Run` `0x004C7D10`), материализация active/passive FIFO, реальный
//! путь `monsterbaseattack` и runtime-входы `CGame`. `CGame` владеет
//! журналированием, пакетами, привязкой игрока и фактическим
//! спавном/переносом: они приходят через фасады ниже. Отложенная
//! синхронизация хранит метку связанной цели (местная метка
//! [`JiuMaiAiState::linked_target`], машиной не требуется — см. прежнего
//! владельца).
//!
//! Швы к hub-владельцам:
//!
//! - [`JiuMaiDispatcherMonster`] — доступ к состоянию пары, признакам
//!   призыва/повозки, общей hurt-ветви и записи направления формы на
//!   hub-владельце `CMonster`.
//! - [`JiuMaiDispatcherGame`] — `GetRandomPosInRange` с RNG hub-владельца,
//!   фактический `AddSummonedCreature` и мгновенный `ForceMove` монстра.
//! - [`JiuMaiDispatcherPlayer`] — текущее HP игрока для min-HP селектора;
//!   проходы кандидатов повторяют общий шов `ai/lord.rs`
//!   (`EnemySearchDispatcherRegion`).

use nebokrai_shared::resources::MonsterProperties;
use nebokrai_shared::runtime::get_line_direction;

use crate::combat::MasterInfo;
use crate::regions::ShapeIdentity;
use crate::regions::moveshape::is_died;
use crate::regions::region::RegionRandomContext;
use crate::regions::shape::{CShape, ShapeAreaCoordinates, ShapeView};

use super::lord::{EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherOwner, MonsterDispatcherPlayer, MonsterDispatcherRegion,
    move_owned_monster_to,
};

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

/// Каноническое состояние `CJiuMai`: `m_lTwinsID` (`[this+0x7C]`) и локальная
/// метка ранее связанной цели (hub-память отложенной синхронизации, машиной
/// не требуется). Хранилище перенесено целиком; владелец экземпляра —
/// переходный `CMonster` старого пакета.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JiuMaiAiState {
    twins_id: i32,
    linked_target: bool,
}

impl JiuMaiAiState {
    pub const fn twins_id(&self) -> i32 {
        self.twins_id
    }

    pub const fn set_twins_id(&mut self, twins_id: i32) {
        self.twins_id = twins_id;
    }

    pub const fn linked_target(&self) -> bool {
        self.linked_target
    }

    pub const fn set_linked_target(&mut self, linked_target: bool) {
        self.linked_target = linked_target;
    }
}

/// Монстр-близнец: переходный фасад прежнего `CMonster` (состояние пары,
/// призыв/повозка, общая hurt-ветвь и запись направления формы).
pub trait JiuMaiDispatcherMonster: MonsterDispatcherMonster {
    fn jiu_mai_ai(&self) -> Option<&JiuMaiAiState>;

    fn jiu_mai_ai_mut(&mut self) -> Option<&mut JiuMaiAiState>;

    /// `dynamic_cast<CSummonedCreature*>` владельца.
    fn is_summoned_creature(&self) -> bool;

    /// `CMonster::IsCarriage` (`0x004E6D30`).
    fn is_carriage(&self, property: &MonsterProperties) -> bool;

    /// Общая Defense-ветвь `CBaseAI::WhenBeenHurted` hub-владельца
    /// (`0x004C93E0`).
    fn when_been_hurted(&mut self, now_ms: u32);

    /// `CShape::SetDir` (owner vtable `+0x60`): тихая запись направления формы
    /// до пространственного шага hurt-отхода.
    fn set_shape_direction(&mut self, direction: i32);
}

/// Игрок-кандидат min-HP селектора AI101: переходный фасад прежнего `CPlayer`.
pub trait JiuMaiDispatcherPlayer: EnemySearchDispatcherPlayer {
    /// Текущее HP (`GetHitPoint`, vtable `+0xD0`).
    fn hit_points(&self) -> u32;
}

/// Игра-владелец пары Цзюмай: переходный фасад прежнего `CGame` (RNG позиции,
/// фактический спавн призванного близнеца и мгновенный перенос).
pub trait JiuMaiDispatcherGame: MonsterDispatcherGame {
    /// `CRegion::GetRandomPosInRange` с RNG hub-владельца; `None` соответствует
    /// отказу позиции (см. PARTIAL-оговорку шапки про ветку `−1`).
    fn random_region_position(
        &mut self,
        region: &<Self::RegionOwner as MonsterDispatcherOwner>::Region,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    ) -> Option<ShapeAreaCoordinates>;

    /// `CServerRegion::AddSummonedCreature` с runtime-эффектами hub-владельца;
    /// `None` — отказ спавна (исходный `NULL` → `m_lTwinsID = −1`).
    #[allow(clippy::too_many_arguments, reason = "поля spawn-записи сохраняют исходный вызов")]
    fn add_summoned_creature(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        property: &MonsterProperties,
        master: MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
    ) -> Option<i32>;

    /// `CMoveShape::ForceMove` монстра hub-владельца; возврат — исходный
    /// признак переноса, отбрасываемый вызывающей стороной, как и прежде.
    fn force_move_owned_monster(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        tile_x: i32,
        tile_y: i32,
        run: i32,
    ) -> bool;
}

/// Выполняет достигнутый префикс `OnIdle` AI101: призванный близнец берёт
/// обратный ID мастера, обычный монстр один раз создаёт бессрочного близнеца
/// того же свойства в квадрате 11×11 вокруг себя. Базовый общий `OnIdle`
/// ставится внешним caller-ом при любом исходе.
pub fn ensure_jiumai_twin<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
) -> bool
where
    Game: JiuMaiDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: JiuMaiDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some((twins_id, summoned, master, owner)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                JiuMaiDispatcherMonster::jiu_mai_ai(monster)?.twins_id(),
                JiuMaiDispatcherMonster::is_summoned_creature(monster),
                monster.master_info(),
                monster.shape_view(property)?,
            ))
        })
    else {
        return false;
    };
    if twins_id != 0 {
        return true;
    }
    if summoned {
        if let Some(state) = region
            .find_monster_by_id_mut(monster_id)
            .and_then(JiuMaiDispatcherMonster::jiu_mai_ai_mut)
        {
            state.set_twins_id(master.master_id);
        }
        return true;
    }

    let position = game.random_region_position(
        region,
        owner.tile_x.wrapping_sub(5),
        owner.tile_y.wrapping_sub(5),
        10,
        10,
    );
    let spawned = position.and_then(|position| {
        game.add_summoned_creature(
            region,
            property,
            MasterInfo {
                master_type: MONSTER_TYPE,
                master_id: monster_id,
                ..MasterInfo::default()
            },
            position.x,
            position.y,
            -1,
            u32::MAX,
        )
    });
    if let Some(state) = region
        .find_monster_by_id_mut(monster_id)
        .and_then(JiuMaiDispatcherMonster::jiu_mai_ai_mut)
    {
        state.set_twins_id(spawned.unwrap_or(-1));
    }
    true
}

/// Сохраняет достигнутый префикс `OnSchedule` AI101: живой близнец дальше
/// пяти клеток при текущей цели не ближе к владельцу получает случайный
/// пункт в квадрате 11×11 около близнеца и исходный `ForceMove(run=0)`.
/// Боевой хвост расписания — общий диспетчер навыка (`ai/monsterai.rs`).
pub fn maintain_jiumai_twin<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool
where
    Game: JiuMaiDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: JiuMaiDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
    Runtime: RegionRandomContext,
{
    let Some((twins_id, owner, target)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            if monster.active_primary_ai_type() != Some(101)
                || is_died(monster.hit_points())
                || !monster.primary_ai_queues_idle()
            {
                return None;
            }
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some((
                JiuMaiDispatcherMonster::jiu_mai_ai(monster)?.twins_id(),
                monster.shape_view(property)?,
                monster.ai_target(),
            ))
        })
    else {
        return false;
    };
    let Some(twin) = region.find_monster_by_id(twins_id).and_then(|monster| {
        (!is_died(monster.hit_points())).then(|| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            monster.shape_view(property)
        })?
    }) else {
        return true;
    };
    if owner.real_distance(Some(twin)) <= 5 {
        return true;
    }

    let target = target.and_then(|identity| match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).and_then(|player| player.shape_view()),
        MONSTER_TYPE => region.find_monster_by_id(identity.id).and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            monster.shape_view(property)
        }),
        _ => None,
    });
    if target.is_some_and(|target| {
        target.real_distance(Some(owner)) <= target.real_distance(Some(twin))
    }) {
        return true;
    }

    let Ok(destination) = region.base_region().get_random_pos_in_range(
        twin.tile_x.wrapping_sub(5),
        twin.tile_y.wrapping_sub(5),
        10,
        10,
        runtime,
    ) else {
        return true;
    };
    let _ = game.force_move_owned_monster(
        region,
        owner.identity.id,
        destination.x,
        destination.y,
        0,
    );
    true
}

/// Достигнутый `OnSearchEnemy` AI101: среди живых игроков и питомцев внутри
/// дальности охраны остаётся первая цель с минимальным текущим HP; равный HP
/// сохраняет более раннюю запись прохода.
pub fn select_jiumai_enemy<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity>
where
    Game: MonsterDispatcherGame,
    Game::Player: JiuMaiDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    let mut selected: Option<(ShapeIdentity, i32, u32)> = None;
    let mut consider = |view: ShapeView, hit_points: u32| {
        let distance = owner.real_distance(Some(view));
        if distance > guard_range {
            return;
        }
        if selected.is_none_or(|(_, _, current)| current > hit_points) {
            selected = Some((view.identity, distance, hit_points));
        }
    };
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.region_id()) || player.is_dead() {
            continue;
        }
        let Some(view) = player.shape_view() else {
            continue;
        };
        consider(view, player.hit_points());
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((view, hit_points)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((pet.shape_view(property)?, pet.hit_points()))
            })
        else {
            continue;
        };
        consider(view, hit_points);
    }
    selected.map(|(identity, _, _)| identity)
}

/// Повторяет `CJiuMai::SetTarget`: основной владелец получает цель всегда,
/// живой близнец — только когда ещё не ведёт собственный бой.
pub fn assign_jiumai_target<Region>(
    region: &mut Region,
    monster_id: i32,
    target: ShapeIdentity,
) -> bool
where
    Region: MonsterDispatcherRegion,
    Region::Monster: JiuMaiDispatcherMonster,
{
    let Some(twins_id) = region
        .find_monster_by_id(monster_id)
        .and_then(JiuMaiDispatcherMonster::jiu_mai_ai)
        .map(JiuMaiAiState::twins_id)
    else {
        return false;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_ai_target(target);
        if let Some(state) = JiuMaiDispatcherMonster::jiu_mai_ai_mut(monster) {
            state.set_linked_target(true);
        }
    }
    if twins_id > 0
        && let Some(twin) = region.find_monster_by_id_mut(twins_id)
        && !is_died(twin.hit_points())
        && twin.ai_target().is_none()
    {
        twin.set_ai_target(target);
    }
    true
}

/// Завершает достигнутый `OnLoseTarget` после общего боевого такта. Метка
/// отличает реальный переход ранее связанной цели от обычного бездействия:
/// независимый бой близнеца без предшествующего `SetTarget` не стирается.
pub fn synchronize_jiumai_target_loss<Region>(region: &mut Region, monster_id: i32) -> bool
where
    Region: MonsterDispatcherRegion,
    Region::Monster: JiuMaiDispatcherMonster,
{
    let Some((twins_id, lost_linked_target)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let state = JiuMaiDispatcherMonster::jiu_mai_ai(monster)?;
            Some((
                state.twins_id(),
                state.linked_target() && monster.ai_target().is_none(),
            ))
        })
    else {
        return false;
    };
    if !lost_linked_target {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id)
        && let Some(state) = JiuMaiDispatcherMonster::jiu_mai_ai_mut(monster)
    {
        state.set_linked_target(false);
    }
    if twins_id > 0
        && let Some(twin) = region.find_monster_by_id_mut(twins_id)
        && !is_died(twin.hit_points())
        && twin.ai_target().is_some()
    {
        twin.release_ai_target_for_death();
    }
    true
}

/// Материализует виртуальный `CJiuMai::OnLoseTarget` для расписания и death
/// FIFO. Сначала общий monster-owner отпускает цель погибшей половины, затем
/// тот же базовый переход получает живой сражающийся близнец (у
/// виртуального каскада его цель уже пуста, поэтому один переход здесь
/// эквивалентен). Оба перехода сохраняют уже поставленный
/// `ASA_MOVE`, как исходный `CMonsterAI::OnLoseTarget`.
pub fn release_jiumai_target<Region>(region: &mut Region, monster_id: i32) -> bool
where
    Region: MonsterDispatcherRegion,
    Region::Monster: JiuMaiDispatcherMonster,
{
    let Some(twins_id) = region
        .find_monster_by_id(monster_id)
        .and_then(JiuMaiDispatcherMonster::jiu_mai_ai)
        .map(JiuMaiAiState::twins_id)
    else {
        return false;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.release_ai_target_for_death();
        if let Some(state) = JiuMaiDispatcherMonster::jiu_mai_ai_mut(monster) {
            state.set_linked_target(false);
        }
    }
    if twins_id > 0
        && let Some(twin) = region.find_monster_by_id_mut(twins_id)
        && !is_died(twin.hit_points())
        && twin.ai_target().is_some()
    {
        twin.release_ai_target_for_death();
        if let Some(state) = JiuMaiDispatcherMonster::jiu_mai_ai_mut(twin) {
            state.set_linked_target(false);
        }
    }
    true
}

/// Выполняет достигнутые ветви `WhenBeenHurted` AI101 после базовой Defense.
/// Свободная пара принимает существующего игрока региона либо приручённого
/// монстра или повозку. При исчезнувшем игроке: отход от ближайшего игрока,
/// иначе шаг к ближайшему монстру, иначе шаг по текущему направлению. Каждый
/// ориентир пишет направление формы `CShape::SetDir` до `GetDirPos` и общего
/// `MoveTo(run=0)` — машинный порядок `0x0060A89F`/`0x0060A8E7`.
pub fn retarget_jiumai_after_hurt<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    attacker: ShapeIdentity,
    mut now: impl FnMut() -> u32,
) -> bool
where
    Game: JiuMaiDispatcherGame,
    Region: EnemySearchDispatcherRegion,
    Region::Monster: JiuMaiDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some((owner, area_index, direction, fighting)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            JiuMaiDispatcherMonster::jiu_mai_ai(monster)?;
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some((
                monster.shape_view(property)?,
                monster.move_shape().shape().area_index(),
                monster.move_shape().shape().get_direction(),
                monster.ai_target().is_some(),
            ))
        })
    else {
        return false;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        JiuMaiDispatcherMonster::when_been_hurted(monster, now());
    }
    if fighting {
        return true;
    }
    let eligible = match attacker.object_type {
        PLAYER_TYPE => game.find_player(attacker.id).is_some_and(|player| {
            player.server_region_id() == Some(region.region_id())
        }),
        MONSTER_TYPE => region.find_monster_by_id(attacker.id).is_some_and(|monster| {
            monster.is_tamed()
                || monster
                    .base_property_key()
                    .and_then(|key| game.find_monster_property_by_origin_name(key))
                    .is_some_and(|property| JiuMaiDispatcherMonster::is_carriage(monster, property))
        }),
        _ => false,
    };
    if eligible {
        let _ = assign_jiumai_target(region, monster_id, attacker);
        return true;
    }
    if attacker.object_type != PLAYER_TYPE {
        return true;
    }

    let Some(area_index) = area_index else {
        return true;
    };
    let retreat = nearest_jiumai_player(game, region, area_index, owner)
        .and_then(|threat| jiumai_step(owner, threat, true))
        .or_else(|| {
            nearest_jiumai_monster(game, region, area_index, owner, monster_id)
                .and_then(|companion| jiumai_step(owner, companion, false))
        })
        .or_else(|| {
            Some((
                direction,
                CShape::get_direction_position(direction, shape_coordinates(owner)).ok()?,
            ))
        });
    if let Some((step_direction, destination)) = retreat {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            JiuMaiDispatcherMonster::set_shape_direction(monster, step_direction);
        }
        move_owned_monster_to(game, region, monster_id, destination, 0, &mut now);
    }
    true
}

fn nearest_jiumai_player<Game, Region>(
    game: &Game,
    region: &Region,
    area_index: usize,
    owner: ShapeView,
) -> Option<ShapeView>
where
    Game: MonsterDispatcherGame,
    Region: EnemySearchDispatcherRegion,
{
    nearest_jiumai_view(
        region
            .player_ids_around_area(area_index)
            .into_iter()
            .filter_map(|player_id| {
                game.find_player(player_id).and_then(|player| {
                    (player.server_region_id() == Some(region.region_id()))
                        .then(|| player.shape_view())
                        .flatten()
                })
            }),
        owner,
    )
}

fn nearest_jiumai_monster<Game, Region>(
    game: &Game,
    region: &Region,
    area_index: usize,
    owner: ShapeView,
    owner_id: i32,
) -> Option<ShapeView>
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
{
    nearest_jiumai_view(
        region
            .monster_ids_around_area(area_index)
            .into_iter()
            .filter(|candidate_id| *candidate_id != owner_id)
            .filter_map(|candidate_id| {
                let candidate = region.find_monster_by_id(candidate_id)?;
                let property = game
                    .find_monster_property_by_origin_name(candidate.base_property_key()?)?;
                candidate.shape_view(property)
            }),
        owner,
    )
}

fn nearest_jiumai_view(
    candidates: impl Iterator<Item = ShapeView>,
    owner: ShapeView,
) -> Option<ShapeView> {
    candidates
        .fold(None, |nearest, candidate| {
            let distance = owner.real_distance(Some(candidate));
            match nearest {
                Some((_, current_distance)) if current_distance <= distance => nearest,
                _ => Some((candidate, distance)),
            }
        })
        .map(|(candidate, _)| candidate)
}

/// Один исходный шаг hurt-отхода: отход строит направление от ориентира к
/// владельцу (`GetLineDir(threat → owner)`), сближение с монстром — от
/// владельца к нему. Возвращает пару машинных вызовов
/// `SetDir` + `GetDirPos(owner, dir)`.
fn jiumai_step(
    owner: ShapeView,
    reference: ShapeView,
    away_from_reference: bool,
) -> Option<(i32, ShapeAreaCoordinates)> {
    let direction = if away_from_reference {
        get_line_direction(
            reference.tile_x,
            reference.tile_y,
            owner.tile_x,
            owner.tile_y,
        )
    } else {
        get_line_direction(
            owner.tile_x,
            owner.tile_y,
            reference.tile_x,
            reference.tile_y,
        )
    };
    let destination =
        CShape::get_direction_position(direction, shape_coordinates(owner)).ok()?;
    Some((direction, destination))
}

const fn shape_coordinates(view: ShapeView) -> ShapeAreaCoordinates {
    ShapeAreaCoordinates {
        x: view.tile_x,
        y: view.tile_y,
    }
}
