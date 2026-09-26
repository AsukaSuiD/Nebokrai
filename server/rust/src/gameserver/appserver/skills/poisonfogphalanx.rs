//! Путь к живой форме области ядовитого тумана в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/poisonfogphalanx.cpp/.h.
//! Композит `CPoisonFogPhalanx` (CShape + область) перенесён буквально в
//! `nebokrai_zone::skills::poisonfog` порцией T5 «zonalcast-хаб» (основание
//! и статусы — там). Здесь — реэкспорт и снимок целей клетки прежней
//! вспышки; последний остаётся у Game-арены (он не входил в переносимые
//! тела навыка).

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::skills::{CPoisonFogPhalanx, PoisonFogPhalanxTick};

pub(crate) fn poison_fog_targets(game: &CGame, region_id: i32, phalanx: &CPoisonFogPhalanx) -> Vec<ShapeIdentity> {
    if !phalanx.cell_active() { return Vec::new(); }
    let (Ok(x), Ok(y)) = (phalanx.shape().get_tile_x(), phalanx.shape().get_tile_y()) else { return Vec::new() };
    super::flash::cell_views(game, region_id, x, y).into_iter().map(|shape| shape.identity).collect()
}
