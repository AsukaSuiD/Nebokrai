//! Делегат состояния и расписаний `CPlayerAI` в Zone.
//!
//! Назначения клиента (destination FIFO), независимые очереди навыков
//! игрока и боевого духа, выбранные/текущие команды и хвост `Run`
//! (авто-прирост опыта/бодрости, регенерация энергии) перенесены буквально
//! в `nebokrai_zone::ai::playerai` — машинная база VERIFIED по точной паре
//! `4F5C98E0…` + GameServer.pdb (RSDS match), RVA-якоря `OnSchedule`,
//! перегрузок `Attack`, WarSoul-семьи, виртуального `MoveTo` и свежая
//! точечная сверка волны Z-AI-player (константа авто-прироста
//! `0.00011574074f` у `0x5094D4`, пустой RET слота +0x48, общие f32
//! `1e6`/`1.414e6`/`0.001` у MoveTo игрока), а также честные UNKNOWN —
//! в её шапке (`zone/src/ai/playerai.rs`). Базовые FIFO, object-цель,
//! back-stage список, Slip и задержка шага — в `nebokrai_zone::ai::baseai`.
//! Здесь:
//!
//! - реализация узкого фасада `AutoIncPlayer` Zone над прежним `CPlayer`:
//!   только аксессорные операции хвоста `Run`, имена членов сохраняют
//!   исходные операции, реестр игроков, публикация AI на время callbacks
//!   и издатели/подписчики исполнения остаются hub-владением (`game.rs`,
//!   `player.rs`);
//! - переэкспорт типов с прежними именами — потребители (`player.rs`,
//!   skills-executors, `states/skill.rs`, `game.rs`) не меняются;
//! - часы хвоста `Run` приходят как и прежде — аргументом делегата
//!   старого main loop (`&mut dyn FnMut() -> u32`).

use crate::gameserver::appserver::player::CPlayer;

pub(crate) use nebokrai_zone::ai::playerai::{CPlayerAI, PlayerAutoProgress};
use nebokrai_zone::ai::playerai::AutoIncPlayer;

/// `CPlayer` как игрок хвоста авто-прироста/регенерации `CPlayerAI::Run`.
impl AutoIncPlayer for CPlayer {
    fn player_id(&self) -> i32 {
        self.player_id()
    }

    fn is_dead(&self) -> bool {
        self.is_dead()
    }

    fn faction_id(&self) -> i32 {
        self.faction_id()
    }

    fn faction_level(&self) -> u16 {
        self.faction_level()
    }

    fn level(&self) -> u8 {
        self.level()
    }

    fn experience(&self) -> u32 {
        self.experience()
    }

    fn set_experience(&mut self, value: u32) {
        self.set_experience(value);
    }

    fn vigour(&self) -> u32 {
        self.vigour()
    }

    fn set_vigour(&mut self, value: u32) {
        self.set_vigour(value);
    }

    fn energy(&self) -> u32 {
        self.energy()
    }

    fn maximum_energy(&self) -> u32 {
        self.maximum_energy()
    }

    fn set_energy(&mut self, value: u32) {
        self.set_energy(value);
    }
}
