//! Базовая атака монстра и приручённого питомца (`CMonsterBaseAttack`).
//! На время прямого удара настоящий CPlayerAI опубликован в CPlayer:
//! вложенные обработчики смерти видят и изменяют ту же очередь источника.
//! Достигнутый OnSchedule с целью и пустыми active/passive FIFO вызывается
//! до background/passive (CBaseAI::Run 0x004C7D10; проверки CMonsterAI
//! 0x005DCFA2..0x005DCFB0). Первые входы owners сохраняют только Begin;
//! немедленный навык тоже получает общий cast и Attack до первого эффекта.
//! Его End виден обоим проходам; OnFighting снимает завершённое исполнение
//! при следующем входе, не повторяя AI после background-End. Idle и поиск
//! без цели сохраняют отдельные производные пути и требуют дальнейшего
//! согласования полного OnSchedule/OnIdle для всех AI-типов.
//! Сохранённый cast CMoveShape не определяет фазу вызова: продолжение
//! доступно только достигнутому Attack выбранного GetAI и совпадающему
//! зарегистрированному current skill. Пустая FIFO другого AI после смены
//! хозяина проходит собственные Schedule/Idle, не продолжая и не отменяя
//! чужое исполнение. Begin, progress, reuse и End обращаются к экземпляру
//! зарегистрированного навыка по ID собственного dispatch или concrete owner-а;
//! состояние другого навыка не используется как запасное исполнение.
//! OnFighting уже начатого immediate вызывает тот же owner до target/range
//! расписания. Семейство постоянных свойств сохраняет цель общего Begin;
//! выбор U/S и результата остаётся у его AI. Активный и фоновый входы используют один
//! kernel, без visual, Move и reuse-допуска. Фоновый Begin не ставит Attack
//! в очередь; завершение FIFO остаётся следующим active-проходом.
//! MonsterThorn AI без свойств (0x005423E2), MachineryStomp (0x00532836)
//! и LordWiderangingAttack (0x00530366), MonsterRangeAttack (0x00512900)
//! вызывают owner End(0).
//! CBaseAttack (0x005B2E40) не проверяет reuse в Begin: общий хвост
//! не добавляет его сверх таймера OnSchedule. Поворот/старт и проверка
//! дальности перенесены в первый AI его owner-а (0x005B39B0).
//! Общий lookup не поглощает этот отказ живого cast; до Begin свойства
//! по-прежнему необходимы расписанию для расчёта диапазона.
//! Зарегистрированные навыки ниже сохраняют getters диапазона при отсутствии
//! свойств. Новый Begin идёт после диапазона либо Tracing и интервала ИИ;
//! уже начатый навык получает AI без повторного допуска расписанием.
//! Базовые снаряды, FireBolt, FireBall, GodPunishment и Heal используют зарегистрированный цикл
//! игрока/монстра и minimum1/положительный maximum, в том числе при поиске цели.
//! Default в выборе и OnChangeSkill берётся из зарегистрированных навыков
//! CMoveShape (GetDefaultAttackSkillID, 0x004CE240), как при Stiffen.
//! Таблица MonsterProperties задаёт взвешенный выбор, но не заменяет реестр
//! владельца: неуспешно загруженный навык не участвует в выборе default.
//! CMonsterAI::OnSchedule (0x005DCF80) и CPet::OnAttackingSchedule
//! (0x004E9A20) проверяют цель до выбора навыка/RNG. Если GetCurrentSkill
//! не разрешает выбранный ID, выполняется полный OnChangeSkill с IsRestored,
//! затем повторный поиск в реестре. Его отказ вызывает только виртуальный
//! OnLoseTarget, без внешнего SearchEnemy; FIFO-обёртка здесь не исполняется.
//! IsRestored в OnChangeSkill читает уровень зарегистрированного CSkill,
//! как CMonsterAI::OnChangeSkill (0x005DCBC0), а не максимум уровней в setup.
//! Неудачный AddSkill может удалить прежнюю запись: наличие ID в настройках
//! после этого не означает ни восстановленного навыка, ни допустимого cooldown.
//! OnSearchEnemy также получает ID и уровень через GetCurrentSkill: дальность
//! навыка берётся из того же зарегистрированного объекта, что и IsRestored.
//! Повторный максимум уровней setup не восстанавливает удалённый AddSkill-ом
//! объект и не подменяет фактический уровень оставшегося владельца.
//! Отсутствие навыка не блокирует общий dispatch поиска. CGladiator
//! (0x006112C0), CSmartGladiator (0x00610AC0), CJiuMai (0x0060AD10) ищут
//! по guard range без GetCurrentSkill; требование навыка остаётся только
//! у ветвей, использующих его минимальную дистанцию. Общая проекция этой
//! дистанции не вводит второй реестр и не меняет порядок обхода кандидатов.
//! Ранний HasTarget в CSmartGladiator (0x00610B06) и CJiuMai (0x0060AD5A)
//! завершает OnSearchEnemy до разрешения региона и обхода целей. Уже заданная
//! цель не заменяется; новый шаг отхода и передача цели близнецу не выполняются.
//! Общий caller по-прежнему завершает достигнутое FIFO-событие после возврата.
//! Фабричный CMonsterAI наследует пустой CBaseAI::OnSearchEnemy (0x0047B150,
//! mov eax,1; ret), а не проверку смерти/области/навыка. Этот dispatch
//! завершается до разрешения ShapeView и area_index; его прежняя проверка
//! в конце общего поиска ошибочно зависела от инфраструктуры региона.
//! Начало атаки получает ID/уровень из этого же реестра; после Begin источником
//! обоих значений является MonsterBaseAttackDispatch. Повторный проход не
//! перечитывает максимум setup и не смешивает сохранённую цель с другим уровнем.
//! WORD-уровень monster-dispatch проверяется до Begin без усечения; настройка
//! монстра изначально хранит u16, а общий реестр CMoveShape допускает i32.
//! Выбор не требует реализации всех записей setup: CMonsterAI::OnChangeSkill
//! (0x005DCBC0) вызывает selector непосредственно, затем проверяет только
//! выбранный объект. Пустой список также проходит исходный RNG/default.
//! Предварительный запрет по всему списку удалён: второй внешней виртуальной
//! ветви в CGame нет. Неподключённый выбранный skill всё ещё возвращает отказ
//! перед Begin; это оставшийся конкретный owner, а не успешная атака.
//! Допуск OnSchedule проверяет CMoveShape::can_fight до цели, RNG и Begin:
//! CMonsterAI 0x005DCFB6, лучники/охрана 0x0060B8D7, BossBlue 0x0060A008,
//! BossFiend 0x006095A7, SmartGladiator 0x0061071C. Запрет вызывает только
//! виртуальный OnLoseTarget, без внешнего SearchEnemy. Уже начатый cast
//! проходит OnFighting; эта проверка не подменяет его отдельный End.
//! CJiuMai отличается: 0x0060AC3B при запрете сразу возвращается, сохраняя цель.
//! CPuninessCreature (0x0060F4B0) переходит прямо к Tracing и такой проверки
//! не имеет; исключение относится к первичному AI7, но не к приручённому CPet.
//! CPet vtable 0x00652D0C хранит общий OnChangeSkill (+0x24 → 0x005DCBC0)
//! и SelectAttackSkill (+0x8C → 0x005DD0B0). Поэтому приручение отключает
//! boss/lord-selector и производный restore-delay AI5/AI103, но сохраняет
//! исходный список odds, один RNG и общий default/IsRestored.
//! CCarriage vtable 0x00653F9C наследует те же OnChangeSkill/SelectAttackSkill,
//! а OnSearchEnemy (+0x18) указывает на RET1 0x0047B150. Обе формы повозки,
//! первичная AI24 и auxiliary GetAI, проходят этот dispatch без поиска целей;
//! номер первичного AI разрешается только у активного primary owner-а.
//! CPet::OnSchedule (0x004E9DC0) выбирает собственные Attack/Follow/Stay:
//! сохранённый property.ai не включает связывание Цзюмай, пост стража,
//! стационарный поиск дальности или исключение AI13 из проверки цели.
//! Выбор этих методов использует GetAI, а не игровой tamed sign; нулевой
//! auxiliary pointer не подменяется первичным AI. Признак приручения отдельно
//! сохраняется для combat/scaling и отношений хозяина.
//! CPet::OnAttackingSchedule/OnStayingSchedule проверяют допустимость цели.
//! OnStayingSchedule (0x004E9650) проверяет включительный min/max диапазон
//! текущего навыка до любого concrete Begin, включая immediate-навыки.
//! При выходе за диапазон выполняется OnLoseTarget → SearchEnemy; Tracing
//! в Stay не вызывается. Проверки самого Begin этим не подменяются.
//! Стационарное OnSchedule 0x0060B890 проходит ту же проверку диапазона
//! до диспетчеризации всех навыков, без Tracing и общей проверки прямого пути.
//! Отказ вызывает virtual OnLoseTarget и затем ставит SearchEnemy.
//! Этот OnSchedule проверяет существование/смерть цели, но не IsAttackable;
//! исключение относится ко всей стационарной семье, не только AI13.
//! OnAttackingSchedule (0x004E9A20) ограничивает дистанцию цели от хозяина
//! (при его отсутствии — от питомца) до GetCurrentSkill/OnChangeSkill/Begin.
//! Граница distance >= MaxPetTracingDistance сбрасывает цель и ставит поиск,
//! не расходуя RNG выбора навыка и не затрагивая уже начатое исполнение.
//! Мёртвая цель CPet теряется до IsAttackable: уведомление об уровне и
//! встречный OnLoseTarget относятся только к отказу живой цели от атаки.
//! Обычный virtual OnLoseTarget (0x005DCC30 → 0x004C7DA0) очищает только
//! цель, не вызывает End и не снимает Move. Общий release использует эту
//! узкую операцию; полная отмена монстра остаётся отдельным lifecycle-действием.
//! Отказ базового Begin по reuse (CheckCastCondition 0x00514340) для CPet
//! завершает попытку через OnLoseTarget → SearchEnemy, а не оставляет цель
//! в ожидании. Проверка идёт после Tracing; движение не считается отказом.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/monsterbaseattack.cpp`. Модуль навыка хранит выбор цели,
//! стадии атаки, преследование и исходный физический разброс. Встроенный
//! `GetAddElementAtk` возвращает ноль; диапазон monster element здесь не
//! разыгрывается. Общие защита,
//! применение повреждений и точные пакеты принадлежат узкому
//! `monsterattack`; `CGame` оставляет возврат владельца региона и
//! межвладельческие последствия смерти. RAW вариантов `Begin` сохранён для
//! ещё не подключённого координатного входа монстра; координатный player-вход
//! разрешает цель через существующий `CState::GetSufferer` на каждом такте.
//! Назначенный питомцу NPC остаётся допустимым `CMoveShape` на входе команды,
//! но `CMonster::IsAttackAble` отвергает любой тип кроме игрока и монстра;
//! расписание поэтому выполняет обычный `OnLoseTarget` и ставит поиск заново.
//! Конструктор и ветвь `SKILL_MONSTER_BASE_ATTACK` фабрики подтверждают ID
//! `0x2bd`; навык игрока `1` принадлежит другому модулю и не подменяет этот ID.
//! Проверка reuse в расписании делегируется общему exact `CSkill::IsRestored`:
//! его wrapped DWORD deadline намеренно отличается от длительностей стадий.
//! Активный выбор принимает также пять немедленных состояний, четыре Swordship
//! и пять WuXing: они не блокируют весь список навыков монстра. Concrete owner
//! задаёт эффект, а `OnFighting` завершает активный ход даже после `End(0)`:
//! Swordship устанавливает состояние без reuse, WuXing отвергает type `600`
//! без эффекта и reuse. Это завершение не добавляется фоновой очереди.
//! Все 50 NonFun проходят тот же активный Begin/Attack/End(0) без своих
//! проверок и эффектов. Их minimum равен 1; AutoStart они не используют.
//! Объектный `CBaseAttack` (`1`) получает здесь только Begin расписания;
//! его собственные AI/Calculate/Attack находятся в baseattack и публикуют
//! исходный регион для общих visual/OnBeenAttacked/End. ID навыка остаётся
//! в состоянии и visual, но сам удар сохраняет конструкторские UNKNOWN/1.
//! Различия расчёта родственных владельцев не стираются:
//! `CBaseAttack` (`0x005B3600`) берёт `max(max-min,0)`, MonsterBase/Fast
//! (`0x00514460/0x00513490`) прибавляют единицу, LordFast (`0x00530D60`)
//! использует `abs(max-min)+1` с DWORD-переполнением. MonsterBase/Fast перед
//! critical-roll требуют успешный cast в CPlayer, поэтому на монстре этого
//! RNG-вызова нет; BaseAttack/LordFast выполняют его даже при `GetCCH == 0`.
//! `BaseAttack/MonsterBaseAttack::AI` сравнивают задержку с абсолютным
//! wrapping DWORD deadline (`0x005B3B0E/0x0051497E`), а не с elapsed-time.
//! Player-путь `0x2bd` хранит kernel и reuse в зарегистрированном навыке CMoveShape.
//! `CheckCastCondition` (VA `0x00514340`) требует источник и свойства,
//! проверяет reuse с failure 13 и `GS1143`, но не цель/MP/дальность/путь.
//! Первая AI-фаза проверяет дальность беззнаковым сравнением и поворачивает
//! источник; через delay посылается fire с identity и координатами цели,
//! затем один удар. Пустая цель не блокирует анимацию; self/NPC/недопустимая
//! цель не получают урон и не расходуют RNG. Мёртвая цель даёт failure 2
//! и `End(1)`, превышенная дальность — failure 11 и `End(0)`.
//! `End` (VA `0x005b3010`) не меняет движение и не пересчитывает свойства;
//! только успех изнашивает оружие и фиксирует reuse. Формула player-урона
//! общая с MonsterFastAttack, включая личный критический множитель; защита,
//! RP, смерть и сообщения используют существующий владелец применения атаки.
//! Постройки и ворота проходят war/camp-проверки до расчёта, затем общий
//! OnBeenAttacked формы без отдельного сценария постройки. RP источнику начисляется
//! после возврата из попадания независимо от отказа защиты; NPC не атакуются.
//! Рассчитанный AttackInformation сохраняет конструкторские UNKNOWN/уровень 1,
//! поэтому wire-id не определяет начисление RP конкретного навыка.
//! У NPC нулевой combat HP: общая IsDied-проверка даёт failure 2 и End(1)
//! до начала анимации, а не пустую атаку по истечении delay.
//! Цепочка попадания передаёт Option владельца региона до синхронной смерти.
//! Заимствование базы не переживает эту границу; продолжение заново получает
//! оставшегося владельца, не создавая замену исчезнувшему региону.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp

// ============================================================================
// FUNCTION: CMonsterBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp:111
// RVA: 0x00113B80
// ADDRESS: 00513b80
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp:127
// RVA: 0x00113C50
// ADDRESS: 00513c50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::gameserver::game::ServerRegionOwner;

use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use super::baseattack::{
    BASE_ATTACK_SKILL_ID as COMMON_BASE_ATTACK_SKILL_ID,
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
    SKILL_USAGE_USER_HIT_MODIFIER, time_reached,
};
use super::monsterfastattack::{
    MONSTER_FAST_ATTACK_SKILL_ID, MonsterFastAttackProgress, SKILL_USAGE_FIRST_TIME, SKILL_USAGE_SECOND_TIME,
    fast_attack_fire_message,
};
use super::monsterattack::{
    apply_owned_monster_attack_hit,
    resolve_owned_monster_attack_target,
};
use super::monsterrangeattack::{
    MONSTER_RANGE_ATTACK_SKILL_ID, MonsterRangeAttackDispatch,
    prepare_owned_monster_range_cast,
};
use super::chuckstone::CHUCK_STONE_SKILL_ID;
use super::archery::{ARCHERY_SKILL_ID, execute_owned_monster_archery};
use super::basemagic::{BASE_MAGIC_SKILL_ID as BASE_MAGIC_PROJECTILE_SKILL_ID, execute_owned_monster_base_magic};
use super::firebolt::{FIRE_BOLT_SKILL_ID, execute_owned_monster_fire_bolt};
use super::fireball::{FIRE_BALL_SKILL_ID, execute_owned_monster_fire_ball};
use super::godpunishment::{GOD_PUNISHMENT_SKILL_ID, execute_owned_monster_god_punishment};
use super::godbless::{GOD_BLESS_SKILL_ID, execute_owned_monster_god_bless};
use super::godbless2::GOD_BLESS_2_SKILL_ID;
use super::heal::{HEAL_SKILL_ID, execute_owned_monster_heal, is_heal_skill};
use super::nonfun::{execute_owned_monster_non_fun, is_non_fun_skill};
use super::heal2::HEAL_2_SKILL_ID;
use super::superheal::SUPER_HEAL_SKILL_ID;
use super::superheal2::SUPER_HEAL_2_SKILL_ID;
use super::bossbluefury::{BOSS_BLUE_FURY_SKILL_ID, execute_owned_boss_blue_fury};
use super::bossbluequake::{BOSS_BLUE_QUAKE_SKILL_ID, execute_owned_boss_blue_quake};
use super::bossfiendsummon::BOSS_FIEND_SUMMON_SKILL_ID;
use super::bossfiendpenetrate::{
    BOSS_FIEND_PENETRATE_SKILL_ID, execute_owned_boss_fiend_penetrate,
};
use super::corpsecandleblasting::{
    CORPSE_CANDLE_BLASTING_SKILL_ID, execute_owned_corpse_candle_blasting,
};
use super::corpseptomaine::{CORPSE_PTOMAINE_SKILL_ID, execute_owned_corpse_ptomaine};
use super::energybolt::{ENERGY_BOLT_SKILL_ID, execute_owned_energy_bolt};
use super::fury::{FURY_SKILL_ID, execute_owned_fury};
use super::ragebreak::{RAGE_BREAK_SKILL_ID, execute_owned_monster_rage_break};
use super::immediatestate::{
    check_immediate_state_cast, execute_monster_immediate_state, is_immediate_state_skill,
};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillTermination};
use super::littlestar::{LITTLE_STAR_SKILL_ID, execute_owned_little_star};
use super::lordfastattack::LORD_FAST_ATTACK_SKILL_ID;
use super::lordwiderangingattack::{
    LORD_WIDERANGING_ATTACK_SKILL_ID,
};
use super::machinerystomp::{
    MACHINERY_STOMP_SKILL_ID, WideArcAttackDispatch, prepare_owned_wide_arc_attack,
};
use super::monsterprojectile::{MonsterProjectileDispatch, prepare_owned_monster_projectile};
use super::monsterthorn::{MONSTER_THORN_SKILL_ID, execute_owned_monster_thorn};
use super::knockoutruntime::{KNOCK_OUT_SKILL_ID, execute_owned_monster_knock_out};
use super::promotion::{PROMOTION_SKILL_ID, execute_owned_monster_promotion};
use super::cure::{CURE_SKILL_ID, execute_owned_monster_cure};
use super::hearten::{HEARTEN_SKILL_ID, execute_owned_monster_hearten};
use super::skeletonarchery::SKELETON_ARCHERY_SKILL_ID;
use super::snakebolt::{SNAKE_BOLT_SKILL_ID, execute_owned_snake_bolt};
use super::snowstorm::{SNOW_STORM_SKILL_ID, execute_owned_monster_snow_storm};
use super::spiderpoison::{SPIDER_POISON_SKILL_ID, execute_owned_spider_poison};
use super::spidermist::{SPIDER_MIST_SKILL_ID, execute_owned_spider_mist};
use super::spiderweb::{SPIDER_WEB_SKILL_ID, execute_owned_spider_web};
use super::sporeblasting::{SPORE_BLASTING_SKILL_ID, execute_owned_spore_blasting};
use super::spriteburn::{SPRITE_BURN_SKILL_ID, execute_owned_sprite_burn};
use super::summoncorpsecandle::SUMMON_CORPSE_CANDLE_SKILL_ID;
use super::summoncreatureskill::execute_owned_summon_creature;
use super::summonskeleton::SUMMON_SKELETON_SKILL_ID;
use super::summonspore::SUMMON_SPORE_SKILL_ID;
use super::yunshenglightning::{YUNSHENG_LIGHTNING_SKILL_ID, execute_owned_yunsheng_lightning};
use super::yakshaslash::{YAKSHA_SLASH_SKILL_ID, execute_owned_monster_yaksha_slash};
use super::zombieclaw::{ZOMBIE_CLAW_SKILL_ID, execute_owned_zombie_claw};
use crate::gameserver::appserver::ai::aifactory::{ActiveMonsterAi, MonsterAiKind};
use crate::gameserver::appserver::ai::archer::select_archer_enemy;
use crate::gameserver::appserver::ai::bossblue::{
    choose_boss_blue_attack_skill, select_boss_blue_enemy,
};
use crate::gameserver::appserver::ai::bossfiend::{
    choose_boss_fiend_attack_skill, select_boss_fiend_enemy,
};
use crate::gameserver::appserver::ai::bossidle::queue_boss_idle;
use crate::gameserver::appserver::ai::cityguardwithsword::{
    CitySwordTraceOutcome,
    select_city_guard_enemy, trace_city_sword_target,
};
use crate::gameserver::appserver::ai::fixedpositionarcher::select_fixed_archer_enemy;
use crate::gameserver::appserver::ai::fixedpositionarcher::{
    inherits_fixed_archer_change_skill, queue_fixed_archer_skill_delay,
    queue_stationary_guard_idle,
};
use crate::gameserver::appserver::ai::gladiator::select_gladiator_enemy;
use crate::gameserver::appserver::ai::godsbattlemonster::select_gods_battle_enemy;
use crate::gameserver::appserver::ai::godsbattleguardwithsword::select_gods_battle_guard_enemy;
use crate::gameserver::appserver::ai::guardwithbow::select_guard_with_bow_target;
use crate::gameserver::appserver::ai::guardcountry::select_country_guard_target;
use crate::gameserver::appserver::ai::jiumai::{
    assign_jiumai_target, ensure_jiumai_twin, select_jiumai_enemy,
};
use crate::gameserver::appserver::ai::lord::{select_lord_attack_skill, select_lord_enemy};
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, has_owned_search_enemy, trace_owned_target_state_skill,
    hibernates_without_nearby_players, release_owned_monster_target,
    queue_monster_idle, schedule_attack_interval, select_attack_skill, uses_stationary_attack_schedule,
};
use crate::gameserver::appserver::ai::puninesscreature::search_puniness_enemy;
use crate::gameserver::appserver::ai::pet::{
    PetMasterRef, lose_pet_target_and_search, pet_master_ref, queue_pet_idle,
};
use crate::gameserver::appserver::ai::nationgladiator::select_nation_gladiator_enemy;
use crate::gameserver::appserver::ai::nationcouguardwithsword::select_nation_country_guard_enemy;
use crate::gameserver::appserver::ai::smartgladiator::select_smart_gladiator_enemy;
use crate::gameserver::appserver::ai::stupidarcher::search_stupid_archer_enemy;
use crate::gameserver::appserver::ai::warattackmonster::select_country_war_enemy;
use crate::gameserver::appserver::ai::vilcouguardwithsword::select_village_country_guard_enemy;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::state::{resolve_coordinate_sufferer, resolve_identity_sufferer};
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
pub(crate) const MONSTER_BASE_ATTACK_SKILL_ID: u32 = 0x2bd;

pub(crate) const fn is_player_monster_base_attack(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: MONSTER_BASE_ATTACK_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: MONSTER_BASE_ATTACK_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: MONSTER_BASE_ATTACK_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE | 500 | 1100 | 1200, .. } })
}

fn player_base_attack_outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn end_player_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, runtime: &mut Runtime, success: bool,
) {
    if success {
        game.after_use_player_skill(player_id, MONSTER_BASE_ATTACK_SKILL_ID, runtime);
    }
}

pub(crate) fn finish_player_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime, success: bool,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, MONSTER_BASE_ATTACK_SKILL_ID).map(|kernel| kernel.dispatch()) else { return false };
    end_player_monster_base_attack(game, player_id, runtime, success);
    game.finish_player_skill(player_id, ai, dispatch, if success { SkillTermination::Completed } else { SkillTermination::Cancelled })
}

pub(crate) fn execute_player_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    use super::lordfastattack::{calculate_attack, master_info, send_start};
    let rejected = || player_base_attack_outcome(QueuedSkillExecutionState::Rejected);
    if !is_player_monster_base_attack(dispatch) { return rejected(); }
    let Some((region_id, level, source)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.learned_skill_level(MONSTER_BASE_ATTACK_SKILL_ID, game.skill_factory()), player.shape_view()?))
    }) else { return rejected() };
    let Some(properties) = game.skill_base_properties(MONSTER_BASE_ATTACK_SKILL_ID, level) else {
        end_player_monster_base_attack(game, player_id, runtime, false);
        return rejected();
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(super::basemagic::SKILL_USAGE_CAN_BE_BREAKED);
    if game.player_skill_execution(player_id, MONSTER_BASE_ATTACK_SKILL_ID).is_none() {
        let now = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, MONSTER_BASE_ATTACK_SKILL_ID), reuse, now) {
            game.send_self_state_skill_failure(0x000b_fe01, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS1143");
            end_player_monster_base_attack(game, player_id, runtime, false);
            return rejected();
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now));
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(MONSTER_BASE_ATTACK_SKILL_ID));
        }
        return player_base_attack_outcome(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, MONSTER_BASE_ATTACK_SKILL_ID).is_none_or(|kernel| kernel.dispatch() != dispatch) {
        return rejected();
    }
    let requested = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => resolve_identity_sufferer(game, region_id, target),
        PlayerSkillDispatch::Point { x, y, .. } => resolve_coordinate_sufferer(game, region_id, x, y),
        PlayerSkillDispatch::SelfTarget { .. } => None,
    };
    let target = requested.and_then(|identity| game.base_magic_target_view(region_id, identity).map(|view| (identity, view)));
    if target.is_some_and(|(identity, _)| game.base_magic_target_dead(region_id, identity)) {
        game.send_self_state_skill_failure(0x000b_fe01, player_id, 2);
        end_player_monster_base_attack(game, player_id, runtime, true);
        return player_base_attack_outcome(QueuedSkillExecutionState::Completed);
    }
    let (fallback_x, fallback_y) = match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => (x, y),
        _ => (0, 0),
    };
    let (target_x, target_y) = target.map_or((fallback_x, fallback_y), |(_, view)| (view.tile_x, view.tile_y));
    if game.player_skill_execution(player_id, MONSTER_BASE_ATTACK_SKILL_ID).is_some_and(|kernel| kernel.stage() == SkillStage::Begin) {
        let distance = target.map_or_else(
            || super::baseattack::real_distance(source.tile_x, source.tile_y, target_x, target_y),
            |(_, view)| source.real_distance(Some(view)),
        );
        if maximum_distance != 0 && maximum_distance < distance as u32 {
            game.send_self_state_skill_failure(0x000b_fe01, player_id, 0x0b);
            end_player_monster_base_attack(game, player_id, runtime, false);
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source.tile_x, source.tile_y, target_x, target_y));
        }
        send_start(game, player_id, MONSTER_BASE_ATTACK_SKILL_ID, level);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, MONSTER_BASE_ATTACK_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started = game.player_skill_execution(player_id, MONSTER_BASE_ATTACK_SKILL_ID).map(|kernel| kernel.started_at_ms()).expect("базовая атака хранит начало");
    if !skill_is_restored(started, delay, runtime.now_milliseconds()) {
        return player_base_attack_outcome(QueuedSkillExecutionState::Pending);
    }
    let mut fire = CMessage::new(0x000b_fe01);
    fire.add_byte(2);
    fire.add_long(MONSTER_BASE_ATTACK_SKILL_ID as i32);
    fire.add_short(level as i16);
    fire.add_long(PLAYER_TYPE);
    fire.add_long(player_id);
    fire.add_long(target.map_or(0, |(identity, _)| identity.object_type));
    fire.add_long(target.map_or(0, |(identity, _)| identity.id));
    fire.add_long(target_x);
    fire.add_long(target_y);
    let _ = game.send_player_shape_around(player_id, None, &fire);
    if let Some(kernel) = game.player_skill_execution_mut(player_id, MONSTER_BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
    }
    if let Some((identity, _)) = target
        && !(identity.object_type == PLAYER_TYPE && identity.id == player_id)
        && let Some(master) = game.find_player(player_id).map(master_info)
        && (if matches!(identity.object_type, 1100 | 1200) {
            game.stationary_build_attackable_by_player(player_id, region_id, identity)
        } else {
            game.owned_player_skill_target_attackable(master, identity, region_id)
        })
        && let Some((master, mut attack)) = calculate_attack(game, player_id, MONSTER_BASE_ATTACK_SKILL_ID, level, hit_modifier)
    {
        if matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE | 1100 | 1200) {
            attack.skill_id = super::skillfactory::UNKNOWN_SKILL_ID;
            attack.skill_level = 1;
            game.with_published_player_ai(player_id, player_ai, |game| {
                game.apply_owned_skill_contact(master, identity, region_id, attack, runtime);
                game.increase_owned_player_rp(player_id, true, 0);
            });
        }
    }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, MONSTER_BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    end_player_monster_base_attack(game, player_id, runtime, true);
    player_base_attack_outcome(QueuedSkillExecutionState::Completed)
}

fn is_owned_monster_attack_skill<Runtime: GameMainLoopRuntime>(skill_id: u32) -> bool {
    owned_registered_cast_executor::<Runtime>(skill_id).is_some() || matches!(
        skill_id,
        COMMON_BASE_ATTACK_SKILL_ID
            | MONSTER_BASE_ATTACK_SKILL_ID
            | MONSTER_FAST_ATTACK_SKILL_ID
            | LORD_FAST_ATTACK_SKILL_ID
            | MONSTER_RANGE_ATTACK_SKILL_ID
            | MONSTER_THORN_SKILL_ID
            | SKELETON_ARCHERY_SKILL_ID
            | CHUCK_STONE_SKILL_ID
            | YUNSHENG_LIGHTNING_SKILL_ID
            | CORPSE_PTOMAINE_SKILL_ID
            | CORPSE_CANDLE_BLASTING_SKILL_ID
            | SPORE_BLASTING_SKILL_ID
            | ENERGY_BOLT_SKILL_ID
            | ZOMBIE_CLAW_SKILL_ID
            | LITTLE_STAR_SKILL_ID
            | SNAKE_BOLT_SKILL_ID
            | SPIDER_MIST_SKILL_ID
            | SPRITE_BURN_SKILL_ID
            | MACHINERY_STOMP_SKILL_ID
            | LORD_WIDERANGING_ATTACK_SKILL_ID
            | BOSS_BLUE_FURY_SKILL_ID
            | BOSS_BLUE_QUAKE_SKILL_ID
            | BOSS_FIEND_SUMMON_SKILL_ID
            | BOSS_FIEND_PENETRATE_SKILL_ID
            | SUMMON_CORPSE_CANDLE_SKILL_ID
            | SUMMON_SKELETON_SKILL_ID
            | SUMMON_SPORE_SKILL_ID
            | SNOW_STORM_SKILL_ID
    )
}

/// Точная встречная ветвь `CPet::OnStayingSchedule` и
/// `CPet::OnAttackingSchedule`. `GetAI` цели возвращает AI-owner, чьи поля
/// target type/id сравниваются с самим питомцем; только совпавший AI получает
/// виртуальный `OnLoseTarget`. Derived-переходы питомца и мечевого охранника
/// сохраняются, но внешний `SearchEnemy` текущего питомца сюда не переносится.
fn release_reciprocal_monster_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    target_id: i32,
    pet_identity: ShapeIdentity,
    runtime: &mut Runtime,
) {
    let reciprocal = region.find_monster_by_id(target_id)
        .is_some_and(|target| target.ai_target() == Some(pet_identity));
    if reciprocal {
        release_owned_monster_target(game, region, target_id, runtime);
    }
}

/// `CPet::GetPetMaster` сначала разрешает игрока глобальной таблицей, а для
/// остальных типов — ровно зарегистрированный `CMoveShape` текущего региона.
/// Боевой schedule использует найденную форму только как центр ограничения
/// преследования; отсутствие master-а оставляет прежний центр на питомце.
fn pet_combat_master_anchor(
    game: &CGame,
    region: &CServerRegion,
    master: MasterInfo,
) -> Option<(i32, i32)> {
    let master = match pet_master_ref(master)? {
        PetMasterRef::Player(player_id) => game.find_player(player_id)?.shape_view()?,
        PetMasterRef::Region(identity) => game.find_shape_in_region(region.id, identity)?,
    };
    Some((master.tile_x, master.tile_y))
}

fn select_and_store_monster_attack_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    monster_health: u32,
    runtime: &mut Runtime,
) -> Option<u16> {
    let monster = region.find_monster_by_id(monster_id)?;
    let default_skill_id = monster.move_shape().default_attack_skill_id() as u16;
    let primary_ai = monster.active_primary_ai_type();
    let roll = game.skill_random_below(10_000);
    let selected = if primary_ai == Some(21) {
        choose_boss_blue_attack_skill(
            region,
            monster_id,
            property,
            monster_health,
            roll,
            default_skill_id,
        )
    } else if primary_ai == Some(23) {
        choose_boss_fiend_attack_skill(
            game,
            region,
            monster_id,
            property,
            monster_health,
            roll,
            runtime,
        )
    } else if primary_ai == Some(19) {
        Some(select_lord_attack_skill(
            monster_health,
            property.maximum_hp,
            &property.skills,
            roll,
            default_skill_id,
        ))
    } else {
        Some(select_attack_skill(
            &property.skills,
            roll,
            default_skill_id,
        ))
    }
    .unwrap_or(default_skill_id);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster
            .move_shape_mut()
            .set_current_skill_id(Some(u32::from(selected)));
    }
    Some(selected)
}

/// Выполняет `CMonsterAI::OnChangeSkill` из FIFO либо непосредственно OnSchedule. После
/// единственного weighted RNG выбранный concrete skill проверяется через
/// `CSkill::IsRestored`; отсутствующий или ещё не восстановленный навык общего
/// monster AI заменяется `GetDefaultAttackSkillID`. AI5 и наследующий его
/// AI103 сохраняют существующий навык на cooldown и ставят полный restore
/// delay в хвост FIFO. Boss-specific пороги остаются в своих selector-owner-ах.
pub(crate) fn change_owned_monster_attack_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some((property, monster_health, primary_ai)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?
                    .clone(),
                monster.hit_points(),
                monster.active_primary_ai_type(),
            ))
        })
    else {
        return false;
    };
    let selected = select_and_store_monster_attack_skill(
        game,
        region,
        monster_id,
        &property,
        monster_health,
        runtime,
    );
    let Some(selected_skill_id) = selected else {
        return false;
    };
    if primary_ai.is_some_and(inherits_fixed_archer_change_skill) {
        if queue_fixed_archer_skill_delay(
            game,
            region,
            monster_id,
            &property,
            selected_skill_id,
            runtime,
        )
        {
            return true;
        }
    } else if region.find_monster_by_id(monster_id)
        .and_then(|monster| monster.move_shape().current_skill(game.skill_factory()))
        .and_then(|skill| {
            let properties = game.skill_base_properties(
                skill.id(),
                skill.level(),
            )?;
            let last_used_ms = region
                .find_monster_by_id(monster_id)?
                .skill_last_used_ms(u32::from(selected_skill_id), game.skill_factory());
            Some(skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                runtime.now_milliseconds(),
            ))
        })
        .unwrap_or(false)
    {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let default_skill_id = monster.move_shape().default_attack_skill_id();
        monster
            .move_shape_mut()
            .set_current_skill_id(Some(default_skill_id));
    }
    true
}

/// Выполняет только подтверждённый `OnSearchEnemy` обычного агрессивного
/// монстра, умного и пассивного гладиаторов, слабого существа, двух лучников,
/// военного монстра, участника битвы богов, городского охранника, владыки,
/// близнецов JiuMai и двух боссов. Фабричный fallback `CMonsterAI` выполняет
/// унаследованный пустой `OnSearchEnemy`, но всё равно завершает FIFO-событие.
/// Предшествующее событие уже обработано владельцем FIFO, поэтому здесь не
/// начинается атака в том же такте.
pub(crate) fn search_owned_monster_enemy<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_owner: &mut ServerRegionOwner,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let region = region_owner.base_mut();
    if region.find_monster_by_id(monster_id).is_some_and(|monster| {
        matches!(monster.active_ai(), Some(ActiveMonsterAi::Carriage
            | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)))
    }) {
        return true;
    }
    let Some((property, has_target)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
                monster.ai_target().is_some(),
            ))
        })
    else {
        return false;
    };
    if has_target && MonsterAiKind::from_ai_type(property.ai).has_guard_station() {
        super::super::ai::cityguardwithsword::check_guard_station_target(
            game, region, monster_id, property.chase_range as i32, runtime,
        );
        return true;
    }
    if MonsterAiKind::is_generic_ai_type(property.ai)
        || (matches!(property.ai, 2 | 20) && has_target)
    {
        return true;
    }
    if property.ai == 7 {
        return search_puniness_enemy(game, region, monster_id);
    }
    if property.ai == 1 {
        let Some(owner) = region
            .find_monster_by_id(monster_id)
            .and_then(|monster| monster.shape_view(&property))
        else {
            return false;
        };
        let Some(mut state) = region
            .find_monster_by_id_mut(monster_id)
            .and_then(CMonster::take_passive_gladiator_ai)
        else {
            return false;
        };
        let selected = state.select_target(owner, property.chase_range as i32, |player_id| {
            game.find_player(player_id).and_then(|player| {
                (player.server_region_id() == Some(region.id) && !player.is_dead())
                    .then(|| player.shape_view())
                    .flatten()
            })
        });
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.restore_passive_gladiator_ai(state);
            if let Some(selected) = selected {
                monster.set_ai_target(selected);
            }
        }
        return true;
    }
    let Some((owner, area_index, skill)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let owner = monster.shape_view(&property)?;
            let area_index = monster.move_shape().shape().area_index()?;
            let skill = monster.move_shape().current_skill(game.skill_factory())
                .map(|skill| (skill.id(), skill.level()));
            Some((
                owner,
                area_index,
                skill,
            ))
        })
    else {
        return false;
    };
    if property.ai == 20 {
        if let Some(selected) = select_jiumai_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ) {
            let _ = assign_jiumai_target(region, monster_id, selected);
        }
        return true;
    }
    if property.ai == 2 {
        let selection = select_smart_gladiator_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        );
        if let Some(selected) = selection.vulnerable_target() {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.set_ai_target(selected);
            }
        } else if let Some(destination) = selection.retreat_step(owner)
            && let Some(state) = region
                .find_monster_by_id_mut(monster_id)
                .and_then(CMonster::smart_gladiator_ai_mut)
        {
            state.queue_step(destination);
        }
        return true;
    }
    let minimum_skill_distance = skill.map(|(skill_id, skill_level)| {
        if is_immediate_state_skill(skill_id) || is_heal_skill(skill_id) || is_non_fun_skill(skill_id)
            || matches!(skill_id, ARCHERY_SKILL_ID | BASE_MAGIC_PROJECTILE_SKILL_ID | FIRE_BOLT_SKILL_ID | FIRE_BALL_SKILL_ID | GOD_PUNISHMENT_SKILL_ID | GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID)
        {
            return 1;
        }
        game.skill_base_properties(skill_id, skill_level)
            .map_or(0, |properties| properties.query_property(5_004) as i32)
    });
    if matches!(property.ai, 0 | 3) {
        let selected = select_gladiator_enemy(game, region_owner, owner, area_index, &property);
        if let Some(selected) = selected
            && let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
        {
            monster.set_ai_target(selected);
        }
        return true;
    }
    if matches!(property.ai, 8 | 9 | 17 | 100 | 101) {
        let Some(minimum_skill_distance) = minimum_skill_distance else {
            return false;
        };
        let selected = if matches!(property.ai, 8 | 9) {
            select_guard_with_bow_target(
                game, region_owner, monster_id, &property, minimum_skill_distance,
            )
        } else {
            select_country_guard_target(
                game, region_owner, monster_id, &property, minimum_skill_distance,
            )
        };
        if let Some(selected) = selected
            && let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
        {
            monster.set_ai_target(selected);
        }
        return true;
    }
    let selected = match property.ai {
        4 => select_archer_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ),
        5 => {
            let Some(minimum_skill_distance) = minimum_skill_distance else {
                return false;
            };
            select_fixed_archer_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
        }
        6 => {
            let Some(minimum_skill_distance) = minimum_skill_distance else {
                return false;
            };
            let _ = search_stupid_archer_enemy(
                game,
                region,
                monster_id,
                owner,
                area_index,
                &property,
                minimum_skill_distance,
                runtime,
            );
            return true;
        }
        10 | 11 => {
            let Some(minimum_skill_distance) = minimum_skill_distance else {
                return false;
            };
            select_city_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
            .map(|selected| selected.identity)
        }
        14 | 15 => select_country_war_enemy(
            game,
            region,
            owner,
            area_index,
            property.ai,
            property.guard_range as i32,
        ),
        12 | 13 => {
            let Some(minimum_skill_distance) = minimum_skill_distance else {
                return false;
            };
            select_village_country_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
            .map(|selected| selected.identity)
        }
        16 => {
            let Some(minimum_skill_distance) = minimum_skill_distance else {
                return false;
            };
            select_nation_country_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
                property.race,
            )
            .map(|selected| selected.identity)
        }
        18 => select_nation_gladiator_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
            property.race,
        ),
        103 => {
            let Some(minimum_skill_distance) = minimum_skill_distance else {
                return false;
            };
            select_gods_battle_guard_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
                property.race,
            )
        }
        104 => select_gods_battle_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
            property.race,
        ),
        19 => select_lord_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ),
        21 => select_boss_blue_enemy(
            game,
            region,
            owner,
            area_index,
            property.guard_range as i32,
        ),
        23 => {
            let Some(minimum_skill_distance) = minimum_skill_distance else {
                return false;
            };
            select_boss_fiend_enemy(
                game,
                region,
                owner,
                area_index,
                property.guard_range as i32,
                minimum_skill_distance,
            )
        }
        _ => return false,
    };
    if let Some(selected) = selected
        && let Some(monster) = region.find_monster_by_id_mut(monster_id)
    {
        monster.set_ai_target(selected);
    }
    true
}

type OwnedRegisteredCastExecutor<Runtime> = fn(
    &mut CGame, &mut Option<ServerRegionOwner>, i32, ShapeIdentity, u16, &mut Runtime,
) -> bool;

/// Общий Begin немедленных свойств: активное расписание передаёт цель,
/// AutoStart — самого монстра. После базы Check видит исходный U; отказ
/// завершает тот же зарегистрированный экземпляр, не создавая visual.
pub(crate) fn begin_owned_monster_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_id: u32, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    if !is_immediate_state_skill(skill_id) { return false; }
    let Some(region) = owner.as_ref().map(ServerRegionOwner::base) else { return false; };
    let Some(monster) = region.find_monster_by_id(monster_id) else { return false; };
    let original_user = (region.id, monster.move_shape().shape().identity());
    let target_object = resolve_owned_skill_begin_object(game, region, target);
    let started = runtime.now_milliseconds();
    let prepared = owner.as_mut().and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
        .is_some_and(|monster| monster.prepare_base_attack_cast(
            target, skill_id, skill_level, started, target_object, game.skill_factory(),
        ));
    if !prepared { return false; }
    game.with_published_region(owner, |game| {
        let Some(instance) = game.registered_move_shape_skill(original_user.0, original_user.1, skill_id) else { return false; };
        if let Some(kernel) = game.registered_skill_mut(instance).and_then(|skill| skill.monster_kernel_mut()) {
            kernel.clear_phase_for_end();
        }
        if !check_immediate_state_cast(game, instance, Some(original_user)) {
            let _ = super::stateskill::end_state_skill(game, instance, 0, runtime);
            return false;
        }
        game.registered_skill_mut(instance).is_some_and(|skill| {
            skill.advance_execution(SkillStage::Idle, SkillStage::Begin)
        })
    }).unwrap_or(false)
}

fn execute_owned_monster_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    let Some(monster) = owner.as_ref().and_then(|region| region.base().find_monster_by_id(monster_id)) else { return false; };
    let Some(skill_id) = monster.move_shape().current_skill(game.skill_factory()).map(|skill| skill.id()) else { return false; };
    if !is_immediate_state_skill(skill_id) { return false; }
    if monster.current_active_attack_cast(game.skill_factory()).is_some() {
        return execute_monster_immediate_state(game, owner, monster_id, skill_id, i32::from(skill_level), runtime);
    }
    let begun = begin_owned_monster_immediate_state(game, owner, monster_id, target, skill_id, skill_level, runtime);
    let Some(region) = owner.as_mut().map(ServerRegionOwner::base_mut) else { return false; };
    if begun {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.enqueue_base_attack_cast(runtime.now_milliseconds());
        }
        true
    } else {
        crate::gameserver::appserver::ai::monsterai::finish_monster_skill_call(
            game, region, monster_id,
            crate::gameserver::appserver::ai::monsterai::MonsterSkillCallOutcome::BeginRejected, runtime,
        )
    }
}

fn owned_registered_cast_executor<Runtime: GameMainLoopRuntime>(
    skill_id: u32,
) -> Option<OwnedRegisteredCastExecutor<Runtime>> {
    match skill_id {
        ARCHERY_SKILL_ID => Some(execute_owned_monster_archery),
        BASE_MAGIC_PROJECTILE_SKILL_ID => Some(execute_owned_monster_base_magic),
        FIRE_BOLT_SKILL_ID => Some(execute_owned_monster_fire_bolt),
        FIRE_BALL_SKILL_ID => Some(execute_owned_monster_fire_ball),
        GOD_PUNISHMENT_SKILL_ID => Some(execute_owned_monster_god_punishment),
        GOD_BLESS_SKILL_ID => Some(execute_owned_monster_god_bless::<GOD_BLESS_SKILL_ID, Runtime>),
        GOD_BLESS_2_SKILL_ID => Some(execute_owned_monster_god_bless::<GOD_BLESS_2_SKILL_ID, Runtime>),
        HEAL_SKILL_ID => Some(execute_owned_monster_heal::<HEAL_SKILL_ID, Runtime>),
        HEAL_2_SKILL_ID => Some(execute_owned_monster_heal::<HEAL_2_SKILL_ID, Runtime>),
        SUPER_HEAL_SKILL_ID => Some(execute_owned_monster_heal::<SUPER_HEAL_SKILL_ID, Runtime>),
        SUPER_HEAL_2_SKILL_ID => Some(execute_owned_monster_heal::<SUPER_HEAL_2_SKILL_ID, Runtime>),
        KNOCK_OUT_SKILL_ID => Some(execute_owned_monster_knock_out),
        SPIDER_WEB_SKILL_ID => Some(execute_owned_spider_web),
        YAKSHA_SLASH_SKILL_ID => Some(execute_owned_monster_yaksha_slash),
        SPIDER_POISON_SKILL_ID => Some(execute_owned_spider_poison),
        PROMOTION_SKILL_ID => Some(execute_owned_monster_promotion),
        CURE_SKILL_ID => Some(execute_owned_monster_cure),
        HEARTEN_SKILL_ID => Some(execute_owned_monster_hearten),
        FURY_SKILL_ID => Some(execute_owned_fury),
        RAGE_BREAK_SKILL_ID => Some(execute_owned_monster_rage_break),
        _ if is_immediate_state_skill(skill_id) => Some(execute_owned_monster_immediate_state),
        _ if is_non_fun_skill(skill_id) => Some(execute_owned_monster_non_fun),
        _ => None,
    }
}

pub(crate) fn execute_owned_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    runtime: &mut Runtime,
    range_dispatch: &mut Option<MonsterRangeAttackDispatch>,
    wide_arc_dispatch: &mut Option<WideArcAttackDispatch>,
    projectile_dispatch: &mut Option<MonsterProjectileDispatch>,
    snow_storm_entry: &mut Option<i32>,
) -> bool {
    let Some(region_owner) = owner.as_mut() else { return false; };
    let Some(active_ai) = region_owner.base().find_monster_by_id(monster_id).and_then(CMonster::active_ai) else {
        return false;
    };
    let carriage_ai = matches!(active_ai, ActiveMonsterAi::Carriage
        | ActiveMonsterAi::Primary(MonsterAiKind::Carriage));
    let pet_ai = matches!(active_ai, ActiveMonsterAi::Pet);
    if let Some(cast) = region_owner.base().find_monster_by_id(monster_id)
        .and_then(|monster| monster.current_active_attack_cast(game.skill_factory()))
    {
        let dispatch = cast.dispatch();
        if dispatch.skill_id == COMMON_BASE_ATTACK_SKILL_ID {
            return super::baseattack::execute_owned_monster_base_attack(game, owner, monster_id, runtime);
        }
        if let Some(execute) = owned_registered_cast_executor(dispatch.skill_id) {
            return execute(game, owner, monster_id, dispatch.target, dispatch.skill_level, runtime);
        }
    }
    let Some((
        property,
        monster_shape,
        monster_view,
        monster_health,
        target,
        cast,
        tamed,
        attacker_master,
        pet_attack_properties,
        stop_frame,
        area_index,
        pet_action,
    )) = region_owner.base().find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let monster_view = monster.shape_view(&property)?;
        let pet_attack_properties = monster
            .is_tamed()
            .then(|| monster.pet_attack_properties(&property));
        let stop_frame = monster.stop_frame(&property);
        Some((
            property,
            monster.move_shape().shape().clone(),
            monster_view,
            monster.hit_points(),
            monster.ai_target(),
            monster.current_active_attack_cast(game.skill_factory()),
            monster.is_tamed(),
            monster.master_info(),
            pet_attack_properties,
            stop_frame,
            monster.move_shape().shape().area_index(),
            monster.pet_action(),
        ))
    })
    else {
        return false;
    };
    // OnSchedule повозки не начинает атаку; уже зарегистрированное
    // active-исполнение остаётся у общего OnFighting.
    if carriage_ai && cast.is_none() {
        return false;
    }
    if CMoveShape::is_died(monster_health) {
        return false;
    }
    if !pet_ai && property.ai == 20 && target.is_none() && cast.is_none()
        && !ensure_jiumai_twin(game, region_owner.base_mut(), monster_id, &property)
    {
        return false;
    }
    if cast.is_none() && target.is_some() && (pet_ai || property.ai != 7)
        && region_owner.base().find_monster_by_id(monster_id)
            .is_some_and(|monster| !monster.move_shape().can_fight())
    {
        if pet_ai || property.ai != 20 {
            release_owned_monster_target(game, region_owner.base_mut(), monster_id, runtime);
        }
        return true;
    }
    if !pet_ai && MonsterAiKind::from_ai_type(property.ai).has_guard_station() {
        if target.is_none() && cast.is_none()
            && let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
            && monster.primary_ai_queues_idle()
            && let Some(state) = monster.guard_station_ai_mut()
            && state.station().is_none()
        {
            state.record_station(monster_view);
            monster.begin_active_ai_change_skill(runtime.now_milliseconds());
        }
    }
    if target.is_none()
        && cast.is_none()
        && !pet_ai
        && hibernates_without_nearby_players(
            property.ai,
            region_owner.base()
                .find_monster_by_id(monster_id)
                .and_then(CMonster::smart_gladiator_ai)
                .is_some_and(|state| !state.has_queued_steps()),
        )
        && let Some(area_index) = area_index
        && region_owner.base().player_ids_around_area(area_index).is_empty()
    {
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.hibernate_ai(runtime.now_milliseconds());
            return true;
        }
    }
    if target.is_none() && cast.is_none() && !pet_ai && property.ai == 2
        && region_owner.base().find_monster_by_id(monster_id)
            .and_then(CMonster::smart_gladiator_ai)
            .is_some_and(|state| state.has_queued_steps())
    {
        return true;
    }
    if target.is_none()
        && cast.is_none()
        && !pet_ai
        && matches!(property.ai, 5 | 8 | 11 | 13 | 17 | 100 | 101 | 103)
    {
        return queue_stationary_guard_idle(
            region_owner.base_mut(),
            monster_id,
            stop_frame,
            runtime,
        );
    }
    if target.is_none()
        && cast.is_none()
        && !pet_ai
        && (matches!(property.ai, 0 | 1 | 2 | 3 | 4 | 6 | 9 | 10 | 12 | 14 | 15 | 16 | 18 | 19 | 20 | 104)
            || MonsterAiKind::is_generic_ai_type(property.ai))
    {
        return queue_monster_idle(game, region_owner.base_mut(), monster_id, &property, runtime);
    }
    if target.is_none() && cast.is_none() && pet_ai {
        return queue_pet_idle(region_owner.base_mut(), monster_id, stop_frame, game.skill_factory(), runtime);
    }
    let schedule_target_view = if cast.is_none() && let Some(target) = target {
        let Some(schedule_target) =
            resolve_owned_monster_attack_target(game, region_owner, target)
        else {
            if pet_ai {
                lose_pet_target_and_search(
                    region_owner.base_mut(),
                    monster_id,
                    runtime,
                );
            } else {
                release_owned_monster_target(game, region_owner.base_mut(), monster_id, runtime);
                if has_owned_search_enemy(property.ai, pet_ai)
                    && let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
                {
                    monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
                }
            }
            return true;
        };
        if pet_ai && schedule_target.dead {
            lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
            return true;
        }
        if pet_ai && pet_action == 0 {
            let (anchor_x, anchor_y) = pet_combat_master_anchor(game, region_owner.base_mut(), attacker_master)
                .unwrap_or((monster_view.tile_x, monster_view.tile_y));
            let anchor_distance = schedule_target.shape.distance_to_point(anchor_x, anchor_y);
            if game.globe_setup().maximum_pet_tracing_distance() as i32 <= anchor_distance {
                lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
                return true;
            }
        }
        let attackable = (!pet_ai && uses_stationary_attack_schedule(property.ai)) || game.live_skill_target_attackable_in(region_owner, monster_shape.identity(), target);
        if schedule_target.dead
            || ((pet_ai || !uses_stationary_attack_schedule(property.ai))
                && (schedule_target.god || schedule_target.city_dead || !attackable))
        {
            if !attackable && pet_ai {
                let pet_identity = ShapeIdentity {
                    object_type: MONSTER_TYPE,
                    id: monster_id,
                    ex_id: CGuid::GUID_INVALID,
                };
                if target.object_type == PLAYER_TYPE {
                    game.release_reciprocal_player_target(target.id, pet_identity, runtime);
                } else if target.object_type == MONSTER_TYPE {
                    release_reciprocal_monster_target(
                        game,
                        region_owner.base_mut(),
                        target.id,
                        pet_identity,
                        runtime,
                    );
                }
            }
            if pet_ai {
                lose_pet_target_and_search(
                    region_owner.base_mut(),
                    monster_id,
                    runtime,
                );
            } else {
                release_owned_monster_target(game, region_owner.base_mut(), monster_id, runtime);
                if has_owned_search_enemy(property.ai, pet_ai)
                    && let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
                {
                    monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
                }
            }
            return true;
        }
        Some(schedule_target.view)
    } else {
        None
    };
    let selected_skill_id = if let Some(cast) = cast {
        cast.dispatch().skill_id as u16
    } else if let Some(skill_id) = region_owner.base()
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.move_shape().current_skill(game.skill_factory()))
        .map(|skill| skill.id())
    {
        skill_id as u16
    } else if target.is_none() {
        // Собственный выбор босса в OnIdle не подменяется боевым OnSchedule.
        let Some(selected) = select_and_store_monster_attack_skill(
            game, region_owner.base_mut(), monster_id, &property, monster_health, runtime,
        ) else {
            return false;
        };
        selected
    } else {
        if !change_owned_monster_attack_skill(game, region_owner.base_mut(), monster_id, runtime) {
            return false;
        }
        let Some(selected) = region_owner.base().find_monster_by_id(monster_id)
            .and_then(|monster| monster.move_shape().current_skill(game.skill_factory()))
            .map(|skill| skill.id() as u16)
        else {
            release_owned_monster_target(game, region_owner.base_mut(), monster_id, runtime);
            return true;
        };
        selected
    };
    let (skill_id, skill_level) = if let Some(cast) = cast {
        (cast.dispatch().skill_id, cast.dispatch().skill_level)
    } else {
        let Some(skill) = region_owner.base().find_monster_by_id(monster_id)
            .and_then(|monster| monster.move_shape().skill(u32::from(selected_skill_id), game.skill_factory()))
        else {
            return false;
        };
        let Ok(level) = u16::try_from(skill.level()) else {
            return false;
        };
        (skill.id(), level)
    };
    if !is_owned_monster_attack_skill::<Runtime>(skill_id) {
        return false;
    }
    if target.is_none()
        && cast.is_none()
        && !pet_ai
        && matches!(property.ai, 21 | 23)
        && queue_boss_idle(game, region_owner.base_mut(), monster_id, &property, runtime)
    {
        return true;
    }
    let fast_attack = matches!(skill_id, MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID);
    let target = cast.map(|cast| cast.dispatch().target).or(target);
    let Some(target) = target else {
        return false;
    };
    if let Some(execute) = owned_registered_cast_executor(skill_id) {
        // OnFighting выше уже направлен к экземпляру. Здесь только новый
        // Begin: диапазон либо virtual Tracing, затем часы OnSchedule.
        if (pet_ai && pet_action == 2) || (!pet_ai && uses_stationary_attack_schedule(property.ai)) {
            let Some(target_view) = schedule_target_view else { return false; };
            let distance = monster_view.real_distance(Some(target_view));
            // Эти owner-ы наследуют minimum=1 и signed-положительный maximum.
            // NULL properties допускает дистанцию 1 до собственного CheckCast.
            if distance < 1 || distance > game.skill_base_properties(skill_id, i32::from(skill_level))
                .map(|properties| properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as i32)
                .filter(|maximum| *maximum > 0).unwrap_or(1)
            {
                if pet_ai {
                    lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
                } else {
                    release_owned_monster_target(game, region_owner.base_mut(), monster_id, runtime);
                    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                        monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
                    }
                }
                return true;
            }
        } else if !trace_owned_target_state_skill(game, region_owner, monster_id, runtime) {
            return true;
        }
        if !pet_ai && let Some(interval) = schedule_attack_interval(
            property.ai, pet_attack_properties.map_or(property.attack_speed, |pet| pet.attack_interval),
        ) {
            let attempted = region_owner.base_mut().find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| monster.begin_ai_attack_attempt_with_clock(
                    interval, &mut || runtime.now_milliseconds(),
                ));
            if !attempted { return true; }
        }
        return execute(game, owner, monster_id, target, skill_level, runtime);
    }
    let Some(skill_properties) = game
        .skill_base_properties(skill_id, i32::from(skill_level))
        .cloned()
    else {
        if matches!(skill_id, MONSTER_THORN_SKILL_ID | MACHINERY_STOMP_SKILL_ID | LORD_WIDERANGING_ATTACK_SKILL_ID | MONSTER_RANGE_ATTACK_SKILL_ID)
            && cast.is_some()
        {
            return super::monsterattack::end_owned_monster_skill_without_reuse(region_owner.base_mut(), monster_id, skill_id, game.skill_factory());
        }
        return false;
    };
    if cast.is_none()
        && ((pet_ai && pet_action == 2) || (!pet_ai && uses_stationary_attack_schedule(property.ai)))
    {
        let Some(target_view) = schedule_target_view else {
            return false;
        };
        let distance = monster_view.real_distance(Some(target_view));
        let minimum_distance = skill_properties.query_property(5_004) as i32;
        let maximum_distance = skill_properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as i32;
        if distance < minimum_distance || distance > maximum_distance {
            if pet_ai {
                lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
            } else {
                release_owned_monster_target(game, region_owner.base_mut(), monster_id, runtime);
                if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                    monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
                }
            }
            return true;
        }
    }
    let now_ms = runtime.now_milliseconds();
    if skill_id == SNOW_STORM_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_monster_snow_storm(game, region_owner.base_mut(), monster_id, target, skill_level, &skill_properties, &property, now_ms, runtime, snow_storm_entry);
    }
    if matches!(skill_id, SKELETON_ARCHERY_SKILL_ID | CHUCK_STONE_SKILL_ID) {
        let skill_properties = skill_properties.clone();
        return prepare_owned_monster_projectile(
            game,
            region_owner,
            monster_id,
            target,
            skill_id,
            skill_level,
            &skill_properties,
            now_ms,
            projectile_dispatch,
            runtime,
        );
    }
    if skill_id == YUNSHENG_LIGHTNING_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_yunsheng_lightning(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == CORPSE_PTOMAINE_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_corpse_ptomaine(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == CORPSE_CANDLE_BLASTING_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_corpse_candle_blasting(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == SPORE_BLASTING_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_spore_blasting(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == ENERGY_BOLT_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_energy_bolt(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == ZOMBIE_CLAW_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_zombie_claw(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == LITTLE_STAR_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_little_star(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == SNAKE_BOLT_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_snake_bolt(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == SPRITE_BURN_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_sprite_burn(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if matches!(skill_id, MACHINERY_STOMP_SKILL_ID | LORD_WIDERANGING_ATTACK_SKILL_ID) {
        let skill_properties = skill_properties.clone();
        let outcome = prepare_owned_wide_arc_attack(
            game,
            region_owner,
            monster_id,
            target,
            skill_id,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
            wide_arc_dispatch,
        );
        return crate::gameserver::appserver::ai::monsterai::finish_monster_skill_call(
            game, region_owner.base_mut(), monster_id, outcome, runtime,
        );
    }
    if skill_id == BOSS_BLUE_FURY_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_boss_blue_fury(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == BOSS_BLUE_QUAKE_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_boss_blue_quake(
            game, owner, monster_id, target, skill_level, &skill_properties, now_ms, runtime,
        );
    }
    if skill_id == BOSS_FIEND_PENETRATE_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_boss_fiend_penetrate(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == MONSTER_THORN_SKILL_ID {
        let skill_properties = skill_properties.clone();
        let outcome = execute_owned_monster_thorn(
            game,
            owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
        let Some(region_owner) = owner.as_mut() else { return true; };
        return crate::gameserver::appserver::ai::monsterai::finish_monster_skill_call(
            game, region_owner.base_mut(), monster_id, outcome, runtime,
        );
    }
    if skill_id == SPIDER_MIST_SKILL_ID {
        let skill_properties = skill_properties.clone();
        return execute_owned_spider_mist(
            game,
            region_owner,
            monster_id,
            target,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if matches!(
        skill_id,
        SUMMON_CORPSE_CANDLE_SKILL_ID
            | SUMMON_SKELETON_SKILL_ID
            | SUMMON_SPORE_SKILL_ID
            | BOSS_FIEND_SUMMON_SKILL_ID
    ) {
        let skill_properties = skill_properties.clone();
        return execute_owned_summon_creature(
            game,
            region_owner.base_mut(),
            monster_id,
            target,
            skill_id,
            skill_level,
            &skill_properties,
            now_ms,
            runtime,
        );
    }
    if skill_id == MONSTER_RANGE_ATTACK_SKILL_ID && cast.is_some() {
        let skill_properties = skill_properties.clone();
        return prepare_owned_monster_range_cast(
            game,
            region_owner.base_mut(),
            monster_id,
            &skill_properties,
            runtime,
            range_dispatch,
        );
    }
    let delay_ms = skill_properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = skill_properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = skill_properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let hit_modifier = skill_properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let target_snapshot = super::monsterattack::resolve_owned_monster_attack_target(game, region_owner, target,
    ).filter(|_| {
        target.object_type != MONSTER_TYPE
            || game.live_skill_target_attackable_in(region_owner, monster_shape.identity(), target)
    });
    let Some(super::monsterattack::OwnedMonsterAttackTarget {
        shape: target_shape,
        view: target_view,
        dead: target_dead,
        god: target_god,
        city_dead: target_city_dead,
        ..
    }) = target_snapshot
    else {
        if pet_ai {
            lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
        } else if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    };
    if target_dead
        || ((pet_ai || !uses_stationary_attack_schedule(property.ai) || cast.is_some())
            && (target_god
                || target_city_dead
                || (!tamed
                    && property.kind == 5
                    && target.object_type == PLAYER_TYPE
                    && !game.live_skill_target_attackable_in(region_owner, monster_shape.identity(), target))))
    {
        if pet_ai {
            lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
        } else if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    }
    let (Ok(monster_x), Ok(monster_y), Ok(target_x), Ok(target_y)) = (
        monster_shape.get_tile_x(),
        monster_shape.get_tile_y(),
        target_shape.get_tile_x(),
        target_shape.get_tile_y(),
    ) else {
        return true;
    };

    if tamed
        && target.object_type == PLAYER_TYPE
        && attacker_master.master_type == PLAYER_TYPE
        && attacker_master.master_id != 0
        && game
            .find_player(attacker_master.master_id)
            .is_some_and(|master| master.server_region_id() == Some(region_owner.base().id))
    {
        let pet_identity = ShapeIdentity {
            object_type: MONSTER_TYPE,
            id: monster_id,
            ex_id: CGuid::GUID_INVALID,
        };
        if let Some((string_id, limit)) =
            game.player_base_attack_level_block(attacker_master.master_id, target.id)
        {
            game.send_base_attack_level_block(attacker_master.master_id, string_id, limit);
            game.release_reciprocal_player_target(target.id, pet_identity, runtime);
            lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
            return true;
        }
        if !game.player_base_attackable(attacker_master.master_id, target.id) {
            game.release_reciprocal_player_target(target.id, pet_identity, runtime);
            lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
            return true;
        }
    }

    if !pet_ai && MonsterAiKind::from_ai_type(property.ai).has_guard_station()
        && cast.is_none()
        && trace_city_sword_target(
            game,
            region_owner.base_mut(),
            monster_id,
            monster_view,
            target_view,
            skill_properties.query_property(5_004) as i32,
            maximum_distance as i32,
            property.chase_range as i32,
            runtime,
        ) == CitySwordTraceOutcome::Handled
    {
        return true;
    }

    if let Some(cast) = cast {
        let delay_reached = if cast.dispatch().skill_id == MONSTER_BASE_ATTACK_SKILL_ID {
            cast.started_at_ms().wrapping_add(delay_ms) <= runtime.now_milliseconds()
        } else {
            time_reached(now_ms, cast.started_at_ms(), delay_ms)
        };
        if !delay_reached {
            return true;
        }
        let dispatch = cast.dispatch();
        let (hit_count, finish_cast) = if matches!(
            dispatch.skill_id,
            MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID
        ) {
            let first_time = skill_properties.query_property(SKILL_USAGE_FIRST_TIME);
            let second_time = skill_properties.query_property(SKILL_USAGE_SECOND_TIME);
            let Some(mut progress) = region_owner.base()
                .find_monster_by_id(monster_id)
                .and_then(|monster| monster.skill_progress::<MonsterFastAttackProgress>(dispatch.skill_id, game.skill_factory()).copied())
            else {
                return true;
            };
            if !progress.visual_started() {
                let fire = fast_attack_fire_message(
                    dispatch.skill_id,
                    dispatch.skill_level,
                    monster_id,
                    target_x,
                    target_y,
                );
                let _ = game.send_game_shape_around(region_owner.base_mut(), &monster_shape, None, &fire);
                progress.mark_visual_started();
                if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                    *monster
                        .skill_progress_mut::<MonsterFastAttackProgress>(dispatch.skill_id, game.skill_factory())
                        .expect("состояние быстрой атаки принадлежит текущему cast") = progress;
                    let _ = monster
                        .advance_base_attack_cast(dispatch.skill_id, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
                }
            }
            let first_due = time_reached(
                now_ms,
                cast.started_at_ms(),
                delay_ms.wrapping_add(first_time),
            );
            let second_due = time_reached(
                now_ms,
                cast.started_at_ms(),
                delay_ms.wrapping_add(first_time).wrapping_add(second_time),
            );
            let mut hits = 0;
            if !progress.first_attack_done() && first_due {
                progress.mark_first_attack_done();
                hits += 1;
                if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                    *monster
                        .skill_progress_mut::<MonsterFastAttackProgress>(dispatch.skill_id, game.skill_factory())
                        .expect("состояние быстрой атаки принадлежит текущему cast") = progress;
                }
            }
            if progress.first_attack_done() && second_due {
                hits += 1;
            }
            if hits == 0 {
                return true;
            }
            (hits, second_due)
        } else {
            if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                let _ = monster
                    .advance_base_attack_cast(dispatch.skill_id, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
            }
            let mut fire = CMessage::new(0x000b_fe01);
            fire.add_byte(2);
            fire.add_long(dispatch.skill_id as i32);
            fire.add_short(dispatch.skill_level as i16);
            fire.add_long(MONSTER_TYPE);
            fire.add_long(monster_id);
            fire.add_long(target.object_type);
            fire.add_long(target.id);
            fire.add_long(target_x);
            fire.add_long(target_y);
            let _ = game.send_game_shape_around(region_owner.base_mut(), &monster_shape, None, &fire);
            (1, true)
        };

        for hit_index in 0..hit_count {
            let Some(region_owner) = owner.as_mut() else { return true; };
            if hit_index != 0
                && resolve_owned_monster_attack_target(game, region_owner, target)
                    .is_none_or(|target| target.dead)
            { break; }
            let Some(monster) = region_owner.base().find_monster_by_id(monster_id) else { break };
            let (minimum, maximum) = monster.state_attack_bounds(
                property.minimum_attack,
                property.maximum_attack,
            );
            let soul_attack = monster.soul_attack(&property);
            let physical_minimum = minimum as i32;
            let physical_maximum = maximum as i32;
            let difference = physical_maximum.wrapping_sub(physical_minimum);
            let physical_span = match dispatch.skill_id {
                LORD_FAST_ATTACK_SKILL_ID => difference.wrapping_abs().wrapping_add(1),
                _ => difference.max(0).wrapping_add(1),
            };
            let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
            // `CMonster::GetAddElementAtk` остаётся нулевым даже для pet-owner.
            let element = 0;
            if dispatch.skill_id == LORD_FAST_ATTACK_SKILL_ID {
                let _critical_roll = game.skill_random_below(100);
            }
            let mut attack = AttackInformation {
                hit_modifier,
                damages: vec![
                    AttackPower {
                        kind: AttackPowerType::Physical,
                        hp_damage: physical.max(0),
                        mp_damage: 0,
                    },
                    AttackPower {
                        kind: AttackPowerType::Element,
                        hp_damage: element.max(0),
                        mp_damage: 0,
                    },
                    AttackPower {
                        kind: AttackPowerType::Soul,
                        hp_damage: i32::from(soul_attack),
                        mp_damage: 0,
                    },
                ],
                ..AttackInformation::for_master(MasterInfo {
                    master_type: MONSTER_TYPE,
                    master_id: monster_id,
                    ..MasterInfo::default()
                })
            };
            // MonsterBase::Calculate не заменяет конструкторские UNKNOWN/1.
            // Метаданные самостоятельных Fast-владельцев остаются их контрактом.
            if dispatch.skill_id != MONSTER_BASE_ATTACK_SKILL_ID {
                attack.skill_id = dispatch.skill_id;
                attack.skill_level = dispatch.skill_level as u8;
            }
            if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                if matches!(
                    dispatch.skill_id,
                    MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID
                ) {
                    let _ = monster
                        .advance_base_attack_cast(dispatch.skill_id, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
                    if finish_cast && hit_index + 1 == hit_count {
                        let _ = monster
                            .advance_base_attack_cast(dispatch.skill_id, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
                    }
                } else {
                    let _ = monster
                        .advance_base_attack_cast(dispatch.skill_id, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
                    let _ = monster
                        .advance_base_attack_cast(dispatch.skill_id, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
                }
            }
            apply_owned_monster_attack_hit(game, owner, runtime, target, attack);
        }
        let Some(region_owner) = owner.as_mut() else { return true; };
        if finish_cast
            && let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
        {
            if dispatch.skill_id != MONSTER_BASE_ATTACK_SKILL_ID {
                monster.move_shape_mut().shape_mut().set_action(1);
            }
            let _ = monster.finish_base_attack_cast_with_clock(dispatch.skill_id, game.skill_factory(), || runtime.now_milliseconds());
        }
        return true;
    }

    if !approach_attack_range(
        game,
        region_owner.base_mut(),
        monster_id,
        MonsterTraceTarget::Shape(target_view),
        maximum_distance,
        runtime,
    ) {
        if pet_ai && pet_action == 2 {
            lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
        }
        return true;
    }
    let attack_interval = schedule_attack_interval(
        property.ai,
        pet_attack_properties.map_or(property.attack_speed, |pet| pet.attack_interval),
    );
    if let Some(attack_interval) = attack_interval {
        let attack_started = region_owner.base_mut()
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, attack_interval));
        if !attack_started {
            return true;
        }
    }
    if skill_id == MONSTER_RANGE_ATTACK_SKILL_ID {
        let outcome = super::monsterrangeattack::begin_owned_monster_range_cast(
            region_owner.base_mut(), monster_id, skill_level, &skill_properties, now_ms, game.skill_factory(), runtime,
        );
        return crate::gameserver::appserver::ai::monsterai::finish_monster_skill_call(
            game, region_owner.base_mut(), monster_id, outcome, runtime,
        );
    }
    if skill_id == COMMON_BASE_ATTACK_SKILL_ID {
        super::baseattack::begin_owned_monster_base_attack(game, region_owner.base_mut(), monster_id, target, skill_level, runtime);
        return true;
    }
    let last_used_ms = region_owner.base()
        .find_monster_by_id(monster_id)
        .map(|monster| monster.skill_last_used_ms(skill_id, game.skill_factory()))
        .unwrap_or_default();
    if !skill_is_restored(last_used_ms, reuse_delay_ms, now_ms) {
        if pet_ai {
            lose_pet_target_and_search(region_owner.base_mut(), monster_id, runtime);
        }
        return true;
    }
    let direction = get_line_direction(monster_x, monster_y, target_x, target_y);
    let target_object = resolve_owned_skill_begin_object(game, region_owner.base_mut(), target);
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        monster
            .move_shape_mut()
            .shape_mut()
            .set_direction(direction);
        monster.begin_base_attack_cast(target, skill_id, skill_level, now_ms, target_object, game.skill_factory());
        if fast_attack {
            monster.set_skill_progress(skill_id, MonsterFastAttackProgress::default(), game.skill_factory());
        }
    }
    let mut start = CMessage::new(0x000b_fe01);
    start.add_byte(1);
    start.add_long(skill_id as i32);
    start.add_short(skill_level as i16);
    start.add_long(MONSTER_TYPE);
    start.add_long(monster_id);
    start.add_long(direction);
    let _ = game.send_game_shape_around(region_owner.base_mut(), &monster_shape, None, &start);
    true
}
