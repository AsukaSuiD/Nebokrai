//! Process-owned состояние `CJJcSystem` GameServer в Zone `activities/` (локальное исполнение арен).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/jjcsystem.cpp`. Достигнутые World callbacks сохраняют две
//! ordered map: region → пара участников и player → сведения соперника.
//! Match/start/timeout/week/season и `CPlayer::OnLost -> QuitJJc` вызываются
//! реальным `CGame::ProcessMessage`; player, region, script и network effects
//! выполняет `CGame`. `CScript::JJcFunction 10000..10009` теперь достигает
//! также `ApplyJJc`, `EndPK` и `BackRegion`: этот owner выбирает сведения матча
//! и точку возврата, а `CGame` сохраняет порядок сообщений, воскрешения и
//! смены региона. Неопределённые значения EAX после исходных void-вызовов не
//! считаются игровым контрактом и нормализованы диспетчером в ноль.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JjcInfo {
    pub jjc_level: u32,
    pub old_region_id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub opponent_id: i32,
    pub jjc_region_id: i32,
    pub start_time: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CJJcSystem {
    pk_list: BTreeMap<i32, (i32, i32)>,
    player_info: BTreeMap<i32, JjcInfo>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcReturnTarget {
    pub region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

impl CJJcSystem {
    pub fn on_world_closed(&mut self) {
        self.pk_list.clear();
        self.player_info.clear();
    }

    pub fn on_matched(&mut self, first: JjcInfo, second: JjcInfo) {
        self.pk_list
            .insert(first.jjc_region_id, (second.opponent_id, first.opponent_id));
        self.player_info.insert(second.opponent_id, first);
        self.player_info.insert(first.opponent_id, second);
    }

    pub fn opponent_info(&self, selector: u32, region_id: i32, player_id: i32) -> Option<i32> {
        let info = self.player_info.get(&player_id)?;
        let opponent = self.player_info.get(&info.opponent_id)?;
        let _region_members = self.pk_list.get(&region_id);
        match selector {
            1 => Some(info.opponent_id),
            2 => Some(opponent.jjc_level as i32),
            _ => None,
        }
    }

    pub fn return_target(
        &self,
        player_id: i32,
        current_region_id: i32,
        country: u8,
        region_min: i32,
        region_max: i32,
    ) -> Option<JjcReturnTarget> {
        let info = self
            .player_info
            .get(&player_id)
            .copied()
            .unwrap_or_default();
        if info.old_region_id != 0 {
            return Some(JjcReturnTarget {
                region_id: info.old_region_id,
                tile_x: info.pos_x,
                tile_y: info.pos_y,
            });
        }
        if !(region_min..=region_max).contains(&current_region_id) {
            return None;
        }
        match country {
            1 => Some(JjcReturnTarget {
                region_id: 11_000,
                tile_x: 0x113,
                tile_y: 0x11a,
            }),
            2 => Some(JjcReturnTarget {
                region_id: 12_000,
                tile_x: 0xdc,
                tile_y: 0x106,
            }),
            3 => Some(JjcReturnTarget {
                region_id: 13_000,
                tile_x: 0xdc,
                tile_y: 0x106,
            }),
            4 => Some(JjcReturnTarget {
                region_id: 14_000,
                tile_x: 0x113,
                tile_y: 0x11a,
            }),
            _ => None,
        }
    }

    pub fn finish_pk(&mut self, region_id: i32, player_id: i32) {
        self.player_info.remove(&player_id);
        self.pk_list.remove(&region_id);
    }
}
