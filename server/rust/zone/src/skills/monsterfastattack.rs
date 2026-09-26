//! Двухударная быстрая атака `CMonsterFastAttack` (ID `0x2d1`): константы
//! сроков, wire-кадр выпуска и машинная база исполнения. Монстр проходит
//! через hub `execute_owned_monster_base_attack` прежнего пакета (его перенос
//! — своя порция); player-dispatch, отмена и завершение используют общий
//! двухударный owner `lordfastattack` с отдельными формулой, MP и cooldown
//! (split player-ветви fast по `lordfastattack` не входит в волну A2 — связка
//! `SKILL_USAGE_FIRST_TIME`/`SKILL_USAGE_SECOND_TIME` сохранена здесь для
//! будущего lord-владельца).
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/monsterfastattack.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/monsterfastattack.rs` (кластер A2 полосы
//! Monster, 26 сентября 2026).
//!
//! Машинная база по этой паре (VERIFIED, тела `.local/recon-a2/out/`);
//! сопоставление с исполнением hub прежнего пакета — MATCH по всем пунктам:
//!
//! - ctor (RVA `0x112910`): `[+4] = 0x2d1`, `[+0x4C] = [+0x50] = [+0x58] =
//!   [+0x54] = 0`; фабричный QuerySkill 0x2D1 → этот класс.
//! - Begin-скелет три формы (`0x1129A0`/`0x112A70`/`0x112BB0`): форвард
//!   `CAttackSkill::Begin` → new effect → effect `VT[0](1)` → слот `+0x60`
//!   CheckCastCondition; провал — `End(0)` и ret 0 БЕЗ терминального кадра,
//!   успех — `[+0x4C] = 1`, `[+0x50] = 0`.
//! - CheckCastCondition (RVA `0x1131D0`): U/S null → ret 0 без кадра;
//!   props null → ret 0; reuse (`QueryProperty(10005) + [+0x40]` vs
//!   timeGetTime, unsigned) → кадр `{0xBFE01, 0, 13}` + GS1143 только у
//!   player; `RealDistance(U, S) > QueryProperty(5003)` → `{0, 0xb}`; ячейка
//!   `GetTargetPath` с `block == 2` → `{0, 0xf}`; **нулевая стоимость MP
//!   (`QueryProperty(2) == 0`) — молчаливый ret 0** (0x5133CF); MP-player <
//!   cost → `{0, 7}` + GS1144; успех — SetMoveable(0) (только player веток).
//!   Исходное разыменование null player в MP-проверке не воспроизводится.
//! - AI (RVA `0x113810`): S null или U null → `End(0)` без кадра; мёртвая S
//!   или U==S → кадр `{0, 10}` + `End(0)` БЕЗ reuse. Фазы `[+0x50]/[+0x54]/
//!   [+0x58]`: первая — MP повторно списывается только у player (`SetMP`,
//!   `OnChangeStates` vt+0x164; нехватка → `{0, 7}` + GS1144), SetDir к S и
//!   старт-кадр (mode 0), `[+0x3C] = QueryProperty(10006)`; вторая — по
//!   `timeGetTime >= start + delay(10001)` кадр fire (mode 1), `[+0x3C] = 0`;
//!   удары по двум **кумулятивным** срокам от начала: первый
//!   `delay + 15001`, второй `delay + 15001 + 15002` (unsigned), затем
//!   `End(1)` со штампом reuse.
//! - Attack (RVA `0x113700`): пропуск null U/S до расчёта и OnBeenAttacked;
//!   НЕТ проверки U==S/500/IsAttackAble, НЕТ IncreaseRp; записи 1/3/4 идут в
//!   исходном порядке в `OnBeenAttacked(&info, 0)` (vt+0x15C).
//! - Calculate (RVA `0x113490`): физический разброс `max(max-min, 0) + 1`,
//!   clamp `max(0)` по min+roll; критический roll `random(100) < GetCCH`
//!   только после успешного cast в CPlayer, множитель — float по
//!   `[player+0x414]` с x87-усечением. У монстра этого RNG-вызова нет.
//! - End (RVA `0x112B50`): нули фаз, `[+0x3C] = 1`, SetMoveable(U, 1),
//!   CAttackSkill::End(H): только успех изнашивает оружие и ставит
//!   cooldown.
//!
//! Объявленные швы переноса: состояние двух ударов (`MonsterFastAttackProgress`)
//! живёт в `skills/execution`; wire-кадр ниже — зональный конструктор.
//! Исполнение монстра и оркестрация hub остаются у прежнего владельца.

use crate::app::game_message::CMessage;

pub const MONSTER_FAST_ATTACK_SKILL_ID: u32 = 0x2d1;

/// Первый кумулятивный срок удара после начальной задержки (`QueryProperty`
/// 15001): `delay + first_time <= now` от начала cast.
pub const SKILL_USAGE_FIRST_TIME: u32 = 15_001;

/// Второй кумулятивный срок: `delay + first_time + second_time <= now`.
pub const SKILL_USAGE_SECOND_TIME: u32 = 15_002;

/// Кадр выпуска `0x000BFE01` монстровой быстрой атаки: action 2, навык,
/// уровень, сторона, два нулевых long и координаты цели тика (как у
/// message-owner семьи; прежний конструктор владельца).
pub fn fast_attack_fire_message(
    skill_id: u32,
    skill_level: u16,
    monster_id: i32,
    target_x: i32,
    target_y: i32,
) -> CMessage {
    const MONSTER_TYPE: i32 = 600;
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    message
}
