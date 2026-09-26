//! Поведение AI фигур живого региона: диспетчер-ядро `CMonsterAI`, lifecycle
//! `CPet`, AI-состояния лордов и боссов кластера E1, конкретные производные
//! AI волны Z-AI (близнецы Цзюмай, умный гладиатор, слабое существо,
//! повозка, стационарные лучник и городские охранники) и общая порядковая
//! механика passive-реакций исторического GameServer. Компонент выделен из
//! старого пакета зелёной порцией PassiveGladiator
//! (`appserver/ai/passivegladiator.cpp`, `appserver/ai/baseai.cpp`),
//! кластером A1 (`appserver/ai/monsterai.cpp`, `appserver/ai/pet.cpp`),
//! кластером E1 (`appserver/ai/lord.cpp`, `appserver/ai/bossblue.cpp`,
//! `appserver/ai/bossfiend.cpp`) и волной Z-AI (`appserver/ai/jiumai.cpp`,
//! `smartgladiator.cpp`, `puninesscreature.cpp`, `carriage.cpp`,
//! `fixedpositionarcher.cpp`, `cityguardwithsword.cpp`,
//! `cityguardwithbow.cpp`); поля, постановка событий, три FIFO и active-фаза
//! `CBaseAI` остаются hub-владением до своих порций. Общий monster tick hub
//! (тела Run/OnSchedule/OnIdle/OnMoving этих владельцев и их hurt-schedule
//! входы `baseai`/`playerai`) кластерам E1 и Z-AI не принадлежит и
//! переносится своей порцией.

mod events; // элементы `AI_EVENT` и коды `AI_SHAPE_ACTION`; чистый wrapping-deadline.
pub mod monsterai; // `CMonsterAI`: диспетчер-ядро расписаний боя/idle/tracing и hub-фасады прежних владельцев.
pub mod pet; // `CPet`: lifecycle FSM, active-поиск, follow/idle и OnLoseTarget-семья на hub-фасадах `monsterai`.
mod passivegladiator; // `CPassiveGladiator`: список врагов, hurt-разбор и typed-выбор цели.
mod reactions; // `CBaseAI`: буквальный порядок Defense/Stiffen/Died-реакций над очередями hub-владельца.
pub mod lord; // `CLord` (AI100): hurt-план отвода от призванной формы, фазовый выбор навыка и общий enemy-проход E1.
pub mod bossblue; // `CBossBlue` (AI103): восемь одноразовых порогов ярости и пороговый выбор навыка.
pub mod bossfiend; // `CBossFiend` (AI104): восемь одноразовых порогов призыва, таймер повторного призыва и min-distance проход.
pub mod guardtarget; // общее дистанционное ядро охранников и точка поста.
pub mod jiumai; // `CJiuMai` (AI101): создание/сближение близнецов, min-HP выбор и hurt-поведение пары.
pub mod smartgladiator; // `CSmartGladiator` (AI2): очередь шагов отхода, уязвимый выбор и hurt-ответ.
pub mod puninesscreature; // `CPuninessCreature` (AI7): ближайший выбор и пошаговый отход собственного расписания.
pub mod carriage; // `CCarriage` (AI12): lifecycle-состояние и follow/stay-планы повозки.
pub mod fixedpositionarcher; // `CFixedPositionArcher` (AI5): очереди стационарной семьи, restore-хвост и min-dist выбор.
pub mod cityguardwithsword; // `CCityGuardWithSword` (AI10): пост, городской selector и ветвь Tracing семьи.
pub mod cityguardwithbow; // `CCityGuardWithBow` (AI11): hurt-повторный поиск парой городских selector-ов.

pub use events::{ai_event_deadline_reached, AiEvent, AiShapeAction}; // совместимые данные FIFO обоих AI-владельцев.
pub use monsterai::{
    MonsterActiveAiView, MonsterAiScheduleState, MonsterBaseAttackDispatch,
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherOwner, MonsterDispatcherPlayer, MonsterDispatcherRegion,
    MonsterDispatcherRuntime, MonsterSkillCallOutcome, MonsterTraceTarget, accepts_hurt_target,
    approach_attack_range, finish_monster_skill_call, has_owned_search_enemy,
    hibernates_without_nearby_players, is_generic_ai_type, move_owned_monster_to,
    process_owned_monster_stiffen, queue_monster_idle, release_owned_monster_target,
    schedule_attack_interval, select_attack_skill, trace_owned_target_state_skill,
    uses_stationary_attack_schedule,
}; // ядро диспетчера расписания монстра и его hub-фасады.
pub use pet::{
    PetBehaviorState, PetLifecycleFacts, PetLifecycleNotice, PetLifecycleOutcome, PetMasterRef,
    execute_owned_pet_active_search, execute_owned_pet_follow, lose_pet_target_and_search,
    pet_master_ref, queue_pet_idle, release_pet_target,
}; // lifecycle и расписания приручённого питомца.
pub use passivegladiator::{
    PassiveGladiatorAttackFacts, PassiveGladiatorAttackOutcome, PassiveGladiatorCandidate,
    PassiveGladiatorEnemyNotice, PassiveGladiatorSelection, PassiveGladiatorState,
}; // состояние и typed-контракты пассивного гладиатора.
pub use reactions::{
    begin_reached_death_action, begin_reached_stiffen_action, discard_active_prefix,
    finish_reached_death_action, finish_reached_stiffen_action, process_reached_defense_actions,
    reached_death_action_state, PassiveDeathAction, PassiveReactionQueues, PassiveStiffenAction,
}; // семья действий-порядков реакций и узкая generic-сварка очередей.
pub use lord::{
    EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion, LordDispatcherGame,
    LordDispatcherMonster, LordHurtPlan, apply_lord_hurt_response, plan_lord_hurt_response,
    select_lord_attack_skill, select_lord_enemy, select_nearest_player_or_pet,
}; // состояние владыки и общий enemy-проход кластера E1.
pub use bossblue::{
    BossBlueAiState, BossBlueDispatcherMonster, BossBlueDispatcherMoveShape,
    choose_boss_blue_attack_skill, select_boss_blue_attack_skill, select_boss_blue_enemy,
}; // состояние и выборы синего босса.
pub use bossfiend::{
    BossFiendAiState, BossFiendDispatcherMonster, BossFiendSkillSelection,
    choose_boss_fiend_attack_skill, select_boss_fiend_attack_skill, select_boss_fiend_enemy,
}; // состояние и выборы демона-босса.
pub use guardtarget::{
    GuardDistanceTarget, GuardStationState, consider_guard_distance_target,
    select_guard_target_groups,
}; // дистанционное ядро и точка поста охранников.
pub use jiumai::{
    JiuMaiAiState, JiuMaiDispatcherGame, JiuMaiDispatcherMonster, JiuMaiDispatcherPlayer,
    assign_jiumai_target, ensure_jiumai_twin, maintain_jiumai_twin, release_jiumai_target,
    retarget_jiumai_after_hurt, select_jiumai_enemy, synchronize_jiumai_target_loss,
}; // состояние и тела близнецов Цзюмай.
pub use smartgladiator::{
    SmartGladiatorCandidate, SmartGladiatorDispatcherMonster, SmartGladiatorDispatcherPlayer,
    SmartGladiatorSelection, SmartGladiatorState, apply_monster_hurt_response,
    apply_player_hurt_response, execute_smart_gladiator_retreat, retreat_step_from,
    select_smart_gladiator_enemy,
}; // состояние, выбор и hurt-ответ умного гладиатора.
pub use puninesscreature::{execute_owned_puniness_creature, search_puniness_enemy};
// расписание и выбор слабого существа.
pub use carriage::{
    CARRIAGE_FOLLOWING, CARRIAGE_STAYING, CarriageLifecycleState, CarriageMasterFacts,
    CarriageMasterOutcome, CarriageMovementPlan, plan_carriage_movement,
}; // состояние и планы повозки.
pub use fixedpositionarcher::{
    FixedArcherDispatcherMoveShape, FixedArcherTarget, attack_completion_actions,
    consider_fixed_archer_target, inherits_fixed_archer_change_skill,
    queue_fixed_archer_skill_delay, queue_stationary_guard_idle, select_fixed_archer_enemy,
}; // предикаты и выбор стационарного лучника.
pub use cityguardwithsword::{
    CityGuardDispatcherMoveShape, CityGuardDispatcherPlayer, CityGuardDispatcherRegion,
    CitySwordTraceOutcome, GuardStationDispatcherMonster, check_guard_station_target,
    lose_guard_sword_target, release_guard_sword_target, select_city_guard_enemy,
    trace_city_sword_target,
}; // пост, selector и Tracing городского мечевого охранника.
pub use cityguardwithbow::retarget_city_bow_guard_after_hurt; // hurt-поиск городского лучника.
