//! Общая база прицельных снарядов Archery, BaseMagic и FireBolt и
//! площадных областей FireBall и GodPunishment:
//! снимок полёта, физический контакт Archery, элементный контакт,
//! усилитель душами, общий серверный decoder снимка и живые композиты
//! FireBall и GodPunishment.
//!
//! Размещение в `skills/`: полёт строится на конверте `summonshape`, а
//! боевые формулы принадлежат навыковым владельцам, как у соседних
//! `elementphalanx` и `directelement`.
//!
//! Точная пара: `original/server/Miracle_server/GameServer/gameserver.exe`
//! (SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`)
//! + `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`
//! age 2, совпадение подтверждено `.local/evidence/symbols.py identity`).
//! Исходные владельцы PDB: `appserver/skills/archeryphalanx.cpp/.h`,
//! `basemagicphalanx.cpp/.h`, `fireboltphalanx.cpp/.h`,
//! `fireballphalanx.cpp/.h` и `godpunishmentphalanx.cpp/.h`; базовый класс —
//! `appserver/summonshape.cpp/.h` (кто хранит слоты полёта, master и skill id,
//! см. `skills/summonshape`).
//!
//! Pub-адреса (`.local/evidence/symbols.py pubs`): ctor `CArcheryPhalanx`
//! `1:00200aa0`, общий `End` `1:00200c10`,
//! `CArcheryPhalanx::CalculateAttackPower` `1:00200c40`,
//! `CArcheryPhalanx::AI` `1:00201000`,
//! `CBaseMagicPhalanx::CalculateAttackPower` `1:00201240`,
//! `CFireBoltPhalanx::CalculateAttackPower` `1:001fc970`,
//! `CFireBoltPhalanx::AddToByteArray` `1:001fad20` (то же тело у FireBall),
//! `CGodPunishmentPhalanx::AddToByteArray` `1:001f44d0`
//! (оба encoder-а читают общий `CSummonShape::GetRemainedTime` `1:001e8870`),
//! `CFireBallPhalanx::AI` `1:001f75f0`,
//! `CArcheryPhalanx::DecordFromByteArray` `1:001ea070` (RVA `0x1EB070`),
//! серверные декодеры FireBall `1:001fafa0` (RVA `0x1FBFA0`) и
//! GodPunishment `1:001ff7c0` (RVA `0x2007C0`),
//! базовый `CSummonShape` ctor `1:001e87a0`, `CShape::DecordFromByteArray`
//! pub offset `0x5a280`.
//!
//! Сроки полёта: VERIFIED_DISASSEMBLY. `CArcheryPhalanx::AI` (RVA
//! `0x201000`) дважды вызывает часы через IAT `0x64B264`
//! (`WINMM!timeGetTime`): истечение `now > started + lifetime` и готовность
//! атаки `now > started + attack_delay` считаются dword-сложением по модулю
//! 2^32 с беззнаковыми `ja`/`jbe`; строгость неравенств сохранена дословно.
//! Между проверками стоит NULL-проверка выделенного конструктором `CScope`
//! (`+0xBC` → общий End) — сама область и её время жизни остаются у прежнего
//! владельца AI и не переносятся.
//!
//! Общий End: VERIFIED_DISASSEMBLY. Тело по адресу `1:00200c10` —
//! `mov dword ptr [ecx+0x80], 1; ret`, оно свёрнуто линкером как минимум
//! для Archery и FireBolt: тихая пометка `SHAPE_CHANGE_DELETE` без рассылки.
//!
//! Конструктор `CArcheryPhalanx` (pub `1:00200aa0`): VERIFIED_DISASSEMBLY
//! для хранимого снимка. Базовый `CSummonShape` (pub `1:001e87a0`) принимает
//! master и срок жизни: вложенный `tagMasterInfo` конструируется и
//! присваивается по `+0x84` (ctor/assign VA `0x50A610/0x50A640`, см.
//! `combat/masterinfo`), срок жизни кладётся в `+0xB0`, живой `timeGetTime` —
//! в `+0xB4`, а skill id предустановлен `0x7FFFFFFF` в `+0xB8`. Archery
//! перезаписывает `+0xB8` литералом `2` (= прежнему `ARCHERY_SKILL_ID`
//! адаптера), уровень аргументом в `+0xCC`, неиспользуемые дальше аргументы
//! MIN/MAX/ELEMENT — в `+0xC0/+0xC4/+0xC8` (не дублируются), задержку атаки
//! в `+0xD0`, цель type/id в `+0xD4/+0xD8`; отдельно выделяется не участвующий
//! в расчёте `CScope` (`+0xBC`). Машинный `AI` подтверждает назначение слотов
//! дедлайнами `+0xB4`+`+0xB0` и `+0xB4`+`+0xD0` и поиском цели по
//! `+0xD4/+0xD8`.
//!
//! Физический удар Archery: VERIFIED_DISASSEMBLY по
//! `CArcheryPhalanx::CalculateAttackPower` (pub `1:00200c40`). Сначала
//! `GetGame()` (pub `?GetGame@@YAPAVCGame@@XZ`) и поиск CPlayer в map игроков
//! по `attacker_id` атаки независимо от сохранённого типа, затем NULL-проверка
//! цели и живая таблица через pub
//! `?QuerySkillBaseProperties@CSkillFactory@@SAPAVCSkillBaseProperties@@W4tagSkillID@@J@Z`
//! (skill id из `+0xB8`, уровень из `+0xCC`); отсутствие игрока или таблицы
//! оставляет атаку прежней, записи полей идут после. Из снимка объекта
//! пишутся skill id, байт уровня и нулевой damage modifier; уровень цели
//! читается vtable `+0x110` как байт (movzx), живой weapon modifier — vtable
//! `+0x184` игрока с записью в damage factor, hit — `QueryProperty(20001)`
//! (`push 0x4E21`; чтения 20002 нет). Физический roll: чтения MAX
//! (vtable `+0xE8`) и MIN (`+0xE4`), ширина `max(max - min, 0)` БЕЗ +1
//! (`sub` + `jns`/`xor` — отличие от элементной формулы ниже, сохранено
//! дословно), второе чтение MIN до RNG глобального `random` (pub
//! `?random@@YAHH@Z`), сумма второй MIN со сгенерированным значением и нижняя
//! граница ноль; далее компонент Element (vtable `+0x118`) с той же нижней
//! границей, компонент Soul (vtable `+0x11C`, movzx u16) и критический хвост:
//! CCH (vtable `+0x114`, movzx u16), RNG(100), флаг и float-масштабирование
//! FILD/FMUL/FISTP только компонентов видов 1/3/4 (Physical/Element/Soul) —
//! тот же фильтр и та же последовательность, что у элементной формулы.
//! Выделения `new tagAttackPower` (ctor `0x5D3C80`) и `std::vector::push_back`
//! выражены `Vec::push`; множитель крита читается из изменяемой BSS-глобали
//! (`fmul dword ptr [0xEF3E5C]`) и передаётся параметром `critical_rate`
//! живого чтения владельца, как у элементной формулы.
//!
//! Элементная формула: VERIFIED_DISASSEMBLY по четырём телам семьи —
//! `CFireBoltPhalanx::CalculateAttackPower` (RVA `0x1FC970`),
//! `CBaseMagicPhalanx::CalculateAttackPower` (RVA `0x201240`),
//! `CFireBallPhalanx::CalculateAttackPower` (pub `1:001f6df0`, RVA
//! `0x1F7DF0`) и `CGodPunishmentPhalanx::CalculateAttackPower` (pub
//! `1:001fce10`, RVA `0x1FDE10`). Живой element_modify игрока читается
//! первым (прямая загрузка `[player+0x3F0]`, до трёх записей снимка), живой
//! таблицы навыка в этих телах нет — min/max/element_modifier берутся из
//! снимка; затем уровень цели (vtable `+0x110`) и живой weapon modifier
//! (vtable `+0x184`, float в `damage_factor`), `hit_modifier = 100`,
//! знаковое `element_modifier * element_modify / 100` (магия 0x51EB851F),
//! ширина `abs(max - min) + 1`, RNG, сохранённый MIN, живой AddElementAtk
//! (vtable `+0x118`), необязательное усиление душами и нижняя граница ноль;
//! единственная атака вида Element (исходное kind=3). Критический хвост
//! общий: живой CCH (vtable `+0x114`, movzx u16), RNG(100), затем флаг
//! `critical` и масштабирование компонентов видов 1/3/4 (Physical/Element/
//! Soul) float-множителем с усечением FISTP. Тела различаются только
//! смещениями боевых слотов снимка и наличием усилителя: FireBolt и
//! FireBall — один профиль (min/max/element `+0xC0/+0xC4/+0xC8`, souls
//! count/variable `+0xCC/+0xD0`, уровень `+0xD4`, усилитель есть);
//! BaseMagic — те же min/max/element, уровень `+0xCC`, без souls;
//! GodPunishment — уровень `+0xC8`, min/max/element `+0xBC/+0xC0/+0xC4`,
//! без souls. Сумма трёх слагаемых по модулю 2^32 не зависит от порядка
//! сложения.
//!
//! Усилитель душами: VERIFIED_DISASSEMBLY (участок RVA `0x1FDAC0`..`0x1FDB14`
//! у FireBolt; идентичная цепочка в теле FireBall RVA `0x1F7DF0`, обе
//! ссылаются на одни адреса констант `0x64DBD0` и `0x64DB50`, прочитанные
//! из EXE как 0.01f и 1.0f): обе нулевые проверки `count == 0` /
//! `variable == 0`, затем x87-цепочка `fild variable`, `fimul count`,
//! `fmul float(0.01)`, `fadd float(1.0)`, `fimul damage` и одно усечение
//! FISTP без промежуточной float-записи. У BaseMagic и GodPunishment
//! усилителя в теле нет (между суммой и нижней границей ничего).
//!
//! Клиентский снимок `encode_client_snapshot` делегируется общему конверту
//! `summonshape` (master type/id вложенного `tagMasterInfo`, не сохранённая
//! цель): статус см. там.
//!
//! Серверный `DecordFromByteArray` (pub `1:001ea070`, RVA `0x1EB070`):
//! VERIFIED_DISASSEMBLY. Линкер сливает в одно тело декодеры
//! `CArcheryPhalanx`, `CBaseMagicPhalanx` и `CBFBaseAttackPhalanx`; hub не
//! используется, поэтому тело перенесено. Пять DWORD читаются подряд с
//! продвижением offset: skill id в `+0xB8`, уровень в `+0xCC`, master type/id
//! вложенного `tagMasterInfo` в `+0x84/+0x88` (остальные восемь полей master
//! не трогаются) и остаток времени в слот срока жизни `+0xB0` — зеркально
//! префиксу `encode_related_phalanx_prefix`; затем одно чтение часов через
//! IAT `0x64B264` перезапускает отсчёт в `+0xB4`, и хвост делегируется
//! `CShape::DecordFromByteArray` (pub offset `0x5a280`; достигнутый
//! `decode_from_byte_array` в `regions::shape`) с проброшенным
//! `include_ex_data`. Неконтролируемые native-чтения выражены проверками
//! `UnexpectedEnd`: уже записанные поля префикса сохраняются, как и у
//! оригинала, возвращавшего FALSE после частичных записей. Достижимого
//! caller-а у оригинала нет; часы приходят параметром, как у клиентского
//! encoder-а.
//!
//! Серверные декодеры FireBall (pub `1:001fafa0`, RVA `0x1FBFA0`) и
//! GodPunishment (pub `1:001ff7c0`, RVA `0x2007C0`): VERIFIED_DISASSEMBLY.
//! Линкер оставил каждому своё тело, но оба повторяют общую форму
//! decoder-а семьи: пять DWORD (skill id → `+0xB8`, уровень → слот
//! профиля `+0xD4` FireBall и `+0xC8` GodPunishment, master `+0x84/+0x88`,
//! остаток → `+0xB0`), одно чтение часов через IAT `0x64B264` в `+0xB4`
//! и хвост `CShape::DecordFromByteArray` (pub offset `0x5a280`). Общий
//! префикс `decode_server_snapshot` покрывает оба тела: `skill_level`
//! префикса владелец относит к слоту своего профиля. Достижимого caller-а
//! у оригинала нет.
//!
//! Боевые слоты снимков BaseMagic и FireBolt: VERIFIED_DISASSEMBLY
//! перекрёстной сверкой ctor↔CAP (ctor BaseMagic pub `1:002010b0`, FireBolt
//! pub `1:001fc7d0`; контрольно FireBall pub `1:001f6cc0` и GodPunishment
//! pub `1:001fcd80`): каждый читаемый CAP боевой слот пишется ctor из
//! собственного J-аргумента (порядок аргументов между слотами перемешан,
//! семантику слота фиксируют чтения CAP; для Archery layout сверен выше по
//! её собственному ctor). Аргументы-остатки, не читаемые CAP: у BaseMagic
//! три лишних слота `+0xD0/+0xD4/+0xD8`, у FireBolt `+0xD8/+0xDC/+0xE0` —
//! Archery-quirk неиспользуемых MIN/MAX/ELEMENT сюда не распространяется,
//! у этих типов min/max/element читаются CAP. Литерал навыка кладётся в
//! `+0xB8`: BaseMagic 3, FireBolt 0x132, FireBall 0x13D, GodPunishment
//! 0x13A. BaseMagic, FireBolt и FireBall выделяют CScope (`+0xBC`, остаётся
//! у владельца AI; у FireBall статические шаблоны `g_bScope` заменяет
//! итератор `FireBallPath::scope_cells` окном 3×3 X→Y), у FireBall ctor
//! дополнительно копирует вектор клеток области (аргумент) в `+0xDC..+0xE8`
//! — он хранится независимым снимком `FireBallPath`; у GodPunishment CScope
//! нет, и layout сдвинут на слот (min в `+0xBC`).
//!
//! Оставшиеся слоты движения FireBall: VERIFIED_DISASSEMBLY по ctor
//! `1:001f6cc0` и AI `1:001f75f0`. Ctor кладёт аргумент speed в `+0xD8`
//! и обнуляет `+0xEC` (текущая позиция), `+0xF0/+0xF4` (endpoint) и
//! `+0xF8` (признак ForceMove). AI считает дедлайн очередной клетки
//! dword-сложением `+0xB4 + позиция · speed`, пишет endpoint перед
//! BLOCK3-областью, растит позицию после неё и однократно отправляет
//! ForceMove последней клетки с длительностью `len · speed`, ставя
//! признак после callback. Независимый `Vec` заменяет исходный STL-вектор;
//! endpoint клиентскому encoder-у не нужен.
//! Точные имена полей PDB не фиксировались.
//!
//! Живые композиты `CFireBallPhalanx` и `CGodPunishmentPhalanx` перенесены
//! сюда из старого адаптера буквально порцией замыкания: flight
//! `BaseProjectileFlight` + элементный снимок `ElementProjectileAttack`
//! (у FireBall — с движением `FireBallPath` и усилителем душами из cast-а,
//! у GodPunishment — без CScope и без усилителя). Состав полей и порядок
//! записей новыми машинными основаниями не дополнялись, статусы — выше по
//! шапке. Live-разрешение полей источника и доставка контакта остаются у
//! владельца `CGame` (обёртка `elementprojectileattack` старого пакета).

use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::combat::{AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original};
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeDecodeError};
use nebokrai_shared::protocol::LegacyReader;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseProjectileFlight {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

/// Прочитанный серверным decoder-ом префикс снимка прицельной фаланги до
/// базового `CShape`: идентификатор и уровень навыка владельца и master
/// type/id вложенного `tagMasterInfo`. Назначение и порядок чтений — в шапке
/// модуля; остаток времени ложится сразу в слот срока жизни полёта.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProjectileServerSnapshotPrefix {
    pub skill_id: i32,
    pub skill_level: i32,
    pub master_type: i32,
    pub master_id: i32,
}

impl BaseProjectileFlight {
    pub fn new(
        id: i32, started_at_ms: u32, lifetime_ms: u32,
        attack_delay_ms: u32, target: ShapeIdentity,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, started_at_ms, lifetime_ms, attack_delay_ms,
            target: ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..target },
        }
    }

    /// Снимок полёта областей без прицела (FireBall, GodPunishment): общими
    /// с прицельными снарядами остаются форма, часы и конверт `CSummonShape`;
    /// их конструкторы слотов задержки атаки и цели не содержат, поэтому они
    /// и не задаются.
    pub fn new_untargeted(id: i32, started_at_ms: u32, lifetime_ms: u32) -> Self {
        Self::new(
            id, started_at_ms, lifetime_ms, 0,
            ShapeIdentity { object_type: 0, id: 0, ex_id: CGuid::GUID_INVALID },
        )
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn target(&self) -> ShapeIdentity { self.target }

    /// Общая стартовая отметка полёта (слот `+0xB4`); движение FireBall
    /// считает от неё дедлайн очередной клетки.
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }

    pub fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub fn attack_due_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.attack_delay_ms) < now_ms
    }

    pub fn end(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub fn encode_client_snapshot(
        &self, skill_id: u32, skill_level: i32, master: MasterInfo,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, skill_id as i32, skill_level,
            master.master_type, master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }

    /// Общий серверный `DecordFromByteArray` прицельных фаланг (тело слито
    /// линкером у Archery, BaseMagic и BFBaseAttack): пять DWORD префикса —
    /// навык, уровень, master type/id, остаток времени — затем перезапуск
    /// отсчёта от живых часов и хвост базового `CShape`. Порядок записей и
    /// их назначение сверены в шапке модуля; достижимого caller-а у
    /// оригинала нет.
    pub fn decode_server_snapshot(
        &mut self,
        prefix: &mut ProjectileServerSnapshotPrefix,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Result<(), ShapeDecodeError> {
        prefix.skill_id = read_projectile_i32(source, cursor, "прицельный снаряд: skill id")?;
        prefix.skill_level =
            read_projectile_i32(source, cursor, "прицельный снаряд: уровень навыка")?;
        prefix.master_type =
            read_projectile_i32(source, cursor, "прицельный снаряд: master type")?;
        prefix.master_id = read_projectile_i32(source, cursor, "прицельный снаряд: master id")?;
        self.lifetime_ms =
            read_projectile_u32(source, cursor, "прицельный снаряд: остаток времени")?;
        self.started_at_ms = now_milliseconds();
        self.shape.decode_from_byte_array(source, cursor, include_child)
    }
}

/// Движение области FireBall по пути клеток: независимый снимок пути,
/// курсор и однократный ForceMove. Слоты и назначение — по ctor
/// `1:001f6cc0` и AI `1:001f75f0` (шапка модуля): speed лежит в `+0xD8`
/// между souls и уровнем профиля, вектор клеток — в `+0xDC`, текущая
/// позиция, endpoint и флаг обнуляются конструктором; срок жизни читает
/// свои часы отдельно — у полёта композита.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FireBallPath {
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    current_position: u32,
    destination: (i32, i32),
    force_moved: bool,
}

impl FireBallPath {
    /// Курсор, endpoint и флаг начинаются нулевым состоянием конструктора;
    /// путь сохраняется независимо от владельца cast-а.
    pub fn new(path: Vec<(i32, i32)>, speed_ms: u32) -> Self {
        Self { path, speed_ms, current_position: 0, destination: (0, 0), force_moved: false }
    }

    pub fn is_empty(&self) -> bool { self.path.is_empty() }

    /// Готовность очередной клетки: дедлайн считается dword-сложением
    /// `started + позиция · speed` от общей стартовой отметки полёта.
    pub fn cell_due_at(&self, started_at_ms: u32, now_ms: u32) -> bool {
        started_at_ms.wrapping_add(
            self.current_position.wrapping_mul(self.speed_ms)
        ) <= now_ms
    }

    pub fn current_cell(&self) -> Option<(i32, i32)> {
        self.path.get(self.current_position as usize).copied()
    }

    /// Endpoint допущенной клетки; клиентский конверт его не содержит.
    pub fn set_destination(&mut self, cell: (i32, i32)) { self.destination = cell; }

    pub fn advance(&mut self) { self.current_position = self.current_position.wrapping_add(1); }

    /// Один раз за жизнь области: последняя клетка пути и длительность
    /// `len · speed`; флаг владелец ставит после callback.
    pub fn pending_force_move(&self) -> Option<(i32, i32, u32)> {
        if self.force_moved { return None; }
        let &(x, y) = self.path.last()?;
        Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
    }

    pub fn mark_force_moved(&mut self) { self.force_moved = true; }

    /// Статическая маска области 3×3 X→Y, заменяющая CScope конструктора.
    pub fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        (0..3).flat_map(move |x| (0..3).map(move |y| {
            (center_x.wrapping_add(x - 1), center_y.wrapping_add(y - 1))
        }))
    }
}

/// Живые поля источника и RNG элементного контакта прицельного снаряда.
/// Разрешает их старый владелец в порядке исходного Calculate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementProjectileLiveField {
    RandomBelow(i32), AddElementAttack, CriticalChance, ElementModify,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SoulProjectileAmplification {
    count: i32,
    variable: i32,
}

impl SoulProjectileAmplification {
    pub const fn new(count: i32, variable: i32) -> Self { Self { count, variable } }

    fn apply(self, damage: i32) -> i32 {
        if self.count == 0 || self.variable == 0 { return damage; }
        truncate_original(
            (f64::from(self.variable) * f64::from(self.count) * f64::from(0.01_f32) + 1.0)
                * f64::from(damage),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElementProjectileAttack {
    pub master: MasterInfo,
    pub skill_id: u32,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub element_modifier: i32,
    pub souls: Option<SoulProjectileAmplification>,
}

impl ElementProjectileAttack {
    pub fn attack_master(self) -> MasterInfo {
        phalanx_attack_master(self.master)
    }

    /// Вызывается после разрешения Player по attacker ID: живой
    /// element_modify читается первым, до трёх записей конструктора, и
    /// возвращается для `roll_damage`. Отсутствие поля оставляет исходную
    /// пустую атаку, но не отменяет контакт у владельца.
    pub fn begin_calculation(
        self, attack: &mut AttackInformation,
        mut read_live: impl FnMut(ElementProjectileLiveField) -> Option<i32>,
    ) -> Option<i32> {
        let element_modify = read_live(ElementProjectileLiveField::ElementModify)?;
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
        Some(element_modify)
    }

    /// Вызывается после разрешения уровня цели и живого weapon modifier
    /// (оба остаются у владельца). Отсутствие живого AddElement или CCH
    /// обрывает расчёт с уже записанными полями и компонентами.
    pub fn roll_damage(
        self, attack: &mut AttackInformation, element_modify: i32, weapon_modifier: f32,
        critical_rate: f32, mut read_live: impl FnMut(ElementProjectileLiveField) -> Option<i32>,
    ) {
        attack.damage_factor = weapon_modifier;
        attack.hit_modifier = 100;
        let element = self.element_modifier.wrapping_mul(element_modify).wrapping_div(100);
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1);
        let Some(random) = read_live(ElementProjectileLiveField::RandomBelow(width)) else { return; };
        let rolled = random.wrapping_add(self.minimum_attack);
        let Some(addition) = read_live(ElementProjectileLiveField::AddElementAttack) else { return; };
        let damage = element.wrapping_add(addition.wrapping_add(rolled));
        let damage = self.souls.map_or(damage, |souls| souls.apply(damage)).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0,
        });
        let Some(chance) = read_live(ElementProjectileLiveField::CriticalChance) else { return; };
        let Some(roll) = read_live(ElementProjectileLiveField::RandomBelow(100)) else { return; };
        apply_projectile_critical(attack, chance, roll, critical_rate);
    }
}

/// Литерал навыка огненного шара для слота `+0xB8` (ctor pub `1:001f6cc0`,
/// статус в шапке модуля).
pub const FIRE_BALL_SKILL_ID: u32 = 0x13d;

/// Литерал навыка божественной кары для слота `+0xB8` (ctor pub `1:001fcd80`,
/// статус в шапке модуля).
pub const GOD_PUNISHMENT_SKILL_ID: u32 = 0x13a;

/// Живая форма движущейся площадной области огненного шара: общий полёт,
/// элементный контакт с усилителем душами и движение пути. Души приходят
/// снимком из cast-а, как у FireBolt. Срок жизни и дедлайн очередной клетки
/// читают разные часы.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CFireBallPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
    movement: FireBallPath,
}

impl CFireBallPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора FireBallPhalanx")]
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, path: Vec<(i32, i32)>, speed_ms: u32,
        soul_count: i32, soul_variable: u32,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new_untargeted(id, started_at_ms, lifetime_ms),
            attack: ElementProjectileAttack {
                master,
                skill_id: FIRE_BALL_SKILL_ID,
                skill_level,
                minimum_attack,
                maximum_attack,
                element_modifier,
                souls: Some(SoulProjectileAmplification::new(soul_count, soul_variable as i32)),
            },
            movement: FireBallPath::new(path, speed_ms),
        }
    }

    pub const fn shape(&self) -> &CShape { self.flight.shape() }
    pub const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub const fn master(&self) -> MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }
    pub fn path_is_empty(&self) -> bool { self.movement.is_empty() }

    pub fn expired_at(&self, now_ms: u32) -> bool { self.flight.expired_at(now_ms) }

    pub fn cell_due_at(&self, now_ms: u32) -> bool {
        self.movement.cell_due_at(self.flight.started_at_ms(), now_ms)
    }

    pub fn current_cell(&self) -> Option<(i32, i32)> { self.movement.current_cell() }
    pub fn set_cell_destination(&mut self, cell: (i32, i32)) {
        self.movement.set_destination(cell);
    }
    pub fn advance(&mut self) { self.movement.advance(); }
    pub fn pending_force_move(&self) -> Option<(i32, i32, u32)> {
        self.movement.pending_force_move()
    }
    pub fn mark_force_moved(&mut self) { self.movement.mark_force_moved(); }

    pub fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        FireBallPath::scope_cells(center_x, center_y)
    }

    pub fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            FIRE_BALL_SKILL_ID, self.attack.skill_level, self.attack.master,
            now_milliseconds,
        )
    }
}

/// Живая форма одноклеточной области божественной кары: тот же общий полёт
/// и элементный контакт без усиления душами и без CScope конструктора.
/// Замена игнорирует уровень и завершает форму при совпадении живой клетки.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CGodPunishmentPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
}

impl CGodPunishmentPhalanx {
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new_untargeted(id, started_at_ms, lifetime_ms),
            attack: ElementProjectileAttack {
                master,
                skill_id: GOD_PUNISHMENT_SKILL_ID,
                skill_level,
                minimum_attack,
                maximum_attack,
                element_modifier,
                souls: None,
            },
        }
    }

    pub const fn shape(&self) -> &CShape { self.flight.shape() }
    pub const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub const fn master(&self) -> MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }

    pub fn expired_at(&self, now: u32) -> bool { self.flight.expired_at(now) }

    pub fn replacement_matches(&self, _level: i32, x: i32, y: i32) -> bool {
        self.flight.shape().get_tile_x().unwrap_or(i32::MIN) == x
            && self.flight.shape().get_tile_y().unwrap_or(i32::MIN) == y
    }

    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            GOD_PUNISHMENT_SKILL_ID, self.attack.skill_level, self.attack.master, now,
        )
    }
}

/// Табличный параметр живого hit модификатора Archery (исходный
/// `QueryProperty(20001)`; прежний `USER_HIT_MODIFIER` оружейного прохода
/// старого пакета — тот же номер usage).
pub const ARCHERY_HIT_MODIFIER_PROPERTY: u32 = 20_001;

/// Живые поля источника и RNG физического контакта Archery прицельного
/// снаряда. Разрешает их старый владелец в порядке исходного Calculate;
/// movzx-u16 чтения Soul и CCH принадлежат этой же подстановке.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcheryProjectileLiveField {
    RandomBelow(i32), MinimumAttack, MaximumAttack,
    AddElementAttack, AddSoulAttack, CriticalChance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcheryProjectileAttack {
    pub master: MasterInfo,
    pub skill_id: u32,
    pub skill_level: i32,
}

impl ArcheryProjectileAttack {
    pub fn attack_master(self) -> MasterInfo {
        phalanx_attack_master(self.master)
    }

    /// Вызывается после разрешения игрока по attacker ID и живой таблицы
    /// навыка (оба остаются у владельца): записывает исходные id навыка,
    /// байт уровня и нулевой damage modifier.
    pub fn begin_calculation(self, attack: &mut AttackInformation) {
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
    }

    /// Вызывается после разрешения уровня цели и живого weapon modifier
    /// (оба остаются у владельца). Hit читается параметром
    /// `ARCHERY_HIT_MODIFIER_PROPERTY`, затем физический roll в исходном
    /// порядке MAX→MIN→второй MIN→RNG с шириной `max(max - min, 0)` без +1
    /// (отличие от элементной формулы), нижняя граница ноль, компоненты
    /// Element и Soul и общий критический хвост. Отсутствие живого поля
    /// обрывает расчёт с уже записанными полями и компонентами.
    pub fn roll_damage(
        self, attack: &mut AttackInformation, weapon_modifier: f32, critical_rate: f32,
        mut query_property: impl FnMut(u32) -> u32,
        mut read_live: impl FnMut(ArcheryProjectileLiveField) -> Option<i32>,
    ) {
        attack.damage_factor = weapon_modifier;
        attack.hit_modifier = query_property(ARCHERY_HIT_MODIFIER_PROPERTY) as i32;
        let Some(maximum) = read_live(ArcheryProjectileLiveField::MaximumAttack) else { return; };
        let Some(minimum) = read_live(ArcheryProjectileLiveField::MinimumAttack) else { return; };
        let width = maximum.wrapping_sub(minimum).max(0);
        let Some(minimum) = read_live(ArcheryProjectileLiveField::MinimumAttack) else { return; };
        let Some(random) = read_live(ArcheryProjectileLiveField::RandomBelow(width)) else { return; };
        let physical = minimum.wrapping_add(random).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0,
        });
        let Some(element) = read_live(ArcheryProjectileLiveField::AddElementAttack) else { return; };
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: element.max(0), mp_damage: 0,
        });
        let Some(soul) = read_live(ArcheryProjectileLiveField::AddSoulAttack) else { return; };
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0,
        });
        let Some(chance) = read_live(ArcheryProjectileLiveField::CriticalChance) else { return; };
        let Some(roll) = read_live(ArcheryProjectileLiveField::RandomBelow(100)) else { return; };
        apply_projectile_critical(attack, chance, roll, critical_rate);
    }
}

/// Общее для прицельных фаланг правило владельца атаки: только master типа
/// 400 сохраняет расширенные поля, остальные получают type/id при нулевых
/// разрешениях.
fn phalanx_attack_master(master: MasterInfo) -> MasterInfo {
    if master.master_type == 400 { return master; }
    MasterInfo {
        master_type: master.master_type, master_id: master.master_id,
        ..MasterInfo::default()
    }
}

/// Общий критический хвост: после живого шанса и RNG(100) ставится флаг,
/// а компоненты физического, элементного и душевного видов (исходная
/// фильтрация 1/3/4) масштабируются float-множителем с усечением.
fn apply_projectile_critical(
    attack: &mut AttackInformation, chance: i32, roll: i32, rate: f32,
) {
    if roll >= chance { return; }
    attack.critical = true;
    for power in &mut attack.damages {
        if matches!(power.kind,
            AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul)
        {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

fn read_projectile_i32(
    source: &[u8], cursor: &mut usize, field: &'static str,
) -> Result<i32, ShapeDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: 4, available: block.available,
        })?;
    let value = reader.read_i32().map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: block.needed, available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_projectile_u32(
    source: &[u8], cursor: &mut usize, field: &'static str,
) -> Result<u32, ShapeDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: 4, available: block.available,
        })?;
    let value = reader.read_u32().map_err(|block|
        ShapeDecodeError::UnexpectedEnd {
            field, offset: block.offset, needed: block.needed, available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}
