//! Смазка оружия ядом `CDaubPoison` (`0xDF`): ID навыка, правило срока нового
//! состояния и тело применения после visual(1) с семейной заменой первого
//! непустого 0xDF-слота. Скелет Begin/Check/AI принадлежит hub `selfstatecast`
//! старого пакета (общий для пяти усилений), обвязка состояния —
//! `skills/daubpoisonstate.rs` рядом.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/daubpoison.cpp`; тело apply перенесено буквально.
//!
//! Машинная сверка по этой паре (запись `.local/recon-de/notes/
//! D4-daubpoison.md`, тела `.local/recon-de/disasm/CDaubPoison.txt`)
//! подтверждает всё:
//!
//! - vtable `0x259B74` (VA `0x00659B74`): Begin-скелет трёх форм
//!   `0x565FE0`/`0x565F10`/`0x5660D0` (форвард `CAttackSkill::Begin` → new
//!   effect `0xC` → `[+0x34]` → BeginVisualEffect(1) → Check `vcall+0x64`;
//!   провал — End(0), успех — `[+0x4C]=1`, `[+0x50]=0`), AI `0x566690`,
//!   Check(U) `0x5664B0`, End `0x546090` (ICF `CAgility::End`),
//!   DoesTargetEffective `0x5AFCE0` (mov eax,1, ICF). Check/AI самого
//!   навыка — hub `selfstatecast` (сверено там MATCH): reuse 10005 →
//!   visual(13) + GS0278; player Query(2)==0 → тихий ret 0; signed MP →
//!   visual(7) + GS0288; SetMoveable(0); не-player — без MP/Move0. AI:
//!   IsDied(U) → visual(2) → End(1) (hub `RejectedAfterUse`); MP → SetMP →
//!   `vcall+0x164` (OnChangeStates) → CAN(10006, dword); visual(0) →
//!   delay 10001 unsigned → visual(1) → apply ниже → End(1).
//! - apply после visual(1): скан вектора `[U+0x11C]` — первый НЕпустой слот
//!   id `0xDF` (без фильтра RTTI или ended) → End `vcall+0x1C` →
//!   deleting-dtor `vcall+0x10(1)` свежего остатка позиции → slot = 0;
//!   ctor `0x5F17B0` с keep **только из Query(10002)** (`esp`-трекер
//!   подтвердил ту же таблицу props); Begin(U,U) `vcall+0x08` → true:
//!   push_back append, false: deleting-dtor нового; затем End(1). Проверки
//!   результата установки нет — отказ Begin не отменяет завершение навыка.
//!
//! **Сознательное отклонение hub (подтверждено, не чинится):**
//! hub `selfstatecast` отклоняет источник не типа Player в Begin-стадии AI
//! (`Rejected`), тогда как нативная фаза 0 читает MP `[U+0x284]` без
//! RTTI-гейта (небезопасный доступ для монстра). Разведка подтвердила
//! отклонение осознанным (шапка hub, «deliberate»); zone-файл здесь
//! воспроизводит только машинно достижимый путь, не изменяя поведение hub.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::StateCastGame`
//! (арена `find_state_position`/`end_and_destroy_state_at`) и соседний
//! `daubpoisonstate::begin_primary_daub_poison_state`; драйвер
//! `selfstatecast.rs` старого пакета не меняется. Потребление
//! статическое (generic), dyn-совместимость и `Send`-контракт не вводятся
//! (ADR-0013).

use crate::content::CSkillBaseProperties;
use crate::effects::DAUB_POISON_STATE_ID;
use crate::regions::ShapeIdentity;

use super::daubpoisonstate::begin_primary_daub_poison_state;
use super::statecast::{StateCastGame, StateCastMoveShape};

pub const DAUB_POISON_SKILL_ID: u32 = DAUB_POISON_STATE_ID;
const STATE_PERSIST_TIME: u32 = 10_002;

/// Единственный запрос срока нового состояния выполняется после завершения
/// прежнего слота; сам запрос остаётся у живого Game.
pub fn daub_poison_keep_time_ms(mut query_property: impl FnMut(u32) -> u32) -> u32 {
    query_property(STATE_PERSIST_TIME)
}

/// Применение из AI `0x566690` после visual(1): первый непустой слот `0xDF`
/// завершается с destructor-ом свежего остатка той же позиции, затем ctor
/// keep из `Query(10002)` и primary Begin(U,U) с append у того же держателя;
/// результат установки завершения навыка End(1) не отменяет.
pub fn apply_daub_poison<Game: StateCastGame>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    now: &mut dyn FnMut() -> u32,
) {
    if let Some((position, _)) = game
        .resolve_state_move_shape(source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == DAUB_POISON_STATE_ID))
    {
        let _ = game.end_and_destroy_state_at(source.0, source.1, position);
    }
    let keep = daub_poison_keep_time_ms(|key| properties.query_property(key));
    let _ = begin_primary_daub_poison_state(game, source, keep, now);
}
