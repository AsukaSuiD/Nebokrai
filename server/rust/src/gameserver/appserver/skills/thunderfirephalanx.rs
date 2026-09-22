//! Движущийся громовой огонь `CThunderFirePhalanx` (`0x322`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunderfirephalanx.cpp`. Форма проходит исходный путь,
//! атакует маской 1×1 только в клетках с блоком `3` и прекращается после
//! первой клетки с применённой атакой. Формула сохраняет два вызова MSVCRT RNG.
//! Soul-усиление загружает битовый `u32`-шаблон свойства как исходный знаковый
//! `J`; оно и критический множитель остаются в x87 до усечения урона к нулю.

use super::fightdefense::truncate_original;
use super::itemskill2::ITEM_SKILL_2_ID;
use nebokrai_shared::protocol::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThunderFirePhalanxTick { Pending, Active { force_move: Option<(i32,i32,u32)>, scan: Option<(i32,i32,u32)> }, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CThunderFirePhalanx {
    shape:CShape, master:MasterInfo, started_at_ms:u32, lifetime_ms:u32, skill_level:i32,
    minimum_attack:i32, maximum_attack:i32, element_modifier:i32, path:Vec<(i32,i32)>,
    speed_ms:u32, soul_count:i32, soul_variable:u32, current_position:usize, force_moved:bool,
}


impl CThunderFirePhalanx {
    #[allow(clippy::too_many_arguments, reason="поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id:i32,master:MasterInfo,started_at_ms:u32,lifetime_ms:u32,skill_level:i32,minimum_attack:i32,maximum_attack:i32,element_modifier:i32,path:Vec<(i32,i32)>,speed_ms:u32,soul_count:i32,soul_variable:u32)->Self{
        let mut shape=CShape::with_constructor_defaults();shape.set_identity(ShapeIdentity{object_type:SUMMON_SHAPE_TYPE,id,ex_id:CGuid::GUID_INVALID});Self{shape,master,started_at_ms,lifetime_ms,skill_level,minimum_attack,maximum_attack,element_modifier,path,speed_ms,soul_count,soul_variable,current_position:0,force_moved:false}
    }
    pub(crate) const fn shape(&self)->&CShape{&self.shape} pub(crate) const fn shape_mut(&mut self)->&mut CShape{&mut self.shape} pub(crate) const fn master(&self)->MasterInfo{self.master}
    pub(crate) fn finish(&mut self){self.shape.set_change_state(SHAPE_CHANGE_DELETE);}
    pub(crate) fn tick(&mut self,now:u32)->ThunderFirePhalanxTick{
        if self.started_at_ms.wrapping_add(self.lifetime_ms)<now||self.path.is_empty()||self.current_position>=self.path.len(){self.finish();return ThunderFirePhalanxTick::Expired;}
        let force_move=if self.force_moved{None}else{self.force_moved=true;let &(x,y)=self.path.last().expect("путь непуст");Some((x,y,(self.path.len() as u32).wrapping_mul(self.speed_ms)))};
        let scan=(self.started_at_ms.wrapping_add((self.current_position as u32).wrapping_mul(self.speed_ms))<=now).then(||{let (x,y)=self.path[self.current_position];self.current_position=self.current_position.wrapping_add(1);(x,y,now)});
        if force_move.is_some()||scan.is_some(){ThunderFirePhalanxTick::Active{force_move,scan}}else{ThunderFirePhalanxTick::Pending}
    }
    pub(crate) fn encode_client_snapshot(&self,mut now:impl FnMut()->u32)->Option<Vec<u8>>{
        let first=now();let remained=if self.started_at_ms.wrapping_add(self.lifetime_ms)<=first{0}else{self.lifetime_ms.wrapping_sub(now()).wrapping_add(self.started_at_ms)};let mut payload=Vec::new();{let mut w=LegacyWriter::new(&mut payload);w.write_i32(ITEM_SKILL_2_ID as i32);w.write_i32(self.skill_level);w.write_i32(self.master.master_type);w.write_i32(self.master.master_id);w.write_u32(remained);}self.shape.add_to_byte_array(&mut payload,true).then_some(payload)
    }
}

pub(crate) fn calculate_owned_thunder_fire_attack(game:&mut CGame,p:&CThunderFirePhalanx,target_level:u8)->Option<(AttackInformation,PlayerCombatProperties,u8,u8)>{
    let player=game.find_player(p.master.master_id)?;let mut combat=player.combat_properties();let occupation=player.occupation();let attacker_level=player.level();let(divisor,minimum)=game.globe_setup().weapon_damage_factors();let factor=player.weapon_modifier(game.goods_factory(),i32::from(target_level),divisor,minimum);
    let width_delta=p.maximum_attack.wrapping_sub(p.minimum_attack);let width=if width_delta<0{width_delta.wrapping_neg()}else{width_delta}.wrapping_add(1);let mut damage=p.element_modifier.wrapping_mul(combat.element_modify).wrapping_div(100).wrapping_add(combat.add_element_attack as i32).wrapping_add(game.skill_random_below(width)).wrapping_add(p.minimum_attack);if p.soul_count!=0&&p.soul_variable!=0{damage=truncate_original((f64::from(p.soul_variable as i32)*f64::from(p.soul_count)*f64::from(0.01_f32)+f64::from(1.0_f32))*f64::from(damage));}damage=damage.max(0);
    let mut attack=AttackInformation{skill_id:ITEM_SKILL_2_ID,skill_level:p.skill_level as u8,attacker_type:p.master.master_type,attacker_id:p.master.master_id,attacker_team_id:p.master.master_team_id,attacker_faction_id:p.master.master_guild_id,attacker_union_id:p.master.master_union_id,hit_modifier:100,damage_factor:factor,damage_modifier:0,critical:false,blast_attack:false,full_miss:0,damages:vec![AttackPower{kind:AttackPowerType::Element,hp_damage:damage,mp_damage:0}]};
    if game.skill_random_below(100)<i32::from(combat.cch){attack.critical=true;let rate=combat.critical_rate();for power in &mut attack.damages{power.hp_damage=truncate_original(f64::from(power.hp_damage)*f64::from(rate));}}
    let[ba,bd,eba,ebd,fm]=game.globe_setup().base_combat_scales();if combat.blast_attack_scale()<1.0{combat.blast_attack_scale_bits=ba.max(1.0).to_bits();}if combat.blast_defense_scale()<0.01{combat.blast_defense_scale_bits=bd.max(0.01).to_bits();}if combat.element_blast_attack_scale()<1.0{combat.element_blast_attack_scale_bits=eba.max(1.0).to_bits();}if combat.element_blast_defense_scale()<0.01{combat.element_blast_defense_scale_bits=ebd.max(0.01).to_bits();}if combat.full_miss_scale()<0.01{combat.full_miss_scale_bits=fm.max(0.01).to_bits();}Some((attack,combat,occupation,attacker_level))
}
