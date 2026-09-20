//! Межвладельческая координация громового огня.
//!
//! Формула, часы и путь принадлежат `thunderfirephalanx.rs`; здесь остаются
//! только региональная регистрация, пространственное перемещение и доставка.

use super::*;
use crate::gameserver::appserver::skills::thunderfirephalanx::CThunderFirePhalanx;

impl CGame {
    pub(crate) fn add_thunder_fire_phalanx<Runtime:GameMainLoopRuntime>(&mut self,region_id:i32,phalanx:CThunderFirePhalanx,x:i32,y:i32,now:u32,runtime:&mut Runtime)->Option<Result<i32,RegionMembershipBlock>>{let mut owner=self.take_region_owner(region_id)?;let result=owner.base_mut().add_thunder_fire_phalanx(phalanx,x,y,self.area_width,self.area_height,now,runtime);self.restore_region_owner(owner);Some(result)}
    pub(crate) fn send_thunder_fire_phalanx_entry<Runtime:GameMainLoopRuntime>(&mut self,region_id:i32,id:i32,runtime:&mut Runtime)->Option<()>{let SummonedSkillShape::ThunderFire(p)=self.find_region(region_id)?.base().find_skill_phalanx(id)? else{return None};let identity=p.shape().identity();let x=p.shape().get_tile_x().ok()?;let y=p.shape().get_tile_y().ok()?;let payload=p.encode_client_snapshot(||runtime.now_milliseconds())?;let mut message=CMessage::new(0x000b_f502);message.add_long(identity.object_type);message.add_long(identity.id);message.base_mut().add_guid(identity.ex_id);message.add_long(i32::try_from(payload.len()).ok()?);message.base_mut().add(&payload);message.base_mut().add_char(0);let _=self.send_shape_position_around(region_id,x,y,&message);Some(())}
}
