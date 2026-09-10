//! Общее владение visual-ресурсами навыков GameServer.
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
//! CRageEffect из `appserver/skills/rage.cpp` не добавляет полей: native
//! выделяет 0xC байт (0x005A0859), вызывает base constructor (0x005A0877),
//! назначает vtable 0x0065BEF0 и skill+0x34 (0x005A0888), затем вызывает
//! BeginVisualEffect(1) (0x005A0897) до CheckCastCondition. AI ресурс не создаёт.
//! CKnightCutEffect из `appserver/skills/knightcut.cpp` также не добавляет
//! полей: три Begin выделяют 0xC байт (0x0059B8B0/0x0059B975/0x0059BA59),
//! вызывают тот же constructor, назначают vtable 0x0065BC40 и skill+0x34,
//! затем BeginVisualEffect(1) перед CheckCastCondition. Derived Update
//! (0x0059BAF0) сходится в base tail 0x0059BE8E даже без source/при ended.
//! Здесь concrete вид и одна общая база сохраняются одним owned ресурсом.
//! Source, skill ID, level и свойства не кешируются в ресурсе.
//! Derived Update(mode) (0x005A0900) выполняет свои gates и публикацию через
//! concrete/CGame owner, затем обязательно вызывает update_base_tail, включая
//! ended/missing-source ветви. Режим 3 сам не завершает visual; при loop=1
//! base tail не меняет ended. Одноаргументный Update остаётся базовым.
//!
//! CState constructor (0x005DBCA0) оставляет visual-указатель null, destructor
//! (0x005DBD40) вызывает CVisualEffect destructor и operator delete. Здесь
//! ресурс владеет своим состоянием по значению, без сырых указателей и
//! неявного копирования; Drop воспроизводит базовый destructor 0x005DC220:
//! ended=true, затем loop=0. Освобождение памяти остаётся обычному Rust owner-у.
//! Option<SkillVisualEffect> принадлежит зарегистрированному экземпляру навыка,
//! не concrete execution payload. Поэтому failed Begin без payload сохраняет
//! настоящий ресурс до End. Вид выбирает только derived-публикацию; хранение,
//! Begin, общий Update и Drop не дублируются между эффектами. Ресурс не
//! копируется; обычный Drop вызывает Drop его базы. Остальные эффекты не заменены
//! этими телами; их подключение остаётся у владельцев. Пакет не заменяет Drop.
//! Повторное присваивание visual безопасно освобождает прежний ресурс, вместо
//! утечки старого указателя native Begin; дополнительных пакетов Drop не шлёт.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillVisualEffectKind {
    Rage,
    KnightCut,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SkillVisualEffect {
    kind: SkillVisualEffectKind,
    base: CVisualEffect,
}

impl SkillVisualEffect {
    pub(crate) const fn new(kind: SkillVisualEffectKind, loop_value: i32) -> Self {
        let mut base = CVisualEffect::new();
        base.begin_visual_effect(loop_value);
        Self { kind, base }
    }

    pub(crate) const fn kind(&self) -> SkillVisualEffectKind {
        self.kind
    }

    pub(crate) const fn is_ended(&self) -> bool {
        self.base.is_ended()
    }

    pub(crate) const fn update_base_tail(&mut self) {
        self.base.update_visual_effect();
    }

    pub(crate) const fn begin_visual_effect(&mut self, loop_value: i32) {
        self.base.begin_visual_effect(loop_value);
    }
}

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
