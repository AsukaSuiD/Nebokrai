//! Путь к живой форме области ослабления в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/weakphalanx.cpp/.h.
//! Композит `CWeakPhalanx` (CShape + область) перенесён буквально в
//! `nebokrai_zone::skills::weak` порцией T5 «zonalcast-хаб» (основание и
//! статусы — там). Здесь — реэкспорт и снимок фигур клетки прежней вспышки;
//! последний остаётся у Game-арены (он не входил в переносимые тела навыка).

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::skills::{CWeakPhalanx, WeakPhalanxTick};

pub(crate) fn weak_cell_targets(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeIdentity> {
    super::flash::cell_views(game, region_id, x, y).into_iter().map(|shape| shape.identity).collect()
}
