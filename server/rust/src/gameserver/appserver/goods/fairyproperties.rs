//! Свойства обычной феи старого GameServer перенесены в Zone items.
//! Здесь их реэкспорт для старого пакета, привязка шва упорядоченного журнала
//! эффектов роста к прежнему `GameEffectJournal` и alias report-а с прежним
//! журналом (потребители без правок).

pub(crate) use nebokrai_zone::items::fairyproperties::*;

use crate::gameserver::appserver::gameeffectjournal::GameEffectJournal;

/// Реализация zone-шва прежним журналом эффектов; тело — прежний ordered push
/// с конвертом `FairyGrowLog → GameEffect`.
impl FairyGrowEffectSink for GameEffectJournal {
    fn push(&mut self, log: FairyGrowLog) {
        self.push(log);
    }
}

/// Подстановка прежнего журнала в параметризованный zone-report.
pub(crate) type FairyExpReport =
    nebokrai_zone::items::fairyproperties::FairyExpReport<GameEffectJournal>;
