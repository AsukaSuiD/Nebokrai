# Доказательная база: навыки GameServer (`zone::skills`)

Машинные основания реконструкции навыков зоны: pub-адреса и RVA, формулы,
состав wire-конвертов и машинные quirks, снятые с точной пары EXE/PDB.
Материал вынесен из header-комментариев `server/rust/zone/src/skills/` при
нормализации: верхние комментарии исходников хранят назначение, границы
владения, инварианты и ссылку на этот документ.

Статусы и допустимость утверждений — по [правилам свидетельств](evidence-and-contracts.md).
VERIFIED_DISASSEMBLY означает досмотр машинного тела указанной функции в
точной паре; без дополнительной оговорки клиентское чтение серверных кадров
не исследовалось (UNKNOWN).

## Идентификаторы сборки

- EXE: `original/server/Miracle_server/GameServer/gameserver.exe`,
  SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`.
- PDB: `GameServer/GameServer.pdb`, SHA-256
  `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`,
  RSDS GUID `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`, age 2; совпадение EXE↔PDB
  подтверждено `.local/evidence/symbols.py identity`.
- Конвенция адресов: паблики PDB (S_PUB32) записаны как `seg:off`; сегмент 1
  — `.text` с RVA-базой `0x1000`, поэтому RVA = off + `0x1000`,
  VA = RVA + ImageBase `0x400000`. Адреса вида `0x5xxxxx` в записях — VA.

Все разделы относятся к этой паре; исключений среди файлов
`zone/src/skills/` нет.

## summonshape: семейство `CSummonShape` (идентичность и wire-конверт)

| Функция/symbol | pub/RVA | Статус | Факт |
|---|---|---|---|
| ctor `CSummonShape` | pub `1:001e87a0`, RVA `0x1E97A0` | VERIFIED_DISASSEMBLY | `mov dword ptr [esi+4], 0x3e8` — тип 1000 в `CBaseObject` |
| статик `?g_lID@CSummonShape@@1JA` | pub `3:0000671c` | VERIFIED_DISASSEMBLY | знаковый static long; ctor сперва выдаёт значение в `[esi+8]` (RVA `0x1E9820`), затем `add dword ptr [0x6a471c], 1` → выдаётся прежнее значение, приращение по модулю 2^32 |
| `GetSkillID` | pub `1:001e0030`, RVA `0x1E1030` | VERIFIED_DISASSEMBLY | skill id в `[+0xB8]` |
| `GetRemainedTime` | pub `1:001e8870`, RVA `0x1E9870` | VERIFIED_DISASSEMBLY | two reads `timeGetTime` через IAT `0x64B264`: истечение `now < lifetime + started` (`jb`), остаток `lifetime - now + started`; истёк → 0 |
| `tagMasterInfo` ctor/assign | RVA `0x10A610`/`0x10A640` | VERIFIED_DISASSEMBLY | вложенный master: type/id по `+0x84`/`+0x88`, копия десяти DWORD (см. `zone::combat::masterinfo`) |
| `CShape::AddToByteArray` | RVA `0x5B250` | VERIFIED_DISASSEMBLY | финальный хвост конверта; флаг `include_child` проброшен, перенос фиксирует `true` |

Порядок пяти полей конверта (skill id, уровень владельца, master type/id,
остаток времени, хвост `CShape`) доказан обходом всех 11 уникальных тел
`AddToByteArray@C*Phalanx` — VERIFIED_DISASSEMBLY для всего семейства.
Запись — побайтовый little-endian append (helper RVA `0x7AF00`), совпадает с
`LegacyWriter::write_i32/write_u32`.

Линкер сфолдовал 29 имён PDB в 11 байт-идентичных групп; уровень во всех
телах читается как поле самой фаланги (`this` в esi), различие immediate —
layout владельцев, не контракт:

| Тело (RVA/VA) | Имена PDB | Слот уровня | Соответствие zone-конверту |
|---|---|---|---|
| `0x1E4100` / `0x5E4100` | CFatalBlowPhalanx, CHeartLessArrowPhalanx2/3 | `[+0xBC]` | да |
| `0x1E47A0` / `0x5E47A0` | CArcheryPhalanx, CBaseMagicPhalanx, CBFBaseAttackPhalanx, CSpiderMistPhalanx, CLeimingPhalanx2, CYinYangPhalanx, CYinYangPhalanx2 | `[+0xCC]` | да |
| `0x1EF0C0` / `0x5EF0C0` | CGodThunderPhalanx, CGodThunderPhalanx2, CThunderPhalanx | `[+0xD8]` | префикс да; расширенный хвост (счётчик `n = ([+0xB0]/[+0xC4])·[+0xDC]`, беззнаковое деление) не перенесён |
| `0x1F54D0` / `0x5F54D0` | CWeakPhalanx, CGodPunishmentPhalanx, CThunderBlowPhalanx, CTianhuoPhalanx | `[+0xC8]` | да |
| `0x1F76A0` / `0x5F76A0` | CThunderSlashPhalanx | `[+0xE4]` | да |
| `0x1F8ED0` / `0x5F8ED0` | CSnowStormPhalanx | `[+0xD8]` | префикс да; расширенный хвост `n = ([+0xB0]/[+0xC4])·k2(level)·k1(level)` не перенесён |
| `0x1F9750` / `0x5F9750` | CFallingStarPhalanx, CMeteorArrowPhalanx | `[+0xEC]` | префикс да; расширенный хвост `n = [+0xE4]` не перенесён |
| `0x1F9F30` / `0x5F9F30` | CRainArrowPhalanx | `[+0x108]` | да |
| `0x1FA850` / `0x5FA850` | CLightingArrowPhalanx | `[+0xF4]` | да |
| `0x1FBD20` / `0x5FBD20` | CFireBoltPhalanx, CFireBallPhalanx, CFireWallPhalanx, CPoisonFogPhalanx, CThunderFirePhalanx | `[+0xD4]` | да |
| `0x1FECB0` / `0x5FECB0` | CChaosSpherePhalanx | `[+0xD8]` | да |

Расширенный хвост трёх групп (godthunder, snowstorm, fallingstar) — одна
схема: после остатка времени дописываются `[+0xB0]` и `[+0xC4]` как i32,
счётчик `n` как i32 и `8·n` сырых байт по указателю из `[+0xBC]` через
helper RVA `0x7AD90` (VA `0x47AD90`, raw-append `(vector, ptr, byte_count)`);
различается только вычисление `n`. Семантика полей (PDB TPI + дизассембл
write-sites): `+0xB0` = `m_dwLifeTime` (мс timeGetTime-шкалы), `+0xBC` =
`tagCell* m_pCells` (массив `{long lX, long lY}` — абсолютные tile-координаты
точек удара; raw-блок читается из самого указателя), `+0xC4` =
`m_dwFrequency` (период тика, из `QueryProperty(6001)`), `+0xDC` =
`m_dwNumTargets` (клеток на тик, из `QueryProperty(20010)`; у
FallingStar-семейства то же смещение — `m_lCCH`), `+0xE4` = `m_dwNumArrows`
(FallingStar/Meteor; у GT-семейства — `m_lCCH`).

SnowStorm: k1/k2 выбираются по уровню 1/2/3 (иначе k1(1)/k2(1)) из глобалей
RVA `0x2A52C4`/`0x2A52CC`/`0x2A52D4` (k1 = `g_dwLevelOne/Two/ThreeLength`) и
`0x2A52C8`/`0x2A52D0`/`0x2A52D8` (k2 = `…Height`); для этой сборки области
5×5 на всех уровнях → `n = ticks·25` независимо от уровня.

Серверный массив имеет страйд `H·L` на тик; wire-счётчик GT =
`ticks·NumTargets` самосогласован при конфиге `NumTargets == Length·Height`
(INFERRED: значения конфига вне машинного основания). Wire-dword после
префикса сериализуется, но на декоде пропускается (skip 8 байт — читается
`m_dwFrequency`); `NumTargets`, `NumArrows`, `m_dwAttackCount`, `m_pScope`
декодом НЕ восстанавливаются (особенность оригинала). Клиентское чтение
raw-массива клеток — UNKNOWN. Символов `*Masked*` в PDB этой сборки нет.

Текущая реализация: `zone::skills::summonshape`.

## projectile: прицельные снаряды и композиты FireBall/GodPunishment

| Функция/symbol | pub/RVA | Статус | Факт |
|---|---|---|---|
| ctor `CArcheryPhalanx` | pub `1:00200aa0` | VERIFIED_DISASSEMBLY | layout снимка ниже; выделяет `CScope` (`+0xBC`, в расчёте не участвует) |
| общий `End` семьи | pub `1:00200c10` | VERIFIED_DISASSEMBLY | `mov dword ptr [ecx+0x80], 1; ret` — линкер свернул минимум для Archery и FireBolt: тихая `SHAPE_CHANGE_DELETE` без рассылки |
| `CArcheryPhalanx::AI` | pub `1:00201000`, RVA `0x201000` | VERIFIED_DISASSEMBLY | два чтения часов через IAT `0x64B264`: `now > started + lifetime` и `now > started + attack_delay`, dword-сложение по модулю 2^32 с `ja`/`jbe`; между проверками NULL-гейт `CScope` (`+0xBC` → общий End) |
| `CArcheryPhalanx::CalculateAttackPower` | pub `1:00200c40` | VERIFIED_DISASSEMBLY | физическая формула ниже |
| `CFireBoltPhalanx::CalculateAttackPower` | pub `1:001fc970`, RVA `0x1FC970` | VERIFIED_DISASSEMBLY | элементная формула, профиль с усилителем |
| `CBaseMagicPhalanx::CalculateAttackPower` | pub `1:00201240`, RVA `0x201240` | VERIFIED_DISASSEMBLY | элементная формула без souls |
| `CFireBallPhalanx::CalculateAttackPower` | pub `1:001f6df0`, RVA `0x1F7DF0` | VERIFIED_DISASSEMBLY | элементная формула, профиль с усилителем |
| `CGodPunishmentPhalanx::CalculateAttackPower` | pub `1:001fce10`, RVA `0x1FDE10` | VERIFIED_DISASSEMBLY | элементная формула без souls, layout со сдвигом |
| `CFireBoltPhalanx::AddToByteArray` | pub `1:001fad20` | VERIFIED_DISASSEMBLY | то же тело у FireBall; читает общий `GetRemainedTime` `1:001e8870` |
| `CGodPunishmentPhalanx::AddToByteArray` | pub `1:001f44d0` | VERIFIED_DISASSEMBLY | то же тело у FireBall не здесь — собственный encoder, тот же `GetRemainedTime` |
| `CFireBallPhalanx::AI` | pub `1:001f75f0` | VERIFIED_DISASSEMBLY | движение пути ниже |
| семейный `DecordFromByteArray` | pub `1:001ea070`, RVA `0x1EB070` | VERIFIED_DISASSEMBLY | линкер слил декодеры Archery/BaseMagic/CBFBaseAttackPhalanx в одно тело; hub не используется — тело перенесено |
| серверный декодер FireBall | pub `1:001fafa0`, RVA `0x1FBFA0` | VERIFIED_DISASSEMBLY | форма общего префикса; достижимого caller-а у оригинала нет |
| серверный декодер GodPunishment | pub `1:001ff7c0`, RVA `0x2007C0` | VERIFIED_DISASSEMBLY | то же |
| `CShape::DecordFromByteArray` | pub offset `0x5a280` | VERIFIED_DISASSEMBLY | хвост семейного и серверных декодеров; пробрасывает `include_ex_data` |

Ctor-база `CSummonShape` (pub `1:001e87a0`) принимает master и срок жизни:
`tagMasterInfo` ctor/assign VA `0x50A610`/`0x50A640` по `+0x84`, срок жизни в
`+0xB0`, живой `timeGetTime` в `+0xB4`, skill id предустановлен `0x7FFFFFFF`
в `+0xB8`. Archery перезаписывает `+0xB8` литералом `2` (= прежнему
`ARCHERY_SKILL_ID` адаптера), уровень аргументом в `+0xCC`, неиспользуемые
аргументы MIN/MAX/ELEMENT в `+0xC0/+0xC4/+0xC8` (quirk: хранятся, CAP не
читает), задержку атаки в `+0xD0`, цель type/id в `+0xD4/+0xD8`. Машинный AI
подтверждает слоты дедлайнами `+0xB4`+`+0xB0` и `+0xB4`+`+0xD0` и поиском
цели по `+0xD4/+0xD8`.

Физический удар Archery (`1:00200c40`): `GetGame()`
(pub `?GetGame@@YAPAVCGame@@XZ`) и поиск CPlayer в map игроков по
`attacker_id` атаки независимо от сохранённого типа; NULL-гейт цели; живая
таблица через pub
`?QuerySkillBaseProperties@CSkillFactory@@SAPAVCSkillBaseProperties@@W4tagSkillID@@J@Z`
(skill id из `+0xB8`, уровень из `+0xCC`); отсутствие игрока или таблицы
оставляет атаку прежней. Из снимка — skill id, байт уровня, нулевой damage
modifier; уровень цели vtable `+0x110` (movzx байт), живой weapon modifier
vtable `+0x184` игрока в damage factor, hit — `QueryProperty(20001)`
(`push 0x4E21`; чтения 20002 нет). Roll: MAX (vtable `+0xE8`) и MIN (`+0xE4`),
ширина `max(max − min, 0)` БЕЗ `+1` (`sub` + `jns`/`xor` — отличие от
элементной формулы, сохранено дословно), второе чтение MIN до RNG глобального
`random` (pub `?random@@YAHH@Z`), сумма второй MIN с RNG и нижняя граница
ноль; далее компонент Element (vtable `+0x118`) с той же границей, Soul
(vtable `+0x11C`, movzx u16) и критический хвост: CCH (vtable `+0x114`,
movzx u16), RNG(100), флаг и FILD/FMUL/FISTP-масштабирование только компонентов
видов 1/3/4 (Physical/Element/Soul). `new tagAttackPower` (ctor `0x5D3C80`) и
`std::vector::push_back` выражены `Vec::push`; множитель крита — изменяемая
BSS-глобаль (`fmul dword ptr [0xEF3E5C]`), передаётся параметром
`critical_rate` живого чтения владельца.

Элементная формула (четыре тела): живой `element_modify` игрока читается
первым (прямая загрузка `[player+0x3F0]`, до трёх записей снимка), живой
таблицы навыка в телах нет — min/max/element из снимка; затем уровень цели
(vtable `+0x110`), живой weapon modifier (vtable `+0x184`, float в
`damage_factor`), `hit_modifier = 100`, знаковое
`element_modifier · element_modify / 100` (магия `0x51EB851F`), ширина
`abs(max − min) + 1`, RNG, сохранённый MIN, живой AddElementAtk (vtable
`+0x118`), необязательное усиление душами, нижняя граница ноль; единственная
атака вида Element (kind 3). Критический хвост общий с физической формулой.
Профили слотов: FireBolt и FireBall — min/max/element `+0xC0/+0xC4/+0xC8`,
souls count/variable `+0xCC/+0xD0`, уровень `+0xD4`; BaseMagic — те же
min/max/element, уровень `+0xCC`, без souls; GodPunishment — уровень `+0xC8`,
min/max/element `+0xBC/+0xC0/+0xC4`, без souls. Сумма трёх слагаемых по
модулю 2^32 не зависит от порядка.

Усилитель душами (VERIFIED_DISASSEMBLY; участок RVA `0x1FDAC0`..`0x1FDB14` у
FireBolt, идентичная цепочка в теле FireBall; константы `0x64DBD0`/`0x64DB50`
прочитаны из EXE как 0.01f и 1.0f): нулевые проверки `count == 0` и
`variable == 0`, затем x87-цепочка `fild variable`, `fimul count`,
`fmul 0.01f`, `fadd 1.0f`, `fimul damage` и одно усечение FISTP без
промежуточной float-записи. У BaseMagic и GodPunishment усилителя нет.

Семейный `DecordFromByteArray` и серверные декодеры FireBall/GodPunishment:
пять DWORD подряд (skill id → `+0xB8`, уровень → `+0xCC` у семейного /
слот профиля `+0xD4` FireBall и `+0xC8` GodPunishment, master type/id →
`+0x84/+0x88`, остаток → `+0xB0`), одно чтение часов через IAT `0x64B264`
перезапускает отсчёт в `+0xB4`, хвост `CShape::DecordFromByteArray`.
Неконтролируемые native-чтения выражены проверками `UnexpectedEnd`: уже
записанные поля префикса сохраняются, как у оригинала, возвращавшего FALSE
после частичных записей. Часы приходят параметром; достижимого caller-а у
оригинала нет.

Боевые слоты BaseMagic/FireBolt/FireBall/GodPunishment (VERIFIED_DISASSEMBLY
перекрёстной сверкой ctor↔CAP; ctor BaseMagic pub `1:002010b0`, FireBolt
`1:001fc7d0`, FireBall `1:001f6cc0`, GodPunishment `1:001fcd80`): каждый
читаемый CAP слот пишется ctor из собственного J-аргумента (порядок
аргументов перемешан, семантику фиксируют чтения CAP). Не читаемые CAP
лишние слоты: BaseMagic `+0xD0/+0xD4/+0xD8`, FireBolt `+0xD8/+0xDC/+0xE0` —
Archery-quirk на них не распространяется. Литерал навыка в `+0xB8`: BaseMagic
3, FireBolt `0x132`, FireBall `0x13D`, GodPunishment `0x13A`. BaseMagic,
FireBolt и FireBall выделяют `CScope` (`+0xBC`, у владельца AI; у FireBall
статические `g_bScope` заменяет итератор `FireBallPath::scope_cells` окном 3×3
X→Y); ctor FireBall дополнительно копирует вектор клеток области в
`+0xDC..+0xE8` (независимый снимок `FireBallPath`); у GodPunishment `CScope`
нет и layout сдвинут на слот.

Движение FireBall (ctor `1:001f6cc0`, AI `1:001f75f0`): ctor кладёт speed в
`+0xD8`, обнуляет `+0xEC` (позиция), `+0xF0/+0xF4` (endpoint), `+0xF8`
(ForceMove-флаг). AI считает дедлайн клетки dword-сложением
`+0xB4 + позиция·speed`, пишет endpoint перед BLOCK3-областью, растит позицию
после неё и однократно отправляет ForceMove последней клетки с длительностью
`len·speed`, ставя флаг после callback. Точные имена полей PDB не
фиксировались.

Текущая реализация: `zone::skills::projectile`; клиентский снимок —
конверт `zone::skills::summonshape`.

## state/catalog: клиентская проекция состояний

Значения time/additional/team_name 56 вариантов enum-каталога и план
loop/updated `CVisualEffect` после runtime Begin; порядок полей записи
(ID, time, additional, имя Team) принадлежит проходу
`CMoveShape::AddToByteArray_ForClient` (записывает `state/snapshot`).

| Функция/symbol | pub/RVA | Статус | Факт |
|---|---|---|---|
| `CMoveShape::AddToByteArray_ForClient` | pub off `0xCCD30`, RVA `0xCDD30` | VERIFIED_DISASSEMBLY | двухпроходный писатель записей состояний |
| `?Serialize@CState@@…` | pub off `0x1DAD00`, RVA `0x1DBD00` | VERIFIED_DISASSEMBLY | прежние записи «VA 0x005DAD00» — без учёта смещения секции |
| `?GetTeamName@CTeamState@@QAEPADXZ` | pub off `0x1BEAB0`, RVA `0x1BFAB0` | VERIFIED_DISASSEMBLY | SSO-ветвь: cap `[ecx+0x50] ≥ 0x10` → ptr `[ecx+0x3c]`, иначе inline `ecx+0x3c`; append C-строки через lstrlenA |
| `IsEnded@CState` | RVA `0xD8380` | VERIFIED_DISASSEMBLY | `return [ecx+0x30]`; писателем не вызывается — пишутся все непустые позиции |
| `CPlayer::AddToByteArray_ForClient` | RVA `0x4A480` | VERIFIED_DISASSEMBLY | сначала базовый проход, затем player-поля вне записей состояний |

Писатель (VERIFIED_DISASSEMBLY): байт-флаг из virtual `[this+0xD0]`
(`!= 0` даёт 0), DWORD числа непустых позиций `m_vStates`; оба прохода
(счёт и запись) по индексам вперёд с пропуском NULL — порядок записей равен
порядку позиций арены. На состояние: DWORD ID напрямую из `[state+4]`
(не виртуал), DWORD time — vtable `+0x30`, DWORD additional — vtable `+0x38`;
слоты доказаны совпадением с пабликами `?GetRemainedTime@CState@@…` /
`?GetAdditionalData@CState@@…` и четырьмя derived-переопределениями. Имя
дописывается только при ID `0x186A6`: `__RTDynamicCast` (RTTI CState
`0x69FFE0` → CTeamState `0x69FFF8`). Знаковой обработки time нет: пишется
DWORD как есть; «отрицательный» остаток возможен гонкой второго чтения часов
и воспроизводится тем же двойным чтением.

Значения time (vtable `+0x30`, VERIFIED_DISASSEMBLY):

- базовый `CState::GetRemainedTime` RVA `0x201200` = `xor eax,eax; ret`
  (ICF-слитый адрес — тот же код у `GetAdditionalData@CState` и десятков
  пустых virtual). Унаследован: Agility/Natural/Rapture, TaiJi,
  EnlargeFullMiss/MaxHp/MaxMp, Origin, MeteorArrow, EnergyHolding,
  SoulCollect, Swordship×4, WuXing×5, AutomaticRestore(Hp/Mp, +Fight),
  Particular, Team, TianShenXiaFan, Ride → нулевые time/additional.
- формула timed (deadline = `[ecx+0x2C]` + keep; now ≥ deadline → 0, иначе
  deadline − now, второе чтение hours по IAT `0x64B264`): RVA `0x1F2CD0`
  (keep `+0x38`: Heal/Heal2, Blind, BoaLock, BossBlueQuake, Cure, DaubPoison,
  Hearten, KnightCut, KnockOut, Roar, Rush/Rush2, Seal, SpiderWeb, Strike,
  Restore×2 consumable, Life/Machine/Mana/Promotion щиты, AutoProtect,
  UseGoods×5, SuperHeal×2), RVA `0x1D5F30` (keep `+0x3C`: Agility2,
  Callosity/2, BossBlueFury, Pillar, ImproveExp), RVA `0x205E10` (keep
  `+0x40`: Fury, RageBreak, Weak, Wangsheng, Po/Yu боевой феи ×8),
  RVA `0x206320` (keep `+0x60`: BloodLoss, Kerosene, LeafCut×3, PoisonArrow,
  SpiderPoison, SpriteBurn), RVA `0x201480` (keep `+0x48`: GodBless/2),
  RVA `0x207E00` (keep `+0x50`: PoisonFog). Формула едина, смещения keep —
  C-layout владельцев в `effects`.
- RVA `0x1DA030` (CHBYState и общий CExState) =
  `change_body_client_state_time` (1 после истечения ненулевого срока, до трёх
  чтений часов) — ChangeBody и Extended::Original; RVA `0x1D6320`
  (CExStateNew и CNotDisappearAfterDead) = `guarded_client_state_time`
  (нулевой срок → 0 без чтения часов) — Extended::New и Undead.

Значения additional (vtable `+0x38`):

- база RVA `0x201200` → 0 для всех классов вне пяти переопределений
  (VERIFIED_DISASSEMBLY);
- `return [ecx+0x38]`: MeteorArrow (RVA `0xF96E0`; поле — текущие стрелы,
  AddMeteorArrow RVA `0x1F6970` пишет с потолком `[ecx+0x3C]`) и Particular
  (тот же адрес; ctor RVA `0xF9440` сохраняет аргумент-additional в `+0x38`);
- `return [ecx+0x3C]`: SoulCollect (RVA `0xD7090`, ICF-слит с
  `?GetNumSouls@CSoulCollectState@@…`; AddSoul RVA `0x1E1BC0` инкрементирует
  `+0x3C` с потолком `+0x40`);
- `([ecx+0x38] << 16) | [ecx+0x3C]`: Ride (RVA `0xF8D50`; раскладка
  type/level по `CRideState::Serialize` RVA `0xF8F60`);
- Team (RVA `0x1BFDD0`): `(([ecx+0x68] != 0) << 16) | count`, где `[+0x68]` —
  длина std::string пароля (ctor RVA `0x1BFE60`: имя string по `+0x38`, пароль
  по `+0x54`), count — число элементов списка живого CTeam (count-fn
  RVA `0x107590`) по id из `CPlayer` `[+0xB20]` через синглтон-менеджер и RTTI
  CMoveShape→CPlayer / CSession→CTeam; любой обрыв цепочки даёт 1. Zone
  получает число параметром; hub по умолчанию передаёт 1. Принадлежность
  считанного контейнера членам команды — INFERRED (по классу CTeam); остальная
  структура записи Team VERIFIED_DISASSEMBLY.
- DISCREPANCY (открыто, поведение не менялось): DefenseShield Life/Machine/
  Mana в zone пишут additional = `life()`, а vtable `+0x38` всех трёх
  классов — базовый 0 (Promotion совпадает: 0; time всех четырёх — timed
  RVA `0x1F2CD0`). Оригинал остаток щита в клиентский снимок не пишет; вопрос
  передан владельцу, исправление — отдельным решением.

Итог: time VERIFIED_DISASSEMBLY у всех 56; additional VERIFIED_DISASSEMBLY у
55 (Team — с оговоркой INFERRED о природе младшего слова); единственное
расхождение — additional щитов Life/Machine/Mana. PARTIAL числовых
property-формул боевой феи относится к владельцу `effects/battlefairy.rs`.
Идентичность классов zone ↔ C++ подтверждена vtable/ctor-опорными адресами
(собраны ниже в разделе [«effects: wire-опорные адреса состояний
Zone»](#effects-wire-опорные-адреса-состояний-zone)); отображение перечислено
у каждого варианта в `state/storage`.

Текущая реализация: `zone::skills::state::catalog`; enum-каталог payload
(`StateData`) — `zone::skills::state::storage`.

## effects: wire-опорные адреса состояний Zone

Идентичность классов Zone `effects` ↔ C++ подтверждена ctor/vtable-опорными
адресами точной пары `gameserver.exe + GameServer.pdb`; исходные владельцы
PDB (`appserver/skills/*state.cpp`, `appserver/other states/*state.cpp`,
`appserver/player.cpp`, `appserver/states/*`) указаны в шапках модулей.
Таблица собирает адреса, которые жили в шапках zone-файлов; поведенческие
правила остаются в шапках соответствующих модулей `zone::effects::*`.
Форма записи — как в прежних шапках-владельцах, статус не повышался.

| zone-владелец | опорные адреса и факт |
|---|---|
| `blind` | BlindState-семейство, десять классов: KnightCut (`0x67`) ctor `0x005FCCF0/0x005FCD60`, vt `0x00661254`; KnockOut (`0x192`) `0x005F4F30/0x005F4FA0`, vt `0x00660894`; BoaLock (`0xD2`) `0x005FB560/0x005FB5D0`, vt `0x006610DC`; Blind (`0x76`) `0x00607380`, vt `0x00662214`; Rush (`0x73`) `0x006077E0`, vt `0x00662274`; Rush2 (`0x7C`) `0x005F12E0/0x005F1350`, vt `0x0066041C`; Seal (`0x138`) `0x005FF800/0x005FF870`, vt `0x006615F4`; Strike (`0xDD`) `0x00606830`, vt `0x00662154`; SpiderWeb (`0x199`) `0x005EA6E0/0x005EA750`, vt `0x0065FBCC`; BossBlueQuake (`0x1F8`) `0x005E8590/0x005E8600`, vt `0x0065F934`. Общие: Serialize `0x005F51E0`, Unserialize `0x005EAAC0`, GetRemainedTime `0x005F2CD0`, AI `0x005D5BA0`; у BoaLock — собственный End `0x005FB800` и пустой OnAction |
| `battlefairy` (Po/Yu ×8) | Pojia `0x005E80B0/0x005E8120` vt `0x0065F8CC` (`0x212`); Pobing `0x005E7B20/0x005E7B90` vt `0x0065F864` (`0x213`); Pomo `0x005E7600/0x005E7670` vt `0x0065F7FC` (`0x214`); Pofa `0x005E70C0/0x005E7130` vt `0x0065F794` (`0x215`); Yujia `0x005E6BD0/0x005E6C40` vt `0x0065F72C` (`0x216`); Yubing `0x005E66E0/0x005E6750` vt `0x0065F6C4` (`0x217`); Yumo `0x005E6220/0x005E6290` vt `0x0065F65C` (`0x218`); Yufa `0x005E5D60/0x005E5DD0` vt `0x0065F5F4` (`0x219`). Общие: Serialize `0x005E7330`, Unserialize `0x005FD660`, GetRemainedTime `0x00605E10`, AI `0x005E6E20`, End `0x005DBCE0`. Отображение вида (Po снижает цель, Yu усиливает держателя) подтверждено различием вызовов в Begin восьми классов; формулы — PARTIAL из существующего Rust |
| `heal` (×4) | CHealState `0x005F8940/0x005F89D0` vt `0x00660D9C` (`0xD3`); CHeal2State `0x005EFAC0/0x005EFB50` vt `0x0066024C` (`0xE3`); CSuperHealState `0x005F63B0/0x005F6440` vt `0x00660A64` (`0xD9`); CSuperHeal2State `0x005EE960/0x005EE9F0` vt `0x00660134` (`0xE4`). Общие: AI `0x005EEDF0`, End `0x005EEBA0`, GetRemainedTime `0x005F2CD0`, Serialize `0x005F65F0`, Unserialize `0x005EEC70` |
| `cure` | ctor `0x005E9E50`, Serialize `0x005F51E0`, Unserialize `0x005EAAC0`, AI `0x005D5BA0` |
| `daubpoison` | ctor `0x005F17B0` (ID `0xDF`), vt `0x0066047C`; общие с CCureState: Serialize `0x005F51E0`, Unserialize `0x005EAAC0`, GetRemainedTime `0x005F2CD0`, AI `0x005D5BA0`, End `0x005FD420` |
| `promotion` (`0x142`) | ctor с параметрами `0x005F2BC0`, ctor по умолчанию `0x005F2C40` (оба коэффициента = 1); vt `0x006605F4`: AI `0x005D5BA0`, End `0x005FD420`, GetRemainedTime `0x005F2CD0`; Restart `0x005FD450` (только часы); Serialize `0x005F2E90`, Unserialize `0x005F2FB0` |
| `bossbluefury` (`0x1F7`) | ctor `0x005E8A60` (коэффициент атаки `+0x38`, срок `+0x3C`, слабая фаза `+0x40`); vt `0x0065F994`: AI `0x005E8D50`, End `0x005E8D10`, Restart `0x005FD450`, OnUpdateProperties `0x005E8DC0`, GetRemainedTime `0x005D5F30`, Serialize `0x005E7330`, Unserialize `0x005D6190` (weak_time не читается) |
| `tianshenxiafan` (`0x335`) | ctor `0x00605780`; vt `0x0066202C`: Begin `0x00605B70`, AI `0x005D5BA0`, End `0x006059A0`, OnUpdateProperties `0x00605A10`, GetRemainedTime `0x00601200`, Serialize `0x006059C0`, Unserialize `0x00605C20`; исходный дефект асимметрии 10/12 байт сохранён |
| `changebody` (`0x37`) | ctor `0x005DAC90`; vt `0x0065E41C`: AI `0x005DAAA0`, End `0x005DA240`, OnUpdateProperties `0x005DA0F0`, GetRemainedTime `0x005DA030`, Serialize `0x005DA080`, Unserialize `0x005DA0C0` |
| `undead` (`0x38`) | ctor `0x005D62A0` (72 байта без часов); GetRemainedTime `0x005D6320`; Serialize `0x005D64F0`; Unserialize `0x005D6530`; общий AI `0x005D7C80` |
| `extended` (`0x32`/`0x33`) | ctor `0x005D9230`/`0x005D9990`; Serialize `0x005D9510`/`0x005D9BB0`; Unserialize `0x005D9550`/`0x005D9BF0`; AI `0x005D94E0`/`0x005D9FB0`; getter Original общий с CHBYState, New — `0x005D6320` |
| `consumablerestore` (`100000`/`100001`) | ctor `0x004F8410`/`0x004F87B0`, vt `0x0065355C`/`0x006535BC`; общие с Heal-семейством: Serialize `0x005F65F0`, Unserialize `0x005EEC70`, GetRemainedTime `0x005F2CD0`, additional `0x00601200`; AI `0x004F8650`/`0x004F8AA0` |
| `agility` | CAgility/CNatural/CRapture: ctor `0x005F4195`/`0x005F3795`/`0x005F3C85`, vt `0x00660774`/`0x006606B4`/`0x00660714`, общий writer `0x005F3E40`, reader `0x005F4420`; временная CAgilityState2: writer `0x005F1050`, reader `0x005F48E0`, AI `0x005D60B0` |
| `callosity` (`0x75`/`0x7D`) | ctor `0x005F4651`/`0x005F0E91`, vt `0x006607D4`/`0x006603BC`; общие: writer `0x005F1050`, reader `0x005F48E0`, AI `0x005D60B0`, property callback `0x005F10B0` |
| `hearten` | ctor `0x005EE500`, vt `0x006600D4`, writer `0x005D4D10`, reader `0x004F9D80`, property callback `0x005EE740` |
| `ride` (`100004`) | Serialize `0x004F8F60`, Unserialize `0x004F93B0`, AI `0x004F9110` |
| `teamstate` (`0x186A6`) | ctor `0x005BFE60`, Serialize `0x005BFA50`, Unserialize `0x005BFF20`, GetAdditionalData `0x005BFDD0`, AI `0x005BFD20` |
| `particular` (`0x186A5`) | ctor `0x004F9440`, vt `0x00653684`; Serialize `0x005E23D0`, Unserialize `0x00601350`, AI `0x004F9900` |
| `scriptstate` (5×UseGoods, ImproveExp, AutoProtect) | AutoProtect vt `0x0065E00C` (End `0x005D44E0`, Serialize `0x005F51E0`); пять UseGoods (End `0x005D5B80` с ended-флагом, Serialize `0x005D4D10`, getter `0x005F2CD0`); ImproveExp (getter `0x005D5F30`, Serialize `0x005E7330`, AI `0x005D60B0`); readers `0x004F9D80`/`0x005D6190`/`0x005EAAC0` |
| `attackgain` (Fury/RageBreak) | общий Serialize `0x005E7330`, OnUpdateProperties `0x005FD480`, Unserialize `0x005FD660`, AI `0x005EA4C0` |
| `roar` | ctor `0x005EC7E0`, Serialize `0x005F65F0`, Unserialize `0x005ECC60`, AI `0x005EC9A0`, OnUpdateProperties `0x005ECAA0` |
| `pillar` | ctor `0x005F4A60`, Serialize `0x005E7330`, Unserialize `0x005D6190`, AI `0x005D60B0` |
| `godbless` (×2) | Serialize `0x005EE310`, Unserialize `0x00601830`, GetRemainedTime `0x00601480`, AI `0x00601640`, OnUpdateProperties `0x00601690` |
| `lifeshield` (`0x220`) | Serialize `0x005E2C60`, Unserialize `0x005E2E30`, AI `0x005E2D90`; ветвь `CFightDefense::PreDefense` `0x005B0E64–0x005B1030`, якорь MP-произведения `0x5B0F55–0x5B0F83`. История: прежняя реконструкция округляла MP-фактор до f32 до умножения — установленное расхождение, исправлено (неокруглённый x87-продукт `mp*0.01`; f32-копия `fst [esp+0x1C]` создаётся только для деления mana-ветви) |
| `machineshield` (`0xDE`) | Serialize `0x005F1EE0`, Unserialize `0x005F2000`, AI `0x005F34B0`; ветвь PreDefense `0x005B0CD8–0x005B0E5F`, якорь `0x5B0D83–0x5B0DB3` |
| `manashield` (`0x141`) | Serialize `0x005F3380`, Unserialize `0x005F3520`, AI `0x005F34B0`; ветвь PreDefense `0x005B0ABC–0x005B0C72`, якорь `0x5B0B8A–0x5B0BB8` (f32-копия `fst [esp+0x18]`) |
| `shieldabsorption` | общая числовая часть мана-/машинного щитов тех же ветвей PreDefense (`0x005B0ABC–0x005B0C72`, `0x005B0CD8–0x005B0E5F`); та же установленная поправка округления, что у lifeshield |
| `defenseshield` | диспетчер PreDefense `0x005B0A50–0x005B0A72` (`skill_id < 530 \|\| > 545 \|\| == 544` проходит к обходу состояний); стихийная ветвь Promotion `0x005B0C80` проверяет ID `0x142` и `kind == 3` |
| `energyholding` | ctor `0x005EC410`/`0x005EC450`, AddEnergy `0x005EC490`; общие с SoulCollect Serialize `0x005E1D50`, Unserialize `0x005E1E80` |
| `soulcollect` | ctor `0x005E1AD0`, AddSoul `0x005E1BC0`, Serialize `0x005E1D50`, Unserialize `0x005E1E80`, GetRemainedTime `0x00601200` (ноль), GetAdditionalData `0x004D7090` |
| `swordship` (×4) | vt `0x00660D4C`/`0x006602AC`/`0x0065FEB4`/`0x0065FE64`; property callback `0x005F8870`, writer `0x005ECE70`, reader `0x005F0010` |
| `element` (`0x12D`/`0x130`) | ctor `0x00600F70`/`0x006011A0`, vt `0x006617C4`/`0x00661814`; общий writer `0x005E23D0`, reader `0x00601350` |
| `fullmiss` (`0x25B`) | ctor `0x005E2090`, vt `0x0065F044`, property callback `0x005E2120`, writer `0x005E23D0`, reader `0x00601350` |
| `maxresource` (`0x259`/`0x25A`) | ctor `0x005E22E0`/`0x005E2160`; обе vtable ведут к writer `0x005E23D0` и reader `0x00601350` |
| `meteorarrow` | ctor `0x005F6820`, vt `0x00660AC4`, writer `0x005ECE70`, reader `0x005F0010`, добавление `0x005F6970` |
| `poison` (×4) | vt `0x0065F24C`/`0x0065F9F4`/`0x006620F4`/`0x0065FCE4`; общий writer `0x005E93C0`, getter `0x00606320`; первые три читаются через `0x005E3500`, Kerosene — через `0x005EB800` (берёт время после полей записи) |
| `poisonfog` (`0xC9`) | ctor `0x00607C40`, vt `0x006622D4`, writer `0x00607F50`, reader `0x006084C0`, property callback `0x00608060` |
| `periodicattack` | PoisonArrow: Serialize `0x005E93C0`, Unserialize `0x005E3500`, AI `0x005E3730`; Kerosene: Unserialize `0x005EB800`; общие с семьями: BloodLoss `0x005E3B40`/`0x005E3C70`, LeafCut Serialize `0x005F0820`, LeafCut2 Unserialize `0x005EBF20` |
| `bloodloss` | ctor `0x005E3820`, vt `0x0065F2AC`, writer `0x005E3B40`, reader `0x005E3C70`; хвост записи — два DWORD с битами f32 и два WORD |
| `leafcut` (`0x6B`/`0x80`/`0x8F`) | ctor `0x005FC580`/`0x005F05C0`/`0x005EBC00`, vt `0x006611F4`/`0x0066035C`/`0x0065FD44`; общие writer `0x005F0820`, reader `0x005EBF20`, getter `0x00606320`; хвост записи — 2 DWORD и 4 WORD |
| `wuxing` (×5) | пять vtable направляют writer на `0x005E0030`, reader на `0x005E0880` (ID DWORD и `0x5C` сырых байт с `[this+0x38]`); property callbacks: Metal `0x005E08C0`, остальные `0x005E0080` |
| `weak` (`0x12E`) | vt `0x006621B4`: AI `0x00606EF0`, property callback `0x00607020`, writer `0x00606FC0`, reader `0x006071F0`, getter срока `0x00605E10` |
| `wangsheng` (`0x221`) | ctor `0x00605D90`; общий Serialize `0x005E7330`, Unserialize `0x005FD660`, AI `0x005E6E20`, OnUpdateProperties `0x00605FB0` |
| `automaticrestore` (×4) | AI HP peace `0x004FA8E0` (смерть/полнота HP/мирный режим до часов); все четыре класса вызывают общие тела Serialize `0x005ECE70` / Unserialize `0x004F9D80`; PDB называет общие тела по другим состояниям — принадлежность определяется vtable, не именем символа |
| `visualeffect` | проверенный диапазон базового `CVisualEffect` `0x005DC1B0..0x005DC234` |
| `time` | образец таймед-остатка `CHealState::GetRemainedTime` `0x005F2CD0` (при истёкшем сроке второе чтение `timeGetTime` не выполняется) |

## zonalcast: скелет областных призывов

Скелет Begin/Check/AI/visual/End для Weak (`0x12E`), PoisonFog (`0xC9`),
SnowStorm (`0x193`), YinYang/YinYang2 (`0x139`/`0x146`),
GodThunder/GodThunder2 (`0x140`/`0x143`), FireWall (`0x134`),
ChaosSphere (`0x137`), SoulMirror (`0x13C`). Исходные владельцы PDB:
`appserver/skills/{weak,poisonfog,snowstorm,yinyang,yinyang2,godthunder,godthunder2,firewall,chaossphere,soulmirror}.cpp`.

| Функция/symbol | pub/RVA | Статус | Факт |
|---|---|---|---|
| `CWeak::CheckCastCondition` | RVA `0x1AF050` | VERIFIED | свежая таблица `QuerySkillBaseProperties` → reuse `10005` + `timeGetTime` → visual13 + `GS0278` → длина пути `5003` (`jbe` при нуле) → visual11 + `GS0290` → MP-контракт (MP0 — тихий отказ, иначе signed-разность) → visual7 + `GS0288` с ценой → `SetMoveable(0)`; блок-проверки пути нет (`CastPathBlock::Ignore`) |
| `CWeak::AI` | RVA `0x1AED40` | VERIFIED | активный гейт фазы, одна таблица на тело, GetUser/GetSufferer, смерть S → visual10 + `GS0285` + `End(0)`; два чтения X/Y S (регистры и сохранённые поля) с очисткой identity S; NULL U → `End(0)`; Begin-доля: MP → `OnChangeStates` → CAN `10006` → `GetLineDir` → `SetDir` → visual0 → фаза 1 → проход в тот же тик; unsigned `start + delay(10001)` → visual1 → `Summon` (вирт. `+0x8C`) → `End(1)` независимо от результата |
| `CPoisonFog::CheckCastCondition` | RVA `0x193D50` | VERIFIED | S → точка с очисткой identity до таблицы и reuse; арбалет — `GetAddonProperty` категории 4, отказ → visual14 + `GS0293` |
| `CFireWall::CheckCastCondition` | RVA `0x1AB8B0` | VERIFIED | сканирует путь и запрещает blocker-клетки 1 и 2 (`GroundAndFly`), visual15 + `GS0282` |
| `CChaosSphere::CheckCastCondition` | RVA `0x1A6DC0` | VERIFIED | чтения пути нет вовсе |
| `CChaosSphere::AI` | RVA `0x1A7FA0` | VERIFIED | после гейта смерти читает X/Y S один раз (без повторного Weak) и чистит identity S |
| `CSnowStorm::CheckCastCondition` / `AI` | RVA `0x183E20` / `0x183C40` | VERIFIED | ошибки reuse/пути/MP — только visual-режимы без GS-форматирования; недостаток MP и в Check, и в AI → visual7 без `GS0288` |
| десять тел `*Effect::UpdateVisualEffect` | таблицы переходов RVA: CWeak `0x1AE8C0`, CPoisonFog `0x1935E0`, CSnowStorm `0x1837C0`, CYinYang `0x1A5730`, CYinYang2 `0x1677A0`, CGodThunder `0x172D00`, CGodThunder2 `0x152F40`, CFireWall `0x1AB140`, CChaosSphere `0x1A7940`, CSoulMirror `0x1A4400` | VERIFIED | wire-кадр `0x000BFE01`: switch по 16 режимам с таблицей переходов; личная ветка отказов кадром `[u8=0, u8=mode]` только игроку; mode 0 → кадр `action=1` (навык, уровень, источник, direction); mode 1 → кадр `action=2` с нулевой парой цели и X/Y. Наборы режимов: девять владельцев `0/1/2/7/10/11/13/15`, у PoisonFog добавлен `14` |
| идентичность областей и Summon (опорные VA) | `CGodThunderPhalanx`/`CGodThunderPhalanx2`: ctor `0x005F5B50`/`0x005EF190`, Initialize `0x005F59C0`/`0x005EEF30`, общий AddToByteArray `0x005EF0C0`, AI `0x005F6150`/`0x005EF6B0`, Summon `0x00573840`/`0x00553A80`; server decode `0x005F5D90` (UNKNOWN: подтверждённого вызывающего пути нет) — `zone::skills::godthunder`. `CWeakPhalanx`: ctor `0x600730`, обход `0x600840`, AI `0x600a70`, расчёт срока Summon `0x5AF41B–0x5AF48C` — `zone::skills::weak`. `CSnowStorm`: Summon `0x00584060`, ctor `0x005F9040`, Initialize `0x005F8D70`, EncodeToByteArray `0x005F8ED0`, CalculateAttackPower `0x005F92D0`, Attack `0x005F93B0`, AI `0x005F94B0` — `zone::skills::snowstorm`. `CChaosSpherePhalanx`: Summon `0x005A8290`, ctor `0x005FEE80`, AddToByteArray `0x005FECB0`, AI `0x005FF270` — `zone::skills::chaossphere`. `CYinYang*`: Summon `0x005A6270`/`0x005682E0`, ctor `0x005FE4F0`/`0x005F2410` — `zone::skills::yinyang`. Неподвижные области: ReplaceAffectRegion `0x005FFCD0`/`0x005FE270`/`0x005F2190`, `CScope::SetInScope` `0x005E9990`, AI FireWall `0x006003E0`, YinYang `0x005FE9A0`, YinYang2 `0x005F28C0` — `zone::skills::masked_area` | MATCH (по шапке владельца) | адреса в форме VA (истинный RVA = VA − `0x400000`); тела Summon сверены у владельцев зонального скелета |
| элементальный удар областей (опорные VA) | `CalculateAttackPower`: firewallphalanx `0x00600120`, yinyangphalanx `0x005FE6E0`, yinyangphalanx2 `0x005F2600`, godthunderphalanx `0x005F5E90`, godthunderphalanx2 `0x005EF3F0`, chaosspherephalanx `0x005FEFB0`; `CFireWallPhalanx` Summon `0x005ABC70`, ctor области `0x005FFF30`; `CPoisonFogPhalanx` ctor `0x005FBD90`, AI `0x005FC040` | MATCH (по шапке владельца) | реализация `zone::skills::{elementphalanx, firewall, poisonfog}` |

Источники X/Y mode 1: живой S с откатом к сохранённой точке (проверено для
CWeak/CFireWall/CSnowStorm/CGodThunder/CYinYang/CYinYang2/CChaosSphere; у
CGodThunder2 тот же шаблон компилятора, без отдельного досмотра); PoisonFog
всегда пишет сохранённую точку; SoulMirror — текущий центр U.

Досверка verify-t5 (дизассемблинг точной пары, `.local/verify-t5/`): тела
`CheckCastCondition`/`AI` YinYang/YinYang2 (`0x1A5EA0`/`0x167F10`,
`0x1A5BB0`/`0x167C20`) и GodThunder/GodThunder2 (`0x173470`/`0x1536B0`,
`0x173180`/`0x1533C0`) — VERIFIED-MATCH общему скелету семьи (YinYang2 —
271/228 инструкций ≡ эталону YinYang, остальные клоны).
`CSoulMirror::CheckCastCondition` (`0x1A4850`, арность `…PAVCMoveShape@@0@Z`)
— VERIFIED: MP-контракт по arg1 (U), цель `SetMoveable` — arg2; для
практических self-кастов они совпадают (S≡U — остающаяся оговорка). Не
закрыто: машинный разбор `CSoulMirror::AI` (`0x1A4D10`, дамп готов) и
клиентское чтение кадров (UNKNOWN).

Швы переноса: трейты модуля — переходные фасады прежнего владельца
`CGame`/`CPlayer`/`CMoveShape`; потребляются статически (generic),
dyn-совместимость и `Send`-контракт не вводятся (ADR-0013). `CFireWall::Summon`
остаётся у прежнего владельца (`appserver/skills/firewall.rs` старого
пакета, шов `summon_fire_wall`). Часы прежнего main loop приходят указателем
`now_milliseconds` (делегат перечитывает `runtime.now_milliseconds()`).

Текущая реализация: `zone::skills::zonalcast`; тела Summon и клиентский
снимок — у владельцев `zone::skills::{weak,poisonfog,snowstorm,godthunder,masked_area,chaossphere,soulmirror}`
и `zone::skills::summonshape`; hub-делегат — `src/gameserver/appserver/skills/zonalcast.rs`.

## battlefairygear: экипировка, потенциал и улучшение боевой феи (ядро CPlayer)

| Функция/symbol | pub/RVA | Статус | Факт |
|---|---|---|---|
| `AllocatePotential` | RVA `0xFF480` | VERIFIED_DISASSEMBLY | jump-table VA `0x50000C` по ключам `0x9B..0xA1`; ATTACK/SPRITE — f64 `1.5` (`0x653008`); добавки `0xB9`/`0xBA` (MAX_HP/MAX_MP) у ветвей `0xA1`/`0xA0`; четыре player-ветви ×(−1e-5, `0x653C68`)+`sub` ≡ `+0.00001`; per-call `PropertiesChanged` (virtual `+0x9C`) и хвост `0xBF918`; отключённая feature (байт `0xEF46C4`) — текст «8» (`ZHGS0008`) внутри каждой записи |
| обработчик `0x8FC2A` | RVA `0x95F00..0x962C0` | VERIFIED_DISASSEMBLY | `imul eax, eax, 0x2710` — клиентские очки масштабируются ×10000 до map-insert (first-wins), агрегат немасштабирован (signed `jg` по потенциалу `0xA3`); итерация std::map вызывает `AllocatePotential` по записи; внешний `0xBF918` после цикла, при insufficient — пропуск |
| `BFPropertyAdd` | RVA `0x1020E0` | VERIFIED_DISASSEMBLY | ранний reject только ячейки `0xC`; десять addon-чтений (`0xCA/0xCB/0xCF/0xCC/0xCD/0xCE/0xD0/0xE3/0xC8/0xC9`) × delta; записи `0x9B/0x9C/0xA1/0x9E/0x9F/0xA0/0x9D/0xDA`, `0xB9` (+`0xCF`+`0xC8`), `0xBA` (+`0xCE`+`0xC9`); клампы current 0x99/0x9A по maximum 0xB9/0xBA; player-ветви с occupation-таблицами BSS (GlobeSetup); double-apply quirk подтверждён машинно: второй раунд SetMaxHp/SetStrength/SetIntelligence/SetDexterity (`0x42ACF0/0x42AD30/0x42AD90/0x42AD50`) после производных коэффициентов (`0x1029A7–0x102AEA`); хвост `PropertiesChanged`+`0xBF918` |
| reset reconcile tail | RVA `0xFF031` | VERIFIED_DISASSEMBLY | семь пар tracked→0 / property−=tracked в порядке `0xBB..0xC1`; recovered ((sprite+attack)·2/3 + blast+brave+agility+spiritualism+strength) — f64 `2/3` (`0x653C40`); потенциал `0xA3` += trunc; четыре вычитания из свойств игрока ×(+1e-5, `0x653C38`) ≡ `−0.00001` |
| upgrade player-tail (аудит `0x60202/0x60203`) | RVA `0x100530` | PARTIAL | якоря кадров `0x60203` событий 1/2 (сериализация target+4 gem: name/price/amount, затем money-поля игрока) и их event-гейты подтверждены спотчеком; полный построчный вывер тела улучшения не переоткрывался — опора на разведку BF-семьи |
| container queries | RVA `0xFC9A0/0xFCAB0/0xFCBF0` | MATCH разведки BF-семьи | `GetFailResult` ≡ filter(!=0).fold(4,min), `GetProbability` порядок 13,14,15,16,12 signed→clamp 0..=100, `GetUpgradePrice` запись поля только при наличии предмета ячейки 12 |

Константная спот-сверка: `1.5` — f64 `0x653008` (ветви ATTACK/SPRITE
`AllocatePotential`); `0.00001` — как отрицательная константа `−1e-5` с `sub`
(`0x653C68`) и положительная `1e-5` с `sub` у reset-хвоста (`0x653C38`):
обе ≡ `f64::from(amount) · 0.00001` на достижимых входах. Масштаб ×10000 —
`imul 0x2710` обработчика `0x8FC2A`.

Оговорки допуска и усечения: (1) машинный guard `BFPropertyAdd` отклоняет
ровно ячейку `12`, zone-предикат сужает допуск до восьми gear-позиций `0..=7`
— тождественно на достижимых caller-ах: container `validate_add_at` порождает
`property_effect` только для gear; (2) машина усекает сумму
`current + amount·1.5` одним FISTP, zone — прибавку до сложения: тождественно
на домене opcode-пути (amount = очки×10000 чётное и неотрицательное за гейтом
потенциала); отдельные caller-ы native `AllocatePotential` с иными amount не
исследованы; (3) ulp-порядок FP-множителей `(amount×coeff)×k` против
`(amount×k)×coeff` в player-формулах не переустанавливался — значения
сходятся на целых прибавках домена; (4) вывер upgrade-тела опирается на
разведку BF-семьи, машинные спотчеки покрывают только аудит-кадры `0x60203`.

Текущая реализация: `zone::skills::battlefairygear`; hub-адаптер — `BattleFairyGearPlayerAdapter` в `src/gameserver/appserver/player.rs`.

## bossbluefury: `CBossBlueFury` (0x1F7)

Разведка `.local/recon-de/notes/E4-bossbluefury.md`, тела
`.local/recon-de/disasm/CBossBlueFury.txt`.

| Функция/symbol | pub/RVA (VA) | Статус | Факт |
|---|---|---|---|
| vtable класса / `CBossBlueFuryEffect` | VA `0x6578AC` / `0x657940` | VERIFIED_DISASSEMBLY | `UpdateVisualEffect` `0x52E560` — отдельный адрес, не ICF с `0x59FB90` RageBreak |
| Begin (триада) | VA `0x52D4A0`/`0x52D2E0`/`0x52D3B0` | VERIFIED_DISASSEMBLY | базовый Begin, `new` loop=1 visual, CheckCast `vcall+0x64`; отказ — полный End(0), успех — `[+0x4C]=1`, `[+0x50]=0` |
| End (общий ICF-хвост) | VA `0x546090` | VERIFIED_DISASSEMBLY | `[+0x50]/[+0x4C]` в 0, свежему U движение `0x4CCEE0(1)`, аргумент базовому End; тот же хвост у Fury/BossBlueQuake |
| `Check` | VA `0x52E8E0` | VERIFIED_DISASSEMBLY | `QuerySkillBaseProperties` обязательна (null → ret 0); абсолютный срок reuse `0x2715` + `[+0x40]` против `timeGetTime` — отказ visual(13)+GS0278 только игроку; дальше только RTTI-CPlayer: стоимость RP `Query(3) == 0` — тихий ret 0; movzx RP `[U+0x288]` − `Query(3)` signed < 0 → visual(8)+GS0289 с ценой; успех — SetMoveable(0). Монстр проходит без RP-блока и без SetMoveable(0) |
| `AI` | VA `0x52EAC0` | VERIFIED_DISASSEMBLY | `[+0x4C]==0` — чистый возврат; таблица и U обязаны (иначе End(0)); `IsDied(U)` → visual(2) → End(1); первый проход `[+0x50]==0` списывает RP до дефицита (необратимо; дефицит — visual(8)+GS0289+End(0)), затем `vcall+0x164` = OnChangeStates, CAN `0x2716` → `[+0x3C]`, visual(0), `[+0x50]=1`; задержка `0x2711` — абсолютная `start+delay` unsigned, затем visual(1) |
| продув состояний 0x1F7 | VA `0x52EC10`–`0x52EC72` | VERIFIED_DISASSEMBLY | End `vcall+0x1C` → deleting-dtor(1) свежего остатка позиции → слот в 0; скан до конца вектора без досрочного выхода, позиция всегда +1, длина перечитывается, пропуски не уплотняются — завершает и уничтожает КАЖДЫЙ живой слот id `0x1F7` |
| `CBossBlueFuryState::ctor` | VA `0x5E8A60` | VERIFIED_DISASSEMBLY | `new(0x44)`; factor = `Query(20003)`, keep = `Query(10002)`, weak = `Query(10003)`; вычисление запросов 10003 → 10002 → 20003 по push-порядку дампа; затем Begin(U,U) `vcall+8` → append в хвост | dtor |
| UpdateProperty на U | call site VA `0x52EDAC` | VERIFIED_DISASSEMBLY | `vcall+0x9C` после Begin, затем End(1) |

Разграничение атрибуции: запись «`new(0x44)` CRageBreakState, gain =
`Query(105)`» относится к соседнему `CRageBreak::AI` (VA `0x5A0348`:
`new(0x44)` → ctor `0x5FD1C0`, ключи `Query(0x69)`/`Query(0x2712)`) — у
CBossBlueFury свой ctor и свои ключи.

Порядок состояний в AI: полный продув каждого слота U с id `0x1F7` →
`new CBossBlueFuryState` → Begin(U,U) → append/dtor → UpdateProperty →
End(1). Узкая достижимость свипа: игровой путь держит не более одной записи
`0x1F7` (каждый cast чистит); расхождение с завершением только первого ключа
видимо при дублированных записях БД id `0x1F7`, загруженных Restart-ом без
продува. Свип общий для обеих ветвей (`sweep_boss_blue_fury_states`),
`PublishStateCastGame`-шов совпадает с конфликт-свипом семьи Fury.

Wire `0x000BFE01`: кадры mode 0/1 — action 1/2 around с U (target = сам U,
тайлы живого источника); отказы 13/8/2 — только игроку BYTE-парой
`[byte 0][byte mode]` (форма отдельного тела `0x52E560`, не DWORD-префикс
RageBreak). После CheckCast-отказов Begin — терминальный `[0, 2]`; GS0278 и
GS0289 — без/с суммой стоимости. Отдельное расхождение прежнего тела:
`vcall+0x164` оно отображало на `update_player_current_state(MoveShapeAi)`;
тот же vcall CFury/CDaubPoison/CRageBreak hub-сверен как OnChangeStates —
вызов приведён к `publish_player_states`.

Текущая реализация: `zone::skills::bossbluefury` (+ `bossbluefurystate`);
данные/кодек состояния — `zone::effects::bossbluefury`; hub-швы `statecast`+`fury`, монстр-вход — hub `monsterattack` (старый пакет).

## skillfactory: фабричный каталог и реестр runtime-свойств

| Функция/symbol | pub/RVA (VA) | Статус | Факт |
|---|---|---|---|
| `Rebuild` | RVA `0x0006CE10` | VERIFIED_DISASSEMBLY | точный цикл и malformed-record skip: очистка реестра до чтения `count`, знаковое количество, пропуск нулевой длины, ключ `skill_id << 16 \| level & 0xffff`, повторный ключ заменяется; имя — байтовый префикс C-string, карта usage — последнее значение ключа. Парная сторона wire-контракта — WorldServer serializer |
| `QuerySkill` | VA `0x00469870` | VERIFIED_DISASSEMBLY | все 209 соответствий ID → конкретный класс по переходам и вызовам конструкторов, включая специальные ID и `CNonFun*` |
| базовые конструкторы категорий | VA `0x005DFA80` (CAttackSkill → 0), `0x006019C0` (CDefenseSkill → 1), `0x005DFB50` (CStateSkill → 2), `0x005E0EC0` (CSummonSkill → 3) | VERIFIED_DISASSEMBLY | категория экземпляра — запись `[this+0x48]` базовым конструктором; временные стековые объекты другой категории в concrete-ctor её не меняют (напр. CArchery — Summon, CSeal и CThunderBlow2 — Attack) |
| `QuerySkillType` | VA `0x0046C3D0` | VERIFIED_DISASSEMBLY | admission не проверяет: ID без concrete factory-owner возвращает категорию имеющейся записи |
| End-политики | slot `+0x68` vtable всех 209 владельцев | VERIFIED_DISASSEMBLY | 37 общих тел End(int), без подстановки категории runtime-свойств |
| клиентский minimum range | VA `0x005387A0` (ChuckStone/SkeletonArchery) | VERIFIED_DISASSEMBLY | читают usage5004, signed-положительный результат, иначе 1; обычно 1 |
| maximum range / MP cost getters | VA `0x004D81C0` / `0x004D81F0` | VERIFIED_DISASSEMBLY | свежие properties usage5003/2; отсутствующая запись → 1/0 |
| AfterUse | VA `0x0053CF30` | VERIFIED_DISASSEMBLY | износ оружия разрешённого CPlayer (Attack/State `+0x8C`, Summon `+0x90`, только при ненулевом End перед общим сбросом); пять пустых overrides — `0x00601A70`; Defense AfterUse не имеет |
| `CItemSkill_2` AfterUse | VA `0x005149E0` | VERIFIED_DISASSEMBLY | вместо износа записывает время item-группы из текущих properties ID/level (`usage 0xC351`) |

37 общих тел End(int) (VA): общий хвост — `50E9B0`, `5A30D0`, `5AC2D0`,
`5AFA40`, `5B3010`, `5DFBD0`, `601A40` (чтение GetUser в двух из них не
меняет движение). Возврат движения источнику: `52B410`, `53BF50`, `540890`,
`546090`, `54D760`, `55B2B0`, `56A330`, `577C40`, `57B810`, `582810`,
`58C3A0`, `58DE90`, `58F5E0`, `591AD0`, `598EE0`, `59A410`, `59D6B0`,
`5AE7A0`. `570B20` освобождает пути после возврата движения, остальные
path-owner-ы — до него. `5502F0` при null GetUser пробует GetSufferer.
`512B50` и `5970C0` также ставят available=true; `544830` — false. `5888D0`
ставит available=true и обновляет visual действием 3 только при End(0).
Особые visual-хвосты: `516FB0`, `51A700`, `51E370`, `5222A0`, `5355F0`,
`5A0790` (условия — в `SkillEndEffect`). End не проверяет ended; null source
не заменяется держателем реестра.

`OnChangeRegion` (`+0x2C`) всех concrete CSkill-derived vtable регионы базы
не присваивает: большинство — ret4 `0x00601A70`; overrides `0x0056A370` /
`0x0052AAC0` вызывают End(0)/End(1), а `0x0053CF10` проверяет IsEnded и
вызывает End(0) только при false.

Прочие установленные факты каталога: SpiderMist возвращает движение перед
`CSummonSkill::End`, его Begin не добавляет cast в `m_vStates`, End не
удаляет самостоятельную phalanx. У Po/Yu и transfer-owner-ов End(bool) в
`+0x94` не подменяет End(int) в `+0x68`. Удерживаемый HeartLessArrow при
ненулевом End имеет отдельный ранний release-переход. Каталог не выполняет
End сам: concrete поля/ресурсы, visual и общая source-aware граница —
обязанность runtime-владельцев; неизвестные concrete layout и полный End
не восстановленных skill-owner-ов каталогом не закрыты.

Текущая реализация: `zone::skills::skillfactory`; исходный владелец PDB
`server/gameserver/appserver/skills/skillfactory.cpp`.

## corpseptomaine: `CCorpsePtomaine` (0x19F)

Разведка `.local/recon-de/notes/D3-corpseptomaine.md`, тела
`.local/recon-de/disasm/CCorpsePtomaine.txt`.

| Функция/symbol | pub/RVA (VA) | Статус | Факт |
|---|---|---|---|
| vtable / End | VA `0x257F5C` / `0x546090` | VERIFIED_DISASSEMBLY | End — ICF-фолд общего End (`CAgility::End`: нули `[+0x50]/[+0x4C]`, GetUser → SetMoveable(1) → `CAttackSkill::End`) |
| `Check` | VA `0x539A00` | VERIFIED_DISASSEMBLY | null S/props → 0; reuse 10005 → visual(13); только U.type==400: RTTI player, `Query(2) == 0` → тихий ret 0 (`jbe`), `MP − loss < 0` → visual(7); non-400 → ret 1 без SetMoveable(0) (`0x539A71`→`0x539AB9`) |
| Begin-скелет | VA `0x539840`/`0x539910`/`0x539AE0` | MATCH | скелет семьи |
| `AI` | VA `0x53A230` | VERIFIED_DISASSEMBLY | `[+0x4C]==0` → out; props null → End(0); U null → End(0); Begin-фаза только player: `MP([U+0x284]) − Query(2)` со знаковым `js` → visual(7) → End(0), иначе SetMP(MP−loss) и `vcall CPlayer+0x164`; `Query(10006)` → `[+0x3C]`; visual(0); delay `Query(10001)+[+0x2C]` → out; visual(1); регион — RTTI `[U+0x40]` → null → End(1) (`0x53A4E8`); нормальное завершение также End(1) |
| обход области | (в теле AI) | VERIFIED_DISASSEMBLY | `g_bScope` — девять «1» (полный 3×3, центр включён), length = height = 3, X-внешний/Y-внутренний, `x + 3·y`, центр `GetTileX/Y(U) − 1`; клетка → RTTI CShape→CMoveShape → `IsDied` → `IsAttackAble(U)` `vcall+0x134` → `DoesStateExist(0x131)` → AddState; god-фильтра нет |
| `add_corpse_poison_state` | VA `0x539FF0` | VERIFIED_DISASSEMBLY | MasterInfo local — type/id U; player: `+0xB20` → mi+0xC, `+0xB28` → mi+0x8, `+0xB78` → mi+0x10, байты `+0x278..+0x27B` → mi+0x18..+0x1B, country остаётся 0; ctor `CSpiderPoisonState` (`0x5E90C0`) с порядком Query const(20010) → freq(6001) → keep(10002) до замены; скан первого `0x191` в `S+0x11C` → End `vcall+0x1C` → deleting-dtor `vcall+0x10(1)` → slot 0 → Begin(U,S) `vcall+0x08` → тот же слот, иначе append; провал Begin — deleting-dtor нового |

Предикат целей player-пути (VERIFIED_DISASSEMBLY по телу AI `0x53A230`):
нативный scan принимает любой живой RTTI-CMoveShape по виртуальным
`IsDied`/`IsAttackAble` (+Cure-гейт) БЕЗ allowlist типов — allowlist
`{400, 500, 600, 1100, 1200}` прежней реконструкции при переносе снят.
Достижимость: домен скана задают живые разрешители owner-а —
`base_magic_target_dead` и региональный `IsAttackAble` знают ровно 400/600
(и 1100/1200 через build-гейт); тип 500 (CNpc) машинно мёртв по
`CMoveShape::IsDied` (нулевой combat HP); тип 1000 призванных фаланг вне
домена `resolve_state_move_shape`. Для гипотетических MoveShape-типов вне
пятёрки нативный scan дал бы решение виртуальным вратам, а разрешители Rust
отвечают «мёртв/неатакуем» — зафиксировано как неснимаемый остаток модели
арены; машинная форма scan-ветки сохранена. Monster-путь фильтра не имел:
кандидаты — общим hub-хелпером `monsterattack`.

Player и monster ветви используют абсолютный срок `CSkill::IsRestored`,
задержку — отдельной elapsed-проверкой, MP-списание — wrapping-sub фазы
Begin.

Текущая реализация: `zone::skills::corpseptomaine`; hub `monsterattack` +
общая арена `zone::skills::spiderpoison` (`SpiderPoisonStateArena`);
фасады `finish_summon_skill` — прежний hub `states/summonskill` старого пакета.

## monster-семья: `CMonsterBaseAttack`/`Fast`/`Range`/`Thorn`/`Taming` и диспетчер

Машинная база всякой записи ниже — VERIFIED по телам `.local/recon-a2/out/`
(если у записи не указано иное). Общие швы: hub-трейты
`zone::skills::monsterattack`, часы — fn-параметр `now_milliseconds` делегата
старого main loop.

### monsterbaseattack — `CMonsterBaseAttack` (0x2bd)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| ctor | `0x113AF0` | VERIFIED | `[+4] = 0x2bd`; фабричный QuerySkill 0x2BD → этот класс; default → NULL без fallback (default-атака 1/2/3 — `CMoveShape::GetDefaultAttackSkillID` `0xCE240`, вне фабрики) |
| Begin (три формы) | `0x113B80`/`0x113C50`/`0x113D40` | VERIFIED | форвард `CAttackSkill::Begin` → new effect (vtable `0x656600`) → `VT[0](1)` → слот `+0x60` CheckCastCondition; провал — End(0) и ret 0 БЕЗ терминального кадра; успех — `[+0x4C]=1`, `[+0x50]=0` |
| `CheckCastCondition` | `0x114340` | VERIFIED | S/props null → ret 0 без кадра; только reuse (`QueryProperty(10005)+[+0x40]` vs timeGetTime, unsigned); отказ — `{0xBFE01, 0, 13}` + GS1143 лишь при dyn-cast источника в CPlayer |
| `AI` | `0x114820` | VERIFIED | `[+0x4C]==0` → выход; props/U null → End(0); мёртвая S → кадр failure 2 (mode 2, только player) + **End(1) со штампом reuse**; фаза 1: `RealDistance` unsigned против `QueryProperty(5003)`, превышение — `{0,0xb}` + End(0); SetDir(GetLineDir(U→S)), старт-кадр (mode 0, `[+0x3C] = QueryProperty(10006)`); delay — абсолютный wrapping-срок `timeGetTime >= [+0x2C]+QueryProperty(10001)` (unsigned `jae` по `0x51497E`); fire-кадр (mode 1), `Attack(U, GetS)`, End(1) |
| `Attack` | `0x1146D0` | VERIFIED | null U/S/self → exit; **type S == 500 → exit (500-skip)**; `S->vt+0x134(U)` — IsAttackAble(S) (`0x0E7230`), отказ → exit; info ctor конструкторских UNKNOWN/уровня 1 не замещается; при player-источнике копия pk-полей (`[+0x278..+0x27C]`, `[+0xB20]`, `[+0xB28]`, `[+0xB78]`); Calculate → `S->vt+0x15C(&info, 0)` (OnBeenAttacked) → безусловный `U->IncreaseRp(true, 0)` (vt+0x12C) |
| `CalculateAttackPower` | `0x114460` | VERIFIED | damage_factor = `U->vt+0x184(S->vt+0x110())` float (weapon-фактор от уровня S), `[info+0x20]=0`, hit = `QueryProperty(20001)`; разброс `max(max−min, 0) + 1`, clamp `max(0)` по min+roll; записи видов 1/3/4 в исходном порядке с clamp ≥ 0; критический roll только после dyn-cast в CPlayer: `random(100) < GetCCH` (vt+0x114), `[info+0x10]=1`, каждая запись kinds {1,3,4} — `fmul` по `[player+0x414]` от `fild` без промежуточной записи float, `fistp` x87-усечение; монстр этот RNG не выполняет |
| `End` | `0x1B3010` | VERIFIED | нули `[+0x50]/[+0x4C]`, хвост `CAttackSkill::End(H)` — **без movement-restore и без пересчёта свойств**; только успех изнашивает оружие (AfterUseSkill `0x13CF30`: OnWeaponDamaged только у dyn-CPlayer) и фиксирует reuse |
| `OnChangeRegion` | `0x16A370` | VERIFIED | принудительный End(0) (аргумент замещается нулём до virtual-диспетчеризации) |

Формула player-удара — шов `monster_combat_calculate_attack` владельца lord
(общее тело `lordfastattack::calculate_attack` старого пакета используется
также 0x2bd-ветвью, включая личный критический множитель). Расхождение,
устранённое при переносе: мёртвая цель mid-cast завершается машинным End(1)
со штампом reuse вместо прежнего снятия cast без reuse.

### monsterfastattack — `CMonsterFastAttack` (0x2d1)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| ctor | `0x112910` | VERIFIED | `[+4] = 0x2d1`, `[+0x4C]=[+0x50]=[+0x58]=[+0x54]=0`; фабричный QuerySkill 0x2D1 |
| Begin (три формы) | `0x1129A0`/`0x112A70`/`0x112BB0` | VERIFIED | скелет семьи; провал — End(0) без терминального кадра |
| `CheckCastCondition` | `0x1131D0` | VERIFIED | U/S/props null → ret 0 без кадра; reuse → `{0xBFE01, 0, 13}` + GS1143 только у player; `RealDistance > QueryProperty(5003)` → `{0,0xb}`; ячейка `GetTargetPath` с block==2 → `{0,0xf}`; **нулевая MP-цена → молчаливый ret 0** (`0x5133CF`); MP-player < cost → `{0,7}` + GS1144; успех — SetMoveable(0) (только player-ветки). Исходное разыменование null player в MP-проверке не воспроизводится |
| `AI` | `0x113810` | VERIFIED | S/U null → End(0) без кадра; мёртвая S или U==S → `{0,10}` + End(0) БЕЗ reuse; фазы `[+0x50]/[+0x54]/[+0x58]`: первая — MP повторно только у player (SetMP, OnChangeStates vt+0x164; нехватка → `{0,7}` + GS1144), SetDir к S, старт-кадр, `[+0x3C] = QueryProperty(10006)`; вторая — по `start+delay(10001)` fire-кадр, `[+0x3C]=0`; удары по **кумулятивным** срокам от начала: `delay+15001` и `delay+15001+15002` (unsigned), затем End(1) со штампом reuse |
| `Attack` | `0x113700` | VERIFIED | пропуск null U/S до расчёта и OnBeenAttacked; НЕТ проверки U==S/500/IsAttackAble, НЕТ IncreaseRp; записи 1/3/4 в исходном порядке в `OnBeenAttacked(&info, 0)` (vt+0x15C) |
| `Calculate` | `0x113490` | VERIFIED | разброс `max(max−min, 0) + 1`, clamp `max(0)`; критический roll `random(100) < GetCCH` только после dyn-cast в CPlayer, множитель float по `[player+0x414]` с x87-усечением; у монстра этого RNG нет |
| `End` | `0x112B50` | VERIFIED | нули фаз, `[+0x3C]=1`, SetMoveable(U, 1), `CAttackSkill::End(H)`; только успех изнашивает оружие и ставит cooldown |

Player-ветвь fast остаётся у owner `lordfastattack` старого пакета;
состояние двух ударов (`MonsterFastAttackProgress`) живёт в
`zone::skills::execution`.

### monsterrangeattack — `CMonsterRangeAttack` (0x2ef)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| ctor | `0x111590` | VERIFIED | `[+4] = 0x2ef`; статические `g_dwLength/g_dwHeight == 7`, маска `g_bScope` по `0x006A0ECC` |
| `CheckCastCondition` | `0x111CF0` | VERIFIED | param/props null → ret 0 без кадра; reuse (10005) → `{0xBFE01, 0, 13}` + GS1143 только у dyn-CPlayer; **non-player без MP и без SetMoveable** (`je 0x511E1F`); player: **нулевая MP-цена → молчаливый ret 0** (`jbe 0x511EA1`), MP < cost → `{0,7}` + GS1144(`%u`); успех — SetMoveable(param, 0) |
| `AI` | `0x112500` | VERIFIED | `[+0x4C]==0` → выход; props/U null → End(0); фаза 1: MP повторно только у dyn-CPlayer (`MP−Query(2)`, `js` → `{0,7}` + GS + End(0); SetMP + vt+0x164), start-кадр (mode 0) **без поворота**, `[+0x3C] = QueryProperty(10006)`; delay — абсолютный unsigned-срок `[+0x2C]+QueryProperty(10001)`; fire-кадр (mode 1) с центром U; регион из `[U+0x40]` (null → End(1)); обход 7×7: X внешний, Y внутренний, индекс маски **x+7y**, начало `tile − 3`; клетка читается после предыдущих ударов; `IsAttackAble(U)` кандидата **перед** дедупликацией; цель добавляется в список **после** Attack; конец обхода — End(1) со штампом reuse |
| `Calculate` | `0x112170` | VERIFIED | id/уровень в seed; hit = `Query(20001)`; damage_factor = `U->vt+0x184(S->vt+0x110())` float; **span `abs(max−min) + 1`, вид 3**; значение = AddElementAtk(U) + min(20008) + random(span из 20009−20008) + **trunc(unsigned(EM=20015) × 0.01f(const `0x64DBD0`) × ElementModify)** (unsigned-переход через fild+fadd 2^32, `fimul` по EC); критический roll только у dyn-CPlayer (`random(100) < vt+0x114`, `×[player+0x414]`, x87-усечение); монстр player-ветвей не выполняет — его ElementModify/AddElementAtk == 0 |
| `Attack` | `0x1123E0` | VERIFIED | RP атакующего **не** увеличивается (в отличие от базовой атаки семьи); End общего владельца `0x146090` |

### monsterthorn — `CMonsterThorn` (0x197)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| ctor / Begin-скелет | `0x141490`; `0x141520`/`0x1415F0`/`0x1416E0` | VERIFIED | `[+4] = 0x197`; скелет семьи; провал — End(0) (`0x541751`) и ret 0 БЕЗ терминального кадра (отличие от `CMonsterTaming`); первый AI остаётся невыполненным из Begin |
| `CheckCastCondition` | `0x141C40` | VERIFIED | S/props null → ret 0 без кадров; reuse (10005, unsigned) → `{0xBFE01, 0, 13}`; path size > Query(5003) → `{0,0xb}`; любая ячейка с `[+8] == 2` (`BLOCK_UNFLY`) → `{0,0xf}`; успех — SetMoveable(0) (`0x541DDD`). GS-строк нет |
| `AI` | `0x142180` | VERIFIED | props/U null → End(0); dead S → `{0,10}` + End(0) (`0x5421FA`); S==NULL → fallback `{0, 0, fb_x, fb_y}` из `[+0x24]/[+0x28]`; старт-фаза до delay: `[+0x3C] = QueryProperty(10006)`, SetDir к клетке, старт-кадр (mode 0); delay — абсолютный unsigned wrapping-срок (`jb` по `0x5422AF`); длинный путь → `{0,0xb}` + End(0) (`0x542314`); **`BLOCK_UNFLY` после задержки → updateVE `{0,15}` + End(1) БЕЗ удара** (`0x542386..0x54239B`); fire (mode 1) → Attack → End(1) со штампом AfterUse/reuse. End живого cast не сбрасывает AI-цель и очередь движения |
| `Attack` | `0x142060` | VERIFIED | пропуск только null/self до расчёта и OnBeenAttacked; IsAttackAble/god не являются допуском этого owner-а; IncreaseRp нет |
| `Calculate` | `0x141E10` | VERIFIED | id/уровень/hit (20001) в seed; span `abs(max−min)+1` (cdq/xor/sub), clamp `max(0)`; записи 1/3/4; **второй RNG crit-roll обязателен и у монстра**: `vt+0x114` монстра ≡ 0, крит никогда не срабатывает, `random(100)` выполняется всегда; множитель — глобалка float `0xEF3E5C` (`globe_setup().critical_rate()`), x87-усечение |
| message-owner updateVE | `0x1417A0`, remap16 `00 01 02 08 08 08 08 03 08 08 04 05 08 06 08 07` | VERIFIED | mode 0 → старт-кадр (action 1, направление), mode 1 → fire-кадр (action 2, S/fallback x/y), mode 2 → `{0xBFE01, 0, 2}` self (`0x541B7F`), failure-кадры 7/10/11/13/15 самому игроку |
| shared End | `0x146090` | VERIFIED | нули `[+0x50]/[+0x4C]`, SetMoveable(U, 1), аргумент в `CAttackSkill::End` |

Объектный Begin (type/id) хранит заданную identity; исчезнувшая объектная
цель оставляет пустой GetTargetPath — message-owner пишет нулевые type/id и
fallback x/y, Attack пропускает null, AI выполняет End(1). Координатный
player-вход выбирает первый CMoveShape клетки, включая NPC и постройки;
мёртвую первую цель следующей допустимой не заменяет.

### monstertaming — `CMonsterTaming` (0xd4)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| ctor / Begin (три формы) | `0x17B5C0`; `0x17B650`/`0x17B730`/`0x17B870` | VERIFIED | `[+4] = 0xd4`; скелет: `CAttackSkill::Begin` → new effect (vtable `0x65AAC0`) → `VT[0](1)` → `+0x60` CheckCastCondition; **провал — терминальный кадр mode 2 + End(0) + ret 0 во всех трёх Begin**; успех — `[+0x4C]=1`, `[+0x50]=[+0x54]=[+0x58]=0` |
| message-owner updateVE | `0x17B940`, remap16 как у Thorn | VERIFIED | mode 1 → fire-кадр; mode 2 → **`{0xBFE01, 0, 2}` SendToPlayer** (entry2 == `0x57BCEC`, разбор jump-таблицы `0x57BD74` и remap `0x57BD98`); терминальный кадр посылается после КАЖДОГО CheckCastCondition-отказа, включая молчаливые |
| `CheckCastCondition` | `0x17BDB0` | VERIFIED | U null → ret 0 без кадра; цель не dyn-CMonster или не IsTamable → `{0xBFE01, 0, 10}` + GS0294/GS0312/GS0286; props null → ret 0 без кадра; S null → `{0,10}` + GS0294; reuse (10005, unsigned) → `{0x0d}` + GS0278; path size > Query(5003) → `{0x0b}` + GS0290; ячейка block == 2 → `{0x0f}` + GS0295 (с именем цели); **нулевая MP-цена → молчаливый ret 0 (`jbe 0x57C1D5`)**; MP < cost → `{0,7}` + GS0288(`%u`); успех — SetMoveable(0) |
| `AI` | `0x17C2A0` | VERIFIED | U/S null → End(0); IsDied(S) → `{0,10}` + GS0285 + End(0); фаза 1 повторно списывает MP (`js` → `{0,7}` + GS0288 + End(0)), SetMP + OnChangeStates (vt+0x164), SetDir + старт-кадр; delay (10001) → SetMoveable(U, 1), повторный путь по 5003 (`{0x0b}` + GS0290 + End(0)); fire, далее цепочка: IsTamable → IncreaseTameAttemptCount → safe-cell (block==2 цели/источника → GS0315) → `target > player` уровни → GS0314 → `weaponLevel − Query(20018) < targetLevel` → GS0314 → `GetValidPetsAmount >= Query(31001)` → GS0313 → единственный `random(10000) <= Query(40001)` |
| успешная ветвь AI | `0x57C802..0x57C98E` | VERIFIED | `StopAllSkills` монстра до `DoesCreatureBeenTamed` и назначения master; после счётчика попыток IsTamable не перепроверяется; только auxiliary CPet получает смену режима и OnLoseTarget (vt+0x2C); primary AI, его команды и hate не очищаются; `AddPet` → `UpgradePetLevel` → кадр `0xC0201` (тип/id монстра, тип/id игрока, legacy-C-строка имени, GetPetLevel, GetPetExperience, GetHP, GetMaxHP) → `GetMonsterRefeash(...)[+0x38] −= 1` → End(1) с SetMoveable(U, 1) и reuse-штампом |
| `CMonster::SetTamedSign` | `0x0E64A0` | VERIFIED (хвост прочитан) | пишет только `[monster+0x224] = 1` (гейт после tame-попыток); живую figure НЕ трогает |
| `CMonster::GetFigure` | vt `+0x8C`, тело `0x0E7D20` | VERIFIED | читает байт `[tagMonster+0x20]` живой региональной записи (заполняется при спавне той же setup-строкой `MonsterProperties.figure`) — снимок `property.figure` в AddPet эквивалентен живому байту на момент read |

Прежние расхождения, устранённые при переносе: нулевая MP-цена теперь даёт
молчаливый reject (была приёмка); терминальный кадр `{0xBFE01, 0, 2}`
посылается после каждого CheckCastCondition-отказа Begin (прежний Rust кадр
пропускал). Кадр имени питомца — legacy-C-строка (обрезка по NUL).

### monsterbasedispatch — диспетчер `CMonsterAI`

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `CMonsterAI::OnChangeSkill` | `0x1DCBC0` | VERIFIED | `SelectAttackSkill` → выбранный concrete skill проверяется `IsRestored` (vt `+0x80`: `QueryProperty(10005)+[+0x40] < timeGetTime`; null-props → 1) → готово; иначе `SetCurrentSkill(GetDefaultAttackSkillID())`; возврат всегда 1 |
| `CMonsterAI::SelectAttackSkill` | `0x1DD0B0` | VERIFIED | `dynamic_cast CMonster`, один `random(10000)`, обход списка в исходном порядке, первый ID с префикс-суммой odds ≥ r, иначе default. AI5/AI23 вместо отката ждут полный restore delay в собственном FIFO; CPet (vtable `0x00652D0C`, `+0x24 → 0x1DCBC0`, `+0x8C → 0x1DD0B0`) наследует общий порядок: приручение отключает boss/lord-selector, сохраняя список odds, один RNG и default/IsRestored. WORD-уровень dispatch проверяется до Begin без усечения |
| `CBaseAI::Run` (проекция OnFighting) | `0x0C7D10`, ветка `AI_EVENT 2` | VERIFIED | уже начатый cast продолжается тем же входом: общий объектный навык 1 уходит в `CBaseAttack`, остальные — в executor реестра по точному ID; реестр допуск расписания не меняет и не повторяет |
| `CPet::OnAttackingSchedule`/`OnStayingSchedule` | `0x0E9A20`/`0x0E9650` | VERIFIED | наследуемый порядок расписания |

`GetDefaultAttackSkillID` (`0x004CE240`) — VERIFIED (см.
gameserver-npc-and-regions.md#реестр-навыков-cmoveshape): скан
attack-вектора `+0x130` (ID == 2 → 2), иначе summon-вектора `+0x150`
(ID == 3 → 3), иначе 1; без `QuerySkillType` и раннего выхода; Rust-owner —
`zone::regions::skillregistry::default_attack_skill_id`. Реестр исполнителей
`owned_registered_cast_executor` не переносится и остаётся владением старого
`appserver/skills/monsterbaseattack.rs`.

### monsterattack — общая доставка и допуск

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `CMonster::IsAttackAble` | `0x0E7230` | VERIFIED | первый класс-гейт типа цели: ветвь `[edi+4] == 0x190` (player) и `0x258` (monster); остальные типы отклоняются до PK/tame-ветвей |
| `end_owned_monster_skill_without_reuse` | (контракт) | VERIFIED косвенно | owner `CMonsterThorn::AI` `0x142180` выполняет End(0) для мёртвой/длинной цели; `CMonsterBaseAttack::End` `0x1B3010` не восстанавливает движение и не перечитывает свойства |
| `monster_attack_cell_candidates` | — | PARTIAL | список типов 400/500/600/1100/1200 и исключение источника — свидетельство прежнего владельца; resolver полного региона и фильтр исчезнувших форм — шов старого хоста |

Во время попадания производный регион публикуется целиком: общий получатель
видит живые защиты, HP/MP, источник и выбранный AI цели; возврат из callback
требует заново получить регион и объекты, а не применять сохранённые до
защиты снимки. У NPC нулевой combat HP: наследуемый `CMoveShape::IsDied`
(GetHP == 0) истинен независимо от action. Blanket-часы:
`GameClockContext::now_milliseconds = game_tick_milliseconds`.

Текущая реализация: `zone::skills::{monsterattack,monsterbaseattack,monsterfastattack,monsterrangeattack,monsterthorn,monstertaming,monsterbasedispatch}`;
hub-оркестрация монстра — `appserver/skills/monsterbaseattack.rs` старого пакета;
исходные владельцы PDB: `appserver/skills/monster{baseattack,fastattack,rangeattack,thorn,taming,attack}.cpp`, `appserver/monster.cpp`.

## Ядовая и трупная линия: `CSpiderPoison`, `CPoisonArrow`, `CPoisonMoth`, `CDaubPoison`, `CSpiderMist`, `CCorpseCandleBlasting`

Разведки `.local/recon-de/notes/D2…D8-*.md`, тела
`.local/recon-de/disasm/C*.txt`. RVA записи — VA = RVA + `0x400000`, где у
записи не указано иное.

### spiderpoison — `CSpiderPoison` (0x191)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| vtable эффекта / класса | `0x65B188` / `0x25B0F4` | VERIFIED_DISASSEMBLY | Begin-скелет трёх форм (`0x5853E0`/`0x5854B0`/`0x5855B0`): форвард `CAttackSkill::Begin` → new effect `0xC` → `[+0x34]` → BeginVisualEffect(1) → Check слот `+0x60`; провал — visual(2) → End(0), успех — `[+0x4C]=1`, `[+0x50]=0` (обвязка hub `stateskill`) |
| `Check` | `0x585BB0` | VERIFIED_DISASSEMBLY | null U/S/props → 0 без кадров; reuse (10005, `CSkill::IsRestored`) → visual(13); путь `vcall+0x58` и `Query(5003) != 0 && size > max` → visual(11); SetMoveable(0) на U → ret 1 |
| `AI` | `0x586020` | VERIFIED_DISASSEMBLY | `[+0x4C]==0` → out; props/U/S null → End(0); **IsDied только у S** → visual(10) → End(0); Begin-фаза один раз: `Query(10006)` → `[+0x3C]`, вычисление `Y(S), X(S), Y(U), X(U)` → GetLineDir → SetDir на U → visual(0); задержка `Query(10001)+[+0x2C]` unsigned → out; **`SetMoveable(1)` перед повторной дистанционной проверкой**, длинный путь → visual(11) → End(0); иначе visual(1) → Attack → **End(1) — все исходы состояния завершаются End(1)** |
| `Attack` | `0x585F10` | VERIFIED_DISASSEMBLY | `tagAttackInformation` ctor-дефолт (id `0x7FFFFFFF`, level 1) не переписывается; U.type==400 → MasterInfo-поля игрока (`+0x278..+0x27C`, `+0xB20/+0xB28/+0xB78`) → Calculate → virtual `+0x15C` приёмника |
| `Calculate` | `0x585CE0` | VERIFIED_DISASSEMBLY | props null → info остаётся ctor-дефолтом; `[+0x18]=0` hit, `[+0x1C]=1.0f` factor, `[+0x20]=0` modifier; physical kind 1: `|max−min| + 1` (cdq-abs) → один RNG → min + random → jns-clamp; element kind 3: `vcall+0x118` (у монстра 0) clamp; soul kind 4: `vcall+0x11C` **movzx WORD** clamp; **2 RNG суммарно** (второй `random(100)` обязателен даже при нулевом `vcall+0x114` монстра, movzx WORD cch, знаковое `<`); x87-крит с глобалкой float `0xEF3E5C` |
| поздний бросок яда | (в теле AI) | VERIFIED_DISASSEMBLY | `DoesStateExist(0x131)` на S → пропуск; `random(100) > Query(40001)` **signed jg** → пропуск; master живого U **с country = 0**; ctor `CSpiderPoisonState` (`0x5E90C0`) с порядком Query const(20010) → freq(6001) → keep(10002) **до** замены; скан первого `0x191` в `S+0x11C` → End `vcall+0x1C` → deleting-dtor `vcall+0x10(1)` → slot 0 → Begin(U,S) → тот же слот, иначе append |

Payload состояния `CSpiderPoisonState`: кодек `zone::effects::poison`
(VERIFIED), живой AI/фабрика — hub `states/poison.rs` и `statefactory`
старого пакета. Последний вариант замены — abort при неразрешимой позиции
слота (внутренний отказ арены Rust): для обеих ветвей принята семантика
sibling-каста `corpseptomaine` вместо безусловного append; машинной ветви
этому отказу не соответствует, она возникает только при несогласованности
арены Rust.

### poisonarrow — `CPoisonArrow` (0x21E)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `Check` | `0x519750` | VERIFIED_DISASSEMBLY | null S/props → 0; null-target/self → visual(10) + ZHGS0045; конфликт сканируется **по позиции** исходного вектора состояний S: первый id из `{0x192 → ZHGS0046, 0xD2 → ZHGS0047, 0x67 → ZHGS0046}` → системное сообщение, ret 0; reuse(10005) → visual(13) + ZHGS0048; путь `vcall+0x58`: `Query(5003)!=0` и cells > max (`jbe` проход) → visual(11) + ZHGS0049; клетка third==2 → visual(15) + ZHGS0051 (SSO-имя цели); `Query(2) == 0` (GENESIS MP0) → **ret 1**; GetWarSoulGoods null → тихий ret 0; `GetAddonPropertyValues(0x9A,1) − Query(2) < 0` (`js`) → visual(7) + ZHGS0052 (fild/fmul 0.01 → ftrunc → %u) |
| `AI` | `0x519D70` | VERIFIED_DISASSEMBLY | `[+0x4C]==0` → out; props/U/S null → End(0); IsDied(S) → visual(10) → End(0); фаза 0 player-only (RTTI): GetWarSoulGoods null → **тихий выход без End (Pending)**; нехватка MP → visual(7) + ZHGS0052 → End(0); иначе SetAddon(1,0x9A,остаток) signed → SerializeForOldClient → кадр `0xBF918`; CAN(10006) → visual(0) → `[+0x50]=1`; delay 10001 unsigned → out; повторный путь: дальность → visual(11) + ZHGS0049 → End(0), клетка 2 → visual(15) + ZHGS0051 → End(0); visual(1); MasterInfo живого U с **country = 0**; PK до состояния: player(U) && region([S+0x40]) && player(S) → `CPKSys::OnFirstSkill` с TileX/TileY(U); ctor `0x5E3140` (Query: const 20010 → freq 6001 → keep 10002) → первый старый state End `vcall+0x1C` → dtor свежего остатка → Begin(U,S) `vcall+0x08` → прежний слот либо append; все исходы → **End(1)**; RNG, UpdateProperty и второго End при отказе state Begin нет |
| End навыка | `0x5DFBD0` | VERIFIED_DISASSEMBLY | тело второй половины общего End-контракта координатора (hub `states/skill.rs` старого пакета); собственной логики CPoisonArrow не добавляет |
| `CPoisonArrowState` | ctor `0x5E3140`, vtable `0x0065F24C` | VERIFIED_DISASSEMBLY (кодек — четырёхсторонний) | Calculate kind 5 с единственным clamp отрицательного HP-loss → 0; данные/кодек — `zone::effects::poison` (`PoisonState<0x21E>`), живой AI/Begin и замена первого слота — hub `states/poison.rs` + `states/periodicattack.rs` старого пакета |

### poisonmoth — `CPoisonMoth` (0xCF)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `Check` | `0x58C4C0` | VERIFIED_DISASSEMBLY | null U → 0; self-target → visual(10) + GS0286; далее семейный скелет `check_ranged_weapon_cast` с правилом пути DistanceAndBlocks и арбалетом категории 4: reuse → visual(13) + GS0278; dist 5003 (`jbe`) → visual(11) + GS0290; клетка third==2 (`BLOCK_UNFLY`) → visual(15) + GS0282; арбалет иначе visual(14) + GS0293; `Query(2)==0` → тихий ret 0; signed MP → visual(7) + GS0288; иначе SetMoveable(0), ret 1; non-player — без Move0/MP/арбалета |
| `AI` фаза 0 | `0x58CED0` | VERIFIED_DISASSEMBLY | player: списание MP → `vcall+0x164` (OnChangeStates) → повторная проверка арбалета; CAN(10006) → `[+0x3C]`; GetSufferer: свежие X/Y S, иначе сохранённая точка команды; GetLineDir → SetDir на U; visual(0); `[+0x50]=1` |
| `AI` подготовка пути (`[+0x54]==0`) | `0x58CED0` | VERIFIED_DISASSEMBLY | `Query(10001)+started` unsigned → out; **`SetMoveable(1)`** → GetTargetPath в payload навыка; дальность: отказ iff `size > max + 1` (`jae` проход) — **квазнота MAX+1 против MAX вместо MAX** → visual(11) + GS0290 → End(0); первая клетка third==2 задаёт endpoint (заблокированная), иначе последняя; flight = `Query(10008) × count` до неё; visual(1); запуск фазы атаки |
| `AI` полёт (`[+0x54]==1`) | `0x58CED0` | VERIFIED_DISASSEMBLY | deadline = `Query(10008) × position + Query(10001) + started` (переполнение — wrapping); текущий регион U проверяется **после часов** (нет региона — Pending без End); index ≥ size → visual(3) + End(1); текущая клетка в payload; block 3 (`BLOCK_SHAPE`) → поклеточный удар, попадание → visual(3) + position = size + 1; block 2 (`BLOCK_UNFLY`) → visual(3) + стоп; иначе шаг вперёд; **одна клетка за AI-тик**, дедупликации целей нет; **двойной visual(3)** — при остановке и повторно на тике завершения перед End(1) |
| `Attack(JJ)` | `0x58CD80` | VERIFIED_DISASSEMBLY | семейный `run_poison_moth_cell`: (0,0)-гейт; GetShapes клетки без типового фильтра (RTTI CMoveShape), skip null/self/IsDied; IsAttackAble(U) `vcall+0x134`; **visual-target пишется перед ударом**; ret — был ли удар |
| `Attack(U,S)` / `Calc` | `0x58CC70` / `0x58C9E0` | VERIFIED_DISASSEMBLY | MasterInfo-канва игрока + Calculate + OnBeenAttacked(&info,0); `Calc` — семейный `calculate_crossbow_attack` (kind 5 записей общего хвоста; hit = `−Query(20001)` только у 0xCF, element-добавка 0 вместо Query(20013), jns/jge-clamp, второй RNG крита обязателен) |
| `End` | `0x58C3A0` | VERIFIED_DISASSEMBLY | сброс фазы/счётчиков/цели, освобождение пути до свежего U Move1 и общего Attack End с исходным аргументом — payload `PoisonMothExecutionState::clear_end_paths` (`zone::skills::execution::payload`) + hub-кадр End старого пакета |

### daubpoison — `CDaubPoison` (0xDF) и `CDaubPoisonState`

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| vtable навыка | `0x259B74` (VA `0x00659B74`) | VERIFIED_DISASSEMBLY | Begin-скелет трёх форм `0x565FE0`/`0x565F10`/`0x5660D0`; провал — End(0), успех — `[+0x4C]=1`, `[+0x50]=0`; AI `0x566690`, Check(U) `0x5664B0`, End `0x546090` (ICF `CAgility::End`), DoesTargetEffective `0x5AFCE0` (`mov eax,1`, ICF) |
| apply (после visual(1)) | (в теле AI `0x566690`) | VERIFIED_DISASSEMBLY | скан вектора `[U+0x11C]` — первый НЕпустой слот id `0xDF` (без фильтра RTTI или ended) → End `vcall+0x1C` → deleting-dtor `vcall+0x10(1)` свежего остатка позиции → slot 0; ctor `0x5F17B0` с keep **только из Query(10002)** (та же таблица props); Begin(U,U) `vcall+0x08` → true: push_back append, false: dtor нового; затем End(1); проверки результата установки нет |
| `CDaubPoisonState` Begin JJ/typed | `0x5F18A0`/`0x5F1960` | VERIFIED_DISASSEMBLY | `CState::Begin` (`0x5DBDD0`/`0x5DBE20`) → GetSufferer → null: End `vcall+0x1C` → ret 0; ok → новый visual `0x6604C8` → `[+0x34]` → BeginVisualEffect(1) → UpdateVisualEffect(0) → ret 1. Object Begin `0x5F1A50`: **null U → тихий ret 0**; base clock при non-NULL U до getters, S не проверяется |
| `CDaubPoisonState` End | `0x5FD420` | VERIFIED_DISASSEMBLY | ICF `CAgilityState::End`: optional Update(1) → свежий GetSufferer → RemoveState(pointer) (`0x4CDAB0`), без записи state.ended; чужой или отсутствующий S оставляет запись у держателя |
| `CDaubPoisonState` AI | `0x5D5BA0` | VERIFIED_DISASSEMBLY | ICF `CBlindState::AI`: `now > start + keep` → End, равенство с границей ещё активно |
| GetRemainedTime / Serialize / Unserialize | `0x5F2CD0` / `0x5F51E0` / `0x5EAAC0` | MATCH | двойное чтение часов (race); Serialize — ID до вызова GetRemainedTime; Unserialize — часы сначала, затем keep; реализованы в `zone::effects::daubpoison` |
| wire | — | VERIFIED_DISASSEMBLY | `0x000BFE03` = type/id/0xDF/remaining/additional(0) (публикуется до append, без отдельного UpdateProperty после Begin); `0x000BFE04` = type/id/0xDF |

Check/AI самого навыка — hub `selfstatecast` старого пакета (сверено MATCH):
reuse 10005 → visual(13) + GS0278; player `Query(2)==0` → тихий ret 0;
signed MP → visual(7) + GS0288; SetMoveable(0); не-player — без MP/Move0.
AI: IsDied(U) → visual(2) → End(1) (hub `RejectedAfterUse`); MP → SetMP →
`vcall+0x164` → CAN(10006, dword); visual(0) → delay 10001 → visual(1) →
apply → End(1).

Отклонение hub `selfstatecast` (задокументировано в его шапке): источник не
типа Player отклоняется в Begin-стадии AI, потому что нативная фаза 0 читает
MP `[U+0x284]` без RTTI-гейта (небезопасный доступ для монстра); zone-файл
воспроизводит только машинно достижимый путь, не меняя поведение hub.

### spidermist — `CSpiderMist` (0x198) и `CSpiderMistPhalanx`

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `CheckCastCondition` | `0x540F40` | VERIFIED_DISASSEMBLY | целевая клетка в `[+0x24]/[+0x28]` записью навыка; reuse-пакет; GetTargetPath; лимит `Query(5003)` → отказ 0x0B; `BLOCK_UNFLY` → отказ 0x0F; SetMoveable(0) |
| `AI` | `0x5409B0` | VERIFIED_DISASSEMBLY | `SetDir` направлением к клетке, `SetMoveable(1)` ПЕРЕД выпуском, Summon(user, x, y), End(1); End `0x540890` ≡ CPoisonFog (ICF). Машинно поворота в Begin нет: он исполняется в выпуске AI, не в Begin-стадии (см. `summoncreatureskill`) |
| `CSpiderMistPhalanx` ctor | `0x5EAEB0` (VA) | VERIFIED_DISASSEMBLY | 6 аргументов: (&master, lifetime Query 30001) базе; level→[+0xCC] (level-switch), state_lifetime 10002→[+0xC0], frequency 6001→[+0xC4], hp_loss 20010→[+0xC8]; 0xD0 байт; перекрёст со стеком Summon `0x541140` и AI `0x5EB110` |
| `CSpiderMistPhalanx::AI` | `0x5EB110` | VERIFIED_DISASSEMBLY | deadline `started + lifetime` строго `ja` → Expired; per-cell пропуски мастера, мёртвых, обладателей 0x131/0x191, неатакуемых; новый state с аргументами из `(info&, +0xC0/+0xC4/+0xC8)` |
| `ReplaceAffectRegion` | пустое тело `ret 0xC` `0x1FECA0` | VERIFIED_DISASSEMBLY | ICF-склейка 15 имён пустых реализаций семьи CSummonShape; у CSpiderMistPhalanx реальное тело `0x5EAC30` (есть в Zone) |
| `CSpiderMist::Summon` | `0x541140` | VERIFIED_DISASSEMBLY | единственный живой JJ-вариант мира (J1/J2 = tile X/Y точки через SetTileXY слот `+0x88`) |
| RTTI | `0x0066F16C` | VERIFIED_DISASSEMBLY | CSpiderMist→CSummonSkill→CSkill→CState, а не CStateSkill; wire — `0x000BFE01` действия 1/2 и входной снимок `0x000BF502` (`include_child = true`, группа AddToByteArray `0x1E47A0`) |

PARTIAL: конфиг-зависимость 5×5 таблиц области в этой сборке константна.
Имя слота `+0x2C`, GUID `0xEF3D9C` — INFERRED; ветви 2..0xF jump-таблицы
UpdateVisualEffect вне режимов 0/1 не достраиваются.

### corpsecandleblasting — `CCorpseCandleBlasting` (0x194)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| vtable / Check | `0x25AEF4` / `0x582600` | VERIFIED_DISASSEMBLY | Check — ICF-фолд с `CSporeBlasting`: null S → 0; reuse 10005 → visual(13); срок — SetMoveable(0), ret 1. Begin-скелет `0x582670`/`0x582740`/`0x582870`; End `0x582810` — ICF с SporeBlasting (`[+0x50]=0`, `[+0x4C]=0` → GetUser → SetMoveable(1) → `CAttackSkill::End`). Общая очистка `CMonster` вызывает его после сообщения смерти либо при отмене/Stiffen без взрыва; скрипт, урон и пометка удаления не являются побочными эффектами End |
| `AI` | `0x5830E0` | VERIFIED_DISASSEMBLY | `[+0x4C]==0` → out; props/U null → End(0); фаза 1 — updateVE(0); delay `Query(10001)+[+0x2C]` unsigned → out; updateVE(1); регион — RTTI `[U+0x40]` → null → End(0); центр `GetTileX/Y(U) − 1` (length = height = 3) |
| обход | (в теле AI) | VERIFIED_DISASSEMBLY | внешний X (esi), внутренний Y (ebp), маска `g_bScope[x + 3·y]` — девять байт `[1,1,1,1,0,1,1,1,1]`; GetShape клетки → RTTI CShape→CMoveShape → `IsAttackAble(U)` `vcall+0x134` → Attack |
| `Attack` | `0x582FC0` | VERIFIED_DISASSEMBLY | отдельный отказ 600→600 внутри Attack (scan его не делает); info ctor-дефолт; U==400 → MasterInfo-поля игрока; Calculate → `vcall+0x15C` приёмника |
| после обхода | (в теле AI) | VERIFIED_DISASSEMBLY | `[U+0x80]=1` (stage-for-delete); RTTI CMonster + GetSufferer RTTI→CPlayer: оба живы и `script_file[0] != 0x30` → `RunScript` (stRunScript с регионом/игроком/файлом, NULL point); кадр смерти `0xBF60B`: `long 0, long 0, long 600, long id, long 0, byte 2`; SendToAround(U, 0); End(1) `vcall+0x68` |
| wire updateVE | `0x582930` | VERIFIED_DISASSEMBLY | `0xBFE01`, живы плечи 0/1; кадры старта и исполнения байт-в-байт совпадают с zone-сборщиками |
| `Calculate` | `0x582EA0` | VERIFIED_DISASSEMBLY | MIN/MAX — `push 0x4E28`/`push 0x4E29` (20008/20009, адреса `0x582F33/0x582F45/0x582F68`), hit — `push 0x4E21` (20001 → `[info+0x18]`, `0x582EE8`); `|max − min| + 1` (cdq-abs), один RNG, `GetAddElementAtk` `vcall+0x118` монстра ≡ 0 (свёрнут), jns-clamp, kind 3 (Element), mp_damage = 0 |

Расхождение, устранённое при переносе: прежняя реконструкция читала MIN/MAX
ключами 20_001/20_002 и hit ключом 3; машинные ключи — 20008/20009 и 20001
(см. строку Calculate). Узкая достижимость: значения строки навыка — вне
машинной базы (строки БД — проектный UNKNOWN); отсутствующий ключ даёт
`query_property == 0`, машинная форма чтений не меняется.

Текущая реализация: `zone::skills::{spiderpoison,poisonarrow,poisonmoth,daubpoison,daubpoisonstate,spidermist,corpsecandleblasting}`;
hub-обвязки — `states/skill.rs` (stateskill), `selfstatecast.rs`,
`states/poison.rs`, `appserver/skills/poisonarrow.rs` старого пакета;
исходные владельцы PDB: `appserver/skills/{spiderpoison,poisonarrow,poisonmoth,daubpoison,daubpoisonstate,spidermist,spidermistphalanx,corpsecandleblasting}.cpp`.

## Громовые облака боевого духа: `CThunder`, `CLeiming2`, `CTianhuo` и их области

### thunder — `CThunder` (0x21F, общие Check/AI семьи)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `Check` | `0x121330` (VA `0x521330`) | VERIFIED_DISASSEMBLY | S=null → тихий 0; RTTI → CPlayer; состояния S в порядке первого совпадения (0x192 → ZHGS0046, 0xD2 → ZHGS0047, 0x67 → ZHGS0046); reuse 10005 → visual 13 + ZHGS0048; GetTargetPath + 5003 → visual 11 + ZHGS0049; клетки `[cell]==2` → visual 15 + ZHGS0051; `query(2)=0` → тихо; WarSoul null → тихо; signed-дефицит MP → visual 7 + ZHGS0052 (`fild` → `fmul [0x64dbd8]` → FISTP). Объектная S захвачена до callback базового Begin, путь читается через текущий GetS |
| `AI` | `0x121940` (VA `0x521940`) | VERIFIED_DISASSEMBLY | phase/таблица-U/S; S мёртв → visual 10 + ZHGS0050 (только player) → End(0); расход MP `GetWarSoulGoods`/`GetAddonPropertyValues(0x9A)` → SetAddon → `SerializeForOldClient` → `CMessage(0xBF918)`; CAN=10006 → visual0 → advance; срок started+10001 unsigned; visual1 → Summon. Сериализация и `0xBF502` безусловны даже при отказе Add области. После visual1 Summon заново читает Master/предмет/таблицу, затем параметры ctor; clock предшествует ID, SetCenter/Initialize — проверке региона U. Отказ Summon не меняет End(1); область живёт независимо от навыка. RVA тела Summon разведкой не назван; тело следует прежнему буквально |
| BF918-доставка CThunder / CLeiming2 | `0x121BAB` / `0x12080B` (VA `0x521BAB`/`0x52080B`) | VERIFIED_DISASSEMBLY | оба — круг `SendToAround(U-шейп, player)`; payload тот же (id, GUID, len, blob `SerializeForOldClient`), доставка — hub-шов `send_summon_cloud_goods_update_around`; форма кадра совпадает с точечным `send_battle_fairy_goods_update` (второго энкодера нет). UNKNOWN: второй аргумент исходного `SendToAround` (exclude-player) не покрыт; делегат доставляет без исключений (`None`) |
| BF918-доставка CTianhuo | `0x12300B` (VA `0x52300B`) | VERIFIED_DISASSEMBLY | точечный `SendToPlayer`; точечная унификация верна только для tianhuo/ResetSkill/fatalblow |

Численные адаптеры сохраняют x87-усечение и младший DWORD результата i64.
Текущая реализация: `zone::skills::thunder` (+ hub-делегат `appserver/skills/thunder.rs` старого пакета).

### leiming2 — `CLeiming2` (0x21B)

Check/AI общие с CThunder (`0x121330`/`0x121940`), BF918 — круг
`SendToAround` в `CLeiming2::AI` `0x12080B` (установлено заново машинным
чтением call site, VERIFIED_DISASSEMBLY). Собственный Summon сохраняет
порядок CCH → AddElementAtk → max → min → текущий level → lifetime: к
вычисленному стихийному коэффициенту прибавляется AddElementAtk, частота и
число целей не читаются. Clock ctor предшествует ID; центр устанавливается
до допуска региона; initialize-прохода RNG у Leiming2 нет; отказ регистрации
обрабатывает общий publisher области; AI в любом случае заканчивает попытку
внешним End(1). RVA тела Summon в записи свидетельства не назван — перенос
тела буквально.

Текущая реализация: `zone::skills::leiming2`.

### tianhuo — `CTianhuo` (0x21A)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `Check` | (семейный префикс) | VERIFIED_DISASSEMBLY | часы читаются до свойства reuse (`reuse_clock_first`); препятствий по клеткам нет; MP проверяется прямым vcall `+0x5C` по equipment[10] (не `GetWarSoulGoods`) даже при нулевой цене; CAN навыком не изменяется |
| `AI` | `0x122DC0` (VA `0x522DC0`) | VERIFIED_DISASSEMBLY | MP списывается необратимо на первом AI; `0xBF918` точечно `SendToPlayer` (`0x12300B`); затем поворот U (GetLineDir + SetDirection `+0x60`) и повторная проверка длины пути (visual 0xB + ZHGS0049); по абсолютному сроку сохранённая объектная S проверяется на смерть и превращается в точку до visual1; visual1 → Summon → безусловный внешний End(1) |
| `Summon` | `0x123280` (VA `0x523280`) | VERIFIED_DISASSEMBLY | region-RTTI → свежая таблица → очистка тройки `[+0x18/1C/20]` → Master → player-RTTI → мёртвые чтения GAP `0xC3` и usage 20015 (`let _ =`) → ctor-стек (lifetime, level, min, max, elem) → SetCenter → GetShape(клетка) + RTTI + skill==0x21A → `vcall [+0xa8]` свёртка старой области → Add → `0xBF502`; свёртка — регистрационный шов прежнего владельца (`replace_tianhuo_phalanxes_in_cell` до add); отказ Summon не меняет End(1). Gameplay ID области — 0x21A, legacy ID режима применения эффекта — 0x13A |

UNKNOWN: ветка AI с не-player U читает CPlayer после RTTI без проверки —
прежняя реконструкция возвращает отказ вместо разыменования (сохраняется).

Текущая реализация: `zone::skills::tianhuo`; поворот U — шов
`set_summon_cloud_user_direction`.

### Области: `CThunderPhalanx`, `CLeimingPhalanx2`, `CTianhuoPhalanx`

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| `CThunderPhalanx` ctor | `0x1E4FE0` | VERIFIED_DISASSEMBLY | 9 аргументов: (&master, lifetime) базе; level→[+0xD8], frequency 6001→[+0xC4] (**нулевая заменяется единицей**), min 20008→[+0xCC], max 20009→[+0xD0], elem-modifier trunc(20015·1e-6·W)→[+0xD4], target_count 20010→[+0xDC], CCH (word, vcall+0x114)→[+0xE4]; массив [+0xBC] ёмкостью (lifetime/freq)·H·L·8; перекрёст со стеком Summon `0x521D30` и Calc `0x1E5250` |
| `CThunderPhalanx` Calc | `0x1E5250` | VERIFIED_DISASSEMBLY | базовый урон — расширенный порядок x87, усечение в `i64`, чтение младших 32 бит; первый RNG использует диапазон ctor и расходуется до свежего WarSoul/таблицы; второй берёт живые min/max; поздний отказ сохраняет метаданные без записи урона |
| `CThunderPhalanx` AI | `0x1E5560` | VERIFIED_DISASSEMBLY | три чтения часов — срок по `+0xB4`+`+0xB0`, частота `+0xC4`, lastAttack `+0xC8`; обход маски 7×7: X снаружи, Y внутри |
| `CThunderPhalanx` Initialize | ICF `0x1EEF30` (`CGodThunderPhalanx2::Initialize`) | VERIFIED_DISASSEMBLY | центр читается после SetCenter; пары RNG расходуются по окнам ≤ 49 целей — массив для клиента, не выбор серверных попаданий |
| `CThunderPhalanx` AddToByteArray | ICF `0x1EF0C0` | VERIFIED_DISASSEMBLY | группа GodThunder/GodThunder2/ThunderPhalanx; пятипольный префикс zone-конверта; расширенный хвост `n = ([B0]/[C4])·[DC]` + raw 8n не переоткрыт: wire сохраняет исходный счётчик `(lifetime/frequency)·targetCount` и читает префикс массива на 49 ячеек на окно; при некорректном счётчике чтение ограничено буфером вместо выхода за границу |
| `CThunderPhalanx` Replace | ICF `0x1FDCA0` | VERIFIED_DISASSEMBLY | — |
| маска 7×7 | — | VERIFIED_DISASSEMBLY (свойство сборки) | одна для всех уровней; константа сборки `godthunder::ROUNDED_THUNDER_SCOPE` |
| `CLeimingPhalanx2` ctor | `0x1E4810` | VERIFIED_DISASSEMBLY | 7 аргументов: (&master, lifetime) базе; level→[+0xCC], min 20008→[+0xC0], max 20009→[+0xC4], elem+AddElementAtk→[+0xC8], CCH→[+0xD0]; own-поля живыми телами не читаются (Calc `0x1E4A00` — формула из таблицы); перекрёст со стеком Summon `0x120990` |
| `CLeimingPhalanx2` AI | `0x1E4CE0` | VERIFIED_DISASSEMBLY | по истечении срока область обходит свою ячейку, поражает каждую цель один раз и удаляется; End помечает удаление только после всех попаданий (`[+0xAC]`). Все уровни имеют одну активную ячейку (машинный факт) |
| `CLeimingPhalanx2` Replace/AddTo/Decord | `0x1E4520`/`0x1E47A0`/`0x1EA070` | VERIFIED_DISASSEMBLY | AddToByteArray — семейная группа `0x1E47A0`; ReplaceAffectRegion выключает совпавшую клетку; создание Leiming2 и общий AddObject его не вызывают; Decord не перенесён |
| `CTianhuoPhalanx` ctor | `0x1E5890` | VERIFIED_DISASSEMBLY | 6 аргументов без id/часов: (&master, lifetime) базе; level→[+0xC8], min 20008→[+0xBC], max 20009→[+0xC0], elem — **сырой Query(20015)**→[+0xC4]; Calc `0x1E5920` читает те же смещения (min/max/level; elem не читается); перекрёст со стеком Summon `0x123280` |
| `CTianhuoPhalanx` AI | `0x1E5C00` | VERIFIED_DISASSEMBLY | пока срок не истёк, на каждом проходе сканирует свою клетку в исходном порядке региона; после каждой допустимой атаки помечается на удаление и немедленно шлёт `0xBF504` (End→BF504); один проход обрабатывает уже полученный снимок — пакет удаления может повториться |
| `CTianhuoPhalanx` Replace/AddTo/Decord | ICF `0x1F54A0`/`0x1F54D0`/`0x1FF7C0` | VERIFIED_DISASSEMBLY | группа `0x1F54D0` (CWeakPhalanx, CGodPunishmentPhalanx, CThunderBlowPhalanx, CTianhuoPhalanx); совпавшая старая область завершается до регистрации новой |
| формулы областей | — | VERIFIED_DISASSEMBLY | у Leiming2 один вызов legacy RNG лишь при наличии боевого духа и таблицы; у Tianhuo один вызов при наличии предмета в слоте 10 до повторного чтения; базовый урон — общий x87-порядок ThunderPhalanx: слагаемое духа и случайная база складываются в x87 до единственного усечения в `i64`, читаются младшие 32 бита |

Оружейный шов `ThunderPhalanxGame` (факторы globe, живой `weapon_modifier`)
— делегат старого пакета `appserver/skills/thunderphalanx.rs`. Run-делегации
(Attack-обход, регистрация, отправка `0xBF504`, свёртка) остаются у прежнего
владельца.

Текущая реализация: `zone::skills::{thunderphalanx,thunder2phalanx,tianhuophalanx}`.

## Громовые удары: `CThunderBlow`/`2`, `CThunderSlash` и их формы, `CThunderFirePhalanx`

### thunderblow — `CThunderBlow` (0x13F) и `CThunderBlowPhalanx`

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| `AI` (якорь) | `0x17A520` | VERIFIED_DISASSEMBLY | AI в Begin — RTTI player → MP `[+0x284]` → signed-дефицит (visual7 + GS0288 + End0) / **SetMP сразу** → state-update (`+0x164`) → CAN=10006 → `[+0x3C]` → назначение из S/снимка → SetDirection → **повторная дальность** (visual 0x0B + GS0290 + End0 при частичных эффектах) → visual0 → advance. Машинный порядок: списание MP, обновление состояния и поворот идут ДО повторной проверки дальности — её отказ теряет уже списанные MP и поворот, общий хвост AfterUse выполняется. Достижимо только при смене цели/пути между начальным Check и этим AI |
| хвост AI | (в теле) | VERIFIED_DISASSEMBLY | срок unsigned → IsDied S (visual 0x0A + GS0285 + End0) → S → точка (`+0x24/+0x28`) → visual1 → Summon (`+0x8C`) → End(1) |
| `Summon` | (в теле) | VERIFIED_DISASSEMBLY | region-RTTI → block (vcall `+0x44`) == 2 → abort → мёртвое 20015 → ctor (min/max/elem/level/lifetime) → SetCenter → same-cell свёртка → Add → `0xBF502` (фасад `add_thunder_blow_phalanx` прежнего владельца региона) |
| `CThunderBlowPhalanx` ctor | `0x1F5430` | VERIFIED_DISASSEMBLY | 6 аргументов, 0xD8 байт; живое тело с 2 RNG (диапазон урона, затем крит) и x87-критом (множитель в расширенной точности, усечение к нулю при записи в i32) |
| `End` базы `CSummonSkill` | `0x1E0F40` | VERIFIED_DISASSEMBLY | собственный `End` навыка не меняет движение, но вызывает оружейный `CAttackSkill::End`; успех, отказ после Begin и клиентская отмена — один хвост с AfterUseSkill и reuse |

Успешный Begin возвращает Begun до первого AI; исходный отсчёт сохраняет
общий kernel. Восстановление — абсолютный срок `CSkill::IsRestored`, задержка
формы — elapsed. UNKNOWN: машинная запись CAN=10006 → `[+0x3C]` в исполнение
player-kernel отдельно не материализуется (только чтение свойства;
потребители поля не установлены — конвенция summon-полосы).

Текущая реализация: `zone::skills::thunderblow` (+ hub-делегат `appserver/skills/thunderblow.rs` старого пакета); исходные владельцы PDB `appserver/skills/thunderblow.cpp`, `thunderblowphalanx.cpp`.

### thunderblow2 — `CThunderBlow2` (0x14D)

Машинные факты (VERIFIED_DISASSEMBLY по телам `appserver/skills/thunderblow2.cpp`,
визуальный ресурс — тот же cpp, слит в файл навыка): зарегистрированный
Attack Begin создаёт loop1-visual перед Check; отказ — End(0) без пакета.
Check требует отдельную S, проверяет reuse, длину локального пути и
ненулевую цену MP; движение не блокируется. Выпуск и попадание отдельно
читают абсолютный unsigned срок start+delay. При выпуске identity-цель
превращается в точку, но отбрасывание и удар используют S, захваченную в
начале AI; визуальный ресурс заново разрешает S. Отсутствие региона источника
отменяет только отбрасывание. Локальные пути между AI не сохраняются. End
сбрасывает фазу и attacking перед Attack-base; движения не возвращает.
Неиспользуемое исходное поле missile всегда ноль и не материализуется.

Pillar-гейт отбрасывания: цель не отбрасывается при живом состоянии `0x74` =
`PILLAR_SKILL_ID` стойки CPillar (`zone::skills::pillar`; Rage — 0x6D,
RageBreak — 0x6E, другая полоса).

Visual `0x000BFE01`: modes 0/1/3 публикуют wire action 1/2/3; mode 1 заново
разрешает S либо использует точку базы — получатель эффекта после
отбрасывания может отличаться от S текущего AI; personal-таблица ошибок
2/7/10/11/13/15 шлёт пару `[0, mode]` только игроку; базовый хвост выполняется
общим visual-owner также для отсутствующего U, завершённого ресурса и
неизвестного mode.

Формула и raw-контакт без RP принадлежат `impactattack` старого пакета (швы
`knock_back_impact_target`, `apply_thunder_blow_2_attack`). Запись исполнения
— `zone::skills::execution::payload::ThunderBlow2Execution` (только
`attacking_started`).

Текущая реализация: `zone::skills::thunderblow2` (+ hub-делегат `appserver/skills/thunderblow2.rs` старого пакета).

### thunderslash — `CThunderSlash` (0x72) и `CThunderSlashEffect`

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| ctor / vtable | `0x57A240` (0x54 байта, фабричный индекс `0x42`) / `0x65A974` | VERIFIED_DISASSEMBLY | — |
| Begin триада / dtor | `0x57A3A0`/`0x57A2D0`/`0x57A490`; `0x57A470` | VERIFIED_DISASSEMBLY | — |
| `CheckCastCondition` | `0x57AAB0` | VERIFIED_DISASSEMBLY | RTTI-гейт игрока; reuse `0x2715` (visual13 + GS0278); непользовательский U без Move0/оружия/ресурсов; игроку — топор категории 1 слота 2 (`GetGoods(2)` → `GetAddonPropertyValues(5, 1) == 1` `0x4CB1E0`) → visual14 + GS0287; signed-дефицит MP `query(2)` → visual7 + GS0288 и RP `query(3)` (WORD-разность) → visual8 + GS0289 с суммами; успех запрещает движение U (`0x4CCEE0(0)`) |
| `AI` | `0x57ADD0` | VERIFIED_DISASSEMBLY | двухфазный. `[+0x50]==0` (без проверки смерти U и регионального отсечения): повторная проверка оружия → расход первого state `[+4]==0x6E` БЕЗ RTTI/ended-проверок (End `vcall+0x1C` + deleting dtor `vcall+0x10` + обнуление слота), иначе visual4 + GS0304 + End(0) → расход MP (`0x4300D0`) и RP (`0x4300F0`) с частичными эффектами при отказе (visual7/8 + GS + End(0)) → OnChangeStates `vcall+0x164` → CAN `0x2716` → `[+0x3C]` → назначение из живой S либо сохранённой точки `[+0x24]/[+0x28]` → направление `0x41D080` → SetDirection `vcall+0x60` → visual0. `[+0x50]!=0`: задержка `0x2711` unsigned `start + delay <= now` → visual1 → лицевая клетка (GetTile, GetDirectionPos `vcall+0x5C`, `0x45B330`) → RTTI-регион из `[U+0x40]`; без регионального владельца End(1) без Summon. Мёртвый GetShapes/RTTI-скан не воспроизводится: найденный объект в `0x57B225`–`0x57B28C` не использовался и игровых callback-ов там нет |
| `Summon` | `0x57B2E0` | VERIFIED_DISASSEMBLY | отдельная свежая таблица (`0x46C390(0x72, level)`), lifetime `query(0x7531)`, снимок getter-ов Dex(`[+0x3B8]`)→SOUL(`vcall+0x11C`)→CCH(`+0x114`)→ELEMENT(`+0x118`)→MIN(`+0xE4`)→MAX(`+0xE8`), country=0, PK-байты `[+0x278..+0x27B]` только при RTTI CPlayer; безопасная замена ограничивает создание формы игроком (у native Summon после необязательного RTTI есть разыменование NULL), не меняя общий Check/AI |
| End (общий хвост) | `0x5AE7A0` | VERIFIED_DISASSEMBLY | vcall vtable+0x68; `[+0x4C]/[+0x50]` в 0, свежему U движение `0x4CCEE0(1)`, база `CSummonSkill::End` `0x5E0F40` |
| `CThunderSlashEffect` | vtable `0x65AA0C`, UpdateVisualEffect `0x57A550` | VERIFIED_DISASSEMBLY | jump-таблица 0..=15: modes 0/1 — around-пакеты, `[0, mode]` только игроку для 2/4/7/10/11/13/14/15, mode 8/12 без пакета |

UNKNOWN: запись CAN `0x2716` → `[+0x3C]` не материализуется (конвенция
summon-полосы, как у thunderblow). Consume расхода состояния —
`zone::skills::ragebreakstate`.

Текущая реализация: `zone::skills::thunderslash`; исходный владелец PDB `appserver/skills/thunderslash.cpp/.h`.

### thunderslashphalanx — `CThunderSlashPhalanx`

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| ctor | `0x5F75E0` (0xEC байт, 12 аргументов, vtable `0x660BDC`) | VERIFIED_DISASSEMBLY | поля `[+0xCC]` и `[+0xD0]` пишут один и тот же max-аргумент — min-аргумент конструктор не читает (**баг оригинала, воспроизводится**); `[+0xE4]` = уровень, `[+0xB8]` = `[+0xE8]` = 0x72, `[+0xC8]` = 0 |
| `AI` | `0x5F7B40` | VERIFIED_DISASSEMBLY | три чтения `timeGetTime`: срок `[+0xB4]+[+0xB0] < now` unsigned, клетка `[+0xBC]/[+0xC0]` ненулевая, период `[+0xC8]+[+0xC4] < now` строгий; штамп `[+0xC8]` пишется ДО RTTI-разрешения региона и единственного GetShape собственной клетки; первая немобильная форма не заменяется следующей целью |
| `Attack` | `0x5F7A00` | VERIFIED_DISASSEMBLY | IsDied → пропуск; FindPlayer по одному ID даже для снимка иного типа; attackable `vcall+0x134`; OnBeenAttacked `vcall+0x15C`; при найденном source — IncreaseRp(1, 0) |
| `Calculate` | `0x5F77B0` | VERIFIED_DISASSEMBLY | см. ниже |
| AddToByteArray / Decord | `0x5F76A0` / `0x5F7730` | VERIFIED_DISASSEMBLY | префикс 5 dword: id 0x72, уровень `[+0xE4]`, master type/id `[+0x84]/[+0x88]`, remained `0x5E9870` |
| ReplaceAffectRegion | ICF-хвост `CWeakPhalanx` `0x5F54A0` | VERIFIED_DISASSEMBLY | End при совпадении клетки |
| GetSkillID / End / ForceMove | `0x5E1030` / `0x5E9DC0` / `0x5E9AF0` | VERIFIED_DISASSEMBLY | — |

Машинный факт тела Calculate `0x5F77B0`:

```text
005f77ed: mov  edx, dword ptr [ebp + 8]    ; [this+8] — instance-id формы
005f77f5: mov  dword ptr [esi], edx        ; info[+0] := instance-id
```

`info[+0]` получает живой instance-id формы (`g_lID` счётчика CSummonShape),
а не константу 0x72 прежней реконструкции (исправлено при переносе).
Эквивалентность потребителей установлена разведкой (`.local/recon-t4`):
читающие стороны поля +0 — war-soul-предикат `is_war_soul_skill`
(0x212..0x224, 530..548) и wire-хвосты 0xBF60A/0xBF60B отчёта попадания;
применение урона поле +0 не использует. Квази-особенность: при входе
instance-id в окно 0x212..0x224 (боевой октет 0x212..0x219 = 530..545)
атака призванной формы с player-мастером проходит war-soul ветки приёмника —
поведение оригинала сохранено буквально. По той же машинной записи
`info[+4]` — младший байт уровня `[+0xE4]`, `info[+0x20]` = 0, фактор
`f32(query(0x4E23) · double(0.01f32))`, hit = `query(0x4E21)`; RNG диапазона —
до RNG крита; критический множитель — FISTP-усечение к нулю
(`truncate_original`).

Нормализация combat-scale: тело Calculate `0x5F77B0` досмотрено до ret 4
целиком — блока нет (VERIFIED_DISASSEMBLY); в реализацию не вводится.

Текущая реализация: `zone::skills::thunderslashphalanx`; тикер и публикация
удаления — региональный runtime старого пакета.

### thunderfirephalanx — `CThunderFirePhalanx` (0x322, предметный `CItemSkill_2`)

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| ctor | `0x5E11E0` (0xFC байта, 10 аргументов) | VERIFIED_DISASSEMBLY | статический `CScope` 1×1 (`g_dwLength/g_dwHeight` = 1, `g_bScope` из 9 байт с маской только (0,0) — конструктор области `0x5E98B0` + маска `0x5E9960`), vtable `0x65EF2C` |
| `AI` | `0x5E1970` | VERIFIED_DISASSEMBLY | см. disassembly ниже |
| обход области | `0x5E1670` | VERIFIED_DISASSEMBLY | scope-ячейки вокруг клетки, список уже-атакованных, self-пропуск, допуск `vcall+0x134` для player-целей |
| `Attack` по цели | `0x5E1570` | VERIFIED_DISASSEMBLY | IsDied → пропуск; Calculate; OnBeenAttacked `vcall+0x15C`; без IncreaseRp |
| `Calculate` | `0x5E1310` | VERIFIED_DISASSEMBLY | `GetWeaponModifier` (`0x42D980` через `vcall+0x184`) без blast-клампа; x87 soul-формула `trunc((soul_variable · soul_count · 0.01 + 1) · damage)`; hit = 0x64; два вызова MSVCRT RNG `0x41CBA0`: диапазон, затем крит с FISTP |
| AddToByteArray / DecordFromByteArray | `0x5FBD20` / `0x5FBFA0` | VERIFIED_DISASSEMBLY | префикс 5 dword: id 0x322, уровень `[+0xD4]`, master type/id `[+0x84]/[+0x88]`, remained `0x5E9870`, затем `CShape` |
| ReplaceAffectRegion | `0x5FECA0` | VERIFIED_DISASSEMBLY | `ret 0xC` — собственное пустое тело, ICF с CWeakPhalanx отсутствует |
| End / ForceMove | `0x5E9DC0` / `vcall+0xA0` `0x5E9AF0` | VERIFIED_DISASSEMBLY | разовый ForceMove в последнюю ячейку с длительностью `count · speed` (`[+0xF8]`) |
| xref активации | `0x515D6F` | VERIFIED_DISASSEMBLY | единственный вызов ctor — из `CItemSkill_2::Summon`; в `QuerySkill` форма не регистрируется. Item-владелец передаёт путь, `speed` и soul-пару (снимок SoulCollect) |

Машинный факт тела AI `0x5E1970`:

```text
005e19e8: cmp  eax, ecx          ; now ? started + idx*speed
005e19ea: jb   0x5e1a80          ; не due — пропуск без End
005e1a0e: test ebx, ebx          ; RTTI-регион из [U+0x40]
005e1a10: je   0x5e19f0.../0x5e1a80 ; region NULL — пропуск без End
005e1a1d: cmp  edi, eax          ; idx < count ?
005e1a1f: jb   0x5e1a2d
...      call [edx + 0xac]        ; End ТОЛЬКО в ветке due+region
```

Верхний Expired-гейт: `exp(started+lifetime < now)` / scope-патч `[+0xBC]==0` /
данные пути `[+0xE0]==NULL` / `count==0` — и НЕ `idx >= count`: исчерпание
пути завершает форму только в due-ветке с разрешённым регионом (тот же
`vcall+0xAC`, что у expired-веток). Прежняя реконструкция выносила
`current_position >= path.len()` в верхний гейт и завершала форму досрочно
даже вне due; условие снято оттуда, исчерпание проверяется в точке due
(см. `tick`). После End по idx-исчерпанию машинный хвост выполняет разовый
ForceMove — блок уже закрыт первым проходом, различия нет.

Нормализация пяти scale: тело Calculate `0x5E1310` досмотрено до ret 8
целиком — блока нет (VERIFIED_DISASSEMBLY); безосновательный clamp-блок
прежней реконструкции удалён из `calculate_owned_thunder_fire_attack` вместе
с методом трейта.

Текущая реализация: `zone::skills::thunderfirephalanx`; исходный владелец PDB `appserver/skills/thunderfirephalanx.cpp/.h`.

## Боевой дух: координатор `battlefairyskill`, призыв `battlefairysummon` и навыки семейства

### battlefairyskill — координатор и wire visual

End-контракт координатора (цепочка — hub `states/skill.rs` старого пакета):
`End` семейства `CStateSkill` `0x5DFBD0`: arg≠0 → GetUser → virtual
`AfterUseSkill` `0x53CF30`; всегда `CSkill::End(arg)` `0x4D84C0`: user? →
virtual `OnEndSkill` (`+0x158`, пустая база) → зануление 9 DWORD `+0xC..+0x2C`
→ reuse-штамп `timeGetTime` только при arg≠0 → прямой delete effect (без
virtual End) → ended=1. Cooldown — после AfterUse, до разрушения эффекта.

Визуал-таблица (VERIFIED_DISASSEMBLY): 19 отдельных тел без ICF между ними;
две формы диспетча — JT16 (7 атакующих: BFBaseAttack `0x1170E0`, CFatalBlow
`0x11E480`, CTianhuo `0x1223B0`, CThunder `0x120E30`, CLeiming2 `0x11FA80`,
CBloodLoss `0x11A840`, CPoisonArrow `0x1191A0`) и remap14→JT8 (12 кастеров со
своими байтовыми картами). Failure-кадр `[byte 4][byte mode]` точечным
SendToPlayer под `dynamic_cast CPlayer`; wide-8 кастеры пишут
`add_long(4)+add_byte(8)`; все ветки сходятся в базовый хвост
`CVisualEffect::UpdateVisualEffect` (внешний dispatcher `states/skill.rs`).

Установленные поправки переноса:

- (A) CFatalBlow (`0x51E538`/`0x51FB3B`/`0x51E5C8`) и CLeiming2 в case0 и
  fire пишут ЖИВОЙ тип юзера `[user+4]`, CThunder — литерал 700; прежний Rust
  писал 700 всем — поведение сохранено по классам через
  `BattleFairySourceType`. INFERRED: значение `[user+4]` при живом боевом духе
  как источнике считается машинным чтением поля, без отдельной ветки типов.
- (B) Po-семейство идёт в fire без проверки sufferer (CPojia `0x12A09B`) —
  прежняя Required-форма сохранена. UNKNOWN: достижимость ветки fire при
  NULL-sufferer не подтверждена.
- (C) wire `0xBF918` — оригинал шлёт `SendToPlayer(player_id)` точечно
  (машинные якоря `0x501BDE..0x501C61` и `0x51F249`); прежний Rust
  `send_goods_update` использовал рассылку around. Общий помощник
  `send_battle_fairy_goods_update` доставляет точечно; форма кадра та же:
  id, GUID (1+16 или маркер), len, blob `SerializeForOldClient`; BF918
  отправляется и при отказе Serialize.

Правила слотов боевого духа (`zone::skills::battlefairy`, MATCH по шапке
владельца; все VA): проверка уровня `0x0042E7A0–0x0042E81C`; снятие/установка
девяти навыков `0x00430610–0x00430756` и `0x00430760–0x00430921`; пары
несовместимых навыков `m_UnPairSkills` при создании `CBattleFairyContainer`
`0x00504052–0x00504149`; ResetSkill — проверка кандидата `0x00501815–0x00501A8D`,
допуск `0x00501599–0x00501645`, поиск `0x00501663–0x0050178A`, чтение
`0x005017B4–0x00501806`, запись `0x00501892–0x00501AC7`, стоимость
уведомления `0x00501B6B–0x00501B8D`; расход одного предмета — количество
`0x004315E9–0x00431615`, удаление `0x0043169C–0x00431702`; стоимость текста
MP — `CWangsheng::AI` `0x0051E097–0x0051E0E7`.

Текущая реализация: `zone::skills::battlefairyskill` (+ файл-делегат `appserver/skills/battlefairyskill.rs` старого пакета); исходные владельцы — семейство `appserver/skills/*` и базовый `appserver/states/skill.cpp`.

### battlefairysummon — призыв/следование/гибель боевого духа (CPlayer)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `CPlayer::SetWarSoulStaus` | `0x2E190` | MATCH (разведка) | исходное `state==1` публикует around `0xBF930 {400,id}`; нормализация `value==1?{1,1}:{0,0}`; прямая пара raw-записей summoned+state сохранена |
| `CPlayer::ComputeWarSoulXY` | `0x30930` | MATCH (разведка + спотчек дизассембла) | формула ниже |
| `CPlayer::SetWarSoulXY` | `0x2DF50` | VERIFIED_DISASSEMBLY | спотчек: 3×`call GetArea` `0x7BB60` |
| `CPlayer::TellClientMove` | `0x2D4D0` | PARTIAL | xrefs из summon-flow `0x2E3E0/0x2E418` (+`0x52E73/0x918AD/0xB7DA7`); из тел `0x30930`/`0x2DF50` прямого вызова нет |
| wires `0xBF930`/`0xBF92E` | — | MATCH | по разведке cbattlefairycontainer pubs/effector: refusal `0xBF930` around, summon `0xBF92E` |

`ComputeWarSoulXY` `0x30930`: pre-gate `GetSkill(current)` + виртуальный
`+0x28` (на Rust-шве — входной `current_war_soul_skill_restored`),
`dist=|√(dx²+dy²)|`, dead-zone `<0.5`, step-ветка до `5.0`, коэффициенты
`0.265 (>3.75)` / `0.065 (>0.75)` / `0.045`, `step=dist·(k+k)`, per-axis
epsilon `0.1`, оба `fistp` под временным `fnstcw` (x87 RC=truncate). Спотчек
образа: два прямых `call 0x2DF50` в теле `0x30930` (`0x30A81` живая /
`0x30BA6` мёртвая ветви), таблица констант `.rdata 0x24DC00..0x24DC20` —
f32 `0.265/0.065/0.045/0.75` по `0x24DC00/04/10/14` и f64 `3.75/5.0/0.5` по
`0x24DC08/18/20`; единственный caller xref `0x5A107`. Сводка разведки
указывала цель `0x2CF50` — по образу прямая цель `0x2DF50` (исправлено при
переносе).

UNKNOWN: pub-имя `ReviveBattleFairy` не резолвится (inline в CGame-handler;
однозначное xref-основание по `GAP_BF_HP`/153 и `GAP_BF_MAX_HP`/185
недостижимо — поведение остаётся снятой моделью); порядок тела refresh
внутри periodic `CPlayer::AI` — pub префикса не резолвится, зафиксирован
PARTIAL; полный tail `SummonBF` за player-частью — у владельца контейнера
(`CBattleFairyContainer`).

Текущая реализация: `zone::skills::battlefairysummon`; исходный владелец PDB `appserver/player.cpp/.h`.

### Навыки семейства: атрибутный октет, BFBaseAttack, transfer, FatalBlow, LifeShield

| Владелец | Символ/RVA | Статус | Факт |
|---|---|---|---|
| атрибутный октет Po/Yu (`0x212..0x219`) | CYujia vs CPojia — 167/167 инструкций | VERIFIED_DISASSEMBLY | единый скелет и константы октета; собственный `End(bool)` 8-fold `0x1246C0` — у координатора; quirk двойного списания у Yumo (расход в Check и повторно в первом AI) сохранён `YUMO_SKILL_ID`-веткой |
| `CLifeShield` (`0x220`) | Begin×3 `0x118190…`, Check `0x118850`, AI `0x118A60`, собственный End(H) `0x11A700` | VERIFIED_DISASSEMBLY (тела буквально) | state ctor `0x1F29C0`, AddCure `0x1F2FD0` зависит от CureState; End CLifeShieldState — hub-lifecycle `lifeshieldstate.rs` старого пакета |
| `CFatalBlow` (`0x21C`) | ctor `0x11E130`, `Summon(shape,shape)` `0x51F640` | VERIFIED_DISASSEMBLY | машинная цепочка summon-хелперов: MasterInfo → cast→CPlayer → GetWarSoulGoods → пермишены → prop 156 sprite → CCH WORD → EM 20015 → new → level WORD (`+0x114`) → 20010/20009/20008/6001/30001 → set center → initialize → region → `0xBF502`; порядок cch/region — машинно новый по CThunder, отличается от godthunder-аудитного |
| `CHuoxieshu`/`CLingzhishu` | (поведение) | VERIFIED_DISASSEMBLY (тела буквально) | Health оставляет одно HP; Mana в Check допускает точную цену, в AI требует остаток ≥ 1; оба AI при нехватке — ZHGS0052 с ценой без масштабирования, Check Health — ZHGS0054 с ценой+1; visual1 при отсутствии предмета повторяется в следующем такте без повторного расхода; восстановление: current → gain из таблицы начала AI → max, при превышении повтор GetMax, одна запись |
| `CBFBaseAttack` (`0x224`) | (поведение) | VERIFIED_DISASSEMBLY (тела буквально) | игрок временно проходит реальный SetTileXY в POINT боевого духа и возвращается в центр исходной клетки, block/area-поля и отмена захвата сохраняются, дробная исходная позиция не восстанавливается |

Текущая реализация: `zone::skills::{battlefairyattribute,battlefairybasemagic,battlefairytransfer,fatalblow,lifeshield}`;
исходные владельцы PDB: `appserver/skills/{pojia,pobing,pomo,pofa,yujia,yubing,yumo,yufa,lifeshield,fatalblow,huoxieshu,lingzhishu,battlefairybasemagic}.cpp`.

## Призыв существ `CSummonSkill` и снаряды монстров

### summoncreatureskill — семья `CSummonSkill` (0x19A/0x19B/0x19C/0x1F9)

Одноимённого класса `CSummonCreatureSkill` в PDB не существует: Rust-файл —
общий путь дериватов `CSummonSkill` (`CSummonCorpseCandle` 0x19A,
`CSummonSkeleton` 0x19B, `CSummonSpore` 0x19C, `CBossFiendSummon` 0x1F9).

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| база ctor | `0x1E0EC0` | VERIFIED_DISASSEMBLY | `[+0x48] = 3`; dtor `0x5E0F20` пишет sentinel `0x7FFFFFFF` |
| Begin-скелет дериватов | — | VERIFIED_DISASSEMBLY | трио байт-эквивалентно (отличия только effect vtable): форвард в Begin базы (всегда 1) → new effect 0xC → `[+0x34]` → effect `VT[0](1)` → CheckCastCondition (слот `+0x64`) → провал: End(0), ret 0; успех: `[+0x4C]=1`, `[+0x50]=0`, ret 1 |
| `CheckCastCondition` | `0x53E8D0` | VERIFIED_DISASSEMBLY | только reuse (`IsRestored` по абсолютному DWORD), затем `SetMoveable(0)`; проверки пути и дальности внутри нет |
| End базы | `0x5E0F40` | VERIFIED_DISASSEMBLY | flag → AfterUseSkill `0x53CF30` (износ оружия только у игрока) → `CSkill::End` `0x4D84C0`; reuse-clock читается после создания и публикации всех существ |
| `Summon` (K-вариант, слот `+0x94`) | трио-фолд `0x53E260`; собственное тело BossFiend `0x52C610` | VERIFIED_DISASSEMBLY | Summon НЕ 4-классовый фолд; AI-фолд `0x53F270` (читатель `K = Query(20010)`). JJ-вариант слота `+0x5C` — stub `xor eax, eax; ret 0xC` у базы и трио/BossFiend: семантики 4-му аргументу нет; живой JJ только `CSpiderMist::Summon` `0x541140` |
| тело Summon | (в теле) | VERIFIED_DISASSEMBLY | MasterInfo (player-поля — только у игрока), цикл по K существ: `lifetime = Query(30001)` ВНУТРИ цикла, `picture-id = Query(30003)` (у BossFiend `random(3)` ДО нулевой проверки количества → 30003/30004/30005), позиция `GetRandomPosInRange(x−4, y−4, 8, 8)`, `AddSummonedCreature(info, id, x, y, −1, lifetime)`; reuse-clock после |
| wire | — | VERIFIED_DISASSEMBLY | AddToByteArray группы `0x1E47A0` и entry `0xBF502` с `include_child = true` (zone-конверт `summonshape`) |

Пакет визуала `0xBFE01` читает GetDir «как есть»: ни Begins/AI/Summon/
UpdateVisualEffect, ни `CSkill::Begin` поворота не содержат; `set_direction`
в реконструкцию не вводится — generic-хвост диспетчера ставит поворот, но
семья уходит в собственные executors раньше него; внешний поворот монстру
задаёт движение подхода (`CMoveShape::OnMove`,
`zone::regions::moveshape::on_move_wire`), поворот игрока принадлежит
клиенту.

Нормализация переноса (не расхождение): чтения `Query(30001)` и
`Query(30003/4/5)` унифицированы ВНУТРИ цикла создания для обеих ветвей
(monster-ветвь читала их там и раньше) — по машинному порядку тела Summon;
запросы детерминированы, порядок и счёт RNG не меняются.

UNKNOWN: имя слота `+0x2C` тела Summon; константа GUID `0xEF3D9C` в
окрестности тела — INFERRED; ветви 2..0xF jump-таблицы UpdateVisualEffect
вне режимов 0/1.

### Снаряды монстров: direct, path, littlestar, yunshenglightning

| Владелец | Факт | Статус |
|---|---|---|
| `CChuckStone` (0x19D) / `CSkeletonArchery` (0x1A1) | ICF-свёртка классов по RVA: ChuckStone ≡ SkeletonArchery 4 тела `0x5377A0/0x53BF10/0x53BF30/0x569330`; общий End `0x0056A330`; базовые Begin — CAttackSkill `0x5DEB00…` | VERIFIED_DISASSEMBLY |
| `CEnergyBolt` (0x1A0) / `CSnakeBolt` (0x1A5) / `CZombieClaw` (0x1A2) | ICF: EnergyBolt ≡ SnakeBolt ≡ ZombieClaw, общий End `0x53AF50`; базовые Begin — CAttackSkill `0x5DEB00…` | VERIFIED_DISASSEMBLY |
| `CLittleStar` (0x1A4) | ICF: `CLittleStar::End` ≡ `CSevenShootingStar::End` `0x5345F0`, `CheckAttackPath` ≡ `CChainLightning` `0x534650`; AI `0x00535E34` (fallback-координаты фиксации), объектный Begin `0x005DBDBA` (обнуление fallback), player/monster End `0x005355F0` | VERIFIED_DISASSEMBLY |
| `CYunShengLightning` (0x19E) | общий End `0x0057B810` (ICF со SpiderWeb); `CSkill::End` `0x004D84C0` (reuse после очистки полёта); player-выпуск — prepared после эффекта 1 (`0x0053B373`) | VERIFIED_DISASSEMBLY |

Поведенческие итоги (машинно сверены в тех же телах): у direct-снарядов Check
без MP/reuse/блоков пути/самонацеливания, обход клеток после полёта без
допуска/фильтра типа/проверки смерти/дедупликации, NULL S на входе AI — два
End(1) после удара; ChuckStone форсирует длину пути MAX. У path-снарядов путь
форсирован MAX и сохраняется после выпуска, обработка по одной клетке через
`start+delay+flying·position`; область 1×1 уровням 1/2, иначе 3×3, сохраняется
между End; позиция начинается с 0 в Begin и не перезаписывается выпуском; у
EnergyBolt нулевая MP-цена не запрещает движение, у SnakeBolt/ZombieClaw —
запрещает. LittleStar: после задержки один путь предельной длины с
публикацией конечной клетки, периодический обход до первой `BLOCK_UNFLY`,
один вызов legacy RNG на цель, x87-прибавка из целых свойств с f32-константой
и усечением к нулю; объектный Begin обнуляет fallback (+0x24/+0x28), поэтому
до построения пути он равен (0,0). YunShengLightning: нулевой
`SKILL_USAGE_USER_MP_LOSE` отклоняет player-cast; нулевой `base_element`
монстра (Windows `CMonster::GetAddElementAtk` возвращает ноль даже для
приручённого) — ровно один RNG-вызов.

Текущая реализация: `zone::skills::{summoncreatureskill,directprojectile,pathprojectile,littlestar,yunshenglightning}`;
hub-делегаты — `appserver/skills/{summoncreatureskill,directprojectile,energybolt}.rs` старого пакета;
исходные владельцы PDB: `appserver/skills/{summoncreatureskill,summoncorpsecandle,summonskeleton,summonspore,bossfiendsummon,chuckstone,skeletonarchery,energybolt,snakebolt,zombieclaw,littlestar,yunshenglightning}.cpp`.

## State-касты: пятёрка, heal-квартет, rage-семья и self/zone-касты

### statecast — общий wire-кадр visual

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| `UpdateVisualEffect@CRageBreakEffect` | VA `0x59FB90` | VERIFIED_DISASSEMBLY | DWORD-форма отказов `[dword 0][byte mode]` принадлежит Fury/RageBreak; у пятёрки и heal-квартета — BYTE-пара `[0, mode]` только игроку; общий кадр `0xBFE01` с around-веткой mode 1 |

### selfcast — self/zone-касты (якоря семьи)

| Владелец | RVA | Статус | Факт |
|---|---|---|---|
| CBlind | Begin `0x16DA30`, AddBlindState `0x16E500` | VERIFIED (разведка) | AddBlindState → `new 0x3C` + ctor CRushState2 `0x5F12E0`; codec 8-байт (`zone::effects::blind`) |
| CEnergyHolding | Begin `0x149BF0`, Check `0x14A190`, AI `0x14A4A0`, state ctor `0x1EC410`, AddEnergy `0x1EC490`, GetRemainedTime `0x201200`, skill End(H) 3-fold `0x1502F0` | VERIFIED | — |
| CRoar | Begin `0x14A7D0`, AI `0x14B060` | VERIFIED | окно `roar_bounds` VA `0x54B1CC..0x54B23E`; CRoarState Serialize 5-fold `0x1F65F0` (с heal-квартетом) |
| CPillar | Begin `0x16FA00`, AI `0x170110`, state Begin `0x1F4B60` | VERIFIED | — |
| CCallosity/CCallosity2 | — | VERIFIED | собственные Check/AI; методы состояний почти полностью попарно folded (9 методов, Restart-fold с CPromotionState `0x1FD450`) |
| CSoulMirror | GetScope/Length/Height `0x1A40D0/0x1A4120/0x1A4150`, CalculateAttackPower `0x1A4A30`, Attack `0x1A4BF0` | VERIFIED | End(H) 13-fold `0x146090` — общий CStateSkill tail нескольких навыков |

### cure — `CCure` (0x131)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| Begin / DoesTargetEffective | `0x1AD3A0` / `0x1AD590` | VERIFIED | собственное DoesTargetEffective — NULL S с fallback U |
| `CastCure` | `0x1ADB10` | VERIFIED | порядок обхода позиций вперёд и расход RNG на каждую подходящую |
| AI | `0x1AE110` | VERIFIED | — |
| codec CCureState | Serialize `0x1F51E0`, Unserialize `0x1E9AC0` | VERIFIED | 8-байтный fold |

`cure_threshold`: `FILD` unsigned-модификатора с поправкой 2^32, затем
`FMUL 0.01`, `FIMUL` свойства игрока и `FISTP DWORD` перед DWORD-арифметикой.
Новый CureState начинает действие до End первого прежнего Cure; после замены
нет дополнительного UpdateProperty.

### hearten — `CHearten` (0x144)

Begin `0x150F30`, AI `0x151950`; состояние — общий AI-fold `0x1D5BA0`
(VERIFIED). AI без задержки: первый проход расходует MP, задаёт
прерываемость/направление и сразу накладывает состояние; дикий монстр не
повозки меняет сохранённые type/id цели на U — после замены игрок может
усилить только игрока; прежний первый state144 — End + dtor той же позиции,
новый — Begin(U,S) → append → UpdateProperty цели → End(1).

### promotion — `CPromotion` (0x142)

Begin `0x1686A0`, AI `0x169110`; Restart состояния — fold `0x1FD450`
(VERIFIED). U без region-link завершает навык до проверки смерти S; после
delay повторного пути нет; первый прежний CPromotionState получает только
Restart; End возвращает движение фактическому U.

### godbless — `CGodBless`/`CGodBless2` (0x12F/0x145)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `CGodBless` Check | `0x1B02A0` | VERIFIED | не-Player возвращает 1 ДО MP/Move0; arg-null → тихий 0; cost0 → тихий 0; signed-diff → visual7/GS0288 с number или Move0 → 1 |
| `CGodBless` AI / `CGodBless2` AI | `0x1B0480` / `0x150990` | VERIFIED | GodBless2 требует именно Monster при каждом входе AI, иначе visual10/GS0305 + End0 |
| End(H) 3-fold | `0x1502F0` | VERIFIED | общий хвост семьи |

Порядок чтений: MIN_COEFF→MIN→MAX_COEFF→MAX→ELEMENT_COEFF→ELEMENT (каждое
unsigned wrapping-произведение с 0.01f сохраняется в f32, константа — отдельным
f32); значения живут через удаление прежнего состояния, затем PERSIST→FISTP
ELEMENT/MAX/MIN→ctor→primary Begin→append→безусловный UpdateProperty; выпуск
— End1 даже при отказе state Begin. Обычный неприручённый монстр без
Carriage AI заменяется свежим U с записью базовых type/id S — GodBless2 после
такой замены может отвергнуть уже изменившуюся S на следующем AI.

### heal — квартет `CHeal/CHeal2/CSuperHeal/CSuperHeal2` (0xD3/0xE3/0xD9/0xE4)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| якорь AI CHeal (FREQ-ключ) | `0x581861` | VERIFIED_DISASSEMBLY | прямое `QueryProperty(6001)` перед `QueryProperty(10002)` во всех четырёх AI: `TARGET_AFFECT_FREQUENCY` = 6001, не 5002 прежней реконструкции (исправлено при переносе) |

Порядок AI: MP → OnChangeStates → CAN → направление → visual0 → delay →
visual1 → формула `coeff · weapon_level · 0.01 + const` (FISTP) →
QueryProperty(6001) → QueryProperty(10002) → ctor state(J,J) → Begin(U,S).
Прибавка Player: COEFF→живой уровень оружия, unsigned wrapping-произведение с
0.01f в f32, CONST отдельным f32, значение усекается FISTP после End/dtor
первого прежнего состояния; у источника другого типа прибавка нулевая.
SuperHeal заменяет только Heal; SuperHeal2 хранится у выбранной S, но primary
Begin получает U/U. Выпуск всегда End1; ранние отказы — End0. PLAYER MP0 —
тихий отказ; препятствия не запрещают лечение; GetTargetPath читается
безусловно даже при самоцели.

### ragebreak — `CRageBreak` (0x6E) и `CRageBreakState`

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| ctor навыка / vtable | `0x59F880` (0x54 байта, фабричный индекс `0x15`, `CGoodsContainer::Add` `0x469C4B`) / `0x65BDB4` | VERIFIED | — |
| Begin триада / dtor | `0x59F9E0`/`0x59F910`/`0x59FAD0`; `0x59FAB0` | VERIFIED | — |
| CheckCastCondition | `0x59FF10` | VERIFIED | reuse `0x2715` (visual13+GS0278); RTTI-гейт игрока; RP `query(3)==0` — тихий отказ без visual; signed-дефицит RP — visual8+GS0289 с суммой; успех — `0x4CCEE0(0)` |
| AI | `0x5A00F0` | VERIFIED | `[+0x4C]==0` — чистый возврат; таблица и U обязаны; `IsDied(U)` → visual2 + End(1); первый проход `[+0x50]==0` расходует RP (дефицит → visual8+GS0289+End0), CAN `0x2716` → `[+0x3C]`, visual0; задержка `0x2711` абсолютная unsigned |
| порядок состояний | (в теле AI) | VERIFIED | End+dtor первого прежнего 0x6E → `new CRageBreakState` (ctor `0x5FD1C0`; gain = `query(105)`, keep = `query(0x2712)`; Begin(U,U), append в хвост) → свип девяти id {0x138, 0xD2, 0xC9, 0x67, 0x192, 0x191, 0x198, 0x199, 0x1A6} (End+dtor+обнуление) → у первого 0x131 только End() (без dtor и обнуления) → `new CCureState` (ctor `0x5E9E50`; keep = `query(0x2712)`; Begin(U,U), append) → UpdateProperty `vcall+0x9C` → End(1) |
| End (общий хвост) | `0x546090` | VERIFIED | DoesTargetEffective `0x5AFCE0` (всегда 1); `CRageBreakEffect` vtable `0x65BE48`, UpdateVisualEffect `0x59FB90` |
| `CRageBreakState` ctor | `0x5FD1C0` / `0x5FD240` (default) | VERIFIED | два аргумента: gain → `[+0x38]`, keep → `[+0x40]`; `[+0x8]` и `[+0x3C]` обнуляются; vtable `0x6612B4` |
| `CRageBreakState` Begin триада | `0x5FD370`/`0x5FD2C0`/`0x5FD5C0` | VERIFIED | часы читаются только при U; loop1 создан без Update |
| `CRageBreakState` AI / End / Restart | `0x5EA4C0` / `0x5FD420` / `0x5FD450` | VERIFIED | строгий unsigned-срок `[+0x2C]+[+0x40] < now` → End; End — ICF `CTeamState::End` (удаление через `0x4CDAB0` именно этого экземпляра); Restart — только `[+0x2C] = timeGetTime()` |
| OnUpdateProperties / OnChangeRegion | `0x5FD480` / `0x5D9BA0` | VERIFIED | последняя — только запись региона User |
| GetRemainedTime / Serialize / Unserialize | `0x605E10` / `0x5E7330` / `0x5FD660` | VERIFIED | unsigned-граница и повторное чтение часов; `CRageBreakStateVisualEffect` `0x661300`, Update `0x5FD690` |

`CRageBreakState` ICF-разделяет Serialize/OnUpdateProperties/Unserialize/AI с
`CFuryState` — общие live-callbacks AttackGain-семьи
(`begin_primary_attack_gain_state`, `update_attack_gain_state_properties`)
оформлены в `ragebreakstate` один раз; прежний `appserver/skills/furystate.rs`
им делегирует. Consume для ThunderSlash: первый ID-0x6E слот без RTTI/ended —
End + destructor свежего остатка той же позиции, без чтений часов или
UpdateProperty сверх callbacks самого завершения.

### bossbluefurystate — `CBossBlueFuryState` (0x1F7)

| Функция/symbol | RVA (VA) | Статус | Факт |
|---|---|---|---|
| ctor | `0x5E8A60` | VERIFIED | arg1 factor → `[+0x38]`, arg2 keep → `[+0x3C]`, arg3 weak → `[+0x40]`; ID `0x1F7`, без чтения часов |
| Begin триада | `0x5E8B60`/`0x5E8C30`/`0x5E8ED0` | VERIFIED | объектная с NULL U → ret 0; base Begin, затем SetMoveable(0) и SetFightable(0) на S, затем `new` loop=1 visual `CBossBlueFuryStateVisualEffect` без немедленного пакета |
| AI | `0x5E8D50` | VERIFIED | часы №1 строго позже `[+0x40]+[+0x2C]` → unlock обоих запретов на sufferer КАЖДЫЙ проход, без one-shot; часы №2 строго позже `[+0x3C]+[+0x2C]` → vcall End |
| End | `0x5E8D10` | VERIFIED | visual(1)? → GetSufferer → `CMoveShape::RemoveState(this)` `0x4CDAB0` → SetMoveable(1) → SetFightable(1) на том же sufferer |
| Restart | `0x5FD450` | VERIFIED | только `[+0x2C] = timeGetTime()` |
| OnUpdateProperties | `0x5E8DC0` | VERIFIED | GetSufferer → visual(0) → sufferer type `0x258` + RTTI CMonster: max живым getter `vcall+0xE8` → trunc-gain → `vcall+0x1AC`, затем min живым getter `vcall+0xE4` → gain → `vcall+0x1A8`; иначе ret 1; порядок maximum → minimum, каждый процент от живого getter после предыдущей прибавки |
| GetRemainedTime / Serialize / Unserialize | `0x5D5F30` (ICF) / `0x5E7330` / `0x5D6190` | VERIFIED | загруженный weak_time остаётся 0 |

Диспетчерский restart моделирует путь `Begin(NULL, holder)` загрузки: base
Begin сохраняет timestamp/user, запреты и loop=1 visual повторяются, готовая
запись и её ключ не заменяются. Узкая достижимость нескольких записей 0x1F7
снимается машинным продувом навыка (`bossbluefury`).

### seal — `CSeal` (0x138)

Живые различия от общего TargetedProjectile (полёт — hub-владение, не
переносится): Check принимает лишь S типа CMonster (visual-отказ 10 с
GS0317), общие reuse/путь (именованная преграда GS0295), MP/Move0 только у
Player. Impact пропускается при Cure; общий DirectElement-профиль — generic
MasterInfo, Player-only EM, один RNG, расширенная EM/FISTP-прибавка с
f32-константой; уровни читаются S→U даже после фатального попадания; новый
SealState создаётся до полного End/dtor прежнего ID и занимает его место;
State Begin получает только keep-time.

Текущая реализация: `zone::skills::{statecast,selfcast,cure,hearten,promotion,godbless,heal,ragebreak,ragebreakstate,bossbluefurystate,seal}`;
hub-реализация швов — делегаты `appserver/skills/{statecast,selfcast,seal,...}.rs` старого пакета;
исходные владельцы PDB: `appserver/states/stateskill.cpp` и совместимые `appserver/skills/*` перечисленных владельцев.

## Рывки, self/zone-владельцы и прочие state-владельцы

### dash/flash/littleflash — рывки

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| AI-стадии CFlash | (тело `appserver/skills/flash.cpp`) | VERIFIED_DISASSEMBLY | порядок GetTargetPath → direction → path → player-only weapon addon==2 → RageBreak id `0x6E` → MP → RP → OnChangeStates → teleport back → VE(1) → condition=1 → attack-фаза `condition && !attacked` (region else — End(0) без VE) → tick ≤ started+10009 → pending; иначе SetMoveable(1) → VE(3) → End(1) |
| visual CFlash | (тело) | VERIFIED_DISASSEMBLY | прямой 16-switch с `{0→1, 1→2, 3→3}` и personal-множеством `{2,4,7,8,10,11,13,14,15}`; GS-строки 0278/0288/0289/0301-0304 байт-сверены |
| LittleFlash/LittleFlash2 | (тела `littleflash.cpp`, `littleflash2.cpp`) | VERIFIED_DISASSEMBLY | LF — MP→OnChangeStates до повторного weapon (GS0292), CAN=raw, delay → VE(1) до SetTileXY(back), финал delay+10009; LF2 — sufferer опционален, gate assigned region, не требует занятую клетку и очищает одиночный закрытый выход (`clear_single_blocked`); GS 0278/0288/0290/0292/0302/0309 байт-сверены |

Сознательные отклонения от native-UB (сохранены): `check_dash_path`
обращается к `back()` пути даже после подрезки `truncate(maximum)` до нуля —
пустой путь остаётся безопасным отказом без выдуманной клетки назначения.
UNKNOWN: порядок геттеров X/Y (сквозная согласованность подтверждена), место
push в Attack-списке CFlash (низкий риск), потребители raw CAN `available`.

### rush — Rush/Rush2 (0x73/0x7C)

VERIFIED_DISASSEMBLY: Rush — MP до RP с частичным расходом, weapon category 1
(GS0287), GetTargetPathWithLength, distance-гейты `> max` Rush / `>= max`
Rush2, snapshot типов `{400,600,601,602}`, first-attack контроллер только у
Rush, уровни movzx → scaled keep (query 10002 / линейный scale с clamp),
AddRushState общий (prev по ID → End + освобождение слота → Begin → append →
knockback); Rush2 — безусловный virtual `OnBeenAttacked(false)` хвост после
состояния и отбрасывания; visual — вторая remap-таблица байт
`{0→act1+dir, 1→act2, 2/7/8/13/14 персональные}`, GS 0278/0287/0288/0289/0302
байт-сверены; тела RushState/RushState2 — разделяемый алиас `BlindState<ID>`,
payload 8 байт. OnAction рывных состояний пустой: Defense их не снимает (в
отличие от собственно Blind).

Сознательное отклонение от native-UB: AI держит impact только в локальных
переменных первого входа — повторный вход с condition1 читал бы
неинициализированную клетку; повторный вход завершается безопасным отказом.
UNKNOWN: второй lock RushState::Begin (fight-lock по форме), потребители raw
CAN `available`.

### roar/cpillar/callosity/energyholding — self-касты

| Владелец | Якорь | Статус | Факт |
|---|---|---|---|
| CRoar (0x83) | Begin `0x14A7D0`, AI `0x14B060`; окно `roar_bounds` VA `0x54B1CC–0x54B23E` | VERIFIED | обход берёт живые X/Y источника, окно 5×5 ограничено размерами региона включительно; X — внешний цикл, клетка заново вызывает одиночный GetShape после предыдущего наложения; нет снимка целей/дедупликации/RP; End(1) после обхода даже без целей; отказ Check — visual2 перед End(0) |
| CPillar (0x74) | Begin `0x16FA00`, AI `0x170110` (запросы factor→persist VA `0x0057033C–0x0057039B`) | VERIFIED | переключатель: первый слот ID74 снимается без создания нового состояния; factor — unsigned × f32 0.001 в расширенной точности; ресурсы CPlayer не читаются у чужого CMoveShape (непроверенное приведение native заменено безопасным отказом) |
| CCallosity/CCallosity2 | — | VERIFIED | собственные Check/AI, различаются ID/таблицей/восстановлением; первый AI всегда выполняет GetMP→query→SetMP, затем GetRP→query→SetRP, включая нулевые цены; signed-проверка RP не откатывает списанный MP; первый непустой ID75/7D завершается без RTTI/ended-фильтра; UpdateProperty и при отказе Begin; непроверенный native доступ к ресурсам заменён безопасным отказом |
| CEnergyHolding (0x89) | Begin `0x149BF0`, Check `0x14A190`, AI `0x14A4A0` | VERIFIED | Check — reuse, оружие категории 2, signed MP и RTTI первого state 0x89; отсутствие состояния не подменяется нулём зарядов при пределе; state ctor `0x1EC410`, AddEnergy `0x1EC490`, GetRemainedTime `0x201200`; InverseChopped выбирает первый непустой слот ID89 включая ended (End + dtor даже при несовпадении RTTI); DB 12 байт (ID/уровень/заряды), процент не сохранён — Unserialize оставляет 0; клиентские remaining/additional = 0 |
| `CSoulMirror` (0x13C) | GetScope VA `0x005A40D0`, GetLength/GetHeight VA `0x005A4120/0x005A4150`, AI VA `0x005A4D10`, CalculateAttackPower `0x1A4A30`, Attack `0x1A4BF0` | VERIFIED | GetScope задаёт фронтальную линию ширины `2·level − 1` в таблицах 3×3/5×5/7×7; клетки X→Y не собираются заранее — синхронный контакт меняет следующий снимок; пустая проходимая клетка создаёт CSummonedCreature с fresh Master (country 0); формула directelementattack — один RNG, без damage modifier/RP/CCH/второго RNG |

### blindstate — 8-байтный lifecycle Blind/lock-семейства

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| codec семейства | Serialize `0x1F51E0`, Unserialize `0x1EAAC0`, GetRemainedTime `0x1F2CD0`, AI-fold `0x1D5BA0` | VERIFIED | 8-байтная запись (ID/remaining); часы читаются перед remaining при Load и после ID при Save |
| ctor CBoaLockState / CBlind | `0x5FB560` / `0x607380` | VERIFIED | машинно НЕ одно тело — общее у семейства: данные/codec/AI, не конструктор |

Объектный Begin требует S; NULL U сохраняет timestamp; Visual Update(0)
предшествует move/fight-lock и публикации; перезапуск обновляет visual и S,
не заменяя payload и срок; End: visual → актуальная S → fight-unlock →
move-unlock → RemoveState того же объекта; отсутствующая S не снимает запреты
держателя; строгий wrapping deadline действует и при нулевом сроке; общие
AI/End обслуживают также KnockOut/SpiderWeb/Seal/Strike/KnightCut/BoaLock;
OnAction не объединён: Blind/KnockOut/Seal/KnightCut заканчиваются при
Defense, Rush/Rush2/SpiderWeb/Strike/BoaLock — ничего не делают.

### selfstate — правила self-state семьи

VERIFIED (машинно): Check после reuse не даёт не-Player Move0, у игрока MP0 —
тихий отказ; щиты требуют Player и допускают Move0 даже при MP0; GS0278/
GS0279/GS0288 байт-сверены. AI фазы: MP → signed DWORD-проверка → SetMP →
OnChangeStates → CAN → visual0 → condition → unsigned start+delay → visual1.
Смерть U у пяти усилений — visual2/End(1); ManaShield её не проверяет и
удаляет первый встреченный ID `0x141`; Agility2 заменяет только первый
`0x81`, постоянное семейство завершает все `0xDA/0xDB/0xDC`. Бонус и persist
читаются после завершения старых; Agility2 затем дополнительно читает
persist. Visual object Begin — loop1 постоянные / loop0 временная Agility2;
BFE03/BFE04 несут time=0 (клиентский остаток — у Agility2) и additional=0.
Досверка verify-selfstate (дизассемблинг той же пары, identity match
заново, `.local/verify-selfstate/`): `CNatural::AI` `0x16C0B0`,
`CRapture::AI` `0x16CBB0`, `CMachineShield::AI` `0x1671C0` — VERIFIED-MATCH,
точный клон-шаблон эталона `CManaShield::AI` `0x169DF0`. Natural/Rapture
проверяют `IsDied(U)` → visual2/End(1); щиты смерть не проверяют. Скан
состояний: Natural завершает все `0xDA/0xDB/0xDC`, Rapture — все (порядок
сравнений `0xDA/0xDC/0xDB`), MachineShield — первый непустой `0xDE`
(ManaShield — первый `0x141`). ctor-маппинг MachineShield: usage
`10002`→keep (state `+0x38`), `10010`→life (`+0x3C`), `20024`→hp-фактор
(`+0x40`, WORD), `20025`→mp-фактор (`+0x42`, WORD); MP-отказ —
visual7+GS0288 с ценой usage 2. Natural Q(`112`) → `CNaturalState(G)`
`0x5F36D0`; Rapture Q(`125`) → `CRaptureState(G)` `0x5F3BC0`.
`CDaubPoison::AI` `0x166690` тем же шаблоном подтверждён (recon-de VERIFIED
действует). Клиентское чтение кадров — UNKNOWN.

### wangsheng — `CWangsheng` (0x221)

конструктор ID и vtable VA `0x0051D4E0–0x0051D514`, AI через слот `+0x90` VA
`0x0051DBD0`, участок лечения VA `0x0051E01D–0x0051E043` (VERIFIED). State
owner 0x221 этим навыком не создаётся; живые callbacks сохранённого
CWangshengState — hub-lifecycle прежнего `wangshengstate.rs`. MP списывается
у equipment[10] общим setter с reload fairy-проекций; повторная проверка
GetWarSoulGoods; отказ сохраняет списание без отката; Serialize не подавляет
BF918 при false; после visual1 — wrapping-сумма HP через SetHP и
OnChangeStates.

### immediate — цикл immediate-состояний (15 навыков)

VERIFIED_DISASSEMBLY: общий Begin-цикл пятнадцати навыков выполняет Check
чтением `[skill+0x64]`, при отказе — End(0), при успехе — phase=1; reuse, MP,
visual и Move в цикле отсутствуют. AI читает свойства до GetU и допускает при
NULL U подстановку GetS, кроме Swordship, которому нужен именно U.
`CTaiJi::AI` VA `0x005AF770` и `COrigin::AI` VA `0x005AEA70`: создание и
первичный Begin нового состояния до поиска первого старого ID, затем End
старого, destroy свежего остатка и установка в ту же позицию, UpdateProperty
и End(1). Три Enlarge исполняют End первого ID до чтения прибавки, добавляют
новое состояние в конец, UpdateProperty безусловно даже при отказе Begin;
Swordship1-4 завершают AI вызовом End(0), включая успешную установку;
остальные immediate-семейства — End(1) включая отказ Begin состояния.
Swordship читает MIN перед MAX. UNKNOWN: writer/reader тел
Serialize/Unserialize состояний (шапки данных — `zone::effects`).

### healstate — живые состояния heal-квартета

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| AI/End/Unserialize fold | `0x1EEDF0`/`0x1EEBA0`/`0x1EEC70` | VERIFIED | единый payload, различия только ID |
| Serialize 5-fold | `0x1F65F0` | VERIFIED | с CRoarState |

Порядок: завершается первый прежний ID без RTTI/ended-фильтра (SuperHeal
удаляет D3, остальные — собственный ID), затем новый экземпляр; Begin читает
часы, сохраняет U/S, loop=1 visual до append; счётчик после BeginVisual;
Begin/End не отправляют BFE03/BFE04; запись SuperHeal2 остаётся у выбранной
цели, U/S — заклинатель; End удаляет собственный указатель через свежего S
без записи ended; SetRegion меняет только регион S; Restart сохраняет
U/старт, назначает S=holder, заново начинает visual и обнуляет счётчик. AI
разрешает S и проверяет смерть до часов; Promotion читается перед первым
clock; строгий unsigned wrapping срок — не более одного тика; Count++
предшествует свежим HP/MAX; полный unsigned gain умножается на сохранённый
f32-множитель и усекается FISTP без промежуточного f32; HP складывается с
DWORD-переполнением; OnUpdateProperties возвращает 1 без эффектов; native
читает WORD первого ID без RTTI — чужой layout не имитируется.

Текущая реализация: `zone::skills::{dash,rush,flash,littleflash,roar,pillar,callosity,energyholding,energyholdingstate,blindstate,soulmirror,selfstate,wangsheng,immediate,healstate}`;
исходные владельцы PDB: `appserver/skills/{flash,littleflash,littleflash2,rush,rush2,roar,pillar,callosity,callosity2,energyholding,energyholdingstate,blindstate,boalockstate,soulmirror,agility,agility2,natural,rapture,daubpoison,manashield,machineshield,wangsheng,taiji,origin,enlargefullmiss,enlargemaxhp,enlargemaxmp,swordship{,2,3,4},wuxing*,healstate,healstate2,superhealstate,superhealstate2}.cpp`.

## `CBaseAttack` и инфраструктура состояний/исполнения

### baseattackruntime — исполнение `CBaseAttack` (навык 1)

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| vtable класса | `0x25CB0C` | VERIFIED | класс снят машинно целиком |
| `AI` | `0x001B39B0` | VERIFIED | гейт active → props → user → dead-check → Begin-стадия ровно раз (direction/visual(0)/stage=1) → fall-through в тот же тик → delay-wrap unsigned → visual(1) → Attack со свежим GetSufferer ПОСЛЕ visual → End(1) |
| `Restart` | `0x00113E00` | VERIFIED (тело) / UNKNOWN (достижимость) | диспетчер vtbl+0x20 не установлен; повторный Begin не считается его реализацией |
| свойства/часы | — | VERIFIED | ключи 5003/10001/10006/20001; `WINMM!timeGetTime` |
| `attack` | — | VERIFIED | guards включая отказ object type 500, master fill, общий virtual +0x15C(..., false), затем безусловный caller `IncreaseRp(1,0)`; у `CMonster` IncreaseRp — пустое тело (ICF-факт) |
| `publish_base_attack_visual` (`0x000BFE01`) | — | VERIFIED | 16-входный switch: area 0→u8=1, 1→u8=2 с target/dest, personal {2,7,10,11,13,14,15}→[u8=0][u8=mode]; режимы 3,4,5,6,8,9,12 — без пакета |
| terminal/abort | — | VERIFIED | End(1) изнашивает оружие только у игрока и фиксирует reuse; End(0) — без эффектов; OnChangeRegion — End(0) |

### state/ — арена, кодек и каталог состояний

| Функция/symbol | RVA | Статус | Факт |
|---|---|---|---|
| `RemoveState` | `0x004CDAB0`, `0x004CDB20` | VERIFIED (сохранённый RAW) | append/remove/insert записей — владелец порядка `state/mutations` |
| `AddExStatesToByteArray` | `0x004D10F0` (moveshape.cpp:1814) | VERIFIED (сохранённый RAW) | уплотнение позиций арены в начале mutable GameSave соответствует этому проходу и UpdateAbnormality |
| `UpdateAbnormality` | `0x004CFD00` | VERIFIED | общий, остаётся у владельца hub moveshape (`states/state.rs`) |
| `StartAllStates` | `0x004CE050` | VERIFIED | вызывает Begin(nullptr, holder), не превращая пустой GetUser в держателя — существенно для базового End `0x005DBCE0` |
| `CState` ctor / `DecodeExStates` | `0x005DBCA0` / `0x004D1B18` | VERIFIED | DecodeExStates пишет sufferer type/id держателя, оставляет region=0 из ctor; Object Begin устанавливает текущий sufferer-region |
| RTTI CSpiderMist | `0x0066F16C` | VERIFIED | CSummonSkill→CSkill→CState: активный CSpiderMist не входит в контейнер `m_vStates`, три Begin не вызывают AddState; проверка ID 0x198 внутри CastCure не создаёт state-owner |
| `CStateFactory::Unserialize` | RVA `0x001D7D00`, VA `0x005D7D00` (`?Unserialize@CStateFactory@@SAPAVCState@@PAEAAJ@Z`) | VERIFIED | чтение declared count, switch по ID записи с переходом к привязке владельца; таблица переходов начинается с `0x32` |
| `CState` Begin / `CSkill` Begin/End / `CSkill::IsRestored` | VA `0x005DBD70`/`0x005DBDD0`, VA `0x004D83E0`/`0x004D84C0`, VA `0x004D8220–0x004D825F` | MATCH (по шапке владельца) | lifecycle-база `zone::skills::lifecycle`; native Begin сохраняет сторону при NULL и обновляет часы только при U; point-ветвь очищает S; IsRestored читает уже прочитанный delay |

Особенности контракта арены: порядок добавления и пустые позиции после
удаления принадлежат контейнеру (уплотнение только явное); новый экземпляр
получает новый поколенческий ключ даже при замене в прежней позиции; общий
End вызывается снаружи и может изменить тот же список до перечитывания
позиции. Loaded ctor начинает с ended=true и visual=NULL; runtime-регистрация
фиксирует результат исполненного concrete Begin (ended=false и base visual
из единого callback-каталога, без повторной рассылки); различаются loop=1,
живой loop=0 GodBless2, one-shot Agility2/Promotion и owners без ресурса.
Tian читает 10 байт, но пишет 12 (cache-span ≠ native input-offset);
Serializer заимствует арену в ReadOnly/Save-режиме; только Save допускает
запись вычисленного remaining в текущий ключ до следующей позиции.

### execution/ — запись и исполнения

Payload монстра: End `0x0057B810` пишет только `+0x4C/+0x50/+0x54/+0x58`
перед GetUser; destination у SpiderMist и YunShengLightning не обнуляется
вместе с derived-полётными флагами (VERIFIED по телу End); координаты
базового CState очищаются общим хвостом. Остальное — архитектурный перенос
без новых машинных фактов (порядок Inactive→kernel, `prepare_derived_end`,
`clear_end_paths` в порядке `SkillOwner::end_policy`).

Payload боевого духа (`zone::skills::execution::payload`, MATCH по шапке
владельца): три `End(int)` VA `0x00516FB0`/`0x0051A700`/`0x005222A0` очищают
DWORD-фазу перед visual и AfterUse; два `End(bool)` VA `0x0051BE50`/
`0x005246C0` очищают только BYTE-флаги и не заменяют `End(int)`.
`?End@CChuckStone@@UAEXH@Z` (VA `0x0056A330`) — общий адрес обоих владельцев
(`CChuckStone`/`CSkeletonArchery`), четыре DWORD-зануления `+0x4C/0x50/0x54/0x58`
перед `SetMoveable(true)` и базовым End.

Текущая реализация: `zone::skills::{baseattackruntime,state/{storage,serialization,catalog,mutations,accessors,snapshot,mod},statefactory,execution/*}`;
исходные владельцы PDB: `appserver/skills/baseattack.cpp`, `appserver/moveshape.*`,
`appserver/skills/statefactory.cpp`, `appserver/monster.*`, `appserver/states/state.*`.
