//! Призыв областей CGodThunder и CGodThunder2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godthunder.cpp и
//! godthunder2.cpp. Общий Begin/Check/AI/visual/End находится в zonalcast.
//! Summon: Master(country0)/Player EM→свежая таблица→usage20015/FISTP;
//! ключи и дальнейший порядок живых чтений принадлежат zone/skills/godthunder.rs.
//! SetTile→Initialize/RNG предшествуют повторному
//! чтению actual region captured U; Add→encode/BF502 не зависят от успеха Add.
//! Конструктор явно отклоняет параметры с native делением на ноль или выходом
//! из массива; валидный порядок запросов и RNG не меняется. Маски, окна и
//! дополнительный обход WarSoul второго варианта принадлежат владельцу формы.

use super::godthunderphalanx::CGodThunderPhalanx;
use super::weaponattack::{SourceProperty, source_property};
use super::zonalcast::prepare_element_summon;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::{ElementSummonLiveField, GodThunderSummonParameters};

pub(crate) use nebokrai_zone::skills::GOD_THUNDER_SKILL_ID;

pub(super) fn summon_god_thunder<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, skill_id: u32,
    source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some((master, properties, scaled_element)) = prepare_element_summon(game, instance, source) else { return; };
    let Some(parameters) = GodThunderSummonParameters::read(
        skill_id,
        |property| properties.query_property(property),
        |field| match field {
            ElementSummonLiveField::CriticalChance => source_property(game, source, SourceProperty::CriticalChance).map(|value| value as i32),
            ElementSummonLiveField::AddElementAttack => source_property(game, source, SourceProperty::Element).map(|value| value as i32),
        },
        || game.registered_skill(instance).map(|skill| skill.level()),
        scaled_element,
    ) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = match CGodThunderPhalanx::new(id, master, started, parameters) {
        Ok(phalanx) => phalanx,
        Err(error) => {
            tracing::error!(skill_id = parameters.skill_id, ?error, "некорректные параметры божественного грома");
            return;
        }
    };
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(destination.0) + 0.5) as f32, (f64::from(destination.1) + 0.5) as f32,
    );
    phalanx.initialize(&mut |maximum| game.skill_random_below(maximum));
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.add_god_thunder_phalanx(region, phalanx, started, runtime);
}
