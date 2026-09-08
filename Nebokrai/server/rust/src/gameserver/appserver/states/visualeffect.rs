//! Достигнутый базовый ресурс `CVisualEffect` GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/visualeffect.h/.cpp`.
//! Constructor (0x005DC200) задаёт ended=false и loop=0. BeginVisualEffect
//! (0x005DC1B0) снова снимает ended и сохраняет signed int без нормализации
//! в bool: любой ненулевой loop, включая отрицательный, подавляет базовый End.
//! EndVisualEffect (0x005DC1D0) выставляет ended, но не меняет loop.
//! Обе перегрузки UpdateVisualEffect (0x005DC1E0/0x005DC1F0) игнорируют
//! CState* и дополнительный ULONG, поэтому представлены одной базовой
//! операцией без фиктивных аргументов. При loop=0 она вызывает End даже
//! у уже завершённого эффекта; отдельного IsEnded-gate и часов здесь нет.
//! В native это virtual вызов слота +4. Этот конкретный Rust-тип представляет
//! только базовый класс: производные Update с собственными действиями,
//! например CExStateNewVisualEffect (0x005D9CF0), не подменяются базовым телом.
//!
//! CState constructor (0x005DBCA0) оставляет visual-указатель null, destructor
//! (0x005DBD40) вызывает CVisualEffect destructor и operator delete. Здесь
//! ресурс владеет своим состоянием по значению, без сырых указателей и
//! неявного копирования; Drop воспроизводит базовый destructor 0x005DC220:
//! ended=true, затем loop=0. Освобождение памяти остаётся обычному Rust owner-у.
//! Для владеющего поля состояния достаточно Option<CVisualEffect>, не флага
//! или отправленного пакета. Этот базовый ресурс достаточен для его owned
//! удаления, но не реализует остальные concrete ресурсы и virtual Update
//! производных эффектов. Их подключение и порядок полного CSkill::End
//! остаются обязанностью владельца состояния; сетевой пакет не заменяет Drop.

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CVisualEffect {
    ended: bool,
    loop_value: i32,
}

impl CVisualEffect {
    pub(crate) const fn new() -> Self {
        Self {
            ended: false,
            loop_value: 0,
        }
    }

    pub(crate) const fn is_ended(&self) -> bool {
        self.ended
    }

    pub(crate) const fn loop_value(&self) -> i32 {
        self.loop_value
    }

    pub(crate) const fn begin_visual_effect(&mut self, loop_value: i32) {
        self.ended = false;
        self.loop_value = loop_value;
    }

    pub(crate) const fn end_visual_effect(&mut self) {
        self.ended = true;
    }

    pub(crate) const fn update_visual_effect(&mut self) {
        if self.loop_value == 0 {
            self.end_visual_effect();
        }
    }
}

impl Default for CVisualEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CVisualEffect {
    fn drop(&mut self) {
        self.ended = true;
        self.loop_value = 0;
    }
}
