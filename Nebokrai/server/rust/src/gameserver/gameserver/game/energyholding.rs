//! Межвладельческая координация зарядов `EnergyHoldingState`.
//!
//! Состояние и предел накопления принадлежат владельцу навыка. Здесь остаются
//! доступ к владельцу `CPlayer` и рассылка подтверждённой пары `End → Begin`
//! вокруг него.

use super::*;
use crate::gameserver::appserver::skills::energyholdingstate::{
    EnergyHoldingState, send_energy_holding_state_visual,
};

impl CGame {
    pub(crate) fn add_player_energy_holding(
        &mut self,
        player_id: i32,
        skill_level: u32,
        parameter_percent: u32,
    ) -> bool {
        let installed = self.find_player_mut(player_id).and_then(|player| {
            let region_id = player.server_region_id()?;
            let identity = player.shape().identity();
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            if player.energy_holding_state().is_none() {
                player.begin_energy_holding_state(EnergyHoldingState::new(skill_level, parameter_percent));
            }
            let state = player.energy_holding_state_mut().expect("состояние создано выше или существовало");
            Some(state.add_energy().then_some((region_id, identity, x, y, *state)))
        });
        match installed {
            None => false,
            Some(None) => true,
            Some(Some((region_id, identity, x, y, state))) => {
                send_energy_holding_state_visual(self, region_id, identity, x, y, state, false);
                send_energy_holding_state_visual(self, region_id, identity, x, y, state, true);
                true
            }
        }
    }
}
