//! Базовое состояние GameServer-region `CServerRegion`.
//! BF-области сохраняют заранее установленный центр. FatalBlow и громовые
//! призывы возвращают непринятый объект caller-у для хвоста сериализации;
//! базовая атака боевого духа этого отдельного хвоста не имеет.
//! Добавление Leiming2 не выключает соседние области автоматически;
//! замена прежнего Tianhuo принадлежит его явному caller-у до AddObject.
//!
//! `StartClearPlayerOut` RVA `0x0007BA50`, `ClearPlayerAI` RVA `0x00082FF0`
//! и base `KickOutAllPlayerToReturnPoint` RVA `0x00082820`, resource forwarding
//! `Save/New/Load` `0x0007BA80/0x0007BA90/0x0007C290`, обе `GetArea`
//! `0x00001DB0/0x0007BB60` и `CreateAreaArray` `0x0007BE10`, фазовые defaults
//! `OnWarDeclare/Start/End/Mass` RVA `0x00085560..0x000855B0`, ownership и
//! state accessors `0x000855D0..0x00085680` и `GetReturnPoint`
//! `0x000814F0` имеют статус `IMPLEMENTED, VERIFIED_DISASSEMBLY`.
//! Membership/spatial family `FindChildObject/AddObject/RemoveObject` RVA
//! `0x000821E0/0x00083270/0x0007CE60` и обе `GetShape`
//! `0x0007F390/0x00081300`, а также `GetPlayerAmout` `0x0000C760` имеют этот
//! статус. Spatial transition `RefeashBlock` `0x0007FAD0/0x00081680`,
//! `OnShapeChangeArea` `0x000802A0` и достигнутая `CS_CHANGEAREA` ветвь `AI`
//! `0x00084520` также `IMPLEMENTED, VERIFIED_DISASSEMBLY`.
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`;
//! исходники `serverregion.h/.cpp`. PDB фиксирует
//! `m_listChangeAreaShape +0x1D0`, поля `m_Param +0x214`,
//! `m_lWarNum +0x238`, `m_CityState +0x23C` и clear timer `+0x240..+0x248`.
//! Изменение налога магазина и передача доли вышестоящему региону
//! `0x826E0/0x7C0C0` замкнуты совместно с владельцем канала World в `CGame`.
//! Меню выплаты и ставки `0x7C160/0x7C2B0` сохраняют двухфазный
//! `CNetSession`: клиентские `0xBFF24/0xBFF25`, строгую денежную границу,
//! региональную публикацию `0xBFF26`, журналы и снимок World `0x6012D`.
//! Countdown сохраняет wrapping DWORD comparison и signed remaining; virtual
//! return point, random destination и полный `CPlayer::ChangeRegion` вызываются
//! Game-owner-ом, который владеет region/player maps и runtime side effects.
//!
//! `i32/u32` сохраняют x86 `long/DWORD`; `String` и owned fields заменяют
//! MFC/STL storage без изменения достигнутых эффектов. `timeGetTime` передаётся
//! явным wrapping `now_ms`, пока GameServer runtime-clock owner не подключён.
//! Вход игрока атомарно изымает ID спящих монстров в порядке девяти областей;
//! `CGame` выполняет `WakeUp` и возвращает классифицированный ID после пакета.
//! Встроенный `CRegion` сохраняет исходную inheritance-границу и является
//! владельцем byte-exact tile/security storage; subtype-ы не дублируют клетки.
//! `Save/New` являются точными tail-jump в `CRegion`; `Load` создаёт area-grid
//! только после успешного resource load. `AREA_WIDTH/AREA_HEIGHT` приходят из
//! достигнутого GlobeSetup writer-а и передаются явно. Для положительных
//! размеров grid использует исходное ceiling-деление и row-major
//! `area_x * y + x`; старые zero/negative span и 32-bit allocation overflow
//! становятся локальной typed-границей, а не platform-dependent trap/UB.
//! Старый unbounded `GetArea(index)` заменён `Result`, coordinate overload
//! сохраняет доказанный `nullptr -> Option`.
//! Spatial-tail `CPlayer::SetWarSoulXY/DelWarSoul` (0x0042DF50/0x0042E0A0)
//! использует signed деление координат на 15, включая (-1,-1) -> area (0,0).
//! Наличие нужной области, а не результат Add/DelWarSoul, разрешает запись
//! player point. Выбранный skill завершается через End(int,0) в `CGame`:
//! Set — после target-area gate до изменения map, Delete — до region gate.
//! Старые pointer-valued hash maps выражены registry identity: сами `CShape`
//! остаются у runtime owner-а и разрешаются через `ShapeResolver`. Это
//! сознательная смена формы API без копии shared/derived семантики. Player
//! registry сохраняет vector и first-erase, остальные map assignment —
//! уникальные ключи. После `CArea::PlayerEnter` region вызывает точно
//! подтверждённый virtual `CMoveShape::AutoStartPassiveSkill`; ordered
//! background-очередь принадлежит выбранному `CBaseAI`, а не форме.
//! Технический membership-adapter передаёт целого CMonster/CPlayer, чтобы
//! сохранить GetAI; голый CMoveShape без такого владельца автостарт не делает.
//! Для синхронных callback-ов временно извлечённого региона snapshot хранит
//! только ordered player IDs и координаты areas. Он сохраняет registry-фильтр,
//! row-major и around/team порядок, не копируя owning monsters/NPC и их навыки
//! с единственными visual-ресурсами. После base AI снимок обновляется явно.
//! Завершающую часть GodsBattle для игрока выполняет `CGame` через конкретный
//! подтип после успешного базового пространственного добавления либо удаления.
//! `SetPosXY` только пишет `CS_CHANGEAREA`; ИИ региона не допускает повторных
//! указателей, сбрасывает состояние лишь после первого добавления и применяет
//! очередь после очередей удаления и отсоединения. `OnShapeChangeArea` строит
//! прежнее и новое окружение из девяти областей, отправляет создание и снимок
//! только в области нового окружения, затем выполняет
//! `RemoveObject -> AddObject -> m_pArea` и пробуждает вошедшего игрока.
//! Реальный `CGame::AI` забирает эту упорядоченную очередь после виртуального
//! вызова ИИ, временно проецирует основные хранилища игроков, монстров и NPC и
//! удаляет запись лишь после попытки каждого `OnShapeChangeArea`.
//! Сериализация и отправка сообщений остаются у явного владельца среды;
//! области и отложенная очередь принадлежат `CServerRegion`. `RefeashBlock`
//! сначала снимает все блоки `3`, затем возвращает одиночный блок живым
//! `CMoveShape` и NPC.
//! `m_listDeleteShape/m_listRemoveShape` теперь представлены `IndexSet`:
//! он обеспечивает уникальность и порядок вставки без ручного поиска,
//! а игровой код по-прежнему задаёт порядок применения:
//! Nation clear напрямую ставит туда sleeping monsters, которых active AI
//! scan не видит, сохраняя pointer-unique append исходника. CGame после scan
//! выполняет delete с освобождением owner-а либо remove-only detach, а затем
//! area/region transitions. Player delete-ветвь является post-OnLost tail и
//! не повторяет session callbacks.
//! GM `0x7FC07` использует identity snapshot registry для проверки, что каждый
//! потенциально более ранний `GetShape` candidate разрешим runtime owner-ом;
//! неразрешённый goods/other shape блокирует сценарий до ложного player match.
//! Достигнутые movement commands вызывают здесь именно owner
//! `CShape::SetTileXY`: region дополняет area facts и сам выбирает базовый
//! либо `CMoveShape::SetPosXY`, не перекладывая игровые spatial-факты в
//! процессный runtime и не дублируя tile-center.
//! Aggressive monster tracing использует тот же owner: девяти-area player/pet
//! snapshots сохраняют раздельный storage order, а concrete `OnMove` временно
//! вынимает monster из map, публикует `0xBF506` и меняет block/area membership.
//! Полный startup decoder сохраняет base/area/NPC/cache/monster/weather/
//! setup/param wire-order. NPC и monster создаются собственными factory/spawn
//! methods региона; канонические `CMonsterList/CSkillFactory` приходят прямыми
//! read-only аргументами, а context оставляет только RNG/AI/message effects.
//! Startup NPC-cache воспроизводит достигнутый
//! linked-list traversal MSVC linear hash узким локальным адаптером.
//! War и Country subtype decoder-ы входят в этот owner напрямую.
//! Concrete `AddNpc` уже создаёт `CNpc` через factory type `500`, назначает
//! spawn-поля, проводит его через `AddObject/CArea` и сохраняет owned object.
//! Type `500` lookup безопасно публикует только уникальное имя; observable
//! traversal старого `stdext::hash_map` при дубликатах остаётся у caller
//! context. Owned-NPC удаление сохраняет spatial/registry lifecycle Nation AI.
//! Low-level `AddMonster` аналогично владеет type `600` spawn и хранит
//! original-name key вместо висячего указателя в reloadable MonsterList;
//! skill-init читает фабрику `CGame`, а AI hooks и around serialization
//! остаются concrete context callbacks.
//! Script rectangle removal снимает ordered ID snapshot по live tile и
//! original-name до publication/mutation у `CGame` owner-а.
//! MonsterTalk area lookup использует тот же девяти-area neighborhood и
//! active/sleeping storage, что исходный `CArea::GetAllShapes` pass.
//! Exact EXE подтверждает legacy quirk: его пятый bool не читается, enter
//! message отправляется всегда, а шестой bool подавляет ранний guard-hook.
//! Clock-вариант `AddNpc` получает отдельный tick на каждый объект batch-а:
//! он одновременно питает born-time и concrete spatial membership owner.
//! Монотонные `g_lTotalMonster/g_lTotalNpc` представлены per-region
//! счётчиками с теми же позициями increment: monster после around-send, NPC
//! до `AddObject`; startup owner складывает их без runtime callback-а.
//! One-second AI fragment сохраняет wrapping respawn deadline, сначала пишет
//! last-reset и затем восполняет только deficit `count-living_count`.
//! Для всех concrete region owners этот fragment вызывается реальным
//! `CGame::AI`; typed spawn/spatial block прекращает weather, дальнейший
//! region tail и subtype contender после уже выполненной partial mutation.
//! Следом тот же секундный gate увеличивает weather counter, циклически меняет
//! segment, выбирает первую cumulative RNG-option и публикует `0xBF507`.
//! `BTreeMap` используется только для identity lookup: observable обход
//! старого MSVC `stdext::hash_map` для startup name-cache отдельно сортирует
//! snapshot по exact bucket/key order и не зависит от Rust-map traversal.
//! Уже используемый crate dependency `encoding_rs` заменяет только ANSI
//! преобразование имени в совместимый `String`-view; byte-exact имя остаётся
//! у встроенного `CRegion`, поэтому wire не зависит от Unicode-конверсии.
//! Общий `OnWarTimeOut/OnClearOtherPlayer/OnRefreshRegion` — один PDB-symbol
//! RVA `0x00201A70`, три байта `ret 4`, поэтому базовые defaults — no-op.
//! `GetReturnPoint` сохраняет null-player zero result, local `m_stSetup`
//! priority и fallback в три mutating country-default карты. Constructor
//! `0x000852B0` не инициализирует `m_stSetup`; prefix
//! `DecordSetupFromByteArray` `0x0007EAC0` материализует точные `0x20` байт,
//! затем полностью заменяет ordered set запрещённых для производства товаров;
//! `FindForbidGood` `0x0007D6A0` выполняет точный lookup C-string в этом set.
//! До вызова decoder-а setup остаётся отдельной typed-границей.
//! Ядра weather tick/change, return-setup fallback и war-фазовые с
//! ownership/state accessors делегированы Zone
//! `regions/serverregion/{weather,returnsetup,war}` без смены сигнатур.
//! Достигнутый player-leave call из `RemoveObject` попадает в тот же exact
//! `ret 4` RVA `0x00201A70`, поэтому отдельного наблюдаемого эффекта не имеет.
//! Packet↔ground проход владеет созданными им `CGoods`, точной 49-cell
//! byte-occupancy таблицей `GetDropGoodsPos`, spatial/area membership,
//! protection lookup и delete lifecycle. Остальные внешние ground owners
//! остаются у resolver/runtime и не подменяются второй копией предмета.
//! Оставшиеся отдельные `std::basic_streambuf`/`Unwind@` экспорты сняты общей
//! технической классификацией; domain lifecycle и callbacks не затрагивались.
//! Остальная поверхность файла ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Двухфазные налоговые callback-и передают ещё не применённые действия через
//! общий типизированный `GameEffectJournal`; FIFO между prompt и result
//! сохраняется, а уже выполненные действия в журнал не копируются.

use std::collections::{BTreeMap, BTreeSet};

use encoding_rs::WINDOWS_1251;
use indexmap::IndexSet;

use super::area::{AreaAiContext, AreaWokenMonsterClass, CArea, WarSoulPoint};
use super::build::BuildBlockUpdate;
use super::country::countryparam::CCountryParam;
use super::gameeffectjournal::{GameEffect, SharedGameEffectJournal};
use super::goods::cgoods::CGoods;
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use super::monster::CMonster;
use super::monsterworld::MonsterWorld;
use super::moveshape::{
    CMoveShape, MoveShapeCommandBlock, MoveShapePositionFacts, MoveShapeResolver,
};
use super::npc::CNpc;
use super::player::CPlayer;
use super::region::{
    CRegion, RegionCellAccessBlock, RegionDecodeError, RegionRandomContext, RegionResourceWrite,
    RegionReturnPoint, RegionStorageBlock,
};
use super::shape::{
    BaseShapePositionDispatch, CShape, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeFigure,
    ShapeIdentity, ShapeResolver, ShapeRuntimeFacts, ShapeView,
};
use super::summonshape::{SUMMON_SHAPE_TYPE, SummonedSkillShape};
use crate::nets::netserver::message::GameServerAroundRuntime;
use nebokrai_shared::values::CGuid;
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::public::netsession::{
    NetSessionAsyncResult, NetSessionAsyncResultKind, NetSessionEndpoint,
};
use crate::setup::monsterlist::{
    MonsterProperties, MonsterRegistry, get_monster_property_by_origin_name,
};

pub(crate) use nebokrai_zone::regions::regionparam::RegionParamState;
pub(crate) use nebokrai_zone::regions::serverregion::{
    areagrid::*, blocks::*, geometry::*, membership::*, queries::*, registry::*, returnsetup::*,
    tax::*, transitions::*, war::*, weather::*,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionTaxSessionKind {
    ObtainPayment,
    AdjustRate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionTaxSessionBegin {
    pub(crate) player_id: i32,
    pub(crate) total_tax: u32,
    pub(crate) player_money: u32,
    pub(crate) current_tax_rate: i32,
    pub(crate) max_tax_rate: i32,
}

/// Совмещает исходные владельцы `IAsyncCaller` и `IAsyncCallback` налогового
/// диалога. Очередь переносит сетевой и игровой результат обратно в `CGame`,
/// которому принадлежат игроки, регионы и соединения.
pub(crate) struct RegionTaxSessionEndpoint {
    kind: RegionTaxSessionKind,
    player_id: i32,
    region_id: i32,
    begin: RegionTaxSessionBegin,
    effects: SharedGameEffectJournal,
}

impl RegionTaxSessionEndpoint {
    pub(crate) fn new(
        kind: RegionTaxSessionKind,
        player_id: i32,
        region_id: i32,
        begin: RegionTaxSessionBegin,
        effects: SharedGameEffectJournal,
    ) -> Self {
        Self {
            kind,
            player_id,
            region_id,
            begin,
            effects,
        }
    }
}

impl NetSessionEndpoint for RegionTaxSessionEndpoint {
    fn do_async_call(&self, session_id: i64, password: i32) {
        let begin = self.begin;
        let (first_value, second_value) = match self.kind {
            RegionTaxSessionKind::ObtainPayment => {
                let capacity = 999_999_999_u32.wrapping_sub(begin.player_money);
                (begin.total_tax.min(capacity), None)
            }
            RegionTaxSessionKind::AdjustRate => (
                begin.current_tax_rate as u32,
                Some(begin.max_tax_rate as u32),
            ),
        };
        self.effects.push(GameEffect::RegionTaxPrompt {
            kind: self.kind,
            player_id: begin.player_id,
            session_id,
            password,
            first_value,
            second_value,
        });
    }

    fn on_async_callback(&self, result: NetSessionAsyncResult) {
        if result.kind != NetSessionAsyncResultKind::Result {
            return;
        }
        let Some(value) = result.value else {
            return;
        };
        self.effects.push(GameEffect::RegionTaxResult {
            kind: self.kind,
            player_id: self.player_id,
            region_id: self.region_id,
            value,
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionClearPlayerTick {
    Waiting { remaining_ms: i32, elapsed_ms: u32 },
    Warning { remaining_ms: i32, seconds: i32 },
    Expired,
}

/// BLOCKED_MISSING_FACT: исходный setup decoder не принимает buffer length;
/// truncated `m_stSetup` в safe Rust останавливается на точной границе.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServerReturnSetupInputBlock {
    pub(crate) offset: usize,
    pub(crate) required: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionSetupDecodeError {
    Setup(ServerReturnSetupInputBlock),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionLoadError {
    Region(RegionDecodeError),
    AreaGrid(AreaGridBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServerRegionDecodeInputBlock {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionDecodeError {
    Region(RegionDecodeError),
    AreaGrid(AreaGridBlock),
    Setup(ServerRegionSetupDecodeError),
    Input(ServerRegionDecodeInputBlock),
    Npc(ServerRegionNpcSpawnBlock),
    Monster(ServerRegionMonsterRectBlock),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerRegionNpcSetup {
    pub(crate) show_list: bool,
    pub(crate) picture_id: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) count: i32,
    pub(crate) direction: i32,
    pub(crate) time: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) script: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerRegionMonsterVariant {
    pub(crate) cumulative_odds: u16,
    pub(crate) sign: u16,
    pub(crate) leader_sign: u16,
    pub(crate) leader_distance: u16,
    pub(crate) name: Vec<u8>,
    pub(crate) script: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerRegionMonsterSetup {
    pub(crate) index: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) count: i32,
    pub(crate) reset_time: i32,
    pub(crate) start_time: i32,
    pub(crate) direction: i32,
    pub(crate) living_count: i32,
    pub(crate) last_reset_time_ms: u32,
    pub(crate) variants: Vec<ServerRegionMonsterVariant>,
}

pub(crate) trait ServerRegionDecodeEffectsContext:
    ServerRegionNpcSpawnEffectsContext + ServerRegionMonsterEffectsContext
{
    fn now_millis(&mut self) -> u32;
}

pub(crate) trait ServerRegionDecodeContext:
    ServerRegionDecodeEffectsContext + ServerRegionMonsterContext
{
}

impl<Context> ServerRegionDecodeContext for Context where
    Context: ServerRegionDecodeEffectsContext + ServerRegionMonsterContext + ?Sized
{
}

pub(crate) trait ServerRegionMembershipContext: RegionRandomContext {}

impl<Context: RegionRandomContext + ?Sized> ServerRegionMembershipContext for Context {}

pub(crate) trait RegionMembershipShape {
    fn membership_shape_mut(&mut self) -> &mut CShape;
    fn after_entered_area(&mut self);
}

impl RegionMembershipShape for CShape {
    fn membership_shape_mut(&mut self) -> &mut CShape { self }
    fn after_entered_area(&mut self) {}
}

impl RegionMembershipShape for CMoveShape {
    fn membership_shape_mut(&mut self) -> &mut CShape { self.shape_mut() }
    fn after_entered_area(&mut self) {}
}

impl RegionMembershipShape for CMonster {
    fn membership_shape_mut(&mut self) -> &mut CShape { self.move_shape_mut().shape_mut() }
    fn after_entered_area(&mut self) { self.auto_start_passive_skills(); }
}

impl RegionMembershipShape for CPlayer {
    fn membership_shape_mut(&mut self) -> &mut CShape { self.move_shape_mut().shape_mut() }
    fn after_entered_area(&mut self) { self.auto_start_passive_skills(); }
}

pub(crate) trait ServerRegionNpcSpawnEffectsContext: ServerRegionMembershipContext {
    /// Сохраняет `GS0233` owner-side log при отсутствии свободной позиции.
    fn log_npc_position_failure(&mut self, npc_name: &[u8]);
}

pub(crate) trait ServerRegionNpcContext: ServerRegionNpcSpawnEffectsContext {
    /// Материализует optional `0xBF502`; startup вызывает AddNpc с false.
    fn send_npc_entered_around(&mut self, npc: &CNpc);
}

pub(crate) trait ServerRegionMonsterSpawnEffectsContext: ServerRegionMembershipContext {
    fn log_monster_variant_failure(&mut self, region_id: i32, refresh_index: i32);

    fn log_monster_position_failure(&mut self, origin_name: &[u8]);
}

pub(crate) trait ServerRegionMonsterEffectsContext: ServerRegionMonsterSpawnEffectsContext {
    fn send_monster_entered_around(&mut self, region: &CServerRegion, monster: &CMonster);
}

pub(crate) trait ServerRegionMonsterContext: ServerRegionMonsterEffectsContext {
    /// Virtual `AddGurdMonster(monsterID)` до speed/direction/AddObject для
    /// AI 10/11; concrete war-region сохраняет ID в своём guard owner.
    fn register_guard_monster(&mut self, monster_id: i32);

    /// Virtual `AddGuardIndex(refreshIndex)` после записи refresh metadata
    /// для guard AI 10/11; это регистрация, не немедленный region refresh.
    fn register_guard_index(&mut self, refresh_index: i32);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionMonsterRectBlock {
    MissingRefreshSetup { index: i32 },
    RandomPosition(RegionCellAccessBlock),
    Membership(RegionMembershipBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionNpcSpawnBlock {
    RandomPosition(RegionCellAccessBlock),
    Membership(RegionMembershipBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerRegionNpcSpawnOutcome {
    pub(crate) first_created_id: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServerRegionNpcNameBlock {
    pub(crate) matches: usize,
}

// Для замены покрытия нужны только призванные формы. Заимствованный resolver
// сохраняет обычный area-порядок GetShapes и работает у снятого с CGame региона.
struct RegionSummonShapeResolver<'a>(&'a BTreeMap<i32, SummonedSkillShape>);

impl ShapeResolver for RegionSummonShapeResolver<'_> {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        if identity.object_type != SUMMON_SHAPE_TYPE { return None; }
        let shape = self.0.get(&identity.id)?.shape();
        Some(ShapeView {
            identity: shape.identity(),
            tile_x: shape.get_tile_x().ok()?,
            tile_y: shape.get_tile_y().ok()?,
            pos_x_bits: shape.get_pos_x().to_bits(),
            pos_y_bits: shape.get_pos_y().to_bits(),
            figure: ShapeFigure::default(),
        })
    }
}

pub(crate) use nebokrai_zone::replication::recipients::ServerRegionRecipientsSnapshot;

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerRegion {
    pub(crate) region: CRegion,
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) country: u8,
    pub(crate) war_region_type: i32,
    pub(crate) no_pk: bool,
    pub(crate) no_contribute: bool,
    area_x: i32,
    area_y: i32,
    areas: Vec<CArea>,
    registry: ServerRegionRegistry,
    owned_monsters: MonsterWorld,
    total_spawned_monsters: i32,
    next_monster_id: NextMonsterId,
    owned_npcs: BTreeMap<i32, CNpc>,
    total_spawned_npcs: i32,
    next_npc_id: NextNpcId,
    next_child_id: NextChildId,
    owned_goods: BTreeMap<CGuid, CGoods>,
    owned_skill_phalanxes: BTreeMap<i32, SummonedSkillShape>,
    delete_shapes: IndexSet<ShapeIdentity>,
    remove_shapes: IndexSet<ShapeIdentity>,
    change_area_shapes: IndexSet<ShapeIdentity>,
    change_region_shapes: IndexSet<ShapeIdentity>,
    pub(crate) param: RegionParamState,
    pub(crate) return_setup: Option<ServerReturnSetup>,
    forbidden_make_goods: BTreeSet<Vec<u8>>,
    npc_setups: Vec<ServerRegionNpcSetup>,
    monster_setups: Vec<ServerRegionMonsterSetup>,
    weather_setup: Vec<ServerRegionWeatherTime>,
    current_weather: Vec<ServerRegionWeather>,
    current_weather_segment: usize,
    npc_name_list: Vec<u8>,
    npc_name_list_count: i32,
    current_weather_count: i32,
    pub(crate) war_number: i32,
    pub(crate) city_state: i32,
    pub(crate) kick_out_player: bool,
    pub(crate) kick_out_player_time: i32,
    pub(crate) last_time_ms: u32,
}

impl CServerRegion {
    /// RVA `0x000852B0` и `AddCityGate` `0x001D0F00`: constructor начинает
    /// `m_lChildID` с `1`, factory получает текущее значение, а owner заранее
    /// сохраняет следующее с wrapping `long`-семантикой.
    pub(crate) fn take_child_id(&mut self) -> i32 {
        self.next_child_id.take()
    }

    /// Строит подтверждённый прямой путь навыка с исходным округлением и
    /// снимком блоков; решение об отказе остаётся у конкретного навыка.
    pub(crate) fn straight_skill_path(
        &self,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
        forced_length: Option<u32>,
    ) -> Vec<(i32, i32, u8)> {
        crate::gameserver::appserver::skills::skillpath::straight_skill_path(
            Some(self), source_x, source_y, target_x, target_y, forced_length,
        )
    }

    /// Возвращает живой тип блока клетки для пошагового полёта навыка; ядро
    /// принадлежит Zone `regions/serverregion/blocks`.
    pub(crate) fn skill_cell_block(&self, x: i32, y: i32) -> u8 {
        skill_cell_block(&self.region, x, y)
    }

    pub(crate) const fn tax_rate(&self) -> i32 {
        self.param.current_tax_rate
    }

    pub(crate) const fn max_tax_rate(&self) -> i32 {
        self.param.max_tax_rate
    }

    pub(crate) const fn total_tax_payment(&self) -> u32 {
        self.param.total_tax
    }

    pub(crate) const fn today_tax_payment(&self) -> u32 {
        self.param.today_total_tax
    }

    pub(crate) const fn set_total_tax_payment(&mut self, value: u32) {
        self.param.total_tax = value;
    }

    pub(crate) const fn set_today_tax_payment(&mut self, value: u32) {
        self.param.today_total_tax = value;
    }

    pub(crate) const fn set_tax_rate(&mut self, value: i32) {
        self.param.current_tax_rate = value;
    }

    pub(crate) const fn owned_faction_id(&self) -> i32 {
        self.param.owned_faction_id
    }

    /// Periodic weather fragment `CServerRegion::AI`; caller уже применил
    /// общий one-second gate monster/weather prefix-а. Ядро принадлежит Zone
    /// `regions/serverregion/weather`.
    pub(crate) fn advance_weather_tick(
        &mut self,
        random_below: impl FnMut(i32) -> i32,
    ) -> ServerRegionWeatherTick {
        advance_weather_tick(
            &self.weather_setup,
            &mut self.current_weather,
            &mut self.current_weather_segment,
            &mut self.current_weather_count,
            random_below,
        )
    }

    pub(crate) fn current_weather(&self) -> &[ServerRegionWeather] {
        &self.current_weather
    }

    /// `CServerRegion::ChangeWeather` заменяет текущую погоду единственной
    /// записью с нулевым цветом тумана; сетевую публикацию выполняет владелец
    /// `CGame`, располагающий настоящим сервером сеансов. Ядро принадлежит Zone
    /// `regions/serverregion/weather`.
    pub(crate) fn change_weather(&mut self, weather_index: i32) -> &[ServerRegionWeather] {
        change_weather(&mut self.current_weather, weather_index);
        &self.current_weather
    }

    /// State-owner ядро `AddTaxMoney` принадлежит Zone
    /// `regions/serverregion/tax`; World-публикация superior-доли остаётся у
    /// достигнутого `CGame` caller-а.
    pub(crate) fn add_tax_money(&mut self, amount: u32) -> RegionTaxAddition {
        add_tax_money(&mut self.param, amount)
    }

    /// Exact `CollectTodayTax` принадлежит Zone `regions/serverregion/tax`.
    /// Лог и World-публикация принадлежат достигнутому `CGame` caller-у.
    pub(crate) fn collect_today_tax(&mut self) -> RegionTaxCollection {
        collect_today_tax(&mut self.param)
    }
    /// Сохраняет caller-side gate начала exact `CServerRegion::AI`: signed
    /// `1000 / g_ms`, затем unsigned `s_lAITick % period`.
    pub(crate) fn refresh_monster_groups_for_ai_tick<Context: ServerRegionMonsterContext>(
        &mut self,
        ai_tick: u32,
        tick_interval_ms: i32,
        now_ms: u32,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, ServerRegionMonsterRectBlock> {
        let period = (1_000i32 / tick_interval_ms) as u32;
        if ai_tick % period != 0 {
            return Ok(false);
        }
        self.refresh_monster_groups(
            now_ms,
            area_width,
            area_height,
            monster_registry,
            skill_factory,
            context,
        )?;
        Ok(true)
    }

    /// Выполняет исходный periodic monster refresh fragment после gate region
    /// AI; `now_ms` уже единожды снят caller-ом через `timeGetTime`.
    pub(crate) fn refresh_monster_groups<Context: ServerRegionMonsterContext>(
        &mut self,
        now_ms: u32,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<(), ServerRegionMonsterRectBlock> {
        let move_existing = self
            .return_setup
            .is_some_and(|setup| setup.move_monster_when_refeash != 0);
        let mut due = Vec::new();
        for (index, setup) in self.monster_setups.iter_mut().enumerate() {
            if setup.reset_time > 0
                && setup.reset_time as u32 <= now_ms.wrapping_sub(setup.last_reset_time_ms)
            {
                setup.last_reset_time_ms = now_ms;
                due.push((index, setup.index));
            }
        }

        let due_groups = due.len();
        for (setup_index, refresh_index) in due {
            if move_existing {
                for area in &mut self.areas {
                    if area.plug_list().is_empty() {
                        area.on_refresh_monster(refresh_index);
                    }
                }
            }
            let amount = self.monster_setups[setup_index]
                .count
                .wrapping_sub(self.monster_setups[setup_index].living_count);
            if amount <= 0 {
                continue;
            }
            let setup = self.monster_setups[setup_index].clone();
            self.add_monster_rect(
                &setup,
                amount,
                false,
                false,
                now_ms,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
            )?;
        }
        tracing::trace!(
            region_id = self.id,
            due_groups,
            move_existing,
            "завершено обновление групп монстров региона"
        );
        Ok(())
    }

    /// Country guard-index refresh выбирает одну уже загруженную setup-запись,
    /// немедленно обновляет её DWORD timestamp и восполняет только
    /// `count - living_count`, как отдельная ветвь `RefreshGuard`.
    pub(crate) fn refresh_monster_group_by_index<Context: ServerRegionMonsterContext>(
        &mut self,
        refresh_index: i32,
        now_ms: u32,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, ServerRegionMonsterRectBlock> {
        let Some(setup_index) = self
            .monster_setups
            .iter()
            .position(|setup| setup.index == refresh_index)
        else {
            return Ok(false);
        };
        self.monster_setups[setup_index].last_reset_time_ms = now_ms;
        let amount = self.monster_setups[setup_index]
            .count
            .wrapping_sub(self.monster_setups[setup_index].living_count);
        if amount > 0 {
            let setup = self.monster_setups[setup_index].clone();
            self.add_monster_rect(
                &setup,
                amount,
                false,
                false,
                now_ms,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
            )?;
        }
        Ok(true)
    }

    pub(crate) fn add_monster_rect<Context: ServerRegionMonsterContext>(
        &mut self,
        setup: &ServerRegionMonsterSetup,
        amount: i32,
        remember_setup: bool,
        suppress_guard_registration: bool,
        now_ms: u32,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<(), ServerRegionMonsterRectBlock> {
        if remember_setup {
            self.monster_setups.push(setup.clone());
        }
        let Some(setup_index) = self
            .monster_setups
            .iter()
            .position(|candidate| candidate.index == setup.index)
        else {
            return Err(ServerRegionMonsterRectBlock::MissingRefreshSetup { index: setup.index });
        };

        let mut created = 0usize;
        let mut missing_properties = 0usize;
        let mut remaining = amount;
        while remaining > 0 {
            let random_odds = context.random_below(100);
            let selected = self.monster_setups[setup_index]
                .variants
                .iter()
                .find(|variant| random_odds < i32::from(variant.cumulative_odds))
                .cloned();
            let Some(selected) = selected else {
                let refresh_index = self.monster_setups[setup_index].index;
                context.log_monster_variant_failure(self.id, refresh_index);
                remaining = remaining.wrapping_sub(1);
                continue;
            };

            let refresh = &self.monster_setups[setup_index];
            let position = self
                .region
                .get_random_pos_in_range(
                    refresh.left,
                    refresh.top,
                    refresh.right.wrapping_sub(refresh.left),
                    refresh.bottom.wrapping_sub(refresh.top),
                    context,
                )
                .map_err(ServerRegionMonsterRectBlock::RandomPosition)?;
            if !position.found {
                context.log_monster_position_failure(&selected.name);
            }

            let Some(property) =
                get_monster_property_by_origin_name(monster_registry, &selected.name).cloned()
            else {
                missing_properties = missing_properties.wrapping_add(1);
                remaining = remaining.wrapping_sub(1);
                continue;
            };
            let direction = self.monster_setups[setup_index].direction;
            let id = self
                .add_monster(
                    &property,
                    position.x,
                    position.y,
                    direction,
                    remember_setup,
                    suppress_guard_registration,
                    now_ms,
                    area_width,
                    area_height,
                    skill_factory,
                    context,
                )
                .map_err(ServerRegionMonsterRectBlock::Membership)?;

            let refresh_index = self.monster_setups[setup_index].index;
            self.monster_setups[setup_index].living_count = self.monster_setups[setup_index]
                .living_count
                .wrapping_add(1);
            let monster = self
                .owned_monsters
                .get_mut(&id)
                .expect("успешный AddMonster публикует owned monster");
            monster.set_refresh_data(
                selected.sign,
                selected.leader_sign,
                selected.leader_distance,
                refresh_index,
            );
            if !selected.script.is_empty() && selected.script != b"0" {
                monster.set_script_file(&selected.script);
            }
            created = created.wrapping_add(1);
            if matches!(property.ai, 10 | 11) && !suppress_guard_registration {
                context.register_guard_index(refresh_index);
            }
            remaining = remaining.wrapping_sub(1);
        }
        tracing::trace!(
            region_id = self.id,
            refresh_index = setup.index,
            created,
            missing_properties,
            "завершено создание группы монстров"
        );
        Ok(())
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "literal AddMonster сохраняет исходные spawn flags и owner-границы"
    )]
    pub(crate) fn add_monster<Context: ServerRegionMonsterContext>(
        &mut self,
        property: &MonsterProperties,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        _unused_legacy_flag: bool,
        suppress_guard_registration: bool,
        now_ms: u32,
        area_width: i32,
        area_height: i32,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<i32, RegionMembershipBlock> {
        let id = self.next_monster_id.take();
        let mut monster = crate::gameserver::appserver::baseobject::create_monster(id);
        monster.bind_spawn_property(property);
        monster.initialize_skills(property, skill_factory, &mut |bound| {
            context.random_below(bound)
        });
        monster.initialize_ai(property, now_ms);
        monster
            .move_shape_mut()
            .shape_mut()
            .set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
        if matches!(property.ai, 10 | 11) && !suppress_guard_registration {
            context.register_guard_monster(id);
        }
        monster.set_spawn_speed(property);
        let direction = if (0..8).contains(&direction) {
            direction
        } else {
            context.random_below(8)
        };
        monster
            .move_shape_mut()
            .shape_mut()
            .set_direction(direction);

        let facts = ShapeRuntimeFacts {
            monster: Some(
                if property.tamable == 1 && property.maximum_tame_attempt_count == 0 {
                    super::shape::MonsterAreaClass::Carriage
                } else {
                    super::shape::MonsterAreaClass::Active
                },
            ),
            is_move_shape: true,
            blocks_region_cell: true,
            figure: CMonster::figure(property),
            ..ShapeRuntimeFacts::default()
        };
        self.add_object(
            &mut monster,
            facts,
            area_width,
            area_height,
            now_ms,
            context,
        )?;
        self.owned_monsters.insert(id, monster);
        // Compatibility quirk exact EXE 0x0047EC50: пятый bool не читается,
        // а enter message отправляется безусловно.
        let entered = self
            .owned_monsters
            .get(&id)
            .expect("monster опубликован непосредственно перед send");
        context.send_monster_entered_around(self, entered);
        self.total_spawned_monsters = self.total_spawned_monsters.wrapping_add(1);
        Ok(id)
    }

    /// Создаёт `CSummonedCreature` сразу в каноническом `MonsterWorld`.
    /// Хозяин и срок жизни назначаются до пространственной регистрации и
    /// `0xBF502`, как в исходном `AddSummonedCreature`; отдельной копии
    /// сущности не существует.
    #[allow(
        clippy::too_many_arguments,
        reason = "граница сохраняет исходный порядок создания, назначения хозяина, срока и публикации"
    )]
    pub(crate) fn add_summoned_creature<Context: ServerRegionMonsterContext>(
        &mut self,
        property: &MonsterProperties,
        master: super::masterinfo::MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
        area_width: i32,
        area_height: i32,
        skill_factory: &CSkillFactory,
        context: &mut Context,
        mut now_ms: impl FnMut(&mut Context) -> u32,
    ) -> Result<i32, RegionMembershipBlock> {
        let id = self.next_monster_id.take();
        let mut monster = crate::gameserver::appserver::baseobject::create_monster(id);
        monster.bind_spawn_property(property);
        monster.initialize_skills(property, skill_factory, &mut |bound| {
            context.random_below(bound)
        });
        let special_ai_started_at_ms = if property.ai == 23 {
            now_ms(context)
        } else {
            0
        };
        monster.initialize_ai(property, special_ai_started_at_ms);
        monster
            .move_shape_mut()
            .shape_mut()
            .set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
        monster.set_spawn_speed(property);
        let direction = if (0..8).contains(&direction) {
            direction
        } else {
            context.random_below(8)
        };
        monster
            .move_shape_mut()
            .shape_mut()
            .set_direction(direction);
        monster.set_master_info(master);
        let started_at_ms = now_ms(context);
        let lifecycle = if lifetime_ms == u32::MAX {
            super::summonedcreature::SummonedCreatureLifecycle::new(0, 0)
        } else {
            super::summonedcreature::SummonedCreatureLifecycle::new(started_at_ms, lifetime_ms)
        };
        monster.set_summoned_creature_lifecycle(Some(lifecycle));

        let facts = ShapeRuntimeFacts {
            monster: Some(super::shape::MonsterAreaClass::Active),
            is_move_shape: true,
            blocks_region_cell: true,
            figure: CMonster::figure(property),
            ..ShapeRuntimeFacts::default()
        };
        self.add_object(
            &mut monster,
            facts,
            area_width,
            area_height,
            started_at_ms,
            context,
        )?;
        self.owned_monsters.insert(id, monster);
        let entered = self
            .owned_monsters
            .get(&id)
            .expect("призванный монстр опубликован перед сообщением о входе");
        context.send_monster_entered_around(self, entered);
        self.total_spawned_monsters = self.total_spawned_monsters.wrapping_add(1);
        Ok(id)
    }

    pub(crate) fn find_monster_by_id(&self, id: i32) -> Option<&CMonster> {
        self.owned_monsters.get(&id)
    }

    pub(crate) fn find_monster_by_id_mut(&mut self, id: i32) -> Option<&mut CMonster> {
        self.owned_monsters.get_mut(&id)
    }

    pub(crate) fn add_base_projectile<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: SummonedSkillShape,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, RegionMembershipBlock> {
        self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        )?;
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, phalanx);
        Ok(id)
    }

    pub(crate) fn add_heartless_arrow_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::heartlessarrowphalanx2::CHeartlessArrowPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::heartlessarrowphalanx2::CHeartlessArrowPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        ) { return Err((error, phalanx)); }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::HeartlessArrow(phalanx));
        Ok(id)
    }

    pub(crate) fn add_lighting_arrow_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::lightingarrowphalanx::CLightingArrowPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::lightingarrowphalanx::CLightingArrowPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context,
        ) { return Err((error, phalanx)); }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::LightingArrow(phalanx));
        Ok(id)
    }

    pub(crate) fn add_meteor_arrow_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::meteorarrowphalanx::CMeteorArrowPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::meteorarrowphalanx::CMeteorArrowPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context,
        ) { return Err((error, phalanx)); }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::MeteorArrow(phalanx));
        Ok(id)
    }

    pub(crate) fn add_rain_arrow_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::rainarrowphalanx::CRainArrowPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::rainarrowphalanx::CRainArrowPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context,
        ) { return Err((error, phalanx)); }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::RainArrow(phalanx));
        Ok(id)
    }

    pub(crate) fn add_battle_fairy_base_magic_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::battlefairybasemagicphalanx::CBattleFairyBaseMagicPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, RegionMembershipBlock> {
        self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        )?;
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes
            .insert(id, SummonedSkillShape::BattleFairyBaseMagic(phalanx));
        Ok(id)
    }

    pub(crate) fn add_fatal_blow_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::fatalblowphalanx::CFatalBlowPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::fatalblowphalanx::CFatalBlowPhalanx)> {
        if let Err(block) = self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        ) {
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes
            .insert(id, SummonedSkillShape::FatalBlow(phalanx));
        Ok(id)
    }

    pub(crate) fn add_fire_ball_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::fireballphalanx::CFireBallPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::fireballphalanx::CFireBallPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width,
            area_height, now_ms, context,
        ) { return Err((error, phalanx)); }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::FireBall(phalanx));
        Ok(id)
    }

    pub(crate) fn add_thunder_fire_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::thunderfirephalanx::CThunderFirePhalanx,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, RegionMembershipBlock> {
        phalanx.shape_mut().set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
        self.add_object(phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context)?;
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::ThunderFire(phalanx));
        Ok(id)
    }

    pub(crate) fn add_chaos_sphere_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::chaosspherephalanx::CChaosSpherePhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::chaosspherephalanx::CChaosSpherePhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        ) {
            return Err((error, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::ChaosSphere(phalanx));
        Ok(id)
    }

    pub(crate) fn add_poison_fog_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::poisonfogphalanx::CPoisonFogPhalanx,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::poisonfogphalanx::CPoisonFogPhalanx)> {
        if let Err(block) = self.replace_poison_fog_scopes_in_cell(
            phalanx.skill_level(), tile_x, tile_y, area_width, area_height,
        ) {
            return Err((block, phalanx));
        }
        if let Err(block) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context,
        ) {
            // Caller сохраняет Serialize/Send после отказа AddShape; владение
            // непринятой областью до этого хвоста не переходит региону.
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::PoisonFog(phalanx));
        Ok(id)
    }

    fn replace_poison_fog_scopes_in_cell(
        &mut self, level: i32, tile_x: i32, tile_y: i32, area_width: i32, area_height: i32,
    ) -> Result<usize, RegionMembershipBlock> {
        let mut shapes = Vec::new();
        self.get_shapes(
            tile_x, tile_y, area_width, area_height,
            &RegionSummonShapeResolver(&self.owned_skill_phalanxes), &mut shapes,
        )?;
        let mut replaced = 0;
        for shape in shapes {
            if let Some(SummonedSkillShape::PoisonFog(existing)) =
                self.owned_skill_phalanxes.get_mut(&shape.identity.id)
            {
                existing.replace_affect_region(level, tile_x, tile_y);
                replaced += 1;
            }
        }
        Ok(replaced)
    }

    pub(crate) fn add_god_punishment_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::godpunishmentphalanx::CGodPunishmentPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::godpunishmentphalanx::CGodPunishmentPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context,
        ) {
            return Err((error, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::GodPunishment(phalanx));
        Ok(id)
    }

    pub(crate) fn add_god_thunder_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::godthunderphalanx::CGodThunderPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::godthunderphalanx::CGodThunderPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context,
        ) {
            return Err((error, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::GodThunder(phalanx));
        Ok(id)
    }

    pub(crate) fn add_masked_element_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::maskedelementphalanx::MaskedElementPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::maskedelementphalanx::MaskedElementPhalanx)> {
        if let Err(error) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context,
        ) {
            return Err((error, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::MaskedElement(phalanx));
        Ok(id)
    }

    pub(crate) fn add_thunder_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::thunderphalanx::CThunderPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::thunderphalanx::CThunderPhalanx)> {
        if let Err(block) = self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        ) {
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes
            .insert(id, SummonedSkillShape::Thunder(phalanx));
        Ok(id)
    }

    pub(crate) fn add_thunder_blow_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::thunderblowphalanx::CThunderBlowPhalanx,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, RegionMembershipBlock> {
        phalanx.shape_mut().set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
        for existing in self.owned_skill_phalanxes.values_mut() {
            let SummonedSkillShape::ThunderBlow(existing) = existing else {
                continue;
            };
            if existing.shape().get_tile_x() == Ok(tile_x)
                && existing.shape().get_tile_y() == Ok(tile_y)
            {
                existing.finish();
            }
        }
        self.add_object(phalanx.shape_mut(), ShapeRuntimeFacts::default(), area_width, area_height, now_ms, context)?;
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::ThunderBlow(phalanx));
        Ok(id)
    }

    pub(crate) fn add_thunder_slash_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::thunderslashphalanx::CThunderSlashPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::thunderslashphalanx::CThunderSlashPhalanx)> {
        if let Err(block) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(),
            area_width, area_height, now_ms, context,
        ) {
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::ThunderSlash(phalanx));
        Ok(id)
    }

    pub(crate) fn add_snow_storm_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::snowstormphalanx::CSnowStormPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::snowstormphalanx::CSnowStormPhalanx)> {
        if let Err(block) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(),
            area_width, area_height, now_ms, context,
        ) {
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::SnowStorm(phalanx));
        Ok(id)
    }

    pub(crate) fn add_weak_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::weakphalanx::CWeakPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::weakphalanx::CWeakPhalanx)> {
        if let Err(block) = self.add_object(
            phalanx.shape_mut(), ShapeRuntimeFacts::default(),
            area_width, area_height, now_ms, context,
        ) {
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes.insert(id, SummonedSkillShape::Weak(phalanx));
        Ok(id)
    }

    pub(crate) fn add_leiming2_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::thunder2phalanx::CLeimingPhalanx2,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::thunder2phalanx::CLeimingPhalanx2)> {
        if let Err(block) = self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        ) {
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes
            .insert(id, SummonedSkillShape::Leiming2(phalanx));
        Ok(id)
    }

    pub(crate) fn add_tianhuo_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::tianhuophalanx::CTianhuoPhalanx,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, (RegionMembershipBlock, super::skills::tianhuophalanx::CTianhuoPhalanx)> {
        if let Err(block) = self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        ) {
            return Err((block, phalanx));
        }
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes
            .insert(id, SummonedSkillShape::Tianhuo(phalanx));
        Ok(id)
    }

    pub(crate) fn add_spider_mist_phalanx<Context: ServerRegionMembershipContext>(
        &mut self,
        mut phalanx: super::skills::spidermistphalanx::CSpiderMistPhalanx,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<i32, RegionMembershipBlock> {
        phalanx
            .shape_mut()
            .set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
        self.replace_poison_fog_scopes_in_cell(phalanx.skill_level(), tile_x, tile_y, area_width, area_height)?;
        self.add_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
            area_width,
            area_height,
            now_ms,
            context,
        )?;
        let id = phalanx.shape().identity().id;
        self.owned_skill_phalanxes
            .insert(id, SummonedSkillShape::SpiderMist(phalanx));
        Ok(id)
    }

    pub(crate) fn finish_tianhuo_phalanx(&mut self, id: i32) -> bool {
        let Some(SummonedSkillShape::Tianhuo(phalanx)) =
            self.owned_skill_phalanxes.get_mut(&id)
        else {
            return false;
        };
        phalanx.finish();
        true
    }

    pub(crate) fn find_skill_phalanx(&self, id: i32) -> Option<&SummonedSkillShape> {
        self.owned_skill_phalanxes.get(&id)
    }

    pub(crate) fn find_skill_phalanx_mut(&mut self, id: i32) -> Option<&mut SummonedSkillShape> {
        self.owned_skill_phalanxes.get_mut(&id)
    }

    /// Применяет базовый virtual `SetTileXY` к принадлежащей региону
    /// призванной форме; у этого семейства нет spatial override-а
    /// `CMoveShape`, поэтому area/block membership не перестраивается.
    pub(crate) fn set_owned_skill_phalanx_tile_position(
        &mut self,
        id: i32,
        tile_x: i32,
        tile_y: i32,
    ) -> Option<()> {
        let phalanx = self.owned_skill_phalanxes.get_mut(&id)?;
        phalanx
            .shape_mut()
            .set_tile_xy(
                &mut self.region,
                tile_x,
                tile_y,
                &mut BaseShapePositionDispatch,
            )
            .expect("базовая запись координат не возвращает ошибку");
        Some(())
    }

    pub(crate) fn finish_fatal_blow_phalanx(&mut self, id: i32) -> bool {
        let Some(SummonedSkillShape::FatalBlow(phalanx)) =
            self.owned_skill_phalanxes.get_mut(&id)
        else {
            return false;
        };
        phalanx.finish();
        true
    }

    pub(crate) fn remove_owned_skill_phalanx(
        &mut self,
        id: i32,
    ) -> Result<bool, RegionMembershipBlock> {
        let Some(mut phalanx) = self.owned_skill_phalanxes.remove(&id) else {
            return Ok(false);
        };
        if let Err(error) = self.remove_object(
            phalanx.shape_mut(),
            ShapeRuntimeFacts::default(),
        ) {
            self.owned_skill_phalanxes.insert(id, phalanx);
            return Err(error);
        }
        Ok(true)
    }

    /// `CMonster::OnDied` tail: ordinary refresh-group membership is reduced
    /// once before `Evanish` stages the still-owned shape for region AI.
    pub(crate) fn finish_owned_monster_death(&mut self, id: i32) -> bool {
        let Some(monster) = self.owned_monsters.get_mut(&id) else {
            return false;
        };
        if !monster.is_tamed()
            && let Some(refresh) = self
                .monster_setups
                .iter_mut()
                .find(|setup| setup.index == monster.refresh_index())
            && refresh.living_count > 0
        {
            refresh.living_count -= 1;
        }
        monster.stage_for_delete();
        let identity = monster.move_shape().shape().identity();
        self.stage_delete_shape(identity);
        true
    }

    /// После успешного `CMonsterTaming` приручённый монстр перестаёт занимать
    /// место в исходной группе возрождения, но остаётся живым объектом региона.
    pub(crate) fn finish_owned_monster_taming(&mut self, id: i32) -> bool {
        let Some(monster) = self.owned_monsters.get(&id) else {
            return false;
        };
        if !monster.is_tamed() {
            return false;
        }
        if let Some(refresh) = self
            .monster_setups
            .iter_mut()
            .find(|setup| setup.index == monster.refresh_index())
            && refresh.living_count > 0
        {
            refresh.living_count -= 1;
        }
        true
    }

    pub(crate) fn owned_pet_ids(&self, player_id: i32) -> Vec<i32> {
        self.owned_monsters
            .iter()
            .filter_map(|(id, monster)| monster.is_owned_pet(player_id).then_some(*id))
            .collect()
    }

    pub(crate) fn set_listed_pets_mode(
        &mut self,
        pets: &[super::moveshape::MoveShapePet],
        mode: i32,
    ) -> usize {
        let mut changed = 0usize;
        for pet in pets {
            if pet.object_type != MONSTER_TYPE {
                continue;
            }
            let Some(monster) = self.owned_monsters.get_mut(&pet.id) else {
                continue;
            };
            if !monster.has_pet_ai() {
                continue;
            }
            monster.set_pet_mode(mode);
            changed = changed.wrapping_add(1);
        }
        changed
    }

    pub(crate) fn set_listed_pets_action(
        &mut self,
        pets: &[super::moveshape::MoveShapePet],
        action: i32,
    ) -> usize {
        let mut changed = 0usize;
        for pet in pets {
            if pet.object_type != MONSTER_TYPE {
                continue;
            }
            let Some(monster) = self.owned_monsters.get_mut(&pet.id) else {
                continue;
            };
            if !monster.has_pet_ai() {
                continue;
            }
            monster.set_pet_action(action);
            changed = changed.wrapping_add(1);
        }
        changed
    }

    /// Точный `CMoveShape::SetTargetForAllPets`: target обязан разрешаться как
    /// `CMoveShape`, а питомцы обходятся в порядке canonical `m_vPet` игрока.
    /// Повторная проверка master-а отсутствует и в исходнике: принадлежность
    /// выражает сама запись списка, stale/non-monster элементы пропускаются.
    /// CPet::SetTarget не прерывает текущий навык и не снимает Move;
    /// новая цель AI не подменяет цель сохранённого исполнения.
    pub(crate) fn set_listed_pets_target(
        &mut self,
        pets: &[super::moveshape::MoveShapePet],
        target: ShapeIdentity,
    ) -> usize {
        if !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE | NPC_TYPE)
            || !self.registry.contains(target)
        {
            return 0;
        }
        let mut changed = 0usize;
        for pet in pets {
            if pet.object_type != MONSTER_TYPE {
                continue;
            }
            let Some(monster) = self.owned_monsters.get_mut(&pet.id) else {
                continue;
            };
            if !monster.has_pet_ai() {
                continue;
            }
            monster.set_pet_target(target);
            changed = changed.wrapping_add(1);
        }
        changed
    }

    /// Выполняет условный `CMonster::Evanish` ветви отзыва питомца.
    /// Исходный диспетчер не перепроверяет владельца найденного монстра;
    /// удаление записи из списка игрока выполняется отдельно и безусловно.
    pub(crate) fn evanish_pet_shape(
        &mut self,
        pet_type: i32,
        pet_id: i32,
    ) -> Option<CShape> {
        if pet_type != MONSTER_TYPE {
            return None;
        }
        let pet = self.find_monster_by_id_mut(pet_id)?;
        pet.evanish_pet();
        Some(pet.move_shape().shape().clone())
    }

    /// Snapshot для script `3313`: inclusive tile rectangle и optional exact
    /// original-name filter обходят canonical monster ID order до mutations.
    pub(crate) fn script_monster_ids_in_rect(
        &self,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        original_name: Option<&[u8]>,
    ) -> Vec<i32> {
        self.owned_monsters
            .iter()
            .filter_map(|(monster_id, monster)| {
                let shape = monster.move_shape().shape();
                let (Ok(tile_x), Ok(tile_y)) = (shape.get_tile_x(), shape.get_tile_y()) else {
                    return None;
                };
                (tile_x >= left
                    && tile_x <= right
                    && tile_y >= top
                    && tile_y <= bottom
                    && original_name.is_none_or(|name| monster.original_name() == name))
                .then_some(*monster_id)
            })
            .collect()
    }

    /// `3304 / MonsterTalk` обходит ровно девять area вокруг current player
    /// area и сохраняет внутренний active/sleeping order каждой area.
    pub(crate) fn script_monster_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        let Some(center) = self.areas.get(area_index) else {
            return Vec::new();
        };
        let center = ShapeAreaCoordinates {
            x: center.x(),
            y: center.y(),
        };
        let mut monster_ids = Vec::new();
        for index in self.neighbor_area_indices(center) {
            self.areas[index].append_monster_ids(&mut monster_ids);
        }
        monster_ids.retain(|monster_id| self.owned_monsters.contains_key(monster_id));
        monster_ids
    }

    /// `FindAroundObject(owner, 400)` использует тот же девяти-area порядок:
    /// area идут по `_area`, player ID — в порядке внутреннего vector-а.
    pub(crate) fn player_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        let Some(center) = self.areas.get(area_index) else {
            return Vec::new();
        };
        let center = ShapeAreaCoordinates {
            x: center.x(),
            y: center.y(),
        };
        let mut player_ids = Vec::new();
        for index in self.neighbor_area_indices(center) {
            self.append_registered_player_ids(&self.areas[index], &mut player_ids);
        }
        player_ids
    }

    /// Exact девяти-area `GetAllShapes` обход входа игрока. Area и категории
    /// сохраняют native order; caller сериализует snapshot каждого результата.
    pub(crate) fn shapes_around_area<Resolver: ShapeResolver>(
        &self,
        area_index: usize,
        resolver: &Resolver,
    ) -> Vec<ShapeView> {
        let Some(center) = self.areas.get(area_index) else {
            return Vec::new();
        };
        let center = ShapeAreaCoordinates {
            x: center.x(),
            y: center.y(),
        };
        let registered = RegisteredShapeResolver {
            registry: &self.registry,
            resolver,
        };
        let mut shapes = Vec::new();
        for index in self.neighbor_area_indices(center) {
            self.areas[index].get_all_shapes(&registered, &mut shapes);
        }
        shapes
    }

    /// FindAroundObject для призванных фигур: девять area и внутренний
    /// порядок FindShapes; исключение исходной фигуры остаётся у caller-а.
    pub(crate) fn summon_shapes_around_area<Resolver: ShapeResolver>(
        &self, area_index: usize, resolver: &Resolver,
    ) -> Vec<ShapeView> {
        let Some(center) = self.areas.get(area_index) else { return Vec::new(); };
        let center = ShapeAreaCoordinates { x: center.x(), y: center.y() };
        let registered = RegisteredShapeResolver { registry: &self.registry, resolver };
        let mut shapes = Vec::new();
        for index in self.neighbor_area_indices(center) {
            self.areas[index].find_shapes(
                super::summonshape::SUMMON_SHAPE_TYPE, &registered, &mut shapes,
            );
        }
        shapes
    }

    pub(crate) fn pet_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        let Some(center) = self.areas.get(area_index) else {
            return Vec::new();
        };
        let center = ShapeAreaCoordinates {
            x: center.x(),
            y: center.y(),
        };
        let mut pet_ids = Vec::new();
        for index in self.neighbor_area_indices(center) {
            let _ = self.areas[index].append_pet_ids(&mut pet_ids);
        }
        pet_ids.retain(|pet_id| self.owned_monsters.contains_key(pet_id));
        pet_ids
    }

    pub(crate) fn carriage_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        let Some(center) = self.areas.get(area_index) else {
            return Vec::new();
        };
        let center = ShapeAreaCoordinates {
            x: center.x(),
            y: center.y(),
        };
        let mut carriage_ids = Vec::new();
        for index in self.neighbor_area_indices(center) {
            let _ = self.areas[index].append_carriage_ids(&mut carriage_ids);
        }
        carriage_ids.retain(|carriage_id| self.owned_monsters.contains_key(carriage_id));
        carriage_ids
    }

    pub(crate) fn monster_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        let Some(center) = self.areas.get(area_index) else {
            return Vec::new();
        };
        let center = ShapeAreaCoordinates {
            x: center.x(),
            y: center.y(),
        };
        let mut monster_ids = Vec::new();
        for index in self.neighbor_area_indices(center) {
            self.areas[index].append_monster_ids(&mut monster_ids);
        }
        monster_ids.retain(|monster_id| self.owned_monsters.contains_key(monster_id));
        monster_ids
    }

    /// Exact area-array traversal `FindShapes(600)` без смены pointer owner-а.
    pub(crate) fn area_monster_ids(&self) -> Vec<i32> {
        let mut ids = Vec::new();
        for area in &self.areas {
            area.append_monster_ids(&mut ids);
        }
        ids
    }

    pub(crate) const fn area_count(&self) -> usize {
        self.areas.len()
    }

    pub(crate) fn begin_area_ai<Context: AreaAiContext>(
        &mut self,
        area_index: usize,
        goods_disappear_timer_ms: u32,
        context: &mut Context,
    ) -> Option<Vec<CGuid>> {
        Some(
            self.areas
                .get_mut(area_index)?
                .begin_ai(goods_disappear_timer_ms, context),
        )
    }

    pub(crate) fn finish_area_ground_goods_expiration(&mut self, area_index: usize, ex_id: CGuid) {
        if let Some(area) = self.areas.get_mut(area_index) {
            area.finish_ground_goods_expiration(ex_id);
        }
    }

    pub(crate) fn finish_area_ai<Context: AreaAiContext>(
        &mut self,
        area_index: usize,
        goods_protected_timer_ms: u32,
        context: &mut Context,
    ) {
        if let Some(area) = self.areas.get_mut(area_index) {
            area.finish_ai(goods_protected_timer_ms, context);
        }
    }

    /// Exact `GetDropGoodsPos` использует EXE-таблицу концентрических 7x7
    /// offsets и выбирает первую клетку, где число ground goods меньше
    /// текущего порога. `start_offset` продолжает обход для массового drop.
    pub(crate) fn get_drop_goods_position(
        &self,
        origin_x: i32,
        origin_y: i32,
        mut start_offset: usize,
    ) -> Result<Option<(i32, i32, u32)>, RegionCellAccessBlock> {
        let mut occupancy = [0_u8; 49];
        for goods in self.owned_goods.values() {
            let (Ok(x), Ok(y)) = (goods.shape().get_tile_x(), goods.shape().get_tile_y()) else {
                continue;
            };
            let dx = x.wrapping_sub(origin_x);
            let dy = y.wrapping_sub(origin_y);
            if let Some(index) = DROP_GOODS_OFFSETS
                .iter()
                .position(|offset| *offset == (dx, dy))
            {
                occupancy[index] = occupancy[index].wrapping_add(1);
            }
        }

        let mut threshold = 1_i32;
        for _ in 0..1000 {
            while start_offset < DROP_GOODS_OFFSETS.len() {
                let index = start_offset;
                start_offset += 1;
                if i32::from(occupancy[index]) >= threshold {
                    continue;
                }
                let (dx, dy) = DROP_GOODS_OFFSETS[index];
                let x = origin_x.wrapping_add(dx);
                let y = origin_y.wrapping_add(dy);
                let Some(cell) = self.region.get_cell(x, y)? else {
                    continue;
                };
                if !matches!(cell.block(), 0 | 3) || self.region.get_switch_at(x, y)?.is_some() {
                    continue;
                }
                if cell.block() == 3
                    && self.owned_npcs.values().any(|npc| {
                        npc.move_shape().shape().get_tile_x() == Ok(x)
                            && npc.move_shape().shape().get_tile_y() == Ok(y)
                    })
                {
                    continue;
                }
                let position = self.region.width.wrapping_mul(y).wrapping_add(x) as u32;
                return Ok(Some((x, y, position)));
            }
            threshold = threshold.wrapping_add(1);
            start_offset = 0;
        }
        Ok(None)
    }

    pub(crate) fn add_owned_ground_goods<Context: ServerRegionMembershipContext>(
        &mut self,
        mut goods: CGoods,
        tile_x: i32,
        tile_y: i32,
        particular_attribute: u32,
        now_ms: u32,
        area_width: i32,
        area_height: i32,
        context: &mut Context,
    ) -> Result<ShapeIdentity, (RegionMembershipBlock, CGoods)> {
        goods
            .shape_mut()
            .set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
        let identity = goods.identity();
        let facts = ShapeRuntimeFacts {
            goods: Some(super::shape::GoodsAreaFacts {
                particular_attribute,
            }),
            ..ShapeRuntimeFacts::default()
        };
        if let Err(error) = self.add_object(
            goods.shape_mut(),
            facts,
            area_width,
            area_height,
            now_ms,
            context,
        ) {
            return Err((error, goods));
        }
        self.owned_goods.insert(identity.ex_id, goods);
        Ok(identity)
    }

    pub(crate) fn find_ground_goods(&self, ex_id: CGuid) -> Option<&CGoods> {
        self.owned_goods.get(&ex_id)
    }

    pub(crate) fn find_ground_goods_mut(&mut self, ex_id: CGuid) -> Option<&mut CGoods> {
        self.owned_goods.get_mut(&ex_id)
    }

    pub(crate) fn set_ground_goods_protection(
        &mut self,
        ex_id: CGuid,
        player_id: i32,
        player_team_id: i32,
        now_ms: u32,
    ) -> bool {
        let Some(area_index) = self
            .owned_goods
            .get(&ex_id)
            .and_then(|goods| goods.shape().area_index())
        else {
            return false;
        };
        let Some(area) = self.areas.get_mut(area_index) else {
            return false;
        };
        area.set_goods_protection(ex_id, player_id, player_team_id, now_ms);
        true
    }

    pub(crate) fn can_pick_up_ground_goods(
        &self,
        ex_id: CGuid,
        player_id: i32,
        player_team_id: i32,
    ) -> bool {
        let Some(goods) = self.owned_goods.get(&ex_id) else {
            return false;
        };
        goods
            .shape()
            .area_index()
            .and_then(|index| self.areas.get(index))
            .is_some_and(|area| area.can_pick_up_goods(ex_id, player_id, player_team_id))
    }

    pub(crate) fn remove_owned_ground_goods(
        &mut self,
        ex_id: CGuid,
        particular_attribute: u32,
    ) -> Result<Option<CGoods>, RegionMembershipBlock> {
        let Some(mut goods) = self.owned_goods.remove(&ex_id) else {
            return Ok(None);
        };
        let facts = ShapeRuntimeFacts {
            goods: Some(super::shape::GoodsAreaFacts {
                particular_attribute,
            }),
            ..ShapeRuntimeFacts::default()
        };
        if let Err(error) = self.remove_object(goods.shape_mut(), facts) {
            self.owned_goods.insert(ex_id, goods);
            return Err(error);
        }
        Ok(Some(goods))
    }

    /// Атомарно изымает ID спящих монстров в точном порядке текущей и восьми
    /// соседних областей. `CGame` выполняет `WakeUp` у владельца и возвращает
    /// живые сущности через `restore_woken_monster` после кругового пакета.
    pub(crate) fn take_sleeping_monsters_around_area(
        &mut self,
        center_area_index: usize,
    ) -> Vec<(usize, i32)> {
        let Some(center) = self.areas.get(center_area_index) else {
            return Vec::new();
        };
        let neighbors = center.player_enter_neighbors();
        let mut sleeping = Vec::new();
        for (area_x, area_y) in neighbors {
            let Some(area_index) = self.area_index_by_coordinates(ShapeAreaCoordinates {
                x: area_x,
                y: area_y,
            }) else {
                continue;
            };
            sleeping.extend(
                self.areas[area_index]
                    .take_sleeping_monster_ids()
                    .into_iter()
                    .map(|monster_id| (area_index, monster_id)),
            );
        }
        sleeping
    }

    pub(crate) fn restore_woken_monster(
        &mut self,
        area_index: usize,
        monster_id: i32,
        class: AreaWokenMonsterClass,
    ) {
        if let Some(area) = self.areas.get_mut(area_index) {
            area.push_woken_monster(monster_id, class);
        }
    }

    pub(crate) fn restore_sleeping_monster(&mut self, area_index: usize, monster_id: i32) {
        if let Some(area) = self.areas.get_mut(area_index) {
            area.restore_sleeping_monster(monster_id);
        }
    }

    pub(crate) fn active_shape_candidates(&self, area_index: usize) -> Vec<ShapeIdentity> {
        self.areas
            .get(area_index)
            .map_or_else(Vec::new, CArea::active_shape_candidates)
    }

    pub(crate) fn active_monster_ids_in_area(&self, area_index: usize) -> Vec<i32> {
        let mut ids = Vec::new();
        if let Some(area) = self.areas.get(area_index) {
            area.append_active_monster_ids(&mut ids);
        }
        ids
    }

    /// Row-major monster-подмножество достигнутого active-shape AI-прохода.
    /// `CArea::GetActivedShapes` не включает sleeping monsters; они возвращаются
    /// в этот обход только через подтверждённый переход `WakeUpMonsters`.
    pub(crate) fn active_monster_ids(&self) -> Vec<i32> {
        let mut ids = Vec::new();
        for area in &self.areas {
            area.append_active_monster_ids(&mut ids);
        }
        ids
    }

    pub(crate) fn forget_unresolved_active_shape(
        &mut self,
        area_index: usize,
        identity: ShapeIdentity,
    ) {
        if let Some(area) = self.areas.get_mut(area_index) {
            area.forget_unresolved_active_shape(identity);
        }
    }

    pub(crate) fn owned_shape_change_state(&self, identity: ShapeIdentity) -> Option<i32> {
        match identity.object_type {
            MONSTER_TYPE => self
                .owned_monsters
                .get(&identity.id)
                .map(|monster| monster.move_shape().shape().change_state()),
            NPC_TYPE => self
                .owned_npcs
                .get(&identity.id)
                .map(|npc| npc.move_shape().shape().change_state()),
            GOODS_TYPE => self
                .owned_goods
                .get(&identity.ex_id)
                .map(|goods| goods.shape().change_state()),
            SUMMON_SHAPE_TYPE => self
                .owned_skill_phalanxes
                .get(&identity.id)
                .map(|phalanx| phalanx.shape().change_state()),
            _ => None,
        }
    }

    pub(crate) fn reset_owned_shape_change_state(&mut self, identity: ShapeIdentity) -> bool {
        let shape = match identity.object_type {
            MONSTER_TYPE => self
                .owned_monsters
                .get_mut(&identity.id)
                .map(|monster| monster.move_shape_mut().shape_mut()),
            NPC_TYPE => self
                .owned_npcs
                .get_mut(&identity.id)
                .map(|npc| npc.move_shape_mut().shape_mut()),
            GOODS_TYPE => self
                .owned_goods
                .get_mut(&identity.ex_id)
                .map(CGoods::shape_mut),
            SUMMON_SHAPE_TYPE => self
                .owned_skill_phalanxes
                .get_mut(&identity.id)
                .map(SummonedSkillShape::shape_mut),
            _ => None,
        };
        let Some(shape) = shape else {
            return false;
        };
        shape.set_change_state(SHAPE_CHANGE_NONE);
        true
    }

    /// Второй `OnClearWar` pass читает только sleeping storage каждой area.
    pub(crate) fn sleeping_monster_ids(&self) -> Vec<i32> {
        let mut ids = Vec::new();
        for area in &self.areas {
            area.append_sleeping_monster_ids(&mut ids);
        }
        ids
    }

    /// Сохраняет pointer-unique append в `m_listDeleteShape`; sleeping
    /// monsters попадают сюда напрямую, потому что base AI их не сканирует.
    pub(crate) fn stage_delete_shape(&mut self, identity: ShapeIdentity) -> bool {
        self.delete_shapes.insert(identity)
    }

    pub(crate) fn stage_remove_shape(&mut self, identity: ShapeIdentity) -> bool {
        self.remove_shapes.insert(identity)
    }

    pub(crate) fn has_staged_shape_cleanup(&self) -> bool {
        !self.delete_shapes.is_empty()
            || !self.remove_shapes.is_empty()
            || !self.change_area_shapes.is_empty()
    }

    pub(crate) fn take_staged_remove_shapes(&mut self) -> Vec<ShapeIdentity> {
        std::mem::take(&mut self.remove_shapes).into_iter().collect()
    }

    pub(crate) fn staged_delete_shapes(&self) -> impl Iterator<Item = ShapeIdentity> + '_ {
        self.delete_shapes.iter().copied()
    }

    pub(crate) fn retain_staged_delete_shapes(
        &mut self,
        mut retain: impl FnMut(ShapeIdentity) -> bool,
    ) {
        self.delete_shapes.retain(|identity| retain(*identity));
    }

    pub(crate) fn remove_owned_monster_by_id(
        &mut self,
        id: i32,
        figure: ShapeFigure,
    ) -> Result<bool, RegionMembershipBlock> {
        let Some(mut taken) = self.owned_monsters.take(id) else {
            return Ok(false);
        };
        let facts = ShapeRuntimeFacts {
            monster: Some(super::shape::MonsterAreaClass::Active),
            is_move_shape: true,
            figure,
            ..ShapeRuntimeFacts::default()
        };
        if let Err(error) =
            self.remove_object(taken.monster_mut().move_shape_mut().shape_mut(), facts)
        {
            self.owned_monsters.restore(taken);
            return Err(error);
        }
        self.owned_monsters.discard(taken);
        Ok(true)
    }

    /// `CS_REMOVE` выполняет тот же spatial/registry detach, но сохраняет
    /// concrete owner: в исходнике после virtual `RemoveObject` delete не было.
    pub(crate) fn detach_owned_monster_by_id(
        &mut self,
        id: i32,
        figure: ShapeFigure,
    ) -> Result<bool, RegionMembershipBlock> {
        let Some(mut taken) = self.owned_monsters.take(id) else {
            return Ok(false);
        };
        let facts = ShapeRuntimeFacts {
            monster: Some(super::shape::MonsterAreaClass::Active),
            is_move_shape: true,
            figure,
            ..ShapeRuntimeFacts::default()
        };
        let result = self
            .remove_object(taken.monster_mut().move_shape_mut().shape_mut(), facts)
            .map(|()| true);
        self.owned_monsters.restore(taken);
        result
    }

    pub(crate) fn monster_base_property_keys(&self) -> impl Iterator<Item = &[u8]> {
        self.owned_monsters
            .values()
            .filter_map(CMonster::base_property_key)
    }

    pub(crate) fn add_npc<Context: ServerRegionNpcContext>(
        &mut self,
        setup: &ServerRegionNpcSetup,
        remember_setup: bool,
        send_around: bool,
        now_ms: u32,
        area_width: i32,
        area_height: i32,
        context: &mut Context,
    ) -> Result<ServerRegionNpcSpawnOutcome, ServerRegionNpcSpawnBlock> {
        self.add_npc_with_clock(
            setup,
            remember_setup,
            send_around,
            area_width,
            area_height,
            context,
            |_| now_ms,
        )
    }

    pub(crate) fn add_npc_with_clock<Context: ServerRegionNpcContext>(
        &mut self,
        setup: &ServerRegionNpcSetup,
        remember_setup: bool,
        send_around: bool,
        area_width: i32,
        area_height: i32,
        context: &mut Context,
        now_ms: impl FnMut(&mut Context) -> u32,
    ) -> Result<ServerRegionNpcSpawnOutcome, ServerRegionNpcSpawnBlock> {
        self.add_npc_with_clock_and_entry(
            setup,
            remember_setup,
            send_around,
            area_width,
            area_height,
            context,
            now_ms,
            |_, _, _| {},
            |npc, context| context.send_npc_entered_around(npc),
        )
    }

    /// Вариант для производного региона: завершающая часть виртуального
    /// `AddObject` вызывается после появления NPC в каноническом хранилище и
    /// до следующего NPC, круговой публикации и следующего обращения к
    /// генератору случайных чисел.
    pub(crate) fn add_npc_with_clock_and_entry<Context: ServerRegionNpcSpawnEffectsContext>(
        &mut self,
        setup: &ServerRegionNpcSetup,
        remember_setup: bool,
        send_around: bool,
        area_width: i32,
        area_height: i32,
        context: &mut Context,
        mut now_ms: impl FnMut(&mut Context) -> u32,
        mut after_entry: impl FnMut(&mut CServerRegion, i32, &mut Context),
        mut send_entry: impl FnMut(&CNpc, &mut Context),
    ) -> Result<ServerRegionNpcSpawnOutcome, ServerRegionNpcSpawnBlock> {
        if remember_setup {
            self.npc_setups.push(setup.clone());
        }

        let mut created = 0usize;
        let mut first_created_id = None;
        let mut remaining = setup.count;
        while remaining > 0 {
            let position = self
                .region
                .get_random_pos_in_range(
                    setup.left,
                    setup.top,
                    setup.right.wrapping_sub(setup.left),
                    setup.bottom.wrapping_sub(setup.top),
                    context,
                )
                .map_err(ServerRegionNpcSpawnBlock::RandomPosition)?;
            if !position.found {
                context.log_npc_position_failure(&setup.name);
                remaining = remaining.wrapping_sub(1);
                continue;
            }

            let id = self.next_npc_id.take();
            let mut npc = crate::gameserver::appserver::baseobject::create_npc(id);
            let shape = npc.move_shape_mut().shape_mut();
            shape.base_object_mut().set_name(&setup.name);
            shape.base_object_mut().set_graphics_id(setup.picture_id);
            shape.set_pos_xy_move_order(position.x as f32 + 0.5, position.y as f32 + 0.5);
            let direction = if (0..8).contains(&setup.direction) {
                setup.direction
            } else {
                context.random_below(8)
            };
            shape.set_direction(direction);
            npc.set_show_list(setup.show_list);
            npc.set_script_file(&setup.script);
            npc.set_live_time(setup.time as u32);
            let spawn_tick = now_ms(context);
            if setup.time != 0 {
                npc.set_born_time(spawn_tick);
            }

            self.total_spawned_npcs = self.total_spawned_npcs.wrapping_add(1);
            let facts = ShapeRuntimeFacts {
                is_npc: true,
                is_move_shape: true,
                blocks_region_cell: true,
                figure: ShapeFigure::default(),
                ..ShapeRuntimeFacts::default()
            };
            self.add_object(
                npc.move_shape_mut(),
                facts,
                area_width,
                area_height,
                spawn_tick,
                context,
            )
            .map_err(ServerRegionNpcSpawnBlock::Membership)?;

            self.owned_npcs.insert(id, npc);
            after_entry(self, id, context);
            created = created.wrapping_add(1);
            first_created_id.get_or_insert(id);
            if send_around {
                send_entry(
                    self.owned_npcs
                        .get(&id)
                        .expect("NPC опубликован непосредственно перед send"),
                    context,
                );
            }
            remaining = remaining.wrapping_sub(1);
        }
        tracing::trace!(
            region_id = self.id,
            created,
            requested = setup.count,
            send_around,
            "завершено создание NPC региона"
        );
        Ok(ServerRegionNpcSpawnOutcome { first_created_id })
    }

    pub(crate) fn find_npc_by_id(&self, id: i32) -> Option<&CNpc> {
        self.owned_npcs.get(&id)
    }

    pub(crate) fn find_npc_by_id_mut(&mut self, id: i32) -> Option<&mut CNpc> {
        self.owned_npcs.get_mut(&id)
    }

    /// Возвращает первый matching NPC текущего owned traversal. Повторный
    /// вызов после удаления продолжает очистку со следующего совпадения, как
    /// исходный четырёхкратный lookup в `FourNationWarSys::ClearRegion`.
    pub(crate) fn find_owned_npc_id_by_name(&self, name: &[u8]) -> Option<i32> {
        self.owned_npcs.iter().find_map(|(&id, npc)| {
            (npc.move_shape().shape().base_object().get_name() == name).then_some(id)
        })
    }

    /// Безопасная форма type `500` lookup только для доказанно уникального
    /// имени. Старый `stdext::hash_map` traversal при дубликатах не подменяется
    /// порядком `BTreeMap`: неоднозначный результат становится typed block.
    pub(crate) fn find_npc_by_name(
        &self,
        name: &[u8],
    ) -> Result<Option<&CNpc>, ServerRegionNpcNameBlock> {
        let mut matches = self
            .owned_npcs
            .values()
            .filter(|npc| npc.move_shape().shape().base_object().get_name() == name);
        let Some(first) = matches.next() else {
            return Ok(None);
        };
        let remaining = matches.count();
        if remaining != 0 {
            return Err(ServerRegionNpcNameBlock {
                matches: remaining + 1,
            });
        }
        Ok(Some(first))
    }

    /// Выполняет достигнутый virtual `RemoveObject` для owned NPC и только
    /// после успешного spatial/registry removal отдаёт его Rust owner.
    pub(crate) fn remove_owned_npc_by_id(
        &mut self,
        id: i32,
    ) -> Result<bool, RegionMembershipBlock> {
        let Some(mut npc) = self.owned_npcs.remove(&id) else {
            return Ok(false);
        };
        let facts = ShapeRuntimeFacts {
            is_npc: true,
            is_move_shape: true,
            figure: ShapeFigure::default(),
            ..ShapeRuntimeFacts::default()
        };
        if let Err(error) = self.remove_object(npc.move_shape_mut().shape_mut(), facts) {
            self.owned_npcs.insert(id, npc);
            return Err(error);
        }
        Ok(true)
    }

    pub(crate) fn detach_owned_npc_by_id(
        &mut self,
        id: i32,
    ) -> Result<bool, RegionMembershipBlock> {
        let Some(mut npc) = self.owned_npcs.remove(&id) else {
            return Ok(false);
        };
        let facts = ShapeRuntimeFacts {
            is_npc: true,
            is_move_shape: true,
            ..ShapeRuntimeFacts::default()
        };
        let result = self
            .remove_object(npc.move_shape_mut().shape_mut(), facts)
            .map(|()| true);
        self.owned_npcs.insert(id, npc);
        result
    }

    /// Декодирует полный World -> Game region snapshot в исходном порядке:
    /// base region, area-grid, NPC, NPC-name cache, monster rectangles,
    /// weather, setup и `tagRegionParam`.
    pub(crate) fn decord_from_byte_array<Context: ServerRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, ServerRegionDecodeError> {
        self.decord_from_byte_array_with_npc_entry(
            source,
            cursor,
            include_child,
            area_width,
            area_height,
            monster_registry,
            skill_factory,
            context,
            |_, _, _| {},
        )
    }

    pub(crate) fn decord_from_byte_array_with_npc_entry<
        Context: ServerRegionDecodeContext,
    >(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
        mut after_npc_entry: impl FnMut(&mut CServerRegion, i32, &mut Context),
    ) -> Result<bool, ServerRegionDecodeError> {
        self.region
            .decord_from_byte_array(source, cursor, include_child)
            .map_err(ServerRegionDecodeError::Region)?;
        self.id = self.region.get_id();
        self.country = self
            .region
            .country()
            .expect("успешный CRegion decoder назначает country");
        self.name = WINDOWS_1251.decode(self.region.get_name()).0.into_owned();

        self.war_region_type = read_server_region_i32(source, cursor, "m_WarRegionType")
            .map_err(ServerRegionDecodeError::Input)?;
        self.no_pk = read_server_region_u8(source, cursor, "m_bNoPk")
            .map_err(ServerRegionDecodeError::Input)?
            != 0;
        self.no_contribute = read_server_region_u8(source, cursor, "m_bNoContribute")
            .map_err(ServerRegionDecodeError::Input)?
            != 0;

        self.create_area_array(area_width, area_height)
            .map_err(ServerRegionDecodeError::AreaGrid)?;

        self.npc_setups.clear();
        let npc_count = read_server_region_i32(source, cursor, "m_listNpc.size")
            .map_err(ServerRegionDecodeError::Input)?;
        for _ in 0..npc_count.max(0) {
            let header = read_server_region_bytes(source, cursor, 0x24, "m_listNpc[]")
                .map_err(ServerRegionDecodeError::Input)?;
            let name = read_server_region_c_string(source, cursor, "tagNpc.strName")
                .map_err(ServerRegionDecodeError::Input)?;
            let script = read_server_region_c_string(source, cursor, "tagNpc.strScript")
                .map_err(ServerRegionDecodeError::Input)?;
            let setup = ServerRegionNpcSetup {
                show_list: header[0] != 0,
                picture_id: read_server_region_i32_at(header, 0x04),
                left: read_server_region_i32_at(header, 0x08),
                top: read_server_region_i32_at(header, 0x0C),
                right: read_server_region_i32_at(header, 0x10),
                bottom: read_server_region_i32_at(header, 0x14),
                count: read_server_region_i32_at(header, 0x18),
                direction: read_server_region_i32_at(header, 0x1C),
                // Оригинал безусловно обнуляет wire `lTime` до `AddNpc`.
                time: 0,
                name,
                script,
            };
            // Wire lTime принудительно равен нулю, поэтому AddNpc не читает
            // clock; `now_ms` не участвует ни в одном NPC side effect.
            self.add_npc_with_clock_and_entry(
                &setup,
                true,
                false,
                area_width,
                area_height,
                context,
                |_| 0,
                &mut after_npc_entry,
                |_, _| {},
            )
            .map_err(ServerRegionDecodeError::Npc)?;
        }

        // Compatibility quirk: decoder сбрасывает count, но не очищает bytes
        // прежнего cache-vector перед новым append.
        self.npc_name_list_count = 0;
        for npc_id in legacy_msvc_npc_hash_traversal(self.owned_npcs.keys().copied()) {
            let npc = self
                .owned_npcs
                .get(&npc_id)
                .expect("hash traversal построен из owned NPC keys");
            if !npc.show_list() {
                continue;
            }
            let view = npc
                .shape_view()
                .expect("успешно зарегистрированный startup NPC имеет координаты");
            append_server_region_c_string(&mut self.npc_name_list, npc.name());
            let mut writer = LegacyWriter::new(&mut self.npc_name_list);
            writer.write_i32(view.tile_x);
            writer.write_i32(view.tile_y);
            self.npc_name_list_count = self.npc_name_list_count.wrapping_add(1);
        }

        self.monster_setups.clear();
        let monster_count = read_server_region_i32(source, cursor, "m_listMonster.size")
            .map_err(ServerRegionDecodeError::Input)?;
        for _ in 0..monster_count.max(0) {
            let header = read_server_region_bytes(source, cursor, 0x24, "m_listMonster[]")
                .map_err(ServerRegionDecodeError::Input)?;
            let variant_count =
                read_server_region_i32(source, cursor, "tagMonster.vectorMonsterList.size")
                    .map_err(ServerRegionDecodeError::Input)?;
            let mut variants = Vec::new();
            for _ in 0..variant_count.max(0) {
                let prefix = read_server_region_bytes(
                    source,
                    cursor,
                    0x22,
                    "tagMonster.vectorMonsterList[]",
                )
                .map_err(ServerRegionDecodeError::Input)?;
                variants.push(ServerRegionMonsterVariant {
                    cumulative_odds: read_server_region_u16_at(prefix, 0x00),
                    sign: read_server_region_u16_at(prefix, 0x02),
                    leader_sign: read_server_region_u16_at(prefix, 0x04),
                    leader_distance: read_server_region_u16_at(prefix, 0x06),
                    name: read_server_region_c_string(source, cursor, "tagMonsterList.strName")
                        .map_err(ServerRegionDecodeError::Input)?,
                    script: read_server_region_c_string(source, cursor, "tagMonsterList.strScript")
                        .map_err(ServerRegionDecodeError::Input)?,
                });
            }
            let start_time = read_server_region_i32_at(header, 0x1C);
            let now_ms = context.now_millis();
            let setup = ServerRegionMonsterSetup {
                index: read_server_region_i32_at(header, 0x00),
                left: read_server_region_i32_at(header, 0x04),
                top: read_server_region_i32_at(header, 0x08),
                right: read_server_region_i32_at(header, 0x0C),
                bottom: read_server_region_i32_at(header, 0x10),
                count: read_server_region_i32_at(header, 0x14),
                reset_time: read_server_region_i32_at(header, 0x18),
                start_time,
                direction: read_server_region_i32_at(header, 0x20),
                living_count: 0,
                last_reset_time_ms: now_ms.wrapping_sub(start_time as u32),
                variants,
            };
            let amount = setup.count;
            self.add_monster_rect(
                &setup,
                amount,
                true,
                false,
                now_ms,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
            )
            .map_err(ServerRegionDecodeError::Monster)?;
        }

        self.weather_setup.clear();
        let weather_time_count =
            read_server_region_i32(source, cursor, "m_vectorWeatherSetup.size")
                .map_err(ServerRegionDecodeError::Input)?;
        for _ in 0..weather_time_count.max(0) {
            let time = read_server_region_i32(source, cursor, "tagWeatherTime.lTime")
                .map_err(ServerRegionDecodeError::Input)?;
            let option_count =
                read_server_region_i32(source, cursor, "tagWeatherTime.vectorOption.size")
                    .map_err(ServerRegionDecodeError::Input)?;
            let mut options = Vec::new();
            for _ in 0..option_count.max(0) {
                let cumulative_odds =
                    read_server_region_i32(source, cursor, "tagWeatherTime.tagOption.lOdds")
                        .map_err(ServerRegionDecodeError::Input)?;
                let weather_count = read_server_region_i32(
                    source,
                    cursor,
                    "tagWeatherTime.tagOption.vectorWeather.size",
                )
                .map_err(ServerRegionDecodeError::Input)?;
                let mut weather = Vec::new();
                for _ in 0..weather_count.max(0) {
                    weather.push(ServerRegionWeather {
                        weather_index: read_server_region_i32(
                            source,
                            cursor,
                            "tagWeather.lWeatherIndex",
                        )
                        .map_err(ServerRegionDecodeError::Input)?,
                        fog_color: read_server_region_u32(source, cursor, "tagWeather.dwFogColor")
                            .map_err(ServerRegionDecodeError::Input)?,
                    });
                }
                options.push(ServerRegionWeatherOption {
                    cumulative_odds,
                    weather,
                });
            }
            self.weather_setup
                .push(ServerRegionWeatherTime { time, options });
            let first_time = self
                .weather_setup
                .first()
                .expect("weather time только что добавлен")
                .time;
            self.current_weather_count = first_time.wrapping_mul(60).wrapping_sub(1);
        }

        self.decord_setup_from_byte_array(source, cursor, include_child)
            .map_err(ServerRegionDecodeError::Setup)?;
        let param = read_server_region_bytes(source, cursor, 0x24, "m_Param")
            .map_err(ServerRegionDecodeError::Input)?;
        self.param = RegionParamState {
            region_id: read_server_region_i32_at(param, 0x00),
            max_tax_rate: read_server_region_i32_at(param, 0x04),
            current_tax_rate: read_server_region_i32_at(param, 0x08),
            total_tax: read_server_region_u32_at(param, 0x0C),
            today_total_tax: read_server_region_u32_at(param, 0x10),
            superior_region_id: read_server_region_i32_at(param, 0x14),
            turn_in_tax_rate: read_server_region_i32_at(param, 0x18),
            owned_faction_id: read_server_region_i32_at(param, 0x1C),
            owned_union_id: read_server_region_i32_at(param, 0x20),
        };
        Ok(true)
    }

    pub(crate) const fn npc_name_list_count(&self) -> i32 {
        self.npc_name_list_count
    }

    /// Process-global counters оригинала становятся суммируемым монотонным
    /// state каждого concrete region без изменения позиций increment-ов.
    pub(crate) const fn total_spawned_shapes(&self) -> (i32, i32) {
        (self.total_spawned_monsters, self.total_spawned_npcs)
    }

    pub(crate) fn npc_name_list_length(&self) -> i32 {
        self.npc_name_list.len() as u32 as i32
    }

    pub(crate) fn npc_name_list(&self) -> &[u8] {
        &self.npc_name_list
    }

    pub(crate) fn npc_setups(&self) -> &[ServerRegionNpcSetup] {
        &self.npc_setups
    }

    pub(crate) fn monster_setups(&self) -> &[ServerRegionMonsterSetup] {
        &self.monster_setups
    }

    /// Точный `GetMonsterRefeashTime`: поиск идёт по индексу настройки,
    /// нулевой интервал сброса означает отсутствие таймера. `timeGetTime` и
    /// обе временные отметки остаются в 32-битном кольце; только итоговый
    /// битовый остаток трактуется как знаковый перед ограничением снизу и
    /// переводом в секунды.
    pub(crate) fn monster_refresh_remaining_seconds(&self, index: i32, now_ms: u32) -> i32 {
        let Some(refresh) = self
            .monster_setups
            .iter()
            .find(|refresh| refresh.index == index)
            .filter(|refresh| refresh.reset_time != 0)
        else {
            return -1;
        };
        let remaining = refresh
            .reset_time
            .cast_unsigned()
            .wrapping_sub(now_ms)
            .wrapping_add(refresh.last_reset_time_ms) as i32;
        remaining.max(0) / 1_000
    }

    pub(crate) fn weather_setup(&self) -> &[ServerRegionWeatherTime] {
        &self.weather_setup
    }

    /// Делегирует exact `CServerRegion::New` в базовый `CRegion::New`.
    pub(crate) fn new_region(&mut self) -> Result<i32, RegionStorageBlock> {
        self.region.new_region()
    }

    /// Делегирует exact `CServerRegion::Save` в базовый resource serializer.
    pub(crate) fn save_region_resource(
        &mut self,
    ) -> Result<RegionResourceWrite, RegionStorageBlock> {
        self.region.set_id(self.id);
        self.region.save_resource()
    }

    /// Сохраняет `Load -> success gate -> CreateAreaArray` без domain effects.
    pub(crate) fn load_region_resource(
        &mut self,
        source: Option<&[u8]>,
        area_width: i32,
        area_height: i32,
    ) -> Result<bool, ServerRegionLoadError> {
        self.region.set_id(self.id);
        let loaded = self
            .region
            .load_resource(source)
            .map_err(ServerRegionLoadError::Region)?;
        if !loaded {
            return Ok(false);
        }
        self.create_area_array(area_width, area_height)
            .map_err(ServerRegionLoadError::AreaGrid)?;
        Ok(true)
    }

    /// Пересоздаёт весь area-grid и назначает каждой клетке region parent/X/Y.
    /// Ядро точного `CreateAreaArray` принадлежит Zone
    /// `regions/serverregion/areagrid`.
    pub(crate) fn create_area_array(
        &mut self,
        area_width: i32,
        area_height: i32,
    ) -> Result<(), AreaGridBlock> {
        create_area_array(
            &mut self.areas,
            &mut self.area_x,
            &mut self.area_y,
            self.region.width,
            self.region.height,
            area_width,
            area_height,
        )
    }

    /// Safe-граница исходного unbounded `GetArea(long)`; ядро принадлежит
    /// Zone `regions/serverregion/areagrid`.
    pub(crate) fn get_area_by_index(&self, index: i32) -> Result<&CArea, AreaIndexBlock> {
        get_area_by_index(&self.areas, index)
    }

    /// Сохраняет bounds-check и `nullptr` coordinate-overload-а; ядро
    /// принадлежит Zone `regions/serverregion/areagrid`.
    pub(crate) fn get_area(&self, x: i32, y: i32) -> Option<&CArea> {
        get_area(&self.areas, self.area_x, self.area_y, x, y)
    }

    pub(crate) fn block_at(&self, x: i32, y: i32) -> Option<u8> {
        block_at(&self.region, x, y)
    }

    /// Применяет достигнутый virtual `SetBlock(tile_x, tile_y, block)` к
    /// карте именно того региона, которому принадлежит build.
    pub(crate) fn apply_build_block(&mut self, update: BuildBlockUpdate) -> bool {
        update.region_id == self.id
            && self
                .region
                .set_block(update.tile_x, update.tile_y, update.block as u8)
                .is_ok()
    }

    /// Mutable counterpart точного coordinate-overload `GetArea`; нужен
    /// только owner-у war-soul map, который уже владеет всем area-grid.
    /// Ядро принадлежит Zone `regions/serverregion/areagrid`.
    fn get_area_mut(&mut self, x: i32, y: i32) -> Option<&mut CArea> {
        get_area_mut(&mut self.areas, self.area_x, self.area_y, x, y)
    }

    pub(crate) fn has_war_soul_area(&self, point: WarSoulPoint) -> bool {
        has_war_soul_area(&self.areas, self.area_x, self.area_y, point)
    }

    /// Материализует spatial tail `CPlayer::SetWarSoulXY`: target area должна
    /// существовать. Ядро принадлежит Zone `regions/serverregion/areagrid`.
    pub(crate) fn set_war_soul_position(
        &mut self,
        player_id: u32,
        previous: WarSoulPoint,
        target: WarSoulPoint,
    ) -> bool {
        set_war_soul_position(
            &mut self.areas,
            self.area_x,
            self.area_y,
            player_id,
            previous,
            target,
        )
    }

    /// `CPlayer::DelWarSoul` сбрасывает player point только при найденной
    /// area. Ядро принадлежит Zone `regions/serverregion/areagrid`.
    pub(crate) fn delete_war_soul(&mut self, player_id: u32, point: WarSoulPoint) -> bool {
        delete_war_soul(&mut self.areas, self.area_x, self.area_y, player_id, point)
    }

    /// Точный `GetWarSoulXY` одной области боевого духа; ядро принадлежит
    /// Zone `regions/serverregion/areagrid`.
    pub(crate) fn war_souls_at(&self, x: i32, y: i32) -> BTreeMap<u32, WarSoulPoint> {
        war_souls_at(&self.areas, self.area_x, self.area_y, x, y)
    }

    /// Собирает player IDs одной area без чтения их координат: исходный
    /// `CArea::FindShapes(400)` использовал только RTTI и inherited socket ID.
    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn find_player_ids_in_area(
        &self,
        area_x: i32,
        area_y: i32,
        destination: &mut Vec<i32>,
    ) {
        find_player_ids_in_area(
            &self.areas,
            self.area_x,
            self.area_y,
            area_x,
            area_y,
            &self.registry,
            destination,
        )
    }

    /// Обходит все `CArea` в физическом row-major storage order и сохраняет
    /// exact `FindShapes(400)` filtering через registry owning region-а.
    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn find_all_player_ids(&self, destination: &mut Vec<i32>) {
        find_all_player_ids(&self.areas, &self.registry, destination)
    }

    /// Собирает per-area списки player identities; ядро принадлежит Zone
    /// `regions/serverregion/queries`, snapshot-тип — Zone
    /// `replication/recipients`.
    pub(crate) fn recipients_snapshot(&self) -> ServerRegionRecipientsSnapshot {
        recipients_snapshot(self.id, self.area_x, self.area_y, &self.areas, &self.registry)
    }

    /// Exact `m_vPlayers` storage order, который Nation kick обходит
    /// напрямую, не через area scan `FindAllPlayer`.
    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn registered_player_ids(&self) -> Vec<i32> {
        registered_player_ids(&self.registry)
    }

    /// Регистрирует уже созданный factory-объект без выдуманного второго
    /// `CShape`. В оригинале virtual factory региона создавал объект и сразу
    /// проводил его через `AddObject`; derived city/country owners затем лишь
    /// сохраняли тот же pointer в своих ordered maps.
    pub(crate) fn register_stationary_child(
        &mut self,
        view: ShapeView,
        area_width: i32,
        area_height: i32,
    ) -> Result<(), RegionMembershipBlock> {
        validate_area_span(area_width, area_height)?;
        let facts = ShapeRuntimeFacts {
            figure: view.figure,
            ..ShapeRuntimeFacts::default()
        };
        self.registry.add(view.identity, facts);
        if let Some(area_index) =
            self.area_index_for_tile(view.tile_x, view.tile_y, area_width, area_height)
        {
            self.areas[area_index].add_object(view.identity, facts, 0);
            Ok(())
        } else {
            self.registry.remove(view.identity);
            Ok(())
        }
    }

    /// Owned identity snapshot для проверки полноты resolver-а перед
    /// pointer-sensitive `OnGMMessage 0x7FC07` scan.
    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn registered_shape_identities(&self) -> Vec<ShapeIdentity> {
        registered_shape_identities(&self.registry)
    }

    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn has_registered_shape(&self, identity: ShapeIdentity) -> bool {
        has_registered_shape(&self.registry, identity)
    }

    /// Безопасно заменяет исходный `CArea::m_pFather`: пара принимается только
    /// если area действительно принадлежит этому server-region.
    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn find_player_ids_in_area_object(
        &self,
        area: &CArea,
        destination: &mut Vec<i32>,
    ) -> bool {
        find_player_ids_in_area_object(&self.areas, area, &self.registry, destination)
    }

    /// Ядро принадлежит Zone `regions/serverregion/queries`; обвязка нужна
    /// девяти-area обходу `player_ids_around_area`.
    fn append_registered_player_ids(&self, area: &CArea, destination: &mut Vec<i32>) {
        append_registered_player_ids(&self.registry, area, destination)
    }

    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn find_child_object<Resolver: ShapeResolver>(
        &self,
        object_type: i32,
        id: i32,
        ex_id: CGuid,
        resolver: &Resolver,
    ) -> Option<ShapeView> {
        find_child_object(&self.registry, object_type, id, ex_id, resolver)
    }

    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn contains_child_object<Resolver: ShapeResolver>(
        &self,
        shape: &CShape,
        resolver: &Resolver,
    ) -> bool {
        contains_child_object(&self.registry, shape, resolver)
    }

    /// Ядро принадлежит Zone `regions/serverregion/queries`.
    pub(crate) fn get_player_amount(&self) -> u32 {
        get_player_amount(&self.registry)
    }

    pub(crate) fn add_object<Member: RegionMembershipShape, Context: ServerRegionMembershipContext>(
        &mut self,
        member: &mut Member,
        facts: ShapeRuntimeFacts,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<(), RegionMembershipBlock> {
        self.add_object_with_area_entry(
            member,
            facts,
            area_width,
            area_height,
            now_ms,
            context,
            |_, _, _| {},
        )
    }

    /// Ядро принадлежит Zone `regions/serverregion/membership`; RTTI-факт
    /// derived owner-а приходит через `RegionMembershipShape`, entry-effects
    /// (hook перед входом и virtual `AfterEnteredArea`) выполняет эта обвязка
    /// над всем переходным агрегатом в исходном порядке.
    pub(crate) fn add_object_with_area_entry<Member: RegionMembershipShape, Context: ServerRegionMembershipContext>(
        &mut self,
        member: &mut Member,
        facts: ShapeRuntimeFacts,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
        mut before_move_shape_entry: impl FnMut(&mut CServerRegion, usize, &mut Context),
    ) -> Result<(), RegionMembershipBlock> {
        let entered = add_object_with_area_entry(
            &mut self.region,
            &mut self.areas,
            self.area_x,
            self.area_y,
            &mut self.registry,
            self.id,
            member.membership_shape_mut(),
            facts,
            area_width,
            area_height,
            now_ms,
            context,
        )?;
        if let AreaEntryOutcome::EnteredArea { area_index } = entered {
            before_move_shape_entry(self, area_index, context);
            if facts.is_move_shape {
                member.after_entered_area();
            }
        }
        Ok(())
    }

    /// Ядро принадлежит Zone `regions/serverregion/membership`.
    pub(crate) fn remove_object(
        &mut self,
        shape: &mut CShape,
        facts: ShapeRuntimeFacts,
    ) -> Result<(), RegionMembershipBlock> {
        remove_object(&mut self.region, &mut self.areas, &mut self.registry, shape, facts)
    }

    /// Ядро принадлежит Zone `regions/serverregion/membership`.
    pub(crate) fn set_move_shape_position(
        &mut self,
        shape: &mut CShape,
        x: f32,
        y: f32,
        facts: MoveShapePositionFacts,
    ) -> Result<(), RegionMembershipBlock> {
        set_move_shape_position(&mut self.region, &self.areas, shape, x, y, facts)
    }

    /// Ядро принадлежит Zone `regions/serverregion/membership`.
    pub(crate) fn set_move_shape_tile_position(
        &mut self,
        shape: &mut CShape,
        tile_x: i32,
        tile_y: i32,
        facts: MoveShapePositionFacts,
    ) -> Result<(), RegionMembershipBlock> {
        set_move_shape_tile_position(&mut self.region, &self.areas, shape, tile_x, tile_y, facts)
    }

    /// Временно освобождает значение generational slot, чтобы его virtual
    /// `OnMove` мог одновременно изменить region membership и отправить wire.
    pub(crate) fn move_owned_monster(
        &mut self,
        monster_id: i32,
        destination_x: i32,
        destination_y: i32,
        run: i32,
        figure: ShapeFigure,
        area_width: i32,
        area_height: i32,
        around: &GameServerAroundRuntime<'_>,
    ) -> Option<Result<(), MoveShapeCommandBlock>> {
        let mut taken = self.owned_monsters.take(monster_id)?;
        let facts = taken
            .monster_mut()
            .movement_position_facts(figure, area_width, area_height);
        let result = taken.monster_mut().move_shape_mut().on_move(
            Some(self),
            destination_x,
            destination_y,
            run,
            facts,
            around,
        );
        self.owned_monsters.restore(taken);
        Some(result)
    }

    /// `CPet::OnFallowingSchedule` far-master branch: wire `BF603` и
    /// canonical spatial mutation принадлежат тому же concrete monster.
    pub(crate) fn set_owned_monster_position(
        &mut self,
        monster_id: i32,
        destination_x: i32,
        destination_y: i32,
        figure: ShapeFigure,
        area_width: i32,
        area_height: i32,
        around: &GameServerAroundRuntime<'_>,
    ) -> Option<Result<bool, MoveShapeCommandBlock>> {
        let mut taken = self.owned_monsters.take(monster_id)?;
        let facts = taken
            .monster_mut()
            .movement_position_facts(figure, area_width, area_height);
        let result = taken.monster_mut().move_shape_mut().on_set_position(
            Some(self),
            destination_x,
            destination_y,
            facts,
            around,
        );
        self.owned_monsters.restore(taken);
        Some(result)
    }

    /// Применяет уже опубликованную `CHANGE_POSITION` к каноническому
    /// владельцу монстра без повторной wire-доставки.
    pub(crate) fn set_owned_monster_tile_position(
        &mut self,
        monster_id: i32,
        destination_x: i32,
        destination_y: i32,
        figure: ShapeFigure,
        area_width: i32,
        area_height: i32,
    ) -> Option<Result<(), RegionMembershipBlock>> {
        let mut taken = self.owned_monsters.take(monster_id)?;
        let facts = taken
            .monster_mut()
            .movement_position_facts(figure, area_width, area_height);
        let result = self.set_move_shape_tile_position(
            taken.monster_mut().move_shape_mut().shape_mut(),
            destination_x,
            destination_y,
            facts,
        );
        self.owned_monsters.restore(taken);
        Some(result)
    }

    /// Применяет уже опубликованную `CHANGE_POSITION` к каноническому NPC;
    /// временное изъятие сохраняет единственного владельца spatial-состояния.
    pub(crate) fn set_owned_npc_tile_position(
        &mut self,
        npc_id: i32,
        destination_x: i32,
        destination_y: i32,
        area_width: i32,
        area_height: i32,
    ) -> Option<Result<(), RegionMembershipBlock>> {
        let mut npc = self.owned_npcs.remove(&npc_id)?;
        let facts = npc.movement_position_facts(area_width, area_height);
        let result = self.set_move_shape_tile_position(
            npc.move_shape_mut().shape_mut(),
            destination_x,
            destination_y,
            facts,
        );
        self.owned_npcs.insert(npc_id, npc);
        Some(result)
    }

    /// Временно освобождает поколенческую ячейку, чтобы `ForceMove`
    /// одновременно отправил пакет, изменил пространственное членство и
    /// поставил исходное ожидание искусственному интеллекту.
    #[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, геометрию и длительность перемещения")]
    pub(crate) fn force_move_owned_monster(
        &mut self,
        monster_id: i32,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        figure: ShapeFigure,
        area_width: i32,
        area_height: i32,
        around: &GameServerAroundRuntime<'_>,
        now_ms: impl FnOnce() -> u32,
    ) -> Option<Result<bool, MoveShapeCommandBlock>> {
        let mut taken = self.owned_monsters.take(monster_id)?;
        let facts = taken
            .monster_mut()
            .movement_position_facts(figure, area_width, area_height);
        let result = taken.monster_mut().move_shape_mut().force_move(
            Some(self),
            destination_x,
            destination_y,
            duration_ms,
            facts,
            around,
        );
        // SetTileXY возвращает void: ошибка уже выполненной пространственной
        // попытки не отменяет последующее ожидание AI в ForceMove.
        if matches!(result, Ok(true) | Err(MoveShapeCommandBlock::Position(_)))
            && let Some(ai) = taken.monster_mut().selected_base_ai_mut()
        {
            ai.begin_active_stand(duration_ms, now_ms());
        }
        self.owned_monsters.restore(taken);
        Some(result)
    }

    /// Pointer-unique append area AI scan; marker сбрасывает caller только
    /// после `true`, потому что duplicate исходник оставлял неизменным.
    /// Ядро принадлежит Zone `regions/serverregion/transitions`.
    pub(crate) fn stage_area_transition(&mut self, identity: ShapeIdentity) -> bool {
        stage_area_transition(&mut self.change_area_shapes, identity)
    }

    /// Возвращает ordered snapshot, не очищая исходный list до применения всех
    /// `OnShapeChangeArea`, как в конце original region AI.
    /// Ядро принадлежит Zone `regions/serverregion/transitions`.
    pub(crate) fn staged_area_transitions(&self) -> Vec<ShapeIdentity> {
        staged_area_transitions(&self.change_area_shapes)
    }

    /// Ядро принадлежит Zone `regions/serverregion/transitions`.
    pub(crate) fn clear_staged_area_transitions(&mut self) {
        clear_staged_area_transitions(&mut self.change_area_shapes);
    }

    /// Ядро принадлежит Zone `regions/serverregion/transitions`.
    pub(crate) fn stage_region_transition(&mut self, identity: ShapeIdentity) -> bool {
        stage_region_transition(&mut self.change_region_shapes, identity)
    }

    /// Ядро принадлежит Zone `regions/serverregion/transitions`.
    pub(crate) fn take_staged_region_transitions(&mut self) -> Vec<ShapeIdentity> {
        take_staged_region_transitions(&mut self.change_region_shapes)
    }

    /// Ядро принадлежит Zone `regions/serverregion/transitions`.
    pub(crate) fn plan_area_transition<Resolver: ShapeResolver>(
        &self,
        shape: &CShape,
        resolver: &Resolver,
    ) -> Result<Option<AreaTransitionPlan>, AreaTransitionBlock> {
        plan_area_transition(
            &self.areas,
            self.area_x,
            self.area_y,
            &self.registry,
            shape,
            resolver,
        )
    }

    /// Ядро принадлежит Zone `regions/serverregion/transitions`.
    pub(crate) fn commit_area_transition(
        &mut self,
        shape: &mut CShape,
        facts: ShapeRuntimeFacts,
        now_ms: u32,
        plan: &AreaTransitionPlan,
    ) -> bool {
        commit_area_transition(&mut self.areas, shape, facts, now_ms, plan)
    }

    pub(crate) fn plan_owned_monster_area_transition<Resolver: ShapeResolver>(
        &self,
        monster_id: i32,
        resolver: &Resolver,
    ) -> Option<Result<Option<AreaTransitionPlan>, AreaTransitionBlock>> {
        let monster = self.owned_monsters.get(&monster_id)?;
        Some(self.plan_area_transition(monster.move_shape().shape(), resolver))
    }

    pub(crate) fn commit_owned_monster_area_transition(
        &mut self,
        monster_id: i32,
        figure: super::shape::ShapeFigure,
        now_ms: u32,
        plan: &AreaTransitionPlan,
    ) -> Option<bool> {
        let mut taken = self.owned_monsters.take(monster_id)?;
        let result = self.commit_area_transition(
            taken.monster_mut().move_shape_mut().shape_mut(),
            ShapeRuntimeFacts {
                monster: Some(super::shape::MonsterAreaClass::Active),
                is_move_shape: true,
                figure,
                ..ShapeRuntimeFacts::default()
            },
            now_ms,
            plan,
        );
        self.owned_monsters.restore(taken);
        Some(result)
    }

    pub(crate) fn plan_owned_npc_area_transition<Resolver: ShapeResolver>(
        &self,
        npc_id: i32,
        resolver: &Resolver,
    ) -> Option<Result<Option<AreaTransitionPlan>, AreaTransitionBlock>> {
        let npc = self.owned_npcs.get(&npc_id)?;
        Some(self.plan_area_transition(npc.move_shape().shape(), resolver))
    }

    pub(crate) fn commit_owned_npc_area_transition(
        &mut self,
        npc_id: i32,
        now_ms: u32,
        plan: &AreaTransitionPlan,
    ) -> Option<bool> {
        let mut npc = self.owned_npcs.remove(&npc_id)?;
        let result = self.commit_area_transition(
            npc.move_shape_mut().shape_mut(),
            ShapeRuntimeFacts {
                is_npc: true,
                is_move_shape: true,
                ..ShapeRuntimeFacts::default()
            },
            now_ms,
            plan,
        );
        self.owned_npcs.insert(npc_id, npc);
        Some(result)
    }

    /// Ядро принадлежит Zone `regions/serverregion/blocks`; alive RTTI-fact
    /// приходит из resolver-а переходного владельца `CMoveShape`.
    pub(crate) fn refresh_blocks<Resolver: MoveShapeResolver>(
        &mut self,
        resolver: &Resolver,
    ) -> Result<(), RegionMembershipBlock> {
        refresh_blocks(
            &mut self.region,
            &self.areas,
            &self.registry,
            resolver,
            &|identity| resolver.move_shape_is_alive(identity),
        )
    }

    /// Ядро принадлежит Zone `regions/serverregion/blocks`.
    pub(crate) fn refresh_block<Resolver: MoveShapeResolver>(
        &mut self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        resolver: &Resolver,
    ) -> Result<(), RegionMembershipBlock> {
        refresh_block(
            &mut self.region,
            &self.areas,
            self.area_x,
            self.area_y,
            &self.registry,
            tile_x,
            tile_y,
            area_width,
            area_height,
            resolver,
            &|identity| resolver.move_shape_is_alive(identity),
        )
    }

    /// Ядро принадлежит Zone `regions/serverregion/blocks`.
    pub(crate) fn get_shape<Resolver: ShapeResolver>(
        &self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        resolver: &Resolver,
    ) -> Result<Option<ShapeView>, RegionMembershipBlock> {
        get_shape(
            &self.areas,
            self.area_x,
            self.area_y,
            &self.registry,
            tile_x,
            tile_y,
            area_width,
            area_height,
            resolver,
        )
    }

    /// Ядро принадлежит Zone `regions/serverregion/blocks`.
    pub(crate) fn get_shapes<Resolver: ShapeResolver>(
        &self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        resolver: &Resolver,
        destination: &mut Vec<ShapeView>,
    ) -> Result<(), RegionMembershipBlock> {
        get_shapes(
            &self.areas,
            self.area_x,
            self.area_y,
            &self.registry,
            tile_x,
            tile_y,
            area_width,
            area_height,
            resolver,
            destination,
        )
    }

    /// Порядок девяти-area окружения принадлежит Zone
    /// `regions/serverregion/areagrid`.
    fn neighbor_area_indices(&self, center: ShapeAreaCoordinates) -> Vec<usize> {
        neighbor_area_indices(self.area_x, self.area_y, center)
    }

    fn area_index_by_coordinates(&self, coordinates: ShapeAreaCoordinates) -> Option<usize> {
        area_index_by_coordinates(self.area_x, self.area_y, coordinates)
    }

    fn area_index_for_tile(
        &self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
    ) -> Option<usize> {
        area_index_for_tile(
            self.area_x,
            self.area_y,
            tile_x,
            tile_y,
            area_width,
            area_height,
        )
    }

    pub(crate) fn decode_return_setup_prefix(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), ServerReturnSetupInputBlock> {
        let offset = *cursor;
        let required = 0x20usize;
        let Some(end) = offset.checked_add(required) else {
            return Err(ServerReturnSetupInputBlock {
                offset,
                required,
                available: source.len().saturating_sub(offset),
            });
        };
        let Some(bytes) = source.get(offset..end) else {
            return Err(ServerReturnSetupInputBlock {
                offset,
                required,
                available: source.len().saturating_sub(offset),
            });
        };
        let read_i32 = |at: usize| {
            LegacyReader::at(bytes, at)
                .and_then(|mut reader| reader.read_i32())
                .expect("проверенный 0x20-байтовый setup")
        };
        self.return_setup = Some(ServerReturnSetup {
            region_id: read_i32(0x00),
            left: read_i32(0x04),
            top: read_i32(0x08),
            right: read_i32(0x0C),
            bottom: read_i32(0x10),
            does_recall_when_lost: read_i32(0x14),
            move_monster_when_refeash: read_i32(0x18),
            use_return: read_i32(0x1C),
        });
        *cursor = end;
        Ok(())
    }

    /// Читает полный startup/reload snapshot `regions/{id}.rs`: восемь
    /// signed DWORD setup-а, count и C-string set запрещённых товаров.
    pub(crate) fn decord_setup_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
    ) -> Result<bool, ServerRegionSetupDecodeError> {
        self.decode_return_setup_prefix(source, cursor)
            .map_err(ServerRegionSetupDecodeError::Setup)?;

        // RVA 0x0007EAC0 очищает tree до чтения count и повторяет clear после
        // него. Второй clear не наблюдаем для owned BTreeSet.
        self.forbidden_make_goods.clear();
        let count = read_setup_i32(source, cursor, "m_ForbidMakeGoods.size")?;
        for _ in 0..count.max(0) {
            let value = read_setup_c_string(source, cursor, "m_ForbidMakeGoods[]")?;
            self.forbidden_make_goods.insert(value);
        }
        Ok(true)
    }

    /// Сохраняет `std::set<std::string>::find`: вход рассматривается как
    /// C-string, поэтому байты после первого NUL не участвуют в lookup.
    pub(crate) fn find_forbid_good(&self, name: &[u8]) -> bool {
        let end = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        self.forbidden_make_goods.contains(&name[..end])
    }

    /// Fallback-цепочка exact `GetReturnPoint`; ядро принадлежит Zone
    /// `regions/serverregion/returnsetup`.
    pub(crate) fn get_return_point(
        &self,
        player: Option<ServerReturnPlayer>,
        country_param: &mut CCountryParam,
    ) -> Result<RegionReturnPoint, ServerReturnSetupBlock> {
        get_return_point(self.return_setup, player, country_param)
    }

    /// Exact `DoesRecallWhenLost`; ядро принадлежит Zone
    /// `regions/serverregion/returnsetup`.
    pub(crate) fn does_recall_when_lost(&self) -> Result<i32, ServerReturnSetupBlock> {
        does_recall_when_lost(self.return_setup)
    }

    /// Phase default `OnWarDeclare`; ядро принадлежит Zone
    /// `regions/serverregion/war`.
    pub(crate) fn on_war_declare(&mut self, war_number: i32) {
        on_war_declare(&mut self.war_number, &mut self.city_state, war_number);
    }

    /// Phase default `OnWarStart`; ядро принадлежит Zone `regions/serverregion/war`.
    pub(crate) fn on_war_start(&mut self, _war_number: i32) {
        on_war_start(&mut self.city_state);
    }

    pub(crate) fn on_war_time_out(&mut self, _war_number: i32) {}

    /// Phase default `OnWarEnd`; ядро принадлежит Zone `regions/serverregion/war`.
    pub(crate) fn on_war_end(&mut self, _war_number: i32) {
        on_war_end(&mut self.war_number, &mut self.city_state);
    }

    /// Phase default `OnWarMass`; ядро принадлежит Zone `regions/serverregion/war`.
    pub(crate) fn on_war_mass(&mut self, _war_number: i32) {
        on_war_mass(&mut self.city_state);
    }

    pub(crate) fn on_clear_other_player(&mut self, _war_number: i32) {}

    pub(crate) fn on_refresh_region(&mut self, _war_number: i32) {}

    /// Ownership-default `SetOwnedCityOrg`; ядро принадлежит Zone
    /// `regions/serverregion/war`.
    pub(crate) fn set_owned_city_org(&mut self, faction_id: i32, union_id: i32) {
        set_owned_city_org(&mut self.param, faction_id, union_id);
    }

    pub(crate) fn owned_city_faction(&self) -> i32 {
        self.param.owned_faction_id
    }

    pub(crate) fn owned_city_union(&self) -> i32 {
        self.param.owned_union_id
    }

    /// Exact `SetWarNum`; ядро принадлежит Zone `regions/serverregion/war`.
    pub(crate) fn set_war_number(&mut self, war_number: i32) {
        set_war_number(&mut self.war_number, war_number);
    }

    pub(crate) fn get_war_number(&self) -> i32 {
        self.war_number
    }

    pub(crate) fn get_city_state(&self) -> i32 {
        self.city_state
    }

    /// Exact `SetCityState`; ядро принадлежит Zone `regions/serverregion/war`.
    pub(crate) fn set_city_state(&mut self, state: i32) {
        set_city_state(&mut self.city_state, state);
    }

    /// Exact `ReSetWarState`; ядро принадлежит Zone `regions/serverregion/war`.
    pub(crate) fn reset_war_state(&mut self, war_number: i32, state: i32) {
        reset_war_state(&mut self.war_number, &mut self.city_state, war_number, state);
    }

    pub(crate) fn start_clear_player_out_at(&mut self, delay_ms: i32, now_ms: u32) {
        self.kick_out_player = true;
        self.kick_out_player_time = delay_ms;
        self.last_time_ms = now_ms;
    }

    /// Exact `ClearPlayerAI`: long delay обновляется не чаще 5 секунд,
    /// последние 10 секунд — не чаще секунды; comparison времени остаётся
    /// unsigned DWORD, а remaining и форматируемые секунды — signed long.
    pub(crate) fn clear_player_ai_at(&mut self, now_ms: u32) -> ServerRegionClearPlayerTick {
        let remaining = self.kick_out_player_time as u32;
        let elapsed_ms = now_ms.wrapping_sub(self.last_time_ms);
        if elapsed_ms >= remaining {
            self.kick_out_player = false;
            self.kick_out_player_time = 0;
            return ServerRegionClearPlayerTick::Expired;
        }
        let threshold = if self.kick_out_player_time < 10_001 {
            1_001
        } else {
            5_001
        };
        if elapsed_ms < threshold {
            return ServerRegionClearPlayerTick::Waiting {
                remaining_ms: self.kick_out_player_time,
                elapsed_ms,
            };
        }
        self.kick_out_player_time = remaining.wrapping_sub(elapsed_ms) as i32;
        self.last_time_ms = now_ms;
        ServerRegionClearPlayerTick::Warning {
            remaining_ms: self.kick_out_player_time,
            seconds: self.kick_out_player_time / 1_000,
        }
    }
}

fn read_setup_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ServerRegionSetupDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        ServerRegionSetupDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader.read_i32().map_err(|block| {
        ServerRegionSetupDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        }
    })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_setup_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, ServerRegionSetupDecodeError> {
    let mut value = Vec::new();
    loop {
        let offset = *cursor;
        let mut reader = LegacyReader::at(source, offset).map_err(|block| {
            ServerRegionSetupDecodeError::UnexpectedEnd {
                field,
                offset: block.offset,
                needed: 1,
                available: block.available,
            }
        })?;
        let byte = reader.read_u8().map_err(|block| {
            ServerRegionSetupDecodeError::UnexpectedEnd {
                field,
                offset: block.offset,
                needed: block.needed,
                available: block.available,
            }
        })?;
        *cursor = reader.position();
        if byte == 0 {
            return Ok(value);
        }
        value.push(byte);
    }
}

fn read_server_region_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, 1)?;
    let value = reader
        .read_u8()
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_server_region_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_i32()
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_server_region_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_u32()
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_server_region_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, needed)?;
    let bytes = reader
        .read_bytes(needed)
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn read_server_region_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, ServerRegionDecodeInputBlock> {
    let mut value = Vec::new();
    loop {
        let offset = *cursor;
        let mut reader = server_region_reader(source, offset, field, 1)?;
        let byte = reader
            .read_u8()
            .map_err(|block| server_region_error(field, block))?;
        *cursor = reader.position();
        if byte == 0 {
            return Ok(value);
        }
        value.push(byte);
    }
}

fn read_server_region_i32_at(bytes: &[u8], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("проверенный region DWORD")
}

fn read_server_region_u32_at(bytes: &[u8], offset: usize) -> u32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_u32())
        .expect("проверенный region DWORD")
}

fn read_server_region_u16_at(bytes: &[u8], offset: usize) -> u16 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_u16())
        .expect("проверенное region WORD")
}

fn append_server_region_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    LegacyWriter::new(destination).write_c_string(value);
}

fn server_region_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, ServerRegionDecodeInputBlock> {
    LegacyReader::at(source, cursor).map_err(|block| ServerRegionDecodeInputBlock {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn server_region_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> ServerRegionDecodeInputBlock {
    ServerRegionDecodeInputBlock {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp

// IMPLEMENTED: `CServerRegion::GetArea` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CServerRegion::GetPlayerAmout` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::StartClearPlayerOut
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x0007BA50
//
// IMPLEMENTED выше: clear timer flags и caller-supplied wrapping tick; технические STL/SEH детали удалены.

// IMPLEMENTED: `CServerRegion::Save` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CServerRegion::New` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::AddRegionParamToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:659
// RVA: 0x0007BAA0
// ADDRESS: 0047baa0
// PROTOTYPE: bool __thiscall AddRegionParamToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::DoesRecallWhenLost
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:666
// RVA: 0x0007BAC0
//
// Реализовано выше прямым чтением setup; player pointer исходник не использует.

// ============================================================================
// FUNCTION: CServerRegion::DeleteChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1209
// RVA: 0x0007BAD0
// ADDRESS: 0047bad0
// PROTOTYPE: void __thiscall DeleteChildObject(long param_1, long param_2, CGUID * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::GetArea` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::AdjustTaxRate(CPlayer*)'::__l6::PlayerAdjustTaxRate::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2272
// RVA: 0x0007BBE0
// ADDRESS: 0047bbe0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::ObtainTaxPayment(CPlayer*)'::__l6::PlayerObtainTaxPayment::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2431
// RVA: 0x0007BBF0
// ADDRESS: 0047bbf0
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTotalTaxPayment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:400
// RVA: 0x0007BD00
// ADDRESS: 0047bd00
// PROTOTYPE: void __thiscall SetTotalTaxPayment(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::FindChildObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::FindAroundObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1611
// RVA: 0x0007BD90
// ADDRESS: 0047bd90
// PROTOTYPE: CBaseObject * __thiscall FindAroundObject(CShape * param_1, long param_2, long param_3, CGUID * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::CreateAreaArray` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::SetGoodsProtection
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2160
// RVA: 0x0007BFB0
// ADDRESS: 0047bfb0
// PROTOTYPE: void __thiscall SetGoodsProtection(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::CanPickUpGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2172
// RVA: 0x0007BFE0
// ADDRESS: 0047bfe0
// PROTOTYPE: int __thiscall CanPickUpGoods(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::AdjustTaxRate(CPlayer*)'::__l6::PlayerAdjustTaxRate::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2275
// RVA: 0x0007C010
// ADDRESS: 0047c010
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::ObtainTaxPayment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2416
// RVA: 0x0007C160
// ADDRESS: 0047c160
// PROTOTYPE: void __thiscall ObtainTaxPayment(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AdjustTaxRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2260
// RVA: 0x0007C2B0
// ADDRESS: 0047c2b0
// PROTOTYPE: void __thiscall AdjustTaxRate(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTaxRateAndUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2334
// RVA: 0x0007C370
// ADDRESS: 0047c370
// PROTOTYPE: void __thiscall SetTaxRateAndUpdate(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::GetMonsterRefeash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:990
// RVA: 0x0007C4B0
// ADDRESS: 0047c4b0
// PROTOTYPE: tagMonster * __thiscall GetMonsterRefeash(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SendWeatherInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2086
// RVA: 0x0007C610
// ADDRESS: 0047c610
// PROTOTYPE: void __thiscall SendWeatherInfo(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::DeleteChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1218
// RVA: 0x0007CBE0
// ADDRESS: 0047cbe0
// PROTOTYPE: void __thiscall DeleteChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::RemoveObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::FindMonsterByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1803
// RVA: 0x0007D090
// ADDRESS: 0047d090
// PROTOTYPE: CMonster * __thiscall FindMonsterByID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindNpcById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2613
// RVA: 0x0007D0D0
// ADDRESS: 0047d0d0
// PROTOTYPE: CNpc * __thiscall FindNpcById(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::AdjustTaxRate(CPlayer*)'::__l6::PlayerAdjustTaxRate::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2288
// RVA: 0x0007D740
// ADDRESS: 0047d740
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::CollectTodayTax
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2382
// RVA: 0x0007D9B0
// ADDRESS: 0047d9b0
// PROTOTYPE: void __thiscall CollectTodayTax(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::ObtainTaxPayment(CPlayer*)'::__l6::PlayerObtainTaxPayment::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2443
// RVA: 0x0007DAF0
// ADDRESS: 0047daf0
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: low-level property/factory/spawn/membership/owned
// lifecycle материализован выше; exact InitSkills/InitAI и message payload
// остаются у обязательного `ServerRegionMonsterContext`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:842
// RVA: 0x0007EC50
// ADDRESS: 0047ec50
// PROTOTYPE: CMonster * __thiscall AddMonster(char * param_1, long param_2, long param_3, long param_4, bool param_5, bool param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddSummonedCreature
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:918
// RVA: 0x0007EEC0
// ADDRESS: 0047eec0
// PROTOTYPE: CSummonedCreature * __thiscall AddSummonedCreature(tagMasterInfo * param_1, ulong param_2, long param_3, long param_4, long param_5, ulong param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SendMsgToAroundShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1662
// RVA: 0x0007F100
// ADDRESS: 0047f100
// PROTOTYPE: void __thiscall SendMsgToAroundShape(CShape * param_1, CMessage * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindNearestObjectInArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1704
// RVA: 0x0007F280
// ADDRESS: 0047f280
// PROTOTYPE: CBaseObject * __thiscall FindNearestObjectInArea(CShape * param_1, long param_2, long param_3, long param_4, ushort * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::GetShape` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::GetDropGoodsPos
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1881
// RVA: 0x0007F530
// ADDRESS: 0047f530
// PROTOTYPE: long __thiscall GetDropGoodsPos(long param_1, long param_2, long * param_3, long * param_4, long param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::RefeashBlock` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::ChangeWeather
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:74
// RVA: 0x00080210
// ADDRESS: 00480210
// PROTOTYPE: void __thiscall ChangeWeather(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::OnShapeChangeArea` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::AddMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:746
// RVA: 0x000808B0
// ADDRESS: 004808b0
// PROTOTYPE: int __thiscall AddMonster(char * param_1, long param_2, long param_3, long param_4, long param_5, long param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: concrete factory/spawn/membership/owned lookup и
// optional send dispatch материализованы выше как `add_npc`; здесь остаются
// первичные свидетельства exact MSVC hash traversal, legacy return и message.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1003
// RVA: 0x00080A40
// ADDRESS: 00480a40
// PROTOTYPE: int __thiscall AddNpc(tagNpc * param_1, bool param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindNearestAroundObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1679
// RVA: 0x00080DD0
// ADDRESS: 00480dd0
// PROTOTYPE: CBaseObject * __thiscall FindNearestAroundObject(CShape * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindAroundObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1738
// RVA: 0x00080E70
// ADDRESS: 00480e70
// PROTOTYPE: void __thiscall FindAroundObject(CShape * param_1, long param_2, vector<CShape*,std::allocator<CShape*>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindAroundPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1758
// RVA: 0x00080FE0
// ADDRESS: 00480fe0
// PROTOTYPE: void __thiscall FindAroundPlayer(CShape * param_1, vector<CPlayer*,std::allocator<CPlayer*>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindAroundPets
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1770
// RVA: 0x00081190
// ADDRESS: 00481190
// PROTOTYPE: void __thiscall FindAroundPets(CShape * param_1, vector<CMonster*,std::allocator<CMonster*>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::GetShape` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::GetReturnPoint
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2104
// RVA: 0x000814F0
//
// Реализовано выше: null default, local setup и country-map fallback;
// layout/call sequence имеют статус VERIFIED_DISASSEMBLY.

// IMPLEMENTED: `CServerRegion::RefeashBlock` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::FindChildObjectByName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: type `500` key-order/name lookup материализован выше.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1547
// RVA: 0x00082340
// ADDRESS: 00482340
// PROTOTYPE: CBaseObject * __thiscall FindChildObjectByName(long param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::KickOutAllPlayerToReturnPointExceptOwner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2523
// RVA: 0x000829B0
// ADDRESS: 004829b0
// PROTOTYPE: void __thiscall KickOutAllPlayerToReturnPointExceptOwner(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::DelOneGoodFromAllPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2556
// RVA: 0x00082B60
// ADDRESS: 00482b60
// PROTOTYPE: void __thiscall DelOneGoodFromAllPlayer(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `ClearPlayerAI` countdown, `0xBF807` warning и expiry-tail
// материализованы в `clear_player_ai_at` и `CGame::AI`.

// IMPLEMENTED: `CServerRegion::AddObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::DeleteAllChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1387
// RVA: 0x00083580
// ADDRESS: 00483580
// PROTOTYPE: void __thiscall DeleteAllChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddMonsterRect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: cumulative selection, fallback position, low-level
// spawn, living-count, refresh metadata/script и guard registration реализованы
// выше; RAW сохраняет ещё не закрытые refresh scheduler/AI caller details.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:672
// RVA: 0x00084040
// ADDRESS: 00484040
// PROTOTYPE: int __thiscall AddMonsterRect(tagMonster * param_1, long param_2, bool param_3, bool param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: сбор, unique insert, state reset и отложенное
// применение `CS_CHANGEAREA`, а также monster refresh due/deficit/spawn
// материализованы выше; общий monster/weather prefix подключён к `CGame::AI`,
// delete-list application и change-area/change-region/ClearPlayerAI также
// достигнуты. Row-major active-shape scan, state classification/reset и
// remove queue теперь принадлежат `CGame::AI`; concrete `CArea::AI` и virtual
// shape `AI` остаются узкими runtime owner-границами.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:87
// RVA: 0x00084520
// ADDRESS: 00484520
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::CServerRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:28
// RVA: 0x000852B0
// ADDRESS: 004852b0
// PROTOTYPE: undefined __thiscall CServerRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::OperatorCityGate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:270
// RVA: 0x00085550
// ADDRESS: 00485550
// PROTOTYPE: bool __thiscall OperatorCityGate(long param_1, eOperCityGate param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::OnWarDeclare
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085560
//
// IMPLEMENTED выше: state DUTH и war number; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::OnWarStart
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085580
//
// IMPLEMENTED выше: state Fight; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::OnWarEnd
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085590
//
// IMPLEMENTED выше: state No и нулевой war number; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::OnWarMass
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x000855B0
//
// IMPLEMENTED выше: state Mass; технические STL/SEH детали удалены.

// IMPLEMENTED: `CServerRegion::SymbolIsAttackAble` входит в canonical
// `ServerRegionOwner` virtual family и возвращает true для всех subtype-ов,
// кроме единственного city override.

// ============================================================================
// FUNCTION: CServerRegion::SetOwnedCityOrg
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x000855D0
//
// IMPLEMENTED выше: faction/union ownership; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetOwnedCityFaction
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x000855F0
//
// IMPLEMENTED выше: прямое owned-faction field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetOwnedCityUnion
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085600
//
// IMPLEMENTED выше: прямое owned-union field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetMaxTaxRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:393
// RVA: 0x00085620
// ADDRESS: 00485620
// PROTOTYPE: long __thiscall GetMaxTaxRate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTaxRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:395
// RVA: 0x00085630
// ADDRESS: 00485630
// PROTOTYPE: void __thiscall SetTaxRate(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetWarNum
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085640
//
// IMPLEMENTED выше: прямой war-number field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetWarNum
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085650
//
// IMPLEMENTED выше: прямой war-number field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetCityState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085660
//
// IMPLEMENTED выше: прямой city-state field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::SetCityState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085670
//
// IMPLEMENTED выше: прямой city-state field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::ReSetWarState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085680
//
// IMPLEMENTED выше: ordered war-number/state assignment; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::~CServerRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:53
// RVA: 0x000856B0
// ADDRESS: 004856b0
// PROTOTYPE: void __thiscall ~CServerRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTodayTaxPayment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:403
// RVA: 0x00089590
// ADDRESS: 00489590
// PROTOTYPE: void __thiscall SetTodayTaxPayment(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
