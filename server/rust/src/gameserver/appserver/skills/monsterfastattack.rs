//! Двухударная быстрая атака `CMonsterFastAttack` (ID `0x2d1`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/skills/monsterfastattack.cpp`. Монстр проходит через
//! `monsterbaseattack`; player-dispatch, отмена и завершение используют общий
//! двухударный owner `lordfastattack` с отдельными формулой, MP и cooldown.
//! Состояние ниже хранит визуальную фазу и первый удар; свойства `15001/15002`
//! задают две последовательные границы после начальной задержки.
//!
//! Player `CheckCastCondition` (VA `0x005131d0`) проверяет reuse, расстояние,
//! путь и ненулевую стоимость MP; нулевой cost возвращает отказ (VA
//! `0x005133cf`). AI (VA `0x00513810`) повторно проверяет и списывает MP,
//! публикует `OnChangeStates`, поворачивает источник, отправляет начало и огонь,
//! затем наносит два удара по абсолютным wrapping-срокам. Для каждого удара
//! физический разброс — `max(max-min, 0)+1`; только player-источник выполняет
//! критический RNG и использует свой множитель. У монстра этот cast отсутствует.
//! Исходное разыменование null player в MP-проверке не воспроизводится.
//!
//! `0xbfe01` сохраняет start/fire и адресные failure-коды; reuse/MP игрока
//! дополняются строками `GS1143/GS1144`. Координатная цель повторно разрешается
//! общим `CState::GetSufferer`; пустая клетка/пустой Begin отклоняются,
//! исчезнувшая цель в AI завершает навык без failure 10, мёртвая/самоцель — с ним.
//! End возвращает движение; только успех изнашивает оружие и ставит cooldown.
//! Общий kernel, типизированные owners и существующая доставка заменяют
//! указатели/STL, не игровые сроки и порядок побочных эффектов.
//! Player-dispatch поддерживает NPC, игроков, монстров, постройки и ворота;
//! общий IsDied отвергает нулевой HP NPC. Урон постройкам проходит их owner,
//! а путь использует ближайшую точку footprint. Attack (VA `0x00513700`)
//! не проверяет IsAttackAble до RNG и не увеличивает RP атакующему.

use crate::nets::netserver::message::CMessage;
pub(crate) use nebokrai_zone::skills::execution::{MonsterFastAttackProgress};

pub(crate) const MONSTER_FAST_ATTACK_SKILL_ID: u32 = 0x2d1;
pub(crate) const SKILL_USAGE_FIRST_TIME: u32 = 15_001;
pub(crate) const SKILL_USAGE_SECOND_TIME: u32 = 15_002;

pub(crate) fn fast_attack_fire_message(
    skill_id: u32,
    skill_level: u16,
    monster_id: i32,
    target_x: i32,
    target_y: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(600);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    message
}
