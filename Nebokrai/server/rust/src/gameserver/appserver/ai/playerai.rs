//! Достигнутая часть очередей и исполнения `CPlayerAI` GameServer.
//! Отсчёт CState::Begin фиксируется общим расписанием до OnBeginSkill.
//! Краткоживущий контекст привязан к dispatch и передаётся kernel при его
//! установке, до первого AI. После вызова владельца контекст очищается даже
//! при отказе; активные навыки его не наследуют. Очередь феи использует
//! отдельный типизированный контекст, чтобы два исполнения одного такта
//! не могли получить отсчёт друг друга.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/ai/playerai.cpp`. Трёхаргументный virtual `MoveTo` RVA
//! `0x0010A480` для живого игрока очищает эмоцию, удаляет старейшие назначения
//! до длины не более трёх и затем добавляет `(direction, is_run)`;
//! так очередь после вызова содержит не более четырёх элементов. Здоровье
//! владельца и `ClearEmotion` остаются у вызывающей стороны, чтобы не хранить сырые
//! указатели внутри ИИ. Канонический `CPlayer` владеет очередями навыков;
//! `CMoveShape::AI` передаёт первый элемент конкретному исполнителю и удаляет
//! его только после завершения либо отказа. Базовая атака, базовая магия,
//! стрельба, бессердечная и световая стрелы, семейство ловкости, парная закалка,
//! воодушевление, управление
//! питомцами, усиление, периодическое лечение, огненная стрела, огненная
//! стена, огненный круг, молния, печать, инь-ян, божественная кара, сбор душ
//! и зеркало душ,
//! сфера хаоса, семь падающих звёзд, ядовитый мотылёк, кровавая роза
//! и трёхударный скорпион,
//! семейства бегущего и армейского ударов,
//! рыцарский удар, подготовка яростного удара, ярость, последующий рывок, громовое
//! рассечение, семейство малых рывков, прямой рывок, боевой клич, накопление
//! энергии, обратный рубящий
//! и двойной направленный удары,
//! периодический удар листвы и фронтальный рубящий удар,
//! быстрая атака владыки,
//! прямые снаряды метателя камня и скелета-стрелка,
//! одноцелевая молния Юньшэн,
//! трупный яд с локальной областью,
//! шипастая одноцелевая атака,
//! паучий туман с призываемой областью,
//! паутина с отложенным состоянием,
//! ядовитая атака паука с периодическим состоянием,
//! семейство призыва трупной свечи, скелета и споры,
//! ярость синего босса с отложенным self-состоянием,
//! землетрясение синего босса с фронтальным состоянием и отбрасыванием,
//! проникающая клеточная атака демона-босса,
//! машинный и мана-щит, защитная стойка,
//! оглушение, ослабление, очищение,
//! атака боевой феи и её призываемые области
//! и приручение монстров сохраняют незавершённое состояние между проходами ИИ.
//! Хвост
//! `CPlayerAI::Run` хранит часы
//! автоматического прироста,
//! использует сохранённые факты игрока и фракции и соблюдает беззнаковую
//! проверку срока. Прирост опыта возвращается в полный `CGame::CheckLevel`,
//! энергия публикуется адресным сообщением `0xBF72C`.
//! Четырёхаргументный `MoveTo` теперь использует каноническую очередь
//! `CBaseAI`: свободная соседняя клетка проверяется до `0xBF605`, а задержка
//! шага сохраняет скорость игрока, диагональный множитель и исходный нулевой
//! остановочный кадр. Слот +0x48 таблицы CPlayerAI указывает на пустой RET
//! (0x00485540), поэтому idle игрока не ставит базовый Stand на 1000 мс.
//! CBaseAI::Run (0x004C7D10) вызывает OnSchedule, затем background, passive и
//! active; только AES_HUNG_UP запрещает следующую основную фазу. WarSoul
//! обрабатывается после них независимо от результата passive. В Rust фон
//! уже предшествует passive, а WarSoul вызывается отдельным хвостом даже
//! при Defense/Stiffen. Прерывание concrete навыка завершается до этого
//! хвоста. Разделение расписания и исполнения остальных concrete owner-ов
//! и условное снятие Attack после End(4) ещё требуют замыкания.
//! Обработанный Defense разрешает active в том же Run; Stiffen запрещает
//! active лишь до deadline. Его прерывание при уже истёкшем сроке сохраняется
//! отдельно от ожидания, не останавливая собственный auto-inc
//! хвост `CPlayerAI::Run`. Достигнутые reciprocal/death `OnLoseTarget` всегда
//! возвращают player-а к вычисленному default attack; death-tail делает это и
//! без активного skill, а разорванный concrete owner больше не оставляет
//! current-skill и запрет движения.
//! Отказный `0xBFE01` при `OnLoseTarget` следует только за `End(1)` реально
//! прерванного навыка; одна ожидающая object-команда удаляется без ответа.
//! `OnSchedule` (0x005098D0) извлекает команду до допуска и Begin: текущая
//! команда хранится в Option, ожидающая m_qTarget — в VecDeque. Attack
//! (0x00509FF0/0x0050A230) заменяет только ожидающую команду, не execution.
//! Общий хвост завершения проверяет именно выбранную команду, поэтому End
//! concrete owner-а не позволяет повторно снять следующую совпавшую команду.
//! Освобождение execution также ограничено этой командой: CSkill::End
//! (0x004D84C0) очищает свой экземпляр, не соседние навыки. Option::take_if
//! сохраняет остальные kernel, включая отдельные варианты лечения.
//! OnFighting (0x005092B0) сохраняет Attack до отдельного такта после End;
//! затем ChangeSkill (0x00508E40) возвращает вычисленный после End default.
//! Допуск OnSchedule не повторяется внутри Attack, а завершивший AI не
//! получает лишний Reject от общего координатора. Используется существующая
//! FIFO CBaseAI, включая её очистку Defense/Stiffen и отдельный такт смены.
//! Базовые атака, стрельба и магия возвращают Begun до первого AI: Attack
//! исполняет его в том же Run, без искусственного дополнительного такта.
//! Ещё остаются разделение Begin/первого AI в остальных адаптерах и переход
//! prepared-навыка в фон до его End; эти ветви нельзя подменять ended-состоянием.
//! В активном коде Luvinia MoveShape/PlayerAI используют CNewSkill/stModuParam.
//! Однако старый закомментированный WhenAddBackStageSkill в AI/BaseAI.cpp
//! сохраняет наш контракт 0x004C94B0: при owner != null и ID != SKILL_UNKNOW
//! добавляет ID в m_vBackStageSkills без дедупликации. Этот фрагмент не следует
//! смешивать с новым модульным расписанием. Подготовка через запись регистра
//! в +0x44 сохраняется между AI, например у HeartLessArrow (0x005931BE),
//! EnergyBolt (0x0053CB9C) и GhostCut (0x0059E348); это не только временный
//! флаг синхронного Summon у RainArrow, сбрасываемый в том же AI вызовом End.
//! У боевой феи начатая команда хранится отдельно от сменяемого ожидающего
//! хвоста: новый target не уничтожает уже начатый `SkillExecutionKernel`, а
//! следующий навык продвигается только после завершения текущего. Отмена
//! сохраняет выбранный ID, а успешный `OnChangeSkillWithWarSoul` и потеря цели
//! возвращают его к базовой атаке `0x224`. Обычная очередь действий игрока и
//! очередь боевой феи исполняются независимо: движение игрока не
//! приостанавливает стадии феи.

use std::collections::{BTreeMap, VecDeque};

use super::baseai::{AiShapeAction, CBaseAI, PassiveStiffenAction};
use crate::gameserver::appserver::player::{
    BattleFairySkillDispatch, CPlayer, PlayerSkillDispatch,
};
use crate::gameserver::appserver::skills::agility::{
    AGILITY_2_SKILL_ID, AGILITY_SKILL_ID, AgilityFamilyExecutionState,
};
use crate::gameserver::appserver::skills::archery::ArcheryExecutionState;
use crate::gameserver::appserver::skills::armybreak::{ARMY_BREAK_SKILL_ID, ArmyBreakExecutionState};
use crate::gameserver::appserver::skills::armybreak2::ARMY_BREAK_2_SKILL_ID;
use crate::gameserver::appserver::skills::flash::FlashExecutionState;
use crate::gameserver::appserver::skills::swallow::SwallowExecutionState;
use crate::gameserver::appserver::skills::baseattack::BaseAttackExecutionState;
use crate::gameserver::appserver::skills::basemagic::BaseMagicExecutionState;
use crate::gameserver::appserver::skills::lightning::LightningExecutionState;
use crate::gameserver::appserver::skills::lordfastattack::LordFastAttackExecutionState;
use crate::gameserver::appserver::skills::seal::SealExecutionState;
use crate::gameserver::appserver::skills::battlefairybasemagic::BattleFairyBaseMagicExecutionState;
use crate::gameserver::appserver::skills::battlefairytransfer::BattleFairyTransferKind;
use crate::gameserver::appserver::skills::callosity::CallosityExecutionState;
use crate::gameserver::appserver::skills::chaossphere::ChaosSphereExecutionState;
use crate::gameserver::appserver::skills::chainlightning::ChainLightningExecutionState;
use crate::gameserver::appserver::skills::heartlessarrow::HeartlessArrowExecutionState;
use crate::gameserver::appserver::skills::heartlessarrow2::{
    HEARTLESS_ARROW_2_SKILL_ID, HeartlessArrowAreaExecutionState,
};
use crate::gameserver::appserver::skills::heartlessarrow3::HEARTLESS_ARROW_3_SKILL_ID;
use crate::gameserver::appserver::skills::lightingarrow::LightingArrowExecutionState;
use crate::gameserver::appserver::skills::lightingarrow2::LightingArrow2ExecutionState;
use crate::gameserver::appserver::skills::meteorarrowmass::MeteorArrowMassExecutionState;
use crate::gameserver::appserver::skills::meteorarrow::MeteorArrowExecutionState;
use crate::gameserver::appserver::skills::rainarrow::RainArrowExecutionState;
use crate::gameserver::appserver::skills::poisonmoth::PoisonMothExecutionState;
use crate::gameserver::appserver::skills::bloodrose::BloodRoseExecutionState;
use crate::gameserver::appserver::skills::scorpion::ScorpionExecutionState;
use crate::gameserver::appserver::skills::boalock::BoaLockExecutionState;
use crate::gameserver::appserver::skills::fallingstar::FallingStarExecutionState;
use crate::gameserver::appserver::skills::explosivearrow::{
    ExplosiveArrowExecutionState, ExplosiveArrowVariant,
};
use crate::gameserver::appserver::skills::strike::StrikeExecutionState;
use crate::gameserver::appserver::skills::yakshaslash::YakshaSlashExecutionState;
use crate::gameserver::appserver::skills::ghostcut::{GHOST_CUT_SKILL_ID, GhostCutExecutionState};
use crate::gameserver::appserver::skills::ghostcut2::GHOST_CUT_2_SKILL_ID;
use crate::gameserver::appserver::skills::ghostcut3::GHOST_CUT_3_SKILL_ID;
use crate::gameserver::appserver::skills::knightcut::KnightCutExecutionState;
use crate::gameserver::appserver::skills::littleflash::LittleFlashExecutionState;
use crate::gameserver::appserver::skills::littlestar::PlayerLittleStarExecutionState;
use crate::gameserver::appserver::skills::energybolt::PlayerPathProjectileExecutionState;
use crate::gameserver::appserver::skills::directprojectile::PlayerDirectProjectileExecutionState;
use crate::gameserver::appserver::skills::yunshenglightning::PlayerYunShengLightningExecutionState;
use crate::gameserver::appserver::skills::monsterthorn::PlayerMonsterThornExecutionState;
use crate::gameserver::appserver::skills::spidermist::PlayerSpiderMistExecutionState;
use crate::gameserver::appserver::skills::spiderweb::PlayerSpiderWebExecutionState;
use crate::gameserver::appserver::skills::summoncreatureskill::PlayerSummonCreatureExecutionState;
use crate::gameserver::appserver::skills::bossbluequake::PlayerBossBlueQuakeExecutionState;
use crate::gameserver::appserver::skills::bossfiendpenetrate::PlayerBossFiendPenetrateExecutionState;
use crate::gameserver::appserver::skills::spriteburn::SpriteBurnExecutionState;
use crate::gameserver::appserver::skills::kernel::{
    SkillExecutionKernel, SkillStage, SkillTermination,
};
use crate::gameserver::appserver::skills::natural::NATURAL_SKILL_ID;
use crate::gameserver::appserver::skills::rapture::RAPTURE_SKILL_ID;
use crate::gameserver::appserver::skills::rage::RageExecutionState;
use crate::gameserver::appserver::skills::sevenshootingstar::SevenShootingStarExecutionState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAiDestination {
    pub(crate) direction: i32,
    pub(crate) is_run: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillQueueOutcome {
    ActiveRejected,
    PendingUnchanged,
    Queued { replaced: usize },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CPlayerAI {
    base_ai: CBaseAI,
    destinations: VecDeque<PlayerAiDestination>,
    player_skills: VecDeque<PlayerSkillDispatch>,
    current_player_skill: Option<PlayerSkillDispatch>,
    scheduled_skill_begin: Option<(PlayerSkillDispatch, u32)>,
    scheduled_fairy_skill_begin: Option<(BattleFairySkillDispatch, u32)>,
    selected_battle_fairy_skill_id: u32,
    current_battle_fairy_skill: Option<BattleFairySkillDispatch>,
    battle_fairy_skills: VecDeque<BattleFairySkillDispatch>,
    base_attack: Option<BaseAttackExecutionState>,
    base_attack_last_used_ms: u32,
    archery: Option<ArcheryExecutionState>,
    archery_last_used_ms: u32,
    heartless_arrow: Option<HeartlessArrowExecutionState>,
    heartless_arrow_last_used_ms: u32,
    heartless_arrow_area: Option<HeartlessArrowAreaExecutionState>,
    heartless_arrow_area_last_used_ms: [u32; 2],
    lighting_arrow: Option<LightingArrowExecutionState>,
    lighting_arrow_last_used_ms: u32,
    lighting_arrow_2: Option<LightingArrow2ExecutionState>,
    lighting_arrow_2_last_used_ms: u32,
    meteor_arrow_mass: Option<MeteorArrowMassExecutionState>,
    meteor_arrow_mass_last_used_ms: u32,
    meteor_arrow: Option<MeteorArrowExecutionState>,
    meteor_arrow_last_used_ms: u32,
    rain_arrow: Option<RainArrowExecutionState>,
    rain_arrow_last_used_ms: u32,
    poison_moth: Option<PoisonMothExecutionState>,
    poison_moth_last_used_ms: u32,
    blood_rose: Option<BloodRoseExecutionState>,
    blood_rose_last_used_ms: u32,
    scorpion: Option<ScorpionExecutionState>,
    scorpion_last_used_ms: u32,
    boa_lock: Option<BoaLockExecutionState>,
    boa_lock_last_used_ms: u32,
    falling_star: Option<FallingStarExecutionState>,
    falling_star_last_used_ms: u32,
    explosive_arrow: Option<ExplosiveArrowExecutionState>,
    explosive_arrow_last_used_ms: [u32; 3],
    strike: Option<StrikeExecutionState>,
    strike_last_used_ms: u32,
    daub_poison: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    daub_poison_last_used_ms: u32,
    yaksha_slash: Option<YakshaSlashExecutionState>,
    yaksha_slash_last_used_ms: u32,
    agility_family: Option<AgilityFamilyExecutionState>,
    agility_family_last_used_ms: [u32; 4],
    base_magic: Option<BaseMagicExecutionState>,
    base_magic_last_used_ms: u32,
    fire_bolt: Option<BaseMagicExecutionState>,
    fire_bolt_last_used_ms: u32,
    fire_ball: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    fire_ball_last_used_ms: u32,
    item_skill_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    item_skill_2_last_used_ms: u32,
    chain_lightning: Option<ChainLightningExecutionState>,
    chain_lightning_last_used_ms: u32,
    thunder_blow: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    thunder_blow_last_used_ms: u32,
    thunder_slash: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    thunder_slash_last_used_ms: u32,
    pillar: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    pillar_last_used_ms: u32,
    rush: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    rush_last_used_ms: u32,
    rush_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    rush_2_last_used_ms: u32,
    roar: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    roar_last_used_ms: u32,
    energy_holding: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    energy_holding_last_used_ms: u32,
    inverse_chopped: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    inverse_chopped_last_used_ms: u32,
    thunder_blow_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    thunder_blow_2_last_used_ms: u32,
    mosou: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    mosou_last_used_ms: u32,
    ghost_cut: Option<GhostCutExecutionState>,
    ghost_cut_last_used_ms: [u32; 3],
    knight_cut: Option<KnightCutExecutionState>,
    knight_cut_last_used_ms: u32,
    army_break: Option<ArmyBreakExecutionState>,
    army_break_last_used_ms: [u32; 2],
    rage: Option<RageExecutionState>,
    rage_last_used_ms: u32,
    rage_break: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    rage_break_last_used_ms: u32,
    fury: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    fury_last_used_ms: u32,
    flash: Option<FlashExecutionState>,
    flash_last_used_ms: u32,
    swallow: Option<SwallowExecutionState>,
    swallow_last_used_ms: u32,
    leaf_cut: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    leaf_cut_last_used_ms: u32,
    leaf_cut_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    leaf_cut_2_last_used_ms: u32,
    leaf_cut_3: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    leaf_cut_3_last_used_ms: u32,
    kerosene: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    kerosene_last_used_ms: u32,
    ignition: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    ignition_last_used_ms: u32,
    blind: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    blind_last_used_ms: u32,
    ju_cut: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    ju_cut_last_used_ms: u32,
    lightning_sword: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    lightning_sword_last_used_ms: [u32; 4],
    little_flash: Option<LittleFlashExecutionState>,
    little_flash_last_used_ms: [u32; 2],
    fire_wall: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    fire_wall_last_used_ms: u32,
    poison_fog: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    poison_fog_destination: Option<(i32, i32)>,
    poison_fog_last_used_ms: u32,
    infernol: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    infernol_last_used_ms: u32,
    seven_shooting_star: Option<SevenShootingStarExecutionState>,
    seven_shooting_star_last_used_ms: u32,
    little_star: Option<PlayerLittleStarExecutionState>,
    little_star_last_used_ms: u32,
    path_projectile: Option<PlayerPathProjectileExecutionState>,
    path_projectile_last_used_ms: [u32; 3],
    direct_projectile: Option<PlayerDirectProjectileExecutionState>,
    direct_projectile_last_used_ms: [u32; 2],
    yunsheng_lightning: Option<PlayerYunShengLightningExecutionState>,
    yunsheng_lightning_last_used_ms: u32,
    corpse_ptomaine: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    corpse_ptomaine_last_used_ms: u32,
    monster_thorn: Option<PlayerMonsterThornExecutionState>,
    monster_thorn_last_used_ms: u32,
    spider_mist: Option<PlayerSpiderMistExecutionState>,
    spider_mist_last_used_ms: u32,
    spider_web: Option<PlayerSpiderWebExecutionState>,
    spider_web_last_used_ms: u32,
    spider_poison: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    spider_poison_last_used_ms: u32,
    summon_creature: Option<PlayerSummonCreatureExecutionState>,
    summon_creature_last_used_ms: [u32; 4],
    boss_blue_fury: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    boss_blue_fury_last_used_ms: u32,
    boss_blue_quake: Option<PlayerBossBlueQuakeExecutionState>,
    boss_blue_quake_last_used_ms: u32,
    boss_fiend_penetrate: Option<PlayerBossFiendPenetrateExecutionState>,
    boss_fiend_penetrate_last_used_ms: u32,
    sprite_burn: Option<SpriteBurnExecutionState>,
    sprite_burn_last_used_ms: u32,
    wide_arc_attack: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    wide_arc_attack_last_used_ms: [u32; 2],
    lord_fast_attack: Option<LordFastAttackExecutionState>,
    lord_fast_attack_last_used_ms: u32,
    monster_fast_attack_last_used_ms: u32,
    monster_base_attack: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    monster_base_attack_last_used_ms: u32,
    monster_range_attack: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    monster_range_attack_last_used_ms: u32,
    chaos_sphere: Option<ChaosSphereExecutionState>,
    chaos_sphere_last_used_ms: u32,
    lightning: Option<LightningExecutionState>,
    lightning_last_used_ms: u32,
    seal: Option<SealExecutionState>,
    seal_last_used_ms: u32,
    yin_yang: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    yin_yang_last_used_ms: u32,
    yin_yang_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    yin_yang_2_last_used_ms: u32,
    god_punishment: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_punishment_last_used_ms: u32,
    god_thunder: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_thunder_last_used_ms: u32,
    god_thunder_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_thunder_2_last_used_ms: u32,
    soul_collect: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    soul_collect_last_used_ms: u32,
    soul_mirror: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    soul_mirror_last_used_ms: u32,
    battle_fairy_base_magic: Option<BattleFairyBaseMagicExecutionState>,
    battle_fairy_base_magic_last_used_ms: u32,
    life_shield: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    life_shield_last_used_ms: u32,
    battle_fairy_transfer: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    battle_fairy_transfer_last_used_ms: [u32; 2],
    wangsheng: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    wangsheng_last_used_ms: u32,
    poison_arrow: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    poison_arrow_last_used_ms: u32,
    blood_loss: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    blood_loss_last_used_ms: u32,
    fatal_blow: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    fatal_blow_last_used_ms: u32,
    thunder: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    thunder_last_used_ms: u32,
    leiming2: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    leiming2_last_used_ms: u32,
    tianhuo: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    tianhuo_last_used_ms: u32,
    battle_fairy_attribute: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    battle_fairy_attribute_last_used_ms: [u32; 8],
    callosity: Option<CallosityExecutionState>,
    callosity_last_used_ms: [u32; 2],
    hearten: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    hearten_last_used_ms: u32,
    promotion: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    promotion_last_used_ms: u32,
    heal_family: [Option<SkillExecutionKernel<PlayerSkillDispatch>>; 4],
    heal_family_last_used_ms: [u32; 4],
    pets_control: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    pets_control_last_used_ms: u32,
    monster_taming: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    monster_taming_last_used_ms: u32,
    knock_out: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    knock_out_last_used_ms: u32,
    snow_storm: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    snow_storm_last_used_ms: u32,
    weak: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    weak_last_used_ms: u32,
    god_bless: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_bless_last_used_ms: [u32; 2],
    cure: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    cure_last_used_ms: u32,
    machine_shield: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    machine_shield_last_used_ms: u32,
    mana_shield: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    mana_shield_last_used_ms: u32,
    immediate_state: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    immediate_state_last_used_ms: BTreeMap<u32, u32>,
    non_fun: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    swordship: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    gibe: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    gibe_last_used_ms: u32,
    auto_inc_last_time_ms: u32,
    auto_inc_energy_last_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAutoProgress {
    pub(crate) player_id: i32,
    pub(crate) sampled_at_ms: u32,
    pub(crate) experience_gain: u32,
    pub(crate) vigour_gain: u32,
    pub(crate) previous_experience: u32,
    pub(crate) current_experience: u32,
    pub(crate) previous_vigour: u32,
    pub(crate) current_vigour: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEnergyRegeneration {
    pub(crate) player_id: i32,
    pub(crate) sampled_at_ms: u32,
    pub(crate) increment: u32,
    pub(crate) previous_energy: u32,
    pub(crate) current_energy: u32,
}

impl CPlayerAI {
    pub(crate) fn set_scheduled_skill_begin(&mut self, begin: Option<(PlayerSkillDispatch, u32)>) {
        self.scheduled_skill_begin = begin;
    }

    pub(crate) fn set_scheduled_fairy_skill_begin(&mut self, begin: Option<(BattleFairySkillDispatch, u32)>) {
        self.scheduled_fairy_skill_begin = begin;
    }

    pub(crate) fn when_been_hurted(&mut self, now_ms: u32) {
        self.base_ai.when_been_hurted(now_ms);
    }

    pub(crate) fn when_been_stiffened(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.when_been_stiffened(delay_ms, now_ms);
    }

    pub(crate) fn process_reached_defense_actions(&mut self) -> usize {
        self.base_ai.process_reached_defense_actions()
    }

    pub(crate) fn process_reached_stiffen_action(
        &mut self,
        now_ms: u32,
    ) -> PassiveStiffenAction {
        self.base_ai.process_reached_stiffen_action(now_ms)
    }

    pub(crate) fn queue_client_destination(&mut self, direction: i32, is_run: bool) {
        while 3 < self.destinations.len() {
            self.destinations.pop_front();
        }
        self.destinations
            .push_back(PlayerAiDestination { direction, is_run });
    }

    /// Выполняет достигнутую `ASA_MOVE`-границу до нового `OnSchedule`.
    /// Даже снятое в этом вызове событие удерживает расписание до следующего
    /// такта, как `CBaseAI::ProcessActiveAction`.
    pub(crate) fn advance_active_move(&mut self, now_ms: u32) -> bool {
        self.base_ai.advance_active_move(now_ms)
    }

    pub(crate) fn active_move_unhandled(&self) -> bool {
        self.base_ai.active_move_unhandled()
    }

    pub(crate) fn advance_active_stand(&mut self, now_ms: u32) -> bool {
        self.base_ai.advance_active_stand(now_ms)
    }

    pub(crate) fn active_stand_pending(&self) -> bool {
        self.base_ai.active_stand_pending()
    }

    pub(crate) fn active_stand_unhandled(&self) -> bool {
        self.base_ai.active_stand_unhandled()
    }

    pub(crate) fn active_attack_pending(&self) -> bool {
        self.base_ai.active_attack_pending()
    }

    pub(crate) fn begin_player_fighting(&mut self, now_ms: u32) {
        if !self.base_ai.active_actions().iter().any(|event| event.action == AiShapeAction::Attack) {
            self.base_ai.add_ai_event(AiShapeAction::Attack, 0, 0, now_ms);
        }
    }

    /// CPlayerAI::OnFighting проверяет IsEnded до AI, а не после него. Исчезнувший
    /// concrete execution поэтому обрабатывается только следующим тактом.
    pub(crate) fn finish_ended_player_attack(&mut self, mut now: impl FnMut() -> u32) -> bool {
        if self.current_player_skill.is_some() || !self.base_ai.active_attack_pending() {
            return false;
        }
        self.base_ai.add_ai_event(AiShapeAction::ChangeSkill, 0, 0, now());
        self.base_ai.finish_active_attack(now());
        true
    }

    pub(crate) fn active_change_skill_pending(&self) -> bool {
        self.base_ai.active_change_skill_pending()
    }

    pub(crate) fn finish_active_change_skill(&mut self, now_ms: u32) {
        self.base_ai.lose_target();
        self.base_ai.finish_active_change_skill(now_ms);
    }

    pub(crate) const fn is_hibernated(&self) -> bool {
        self.base_ai.is_hibernated()
    }

    pub(crate) fn has_queued_player_skill(&self) -> bool {
        self.current_player_skill.is_some() || !self.player_skills.is_empty()
    }

    pub(crate) fn next_destination(&self) -> Option<PlayerAiDestination> {
        self.destinations.front().copied()
    }

    /// `CPlayerAI::OnSchedule` удаляет назначение независимо от результата
    /// region cast и `MoveTo`; поэтому изъятие принадлежит самому FIFO-owner-у.
    pub(crate) fn finish_destination(&mut self, expected: PlayerAiDestination) -> bool {
        if self.destinations.front().copied() != Some(expected) {
            return false;
        }
        self.destinations.pop_front();
        true
    }

    pub(crate) fn begin_destination_move(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.begin_active_move(delay_ms, now_ms);
    }

    /// Хвост `CMoveShape::ForceMove`: spatial mutation уже завершена, после
    /// чего concrete player AI получает ожидание `ASA_STAND` на длительность
    /// принудительного перемещения.
    pub(crate) fn begin_forced_stand(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.begin_active_stand(delay_ms, now_ms);
    }

    pub(crate) fn stop_destination_move(&mut self) {
        self.base_ai.cancel_active_move();
    }

    /// Native Attack заменяет только m_qTarget; выбранная OnSchedule команда
    /// уже извлечена из FIFO и сохраняется независимо от текущего ID навыка.
    pub(crate) fn queue_player_skill(&mut self, dispatch: PlayerSkillDispatch) -> usize {
        if self.player_skills.front().copied() == Some(dispatch) {
            return 0;
        }
        let rejected = self.player_skills.len();
        self.player_skills.clear();
        self.player_skills.push_back(dispatch);
        rejected
    }

    pub(crate) fn queue_battle_fairy_skill(
        &mut self,
        dispatch: BattleFairySkillDispatch,
    ) -> BattleFairySkillQueueOutcome {
        if self
            .current_battle_fairy_skill
            .is_some_and(|current| current.skill_id() == dispatch.skill_id())
        {
            return BattleFairySkillQueueOutcome::ActiveRejected;
        }
        if self.battle_fairy_skills.front().copied() == Some(dispatch) {
            return BattleFairySkillQueueOutcome::PendingUnchanged;
        }
        let replaced = self.battle_fairy_skills.len();
        self.battle_fairy_skills.clear();
        self.battle_fairy_skills.push_back(dispatch);
        BattleFairySkillQueueOutcome::Queued { replaced }
    }

    pub(crate) fn player_skills(&self) -> &VecDeque<PlayerSkillDispatch> {
        &self.player_skills
    }

    pub(crate) fn next_player_skill(&self) -> Option<PlayerSkillDispatch> {
        self.current_player_skill.or_else(|| self.player_skills.front().copied())
    }

    pub(crate) const fn current_player_skill(&self) -> Option<PlayerSkillDispatch> {
        self.current_player_skill
    }

    pub(crate) fn finish_scheduled_player_skill(
        &mut self,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        self.current_player_skill == Some(expected)
            && self.finish_player_skill(expected, termination)
    }

    /// OnSchedule извлекает команду до проверок допуска и Begin. Продолжение
    /// текущего выполнения не извлекает следующую, даже если владелец умер.
    pub(crate) fn begin_next_player_skill(&mut self, can_schedule: bool) -> Option<PlayerSkillDispatch> {
        if self.current_player_skill.is_none() && can_schedule {
            self.current_player_skill = self.player_skills.pop_front();
        }
        self.current_player_skill
    }

    pub(crate) fn finish_player_skill(
        &mut self,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        if self.next_player_skill() != Some(expected) {
            return false;
        }
        if self.current_player_skill.take().is_none() {
            self.player_skills.pop_front();
        }
        self.finish_player_skill_execution(expected, termination);
        true
    }

    /// End освобождает только выбранный CSkill, не остальные экземпляры.
    /// Отделено от снятия команды для общего foreground/background lifecycle.
    fn finish_player_skill_execution(
        &mut self,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) {
        if let Some(mut execution) = self.base_attack.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение навыка игрока завершено");
        }
        if let Some(mut execution) = self.archery.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(
                ?expected,
                ?termination,
                stage = ?execution.kernel().stage(),
                "выполнение базовой стрельбы завершено"
            );
        }
        if let Some(mut execution) = self.heartless_arrow.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение бессердечной стрелы завершено");
        }
        if let Some(mut execution) = self.heartless_arrow_area.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение региональной стрелы завершено");
        }
        if let Some(mut execution) = self.lighting_arrow.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение световой стрелы завершено");
        }
        if let Some(mut execution) = self.lighting_arrow_2.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение второй световой стрелы завершено");
        }
        if let Some(mut execution) = self.meteor_arrow_mass.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "накопление метеорных стрел завершено");
        }
        if let Some(mut execution) = self.meteor_arrow.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение метеорной стрелы завершено");
        }
        if let Some(mut execution) = self.rain_arrow.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение дождя стрел завершено");
        }
        if let Some(mut execution) = self.poison_moth.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение ядовитого мотылька завершено");
        }
        if let Some(mut execution) = self.blood_rose.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение кровавой розы завершено");
        }
        if let Some(mut execution) = self.scorpion.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение скорпиона завершено");
        }
        if let Some(mut execution) = self.boa_lock.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение связывания удава завершено");
        }
        if let Some(mut execution) = self.falling_star.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение падающей звезды завершено");
        }
        if let Some(mut execution) = self.explosive_arrow.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение проникающей стрелы завершено");
        }
        if let Some(mut execution) = self.strike.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение оглушающего снаряда завершено");
        }
        if let Some(mut execution) = self.daub_poison.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение смазки оружия ядом завершено");
        }
        if let Some(mut execution) = self.yaksha_slash.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение удара якши завершено");
        }
        if let Some(mut execution) = self.agility_family.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение навыка ловкости завершено");
        }
        if let Some(mut execution) = self.base_magic.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение базовой магии завершено");
        }
        if let Some(mut execution) = self.fire_bolt.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение огненной стрелы завершено");
        }
        if let Some(mut execution) = self.fire_ball.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение огненного шара завершено");
        }
        if let Some(mut execution) = self.item_skill_2.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение громового огня завершено");
        }
        if let Some(mut execution) = self.chain_lightning.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение цепной молнии завершено");
        }
        if let Some(mut execution) = self.thunder_blow.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение громового удара завершено");
        }
        if let Some(mut execution) = self.thunder_slash.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение громового рассечения завершено");
        }
        if let Some(mut execution) = self.pillar.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение защитной стойки завершено");
        }
        if let Some(mut execution) = self.rush.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение прямого рывка завершено");
        }
        if let Some(mut execution) = self.rush_2.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго прямого рывка завершено");
        }
        if let Some(mut execution) = self.roar.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение боевого клича завершено");
        }
        if let Some(mut execution) = self.energy_holding.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение накопления энергии завершено");
        }
        if let Some(mut execution) = self.inverse_chopped.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение обратного рубящего удара завершено");
        }
        if let Some(mut execution) = self.thunder_blow_2.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго громового удара завершено");
        }
        if let Some(mut execution) = self.mosou.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение Мо-шоу завершено");
        }
        if let Some(mut execution) = self.ghost_cut.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение бегущего удара завершено");
        }
        if let Some(mut execution) = self.knight_cut.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение рыцарского удара завершено");
        }
        if let Some(mut execution) = self.army_break.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение армейского удара завершено");
        }
        if let Some(mut execution) = self.rage.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение канала ярости завершено");
        }
        if let Some(mut execution) = self.rage_break.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение подготовки яростного удара завершено");
        }
        if let Some(mut execution) = self.fury.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ярости завершено");
        }
        if let Some(mut execution) = self.flash.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение рывка сквозь строй завершено");
        }
        if let Some(mut execution) = self.swallow.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение двойного направленного удара завершено");
        }
        if let Some(mut execution) = self.leaf_cut.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение периодического удара завершено");
        }
        if let Some(mut execution) = self.leaf_cut_2.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго периодического удара завершено");
        }
        if let Some(mut execution) = self.leaf_cut_3.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение третьего периодического удара завершено");
        }
        if let Some(mut execution) = self.kerosene.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "наложение горючей смеси завершено"); }
        if let Some(mut execution) = self.ignition.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "воспламенение завершено"); }
        if let Some(mut execution) = self.blind.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "ослепление завершено"); }
        if let Some(mut execution) = self.ju_cut.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение рубящего удара завершено");
        }
        if let Some(mut execution) = self.lightning_sword.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение молниеносного меча завершено");
        }
        if let Some(mut execution) = self.little_flash.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение малого рывка завершено");
        }
        if let Some(mut execution) = self.fire_wall.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение огненной стены завершено");
        }
        if let Some(mut execution) = self.poison_fog.take_if(|state| state.dispatch() == expected) { self.poison_fog_destination = None; let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ядовитого тумана завершено"); }
        if let Some(mut execution) = self.infernol.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение огненного круга завершено");
        }
        if let Some(mut execution) = self.seven_shooting_star.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение семи падающих звёзд завершено");
        }
        if let Some(mut execution) = self.little_star.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение малой звезды завершено");
        }
        if let Some(mut execution) = self.path_projectile.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, skill_id = execution.skill_id(), stage = ?execution.kernel().stage(), "выполнение пошагового снаряда завершено");
        }
        if let Some(mut execution) = self.direct_projectile.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, skill_id = execution.skill_id(), stage = ?execution.kernel().stage(), "выполнение прямого снаряда завершено");
        }
        if let Some(mut execution) = self.yunsheng_lightning.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение молнии Юньшэн завершено");
        }
        if let Some(mut execution) = self.corpse_ptomaine.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение трупного яда завершено");
        }
        if let Some(mut execution) = self.monster_thorn.take_if(|state| state.kernel().dispatch() == expected) { let _ = execution.kernel_mut().terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение шипастой атаки завершено"); }
        if let Some(mut execution) = self.spider_mist.take_if(|state| state.kernel().dispatch() == expected) { let _ = execution.kernel_mut().terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение паучьего тумана завершено"); }
        if let Some(mut execution) = self.spider_web.take_if(|state| state.kernel().dispatch() == expected) { let _ = execution.kernel_mut().terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение паутины завершено"); }
        if let Some(mut execution) = self.spider_poison.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ядовитой атаки паука завершено"); }
        if let Some(mut execution) = self.summon_creature.take_if(|state| state.kernel().dispatch() == expected) { let _ = execution.kernel_mut().terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение призыва существа завершено"); }
        if let Some(mut execution) = self.boss_blue_fury.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ярости синего босса завершено"); }
        if let Some(mut execution) = self.boss_blue_quake.take_if(|state| state.kernel().dispatch() == expected) { let _ = execution.kernel_mut().terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение землетрясения синего босса завершено"); }
        if let Some(mut execution) = self.boss_fiend_penetrate.take_if(|state| state.kernel().dispatch() == expected) { let _ = execution.kernel_mut().terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение проникающей атаки демона-босса завершено"); }
        if let Some(mut execution) = self.sprite_burn.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение огненной области завершено");
        }
        if let Some(mut kernel) = self.wide_arc_attack.take_if(|state| state.dispatch() == expected) {
            let _ = kernel.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?kernel.stage(), "выполнение широкой дуговой атаки завершено");
        }
        if let Some(mut execution) = self.lord_fast_attack.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение быстрой атаки владыки завершено");
        }
        if let Some(mut kernel) = self.monster_base_attack.take_if(|state| state.dispatch() == expected) {
            let _ = kernel.terminate(termination);
            tracing::trace!(?expected, ?termination, "исполнение базовой атаки монстра игроком завершено");
        }
        if let Some(mut kernel) = self.monster_range_attack.take_if(|state| state.dispatch() == expected) {
            let _ = kernel.terminate(termination);
            tracing::trace!(?expected, ?termination, "круговая атака монстра игроком завершена");
        }
        if let Some(mut execution) = self.chaos_sphere.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение сферы хаоса завершено");
        }
        if let Some(mut execution) = self.lightning.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение молнии завершено");
        }
        if let Some(mut execution) = self.seal.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение печати завершено");
        }
        if let Some(mut execution) = self.yin_yang.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение инь-ян завершено");
        }
        if let Some(mut execution) = self.yin_yang_2.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго инь-ян завершено");
        }
        if let Some(mut execution) = self.god_punishment.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение божественной кары завершено"); }
        if let Some(mut execution) = self.god_thunder.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение божественного грома завершено"); }
        if let Some(mut execution) = self.god_thunder_2.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго божественного грома завершено"); }
        if let Some(mut execution) = self.soul_collect.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение сбора душ завершено"); }
        if let Some(mut execution) = self.soul_mirror.take_if(|state| state.dispatch() == expected) { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение зеркала душ завершено"); }
        if let Some(mut execution) = self.callosity.take_if(|state| state.kernel().dispatch() == expected) {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение навыка закалки завершено");
        }
        if let Some(mut execution) = self.hearten.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение воодушевления завершено");
        }
        if let Some(mut execution) = self.promotion.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение усиления завершено");
        }
        for mut execution in self
            .heal_family
            .each_mut()
            .into_iter()
            .filter_map(|slot| slot.take_if(|state| state.dispatch() == expected))
        {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение периодического лечения завершено");
        }
        if let Some(mut execution) = self.pets_control.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение управления питомцами завершено");
        }
        if let Some(mut execution) = self.monster_taming.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение приручения монстра завершено");
        }
        if let Some(mut execution) = self.knock_out.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение оглушения завершено");
        }
        if let Some(mut execution) = self.snow_storm.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение снежной бури завершено");
        }
        if let Some(mut execution) = self.weak.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ослабления завершено");
        }
        if let Some(mut execution) = self.god_bless.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение божественного благословения завершено");
        }
        if let Some(mut execution) = self.cure.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение очищения завершено");
        }
        if let Some(mut execution) = self.machine_shield.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение машинного щита завершено");
        }
        if let Some(mut execution) = self.mana_shield.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение мана-щита завершено");
        }
        if let Some(mut execution) = self.immediate_state.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение немедленного состояния завершено");
        }
        if let Some(mut execution) = self.non_fun.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение пустого навыка завершено");
        }
        if let Some(mut execution) = self.swordship.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение корабля мечей завершено");
        }
        if let Some(mut execution) = self.gibe.take_if(|state| state.dispatch() == expected) {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение провокации завершено");
        }
    }

    /// Fallback встречного `CPlayerAI::OnLoseTarget`, когда concrete execution
    /// ещё не материализован: удаляет только текущую object-команду,
    /// действительно направленную на отказавшегося питомца. Уже начатый skill
    /// координатор завершает отдельно через исходный `End(1)`.
    pub(crate) fn release_object_target(
        &mut self,
        target: super::super::shape::ShapeIdentity,
    ) -> bool {
        let Some(dispatch @ PlayerSkillDispatch::Object { target: current, .. }) =
            self.next_player_skill()
        else {
            return false;
        };
        if current != target {
            return false;
        }
        self.finish_player_skill(dispatch, SkillTermination::Cancelled)
    }

    pub(crate) const fn base_attack(&self) -> Option<BaseAttackExecutionState> {
        self.base_attack
    }

    pub(crate) const fn archery(&self) -> Option<ArcheryExecutionState> {
        self.archery
    }

    pub(crate) fn begin_archery(&mut self, mut state: ArcheryExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.archery = Some(state);
    }

    pub(crate) fn archery_mut(&mut self) -> Option<&mut ArcheryExecutionState> {
        self.archery.as_mut()
    }

    pub(crate) const fn archery_last_used_ms(&self) -> u32 {
        self.archery_last_used_ms
    }

    pub(crate) const fn agility_family(&self) -> Option<AgilityFamilyExecutionState> {
        self.agility_family
    }
    pub(crate) fn begin_agility_family(&mut self, mut state: AgilityFamilyExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.agility_family = Some(state);
    }
    pub(crate) fn agility_family_mut(&mut self) -> Option<&mut AgilityFamilyExecutionState> {
        self.agility_family.as_mut()
    }
    const fn agility_family_index(skill_id: u32) -> usize {
        match skill_id {
            AGILITY_SKILL_ID => 0,
            AGILITY_2_SKILL_ID => 1,
            RAPTURE_SKILL_ID => 2,
            NATURAL_SKILL_ID => 3,
            _ => unreachable!(),
        }
    }
    pub(crate) const fn agility_family_last_used_ms(&self, skill_id: u32) -> u32 {
        self.agility_family_last_used_ms[Self::agility_family_index(skill_id)]
    }
    pub(crate) fn mark_agility_family_used(&mut self, skill_id: u32, now_ms: u32) {
        let index = Self::agility_family_index(skill_id);
        self.agility_family_last_used_ms[index] = now_ms;
    }

    pub(crate) const fn mark_archery_used(&mut self, now_ms: u32) {
        self.archery_last_used_ms = now_ms;
    }

    pub(crate) const fn heartless_arrow(&self) -> Option<HeartlessArrowExecutionState> { self.heartless_arrow }
    pub(crate) fn begin_heartless_arrow(&mut self, mut state: HeartlessArrowExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.heartless_arrow = Some(state);
    }
    pub(crate) fn heartless_arrow_mut(&mut self) -> Option<&mut HeartlessArrowExecutionState> { self.heartless_arrow.as_mut() }
    pub(crate) const fn heartless_arrow_last_used_ms(&self) -> u32 { self.heartless_arrow_last_used_ms }
    pub(crate) const fn mark_heartless_arrow_used(&mut self, now_ms: u32) { self.heartless_arrow_last_used_ms = now_ms; }
    pub(crate) const fn heartless_arrow_area(&self) -> Option<HeartlessArrowAreaExecutionState> { self.heartless_arrow_area }
    pub(crate) fn begin_heartless_arrow_area(&mut self, mut state: HeartlessArrowAreaExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.heartless_arrow_area = Some(state);
    }
    pub(crate) fn heartless_arrow_area_mut(&mut self) -> Option<&mut HeartlessArrowAreaExecutionState> { self.heartless_arrow_area.as_mut() }
    const fn heartless_arrow_area_index(skill_id: u32) -> usize {
        match skill_id {
            HEARTLESS_ARROW_2_SKILL_ID => 0,
            HEARTLESS_ARROW_3_SKILL_ID => 1,
            _ => unreachable!(),
        }
    }
    pub(crate) const fn heartless_arrow_area_last_used_ms(&self, skill_id: u32) -> u32 { self.heartless_arrow_area_last_used_ms[Self::heartless_arrow_area_index(skill_id)] }
    pub(crate) fn mark_heartless_arrow_area_used(&mut self, skill_id: u32, now_ms: u32) { self.heartless_arrow_area_last_used_ms[Self::heartless_arrow_area_index(skill_id)] = now_ms; }
    pub(crate) const fn lighting_arrow(&self) -> Option<LightingArrowExecutionState> { self.lighting_arrow }
    pub(crate) fn begin_lighting_arrow(&mut self, mut state: LightingArrowExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.lighting_arrow = Some(state);
    }
    pub(crate) fn lighting_arrow_mut(&mut self) -> Option<&mut LightingArrowExecutionState> { self.lighting_arrow.as_mut() }
    pub(crate) const fn lighting_arrow_last_used_ms(&self) -> u32 { self.lighting_arrow_last_used_ms }
    pub(crate) const fn mark_lighting_arrow_used(&mut self, now_ms: u32) { self.lighting_arrow_last_used_ms = now_ms; }
    pub(crate) fn lighting_arrow_2(&self) -> Option<&LightingArrow2ExecutionState> { self.lighting_arrow_2.as_ref() }
    pub(crate) fn begin_lighting_arrow_2(&mut self, mut state: LightingArrow2ExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.lighting_arrow_2 = Some(state);
    }
    pub(crate) fn lighting_arrow_2_mut(&mut self) -> Option<&mut LightingArrow2ExecutionState> { self.lighting_arrow_2.as_mut() }
    pub(crate) const fn lighting_arrow_2_last_used_ms(&self) -> u32 { self.lighting_arrow_2_last_used_ms }
    pub(crate) const fn mark_lighting_arrow_2_used(&mut self, now_ms: u32) { self.lighting_arrow_2_last_used_ms = now_ms; }
    pub(crate) const fn meteor_arrow_mass(&self) -> Option<MeteorArrowMassExecutionState> { self.meteor_arrow_mass }
    pub(crate) fn begin_meteor_arrow_mass(&mut self, mut state: MeteorArrowMassExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.meteor_arrow_mass = Some(state);
    }
    pub(crate) fn meteor_arrow_mass_mut(&mut self) -> Option<&mut MeteorArrowMassExecutionState> { self.meteor_arrow_mass.as_mut() }
    pub(crate) const fn meteor_arrow_mass_last_used_ms(&self) -> u32 { self.meteor_arrow_mass_last_used_ms }
    pub(crate) const fn mark_meteor_arrow_mass_used(&mut self, now_ms: u32) { self.meteor_arrow_mass_last_used_ms = now_ms; }
    pub(crate) const fn meteor_arrow(&self) -> Option<MeteorArrowExecutionState> { self.meteor_arrow }
    pub(crate) fn begin_meteor_arrow(&mut self, mut state: MeteorArrowExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.meteor_arrow = Some(state);
    }
    pub(crate) fn meteor_arrow_mut(&mut self) -> Option<&mut MeteorArrowExecutionState> { self.meteor_arrow.as_mut() }
    pub(crate) const fn meteor_arrow_last_used_ms(&self) -> u32 { self.meteor_arrow_last_used_ms }
    pub(crate) const fn mark_meteor_arrow_used(&mut self, now_ms: u32) { self.meteor_arrow_last_used_ms = now_ms; }
    pub(crate) fn rain_arrow(&self) -> Option<RainArrowExecutionState> { self.rain_arrow.clone() }
    pub(crate) fn begin_rain_arrow(&mut self, mut state: RainArrowExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.rain_arrow = Some(state);
    }
    pub(crate) fn rain_arrow_mut(&mut self) -> Option<&mut RainArrowExecutionState> { self.rain_arrow.as_mut() }
    pub(crate) const fn rain_arrow_last_used_ms(&self) -> u32 { self.rain_arrow_last_used_ms }
    pub(crate) const fn mark_rain_arrow_used(&mut self, now_ms: u32) { self.rain_arrow_last_used_ms = now_ms; }
    pub(crate) fn poison_moth(&self) -> Option<&PoisonMothExecutionState> { self.poison_moth.as_ref() }
    pub(crate) fn begin_poison_moth(&mut self, mut state: PoisonMothExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.poison_moth = Some(state);
    }
    pub(crate) fn poison_moth_mut(&mut self) -> Option<&mut PoisonMothExecutionState> { self.poison_moth.as_mut() }
    pub(crate) const fn poison_moth_last_used_ms(&self) -> u32 { self.poison_moth_last_used_ms }
    pub(crate) const fn mark_poison_moth_used(&mut self, now_ms: u32) { self.poison_moth_last_used_ms = now_ms; }
    pub(crate) fn blood_rose(&self) -> Option<&BloodRoseExecutionState> { self.blood_rose.as_ref() }
    pub(crate) fn begin_blood_rose(&mut self, mut state: BloodRoseExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.blood_rose = Some(state);
    }
    pub(crate) fn blood_rose_mut(&mut self) -> Option<&mut BloodRoseExecutionState> { self.blood_rose.as_mut() }
    pub(crate) const fn blood_rose_last_used_ms(&self) -> u32 { self.blood_rose_last_used_ms }
    pub(crate) const fn mark_blood_rose_used(&mut self, now_ms: u32) { self.blood_rose_last_used_ms = now_ms; }
    pub(crate) fn scorpion(&self) -> Option<&ScorpionExecutionState> { self.scorpion.as_ref() }
    pub(crate) fn begin_scorpion(&mut self, mut state: ScorpionExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.scorpion = Some(state);
    }
    pub(crate) fn scorpion_mut(&mut self) -> Option<&mut ScorpionExecutionState> { self.scorpion.as_mut() }
    pub(crate) const fn scorpion_last_used_ms(&self) -> u32 { self.scorpion_last_used_ms }
    pub(crate) const fn mark_scorpion_used(&mut self, now_ms: u32) { self.scorpion_last_used_ms = now_ms; }
    pub(crate) fn boa_lock(&self) -> Option<&BoaLockExecutionState> { self.boa_lock.as_ref() }
    pub(crate) fn begin_boa_lock(&mut self, mut state: BoaLockExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.boa_lock = Some(state);
    }
    pub(crate) fn boa_lock_mut(&mut self) -> Option<&mut BoaLockExecutionState> { self.boa_lock.as_mut() }
    pub(crate) const fn boa_lock_last_used_ms(&self) -> u32 { self.boa_lock_last_used_ms }
    pub(crate) const fn mark_boa_lock_used(&mut self, now_ms: u32) { self.boa_lock_last_used_ms = now_ms; }
    pub(crate) fn falling_star(&self) -> Option<&FallingStarExecutionState> { self.falling_star.as_ref() }
    pub(crate) fn begin_falling_star(&mut self, mut state: FallingStarExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.falling_star = Some(state);
    }
    pub(crate) fn falling_star_mut(&mut self) -> Option<&mut FallingStarExecutionState> { self.falling_star.as_mut() }
    pub(crate) const fn falling_star_last_used_ms(&self) -> u32 { self.falling_star_last_used_ms }
    pub(crate) const fn mark_falling_star_used(&mut self, now_ms: u32) { self.falling_star_last_used_ms = now_ms; }
    pub(crate) fn explosive_arrow(&self) -> Option<&ExplosiveArrowExecutionState> { self.explosive_arrow.as_ref() }
    pub(crate) fn begin_explosive_arrow(&mut self, mut state: ExplosiveArrowExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.explosive_arrow = Some(state);
    }
    pub(crate) fn explosive_arrow_mut(&mut self) -> Option<&mut ExplosiveArrowExecutionState> { self.explosive_arrow.as_mut() }
    pub(crate) const fn explosive_arrow_last_used_ms(&self, variant: ExplosiveArrowVariant) -> u32 { self.explosive_arrow_last_used_ms[variant.index()] }
    pub(crate) fn mark_explosive_arrow_used(&mut self, variant: ExplosiveArrowVariant, now_ms: u32) { self.explosive_arrow_last_used_ms[variant.index()] = now_ms; }
    pub(crate) const fn strike(&self) -> Option<StrikeExecutionState> { self.strike }
    pub(crate) fn begin_strike(&mut self, mut state: StrikeExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.strike = Some(state);
    }
    pub(crate) fn strike_mut(&mut self) -> Option<&mut StrikeExecutionState> { self.strike.as_mut() }
    pub(crate) const fn strike_last_used_ms(&self) -> u32 { self.strike_last_used_ms }
    pub(crate) const fn mark_strike_used(&mut self, now_ms: u32) { self.strike_last_used_ms = now_ms; }

    pub(crate) const fn daub_poison(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.daub_poison }
    pub(crate) fn begin_daub_poison(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.daub_poison = Some(state);
    }
    pub(crate) fn daub_poison_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.daub_poison.as_mut() }
    pub(crate) const fn daub_poison_last_used_ms(&self) -> u32 { self.daub_poison_last_used_ms }
    pub(crate) const fn mark_daub_poison_used(&mut self, now_ms: u32) { self.daub_poison_last_used_ms = now_ms; }
    pub(crate) const fn yaksha_slash(&self) -> Option<YakshaSlashExecutionState> { self.yaksha_slash }
    pub(crate) fn begin_yaksha_slash(&mut self, mut state: YakshaSlashExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.yaksha_slash = Some(state);
    }
    pub(crate) fn yaksha_slash_mut(&mut self) -> Option<&mut YakshaSlashExecutionState> { self.yaksha_slash.as_mut() }
    pub(crate) const fn yaksha_slash_last_used_ms(&self) -> u32 { self.yaksha_slash_last_used_ms }
    pub(crate) const fn mark_yaksha_slash_used(&mut self, now_ms: u32) { self.yaksha_slash_last_used_ms = now_ms; }

    pub(crate) fn begin_base_attack(&mut self, mut state: BaseAttackExecutionState) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.base_attack = Some(state);
    }

    pub(crate) fn advance_base_attack(
        &mut self,
        expected: SkillStage,
        next: SkillStage,
    ) -> bool {
        self.base_attack
            .as_mut()
            .is_some_and(|state| state.advance(expected, next))
    }

    pub(crate) const fn base_attack_last_used_ms(&self) -> u32 {
        self.base_attack_last_used_ms
    }

    pub(crate) const fn mark_base_attack_used(&mut self, now_ms: u32) {
        self.base_attack_last_used_ms = now_ms;
    }

    pub(crate) const fn base_magic(&self) -> Option<BaseMagicExecutionState> {
        self.base_magic
    }

    pub(crate) fn begin_base_magic(&mut self, mut state: BaseMagicExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.base_magic = Some(state);
    }

    pub(crate) fn base_magic_mut(&mut self) -> Option<&mut BaseMagicExecutionState> {
        self.base_magic.as_mut()
    }

    pub(crate) const fn base_magic_last_used_ms(&self) -> u32 {
        self.base_magic_last_used_ms
    }

    pub(crate) const fn mark_base_magic_used(&mut self, now_ms: u32) {
        self.base_magic_last_used_ms = now_ms;
    }

    pub(crate) const fn fire_bolt(&self) -> Option<BaseMagicExecutionState> {
        self.fire_bolt
    }

    pub(crate) fn begin_fire_bolt(&mut self, mut state: BaseMagicExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.fire_bolt = Some(state);
    }

    pub(crate) fn fire_bolt_mut(&mut self) -> Option<&mut BaseMagicExecutionState> {
        self.fire_bolt.as_mut()
    }

    pub(crate) const fn fire_bolt_last_used_ms(&self) -> u32 {
        self.fire_bolt_last_used_ms
    }

    pub(crate) const fn mark_fire_bolt_used(&mut self, now_ms: u32) {
        self.fire_bolt_last_used_ms = now_ms;
    }

    pub(crate) const fn fire_ball(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_ball
    }

    pub(crate) fn begin_fire_ball(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.fire_ball = Some(state);
    }

    pub(crate) fn fire_ball_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_ball.as_mut()
    }

    pub(crate) const fn fire_ball_last_used_ms(&self) -> u32 { self.fire_ball_last_used_ms }

    pub(crate) const fn mark_fire_ball_used(&mut self, now_ms: u32) {
        self.fire_ball_last_used_ms = now_ms;
    }

    pub(crate) const fn item_skill_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.item_skill_2
    }

    pub(crate) fn begin_item_skill_2(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.item_skill_2 = Some(state);
    }

    pub(crate) fn item_skill_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.item_skill_2.as_mut()
    }

    pub(crate) const fn item_skill_2_last_used_ms(&self) -> u32 { self.item_skill_2_last_used_ms }

    pub(crate) const fn mark_item_skill_2_used(&mut self, now_ms: u32) {
        self.item_skill_2_last_used_ms = now_ms;
    }

    pub(crate) const fn chain_lightning(&self) -> Option<ChainLightningExecutionState> {
        self.chain_lightning
    }

    pub(crate) fn begin_chain_lightning(&mut self, mut state: ChainLightningExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.chain_lightning = Some(state);
    }

    pub(crate) fn chain_lightning_mut(&mut self) -> Option<&mut ChainLightningExecutionState> {
        self.chain_lightning.as_mut()
    }

    pub(crate) const fn chain_lightning_last_used_ms(&self) -> u32 { self.chain_lightning_last_used_ms }

    pub(crate) const fn mark_chain_lightning_used(&mut self, now_ms: u32) {
        self.chain_lightning_last_used_ms = now_ms;
    }

    pub(crate) const fn thunder_blow(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow }
    pub(crate) fn begin_thunder_blow(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.thunder_blow = Some(state);
    }
    pub(crate) fn thunder_blow_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow.as_mut() }
    pub(crate) const fn thunder_blow_last_used_ms(&self) -> u32 { self.thunder_blow_last_used_ms }
    pub(crate) const fn mark_thunder_blow_used(&mut self, now_ms: u32) { self.thunder_blow_last_used_ms = now_ms; }

    pub(crate) const fn thunder_slash(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_slash }
    pub(crate) fn begin_thunder_slash(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.thunder_slash = Some(state);
    }
    pub(crate) fn thunder_slash_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_slash.as_mut() }
    pub(crate) const fn thunder_slash_last_used_ms(&self) -> u32 { self.thunder_slash_last_used_ms }
    pub(crate) const fn mark_thunder_slash_used(&mut self, now_ms: u32) { self.thunder_slash_last_used_ms = now_ms; }

    pub(crate) const fn pillar(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.pillar }
    pub(crate) fn begin_pillar(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.pillar = Some(state);
    }
    pub(crate) fn pillar_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.pillar.as_mut() }
    pub(crate) const fn pillar_last_used_ms(&self) -> u32 { self.pillar_last_used_ms }
    pub(crate) const fn mark_pillar_used(&mut self, now_ms: u32) { self.pillar_last_used_ms = now_ms; }

    pub(crate) const fn rush(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.rush }
    pub(crate) fn begin_rush(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.rush = Some(state);
    }
    pub(crate) fn rush_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.rush.as_mut() }
    pub(crate) const fn rush_last_used_ms(&self) -> u32 { self.rush_last_used_ms }
    pub(crate) const fn mark_rush_used(&mut self, now_ms: u32) { self.rush_last_used_ms = now_ms; }

    pub(crate) const fn rush_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.rush_2 }
    pub(crate) fn begin_rush_2(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.rush_2 = Some(state);
    }
    pub(crate) fn rush_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.rush_2.as_mut() }
    pub(crate) const fn rush_2_last_used_ms(&self) -> u32 { self.rush_2_last_used_ms }
    pub(crate) const fn mark_rush_2_used(&mut self, now_ms: u32) { self.rush_2_last_used_ms = now_ms; }

    pub(crate) const fn roar(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.roar }
    pub(crate) fn begin_roar(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.roar = Some(state);
    }
    pub(crate) fn roar_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.roar.as_mut() }
    pub(crate) const fn roar_last_used_ms(&self) -> u32 { self.roar_last_used_ms }
    pub(crate) const fn mark_roar_used(&mut self, now_ms: u32) { self.roar_last_used_ms = now_ms; }

    pub(crate) const fn energy_holding(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.energy_holding }
    pub(crate) fn begin_energy_holding(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.energy_holding = Some(state);
    }
    pub(crate) fn energy_holding_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.energy_holding.as_mut() }
    pub(crate) const fn energy_holding_last_used_ms(&self) -> u32 { self.energy_holding_last_used_ms }
    pub(crate) const fn mark_energy_holding_used(&mut self, now_ms: u32) { self.energy_holding_last_used_ms = now_ms; }

    pub(crate) const fn inverse_chopped(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.inverse_chopped }
    pub(crate) fn begin_inverse_chopped(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.inverse_chopped = Some(state);
    }
    pub(crate) fn inverse_chopped_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.inverse_chopped.as_mut() }
    pub(crate) const fn inverse_chopped_last_used_ms(&self) -> u32 { self.inverse_chopped_last_used_ms }
    pub(crate) const fn mark_inverse_chopped_used(&mut self, now_ms: u32) { self.inverse_chopped_last_used_ms = now_ms; }

    pub(crate) const fn thunder_blow_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow_2 }
    pub(crate) fn begin_thunder_blow_2(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.thunder_blow_2 = Some(state);
    }
    pub(crate) fn thunder_blow_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow_2.as_mut() }
    pub(crate) const fn thunder_blow_2_last_used_ms(&self) -> u32 { self.thunder_blow_2_last_used_ms }
    pub(crate) const fn mark_thunder_blow_2_used(&mut self, now_ms: u32) { self.thunder_blow_2_last_used_ms = now_ms; }

    pub(crate) const fn mosou(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.mosou }
    pub(crate) fn begin_mosou(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.mosou = Some(state);
    }
    pub(crate) fn mosou_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.mosou.as_mut() }
    pub(crate) const fn mosou_last_used_ms(&self) -> u32 { self.mosou_last_used_ms }
    pub(crate) const fn mark_mosou_used(&mut self, now_ms: u32) { self.mosou_last_used_ms = now_ms; }

    pub(crate) const fn ghost_cut(&self) -> Option<&GhostCutExecutionState> { self.ghost_cut.as_ref() }
    pub(crate) fn begin_ghost_cut(&mut self, mut state: GhostCutExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.ghost_cut = Some(state);
    }
    pub(crate) fn ghost_cut_mut(&mut self) -> Option<&mut GhostCutExecutionState> { self.ghost_cut.as_mut() }
    const fn ghost_cut_index(skill_id: u32) -> usize {
        match skill_id { GHOST_CUT_SKILL_ID => 0, GHOST_CUT_2_SKILL_ID => 1, GHOST_CUT_3_SKILL_ID => 2, _ => unreachable!() }
    }
    pub(crate) const fn ghost_cut_last_used_ms(&self, skill_id: u32) -> u32 { self.ghost_cut_last_used_ms[Self::ghost_cut_index(skill_id)] }
    pub(crate) fn mark_ghost_cut_used(&mut self, skill_id: u32, now_ms: u32) { self.ghost_cut_last_used_ms[Self::ghost_cut_index(skill_id)] = now_ms; }

    pub(crate) const fn knight_cut(&self) -> Option<&KnightCutExecutionState> { self.knight_cut.as_ref() }
    pub(crate) fn begin_knight_cut(&mut self, mut state: KnightCutExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.knight_cut = Some(state);
    }
    pub(crate) fn knight_cut_mut(&mut self) -> Option<&mut KnightCutExecutionState> { self.knight_cut.as_mut() }
    pub(crate) const fn knight_cut_last_used_ms(&self) -> u32 { self.knight_cut_last_used_ms }
    pub(crate) const fn mark_knight_cut_used(&mut self, now_ms: u32) { self.knight_cut_last_used_ms = now_ms; }
    pub(crate) const fn army_break(&self) -> Option<&ArmyBreakExecutionState> { self.army_break.as_ref() }
    pub(crate) fn begin_army_break(&mut self, mut state: ArmyBreakExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.army_break = Some(state);
    }
    pub(crate) fn army_break_mut(&mut self) -> Option<&mut ArmyBreakExecutionState> { self.army_break.as_mut() }
    const fn army_break_index(skill_id: u32) -> usize { match skill_id { ARMY_BREAK_SKILL_ID => 0, ARMY_BREAK_2_SKILL_ID => 1, _ => unreachable!() } }
    pub(crate) const fn army_break_last_used_ms(&self, skill_id: u32) -> u32 { self.army_break_last_used_ms[Self::army_break_index(skill_id)] }
    pub(crate) fn mark_army_break_used(&mut self, skill_id: u32, now_ms: u32) { self.army_break_last_used_ms[Self::army_break_index(skill_id)] = now_ms; }
    pub(crate) const fn rage(&self) -> Option<&RageExecutionState> { self.rage.as_ref() }
    pub(crate) fn begin_rage(&mut self, mut state: RageExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.rage = Some(state);
    }
    pub(crate) fn rage_mut(&mut self) -> Option<&mut RageExecutionState> { self.rage.as_mut() }
    pub(crate) const fn rage_last_used_ms(&self) -> u32 { self.rage_last_used_ms }
    pub(crate) const fn mark_rage_used(&mut self, now_ms: u32) { self.rage_last_used_ms = now_ms; }
    pub(crate) const fn rage_break(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.rage_break }
    pub(crate) fn begin_rage_break(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.rage_break = Some(state);
    }
    pub(crate) fn rage_break_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.rage_break.as_mut() }
    pub(crate) const fn rage_break_last_used_ms(&self) -> u32 { self.rage_break_last_used_ms }
    pub(crate) const fn mark_rage_break_used(&mut self, now_ms: u32) { self.rage_break_last_used_ms = now_ms; }
    pub(crate) const fn fury(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.fury }
    pub(crate) fn begin_fury(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.fury = Some(state);
    }
    pub(crate) fn fury_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.fury.as_mut() }
    pub(crate) const fn fury_last_used_ms(&self) -> u32 { self.fury_last_used_ms }
    pub(crate) const fn mark_fury_used(&mut self, now_ms: u32) { self.fury_last_used_ms = now_ms; }
    pub(crate) const fn flash(&self) -> Option<&FlashExecutionState> { self.flash.as_ref() }
    pub(crate) fn begin_flash(&mut self, mut state: FlashExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.flash = Some(state);
    }
    pub(crate) fn flash_mut(&mut self) -> Option<&mut FlashExecutionState> { self.flash.as_mut() }
    pub(crate) const fn flash_last_used_ms(&self) -> u32 { self.flash_last_used_ms }
    pub(crate) const fn mark_flash_used(&mut self, now_ms: u32) { self.flash_last_used_ms = now_ms; }
    pub(crate) const fn swallow(&self) -> Option<&SwallowExecutionState> { self.swallow.as_ref() }
    pub(crate) fn begin_swallow(&mut self, mut state: SwallowExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.swallow = Some(state);
    }
    pub(crate) fn swallow_mut(&mut self) -> Option<&mut SwallowExecutionState> { self.swallow.as_mut() }
    pub(crate) const fn swallow_last_used_ms(&self) -> u32 { self.swallow_last_used_ms }
    pub(crate) const fn mark_swallow_used(&mut self, now_ms: u32) { self.swallow_last_used_ms = now_ms; }
    pub(crate) const fn leaf_cut(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.leaf_cut }
    pub(crate) fn begin_leaf_cut(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.leaf_cut = Some(state);
    }
    pub(crate) fn leaf_cut_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.leaf_cut.as_mut() }
    pub(crate) const fn leaf_cut_last_used_ms(&self) -> u32 { self.leaf_cut_last_used_ms }
    pub(crate) const fn mark_leaf_cut_used(&mut self, now_ms: u32) { self.leaf_cut_last_used_ms = now_ms; }
    pub(crate) const fn leaf_cut_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.leaf_cut_2 }
    pub(crate) fn begin_leaf_cut_2(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.leaf_cut_2 = Some(state);
    }
    pub(crate) fn leaf_cut_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.leaf_cut_2.as_mut() }
    pub(crate) const fn leaf_cut_2_last_used_ms(&self) -> u32 { self.leaf_cut_2_last_used_ms }
    pub(crate) const fn mark_leaf_cut_2_used(&mut self, now_ms: u32) { self.leaf_cut_2_last_used_ms = now_ms; }
    pub(crate) const fn leaf_cut_3(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.leaf_cut_3 }
    pub(crate) fn begin_leaf_cut_3(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.leaf_cut_3 = Some(state);
    }
    pub(crate) fn leaf_cut_3_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.leaf_cut_3.as_mut() }
    pub(crate) const fn leaf_cut_3_last_used_ms(&self) -> u32 { self.leaf_cut_3_last_used_ms }
    pub(crate) const fn mark_leaf_cut_3_used(&mut self, now_ms: u32) { self.leaf_cut_3_last_used_ms = now_ms; }
    pub(crate) const fn kerosene(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.kerosene }
    pub(crate) fn begin_kerosene(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.kerosene = Some(state);
    }
    pub(crate) fn kerosene_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.kerosene.as_mut() }
    pub(crate) const fn kerosene_last_used_ms(&self) -> u32 { self.kerosene_last_used_ms }
    pub(crate) const fn mark_kerosene_used(&mut self, now_ms: u32) { self.kerosene_last_used_ms = now_ms; }
    pub(crate) const fn ignition(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.ignition }
    pub(crate) fn begin_ignition(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.ignition = Some(state);
    }
    pub(crate) fn ignition_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.ignition.as_mut() }
    pub(crate) const fn ignition_last_used_ms(&self) -> u32 { self.ignition_last_used_ms }
    pub(crate) const fn mark_ignition_used(&mut self, now_ms: u32) { self.ignition_last_used_ms = now_ms; }
    pub(crate) const fn blind(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.blind }
    pub(crate) fn begin_blind(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.blind = Some(state);
    }
    pub(crate) fn blind_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.blind.as_mut() }
    pub(crate) const fn blind_last_used_ms(&self) -> u32 { self.blind_last_used_ms }
    pub(crate) const fn mark_blind_used(&mut self, now_ms: u32) { self.blind_last_used_ms = now_ms; }
    pub(crate) const fn ju_cut(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.ju_cut }
    pub(crate) fn begin_ju_cut(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.ju_cut = Some(state);
    }
    pub(crate) fn ju_cut_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.ju_cut.as_mut() }
    pub(crate) const fn ju_cut_last_used_ms(&self) -> u32 { self.ju_cut_last_used_ms }
    pub(crate) const fn mark_ju_cut_used(&mut self, now_ms: u32) { self.ju_cut_last_used_ms = now_ms; }
    pub(crate) const fn lightning_sword(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.lightning_sword }
    pub(crate) fn begin_lightning_sword(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.lightning_sword = Some(state);
    }
    pub(crate) fn lightning_sword_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.lightning_sword.as_mut() }
    const fn lightning_sword_index(skill_id: u32) -> usize { match skill_id { 0x70 => 0, 0x77 => 1, 0x78 => 2, 0x7e => 3, _ => unreachable!() } }
    pub(crate) const fn lightning_sword_last_used_ms(&self, skill_id: u32) -> u32 { self.lightning_sword_last_used_ms[Self::lightning_sword_index(skill_id)] }
    pub(crate) fn mark_lightning_sword_used(&mut self, skill_id: u32, now_ms: u32) { self.lightning_sword_last_used_ms[Self::lightning_sword_index(skill_id)] = now_ms; }
    pub(crate) const fn little_flash(&self) -> Option<&LittleFlashExecutionState> { self.little_flash.as_ref() }
    pub(crate) fn begin_little_flash(&mut self, mut state: LittleFlashExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.little_flash = Some(state);
    }
    pub(crate) fn little_flash_mut(&mut self) -> Option<&mut LittleFlashExecutionState> { self.little_flash.as_mut() }
    const fn little_flash_index(skill_id: u32) -> usize {
        match skill_id {
            0x71 => 0,
            0x7f => 1,
            _ => unreachable!(),
        }
    }
    pub(crate) const fn little_flash_last_used_ms(&self, skill_id: u32) -> u32 {
        self.little_flash_last_used_ms[Self::little_flash_index(skill_id)]
    }
    pub(crate) fn mark_little_flash_used(&mut self, skill_id: u32, now_ms: u32) {
        self.little_flash_last_used_ms[Self::little_flash_index(skill_id)] = now_ms;
    }

    pub(crate) const fn fire_wall(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_wall
    }

    pub(crate) fn begin_fire_wall(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.fire_wall = Some(state);
    }

    pub(crate) fn fire_wall_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_wall.as_mut()
    }

    pub(crate) const fn fire_wall_last_used_ms(&self) -> u32 {
        self.fire_wall_last_used_ms
    }

    pub(crate) const fn mark_fire_wall_used(&mut self, now_ms: u32) {
        self.fire_wall_last_used_ms = now_ms;
    }

    pub(crate) const fn poison_fog(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.poison_fog }
    pub(crate) fn begin_poison_fog(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>, destination: (i32, i32)) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.poison_fog = Some(state); self.poison_fog_destination = Some(destination);
    }
    pub(crate) fn poison_fog_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.poison_fog.as_mut() }
    pub(crate) const fn poison_fog_last_used_ms(&self) -> u32 { self.poison_fog_last_used_ms }
    pub(crate) const fn mark_poison_fog_used(&mut self, now_ms: u32) { self.poison_fog_last_used_ms = now_ms; }
    pub(crate) const fn poison_fog_destination(&self) -> Option<(i32, i32)> { self.poison_fog_destination }

    pub(crate) const fn infernol(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.infernol
    }

    pub(crate) fn begin_infernol(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.infernol = Some(state);
    }

    pub(crate) fn infernol_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.infernol.as_mut()
    }

    pub(crate) const fn infernol_last_used_ms(&self) -> u32 {
        self.infernol_last_used_ms
    }

    pub(crate) const fn mark_infernol_used(&mut self, now_ms: u32) {
        self.infernol_last_used_ms = now_ms;
    }

    pub(crate) const fn seven_shooting_star(&self) -> Option<&SevenShootingStarExecutionState> {
        self.seven_shooting_star.as_ref()
    }

    pub(crate) fn begin_seven_shooting_star(
        &mut self,
        mut state: SevenShootingStarExecutionState,
    ) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.seven_shooting_star = Some(state);
    }

    pub(crate) fn seven_shooting_star_mut(
        &mut self,
    ) -> Option<&mut SevenShootingStarExecutionState> {
        self.seven_shooting_star.as_mut()
    }

    pub(crate) const fn seven_shooting_star_last_used_ms(&self) -> u32 {
        self.seven_shooting_star_last_used_ms
    }

    pub(crate) const fn mark_seven_shooting_star_used(&mut self, now_ms: u32) {
        self.seven_shooting_star_last_used_ms = now_ms;
    }

    pub(crate) const fn little_star(&self) -> Option<&PlayerLittleStarExecutionState> {
        self.little_star.as_ref()
    }

    pub(crate) fn begin_little_star(&mut self, mut state: PlayerLittleStarExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.little_star = Some(state);
    }

    pub(crate) fn little_star_mut(&mut self) -> Option<&mut PlayerLittleStarExecutionState> {
        self.little_star.as_mut()
    }

    pub(crate) const fn little_star_last_used_ms(&self) -> u32 {
        self.little_star_last_used_ms
    }

    pub(crate) const fn mark_little_star_used(&mut self, now_ms: u32) {
        self.little_star_last_used_ms = now_ms;
    }

    pub(crate) const fn path_projectile(&self) -> Option<&PlayerPathProjectileExecutionState> {
        self.path_projectile.as_ref()
    }

    pub(crate) fn begin_path_projectile(&mut self, mut state: PlayerPathProjectileExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.path_projectile = Some(state);
    }

    pub(crate) fn path_projectile_mut(&mut self) -> Option<&mut PlayerPathProjectileExecutionState> {
        self.path_projectile.as_mut()
    }

    const fn path_projectile_index(skill_id: u32) -> usize {
        match skill_id {
            0x1a0 => 0,
            0x1a2 => 1,
            0x1a5 => 2,
            _ => unreachable!(),
        }
    }

    pub(crate) const fn path_projectile_last_used_ms(&self, skill_id: u32) -> u32 {
        self.path_projectile_last_used_ms[Self::path_projectile_index(skill_id)]
    }

    pub(crate) fn mark_path_projectile_used(&mut self, skill_id: u32, now_ms: u32) {
        self.path_projectile_last_used_ms[Self::path_projectile_index(skill_id)] = now_ms;
    }

    pub(crate) const fn direct_projectile(&self) -> Option<&PlayerDirectProjectileExecutionState> {
        self.direct_projectile.as_ref()
    }

    pub(crate) fn begin_direct_projectile(&mut self, mut state: PlayerDirectProjectileExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.direct_projectile = Some(state);
    }

    pub(crate) fn direct_projectile_mut(&mut self) -> Option<&mut PlayerDirectProjectileExecutionState> {
        self.direct_projectile.as_mut()
    }

    const fn direct_projectile_index(skill_id: u32) -> usize {
        match skill_id {
            0x19d => 0,
            0x1a1 => 1,
            _ => unreachable!(),
        }
    }

    pub(crate) const fn direct_projectile_last_used_ms(&self, skill_id: u32) -> u32 {
        self.direct_projectile_last_used_ms[Self::direct_projectile_index(skill_id)]
    }

    pub(crate) fn mark_direct_projectile_used(&mut self, skill_id: u32, now_ms: u32) {
        self.direct_projectile_last_used_ms[Self::direct_projectile_index(skill_id)] = now_ms;
    }

    pub(crate) const fn yunsheng_lightning(&self) -> Option<&PlayerYunShengLightningExecutionState> { self.yunsheng_lightning.as_ref() }
    pub(crate) fn begin_yunsheng_lightning(&mut self, mut state: PlayerYunShengLightningExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.yunsheng_lightning = Some(state);
    }
    pub(crate) fn yunsheng_lightning_mut(&mut self) -> Option<&mut PlayerYunShengLightningExecutionState> { self.yunsheng_lightning.as_mut() }
    pub(crate) const fn yunsheng_lightning_last_used_ms(&self) -> u32 { self.yunsheng_lightning_last_used_ms }
    pub(crate) const fn mark_yunsheng_lightning_used(&mut self, now_ms: u32) { self.yunsheng_lightning_last_used_ms = now_ms; }
    pub(crate) const fn corpse_ptomaine(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.corpse_ptomaine }
    pub(crate) fn begin_corpse_ptomaine(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.corpse_ptomaine = Some(state);
    }
    pub(crate) fn corpse_ptomaine_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.corpse_ptomaine.as_mut() }
    pub(crate) const fn corpse_ptomaine_last_used_ms(&self) -> u32 { self.corpse_ptomaine_last_used_ms }
    pub(crate) const fn mark_corpse_ptomaine_used(&mut self, now_ms: u32) { self.corpse_ptomaine_last_used_ms = now_ms; }
    pub(crate) const fn monster_thorn(&self) -> Option<&PlayerMonsterThornExecutionState> { self.monster_thorn.as_ref() }
    pub(crate) fn begin_monster_thorn(&mut self, mut state: PlayerMonsterThornExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.monster_thorn = Some(state);
    }
    pub(crate) fn monster_thorn_mut(&mut self) -> Option<&mut PlayerMonsterThornExecutionState> { self.monster_thorn.as_mut() }
    pub(crate) const fn monster_thorn_last_used_ms(&self) -> u32 { self.monster_thorn_last_used_ms }
    pub(crate) const fn mark_monster_thorn_used(&mut self, now_ms: u32) { self.monster_thorn_last_used_ms = now_ms; }
    pub(crate) const fn spider_mist(&self) -> Option<&PlayerSpiderMistExecutionState> { self.spider_mist.as_ref() }
    pub(crate) fn begin_spider_mist(&mut self, mut state: PlayerSpiderMistExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.spider_mist = Some(state);
    }
    pub(crate) fn spider_mist_mut(&mut self) -> Option<&mut PlayerSpiderMistExecutionState> { self.spider_mist.as_mut() }
    pub(crate) const fn spider_mist_last_used_ms(&self) -> u32 { self.spider_mist_last_used_ms }
    pub(crate) const fn mark_spider_mist_used(&mut self, now_ms: u32) { self.spider_mist_last_used_ms = now_ms; }
    pub(crate) const fn spider_web(&self) -> Option<&PlayerSpiderWebExecutionState> { self.spider_web.as_ref() }
    pub(crate) fn begin_spider_web(&mut self, mut state: PlayerSpiderWebExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.spider_web = Some(state);
    }
    pub(crate) fn spider_web_mut(&mut self) -> Option<&mut PlayerSpiderWebExecutionState> { self.spider_web.as_mut() }
    pub(crate) const fn spider_web_last_used_ms(&self) -> u32 { self.spider_web_last_used_ms }
    pub(crate) const fn mark_spider_web_used(&mut self, now_ms: u32) { self.spider_web_last_used_ms = now_ms; }
    pub(crate) const fn spider_poison(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.spider_poison }
    pub(crate) fn begin_spider_poison(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.spider_poison = Some(state);
    }
    pub(crate) fn spider_poison_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.spider_poison.as_mut() }
    pub(crate) const fn spider_poison_last_used_ms(&self) -> u32 { self.spider_poison_last_used_ms }
    pub(crate) const fn mark_spider_poison_used(&mut self, now_ms: u32) { self.spider_poison_last_used_ms = now_ms; }
    pub(crate) const fn summon_creature(&self) -> Option<&PlayerSummonCreatureExecutionState> { self.summon_creature.as_ref() }
    pub(crate) fn begin_summon_creature(&mut self, mut state: PlayerSummonCreatureExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.summon_creature = Some(state);
    }
    pub(crate) fn summon_creature_mut(&mut self) -> Option<&mut PlayerSummonCreatureExecutionState> { self.summon_creature.as_mut() }
    pub(crate) const fn summon_creature_last_used_ms(&self, index: usize) -> u32 { self.summon_creature_last_used_ms[index] }
    pub(crate) const fn mark_summon_creature_used(&mut self, index: usize, now_ms: u32) { self.summon_creature_last_used_ms[index] = now_ms; }
    pub(crate) const fn boss_blue_fury(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.boss_blue_fury }
    pub(crate) fn begin_boss_blue_fury(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.boss_blue_fury = Some(state);
    }
    pub(crate) fn boss_blue_fury_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.boss_blue_fury.as_mut() }
    pub(crate) const fn boss_blue_fury_last_used_ms(&self) -> u32 { self.boss_blue_fury_last_used_ms }
    pub(crate) const fn mark_boss_blue_fury_used(&mut self, now_ms: u32) { self.boss_blue_fury_last_used_ms = now_ms; }
    pub(crate) const fn boss_blue_quake(&self) -> Option<PlayerBossBlueQuakeExecutionState> { self.boss_blue_quake }
    pub(crate) fn begin_boss_blue_quake(&mut self, mut state: PlayerBossBlueQuakeExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.boss_blue_quake = Some(state);
    }
    pub(crate) fn boss_blue_quake_mut(&mut self) -> Option<&mut PlayerBossBlueQuakeExecutionState> { self.boss_blue_quake.as_mut() }
    pub(crate) const fn boss_blue_quake_last_used_ms(&self) -> u32 { self.boss_blue_quake_last_used_ms }
    pub(crate) const fn mark_boss_blue_quake_used(&mut self, now_ms: u32) { self.boss_blue_quake_last_used_ms = now_ms; }
    pub(crate) fn boss_fiend_penetrate(&self) -> Option<&PlayerBossFiendPenetrateExecutionState> { self.boss_fiend_penetrate.as_ref() }
    pub(crate) fn begin_boss_fiend_penetrate(&mut self, mut state: PlayerBossFiendPenetrateExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.boss_fiend_penetrate = Some(state);
    }
    pub(crate) fn boss_fiend_penetrate_mut(&mut self) -> Option<&mut PlayerBossFiendPenetrateExecutionState> { self.boss_fiend_penetrate.as_mut() }
    pub(crate) const fn boss_fiend_penetrate_last_used_ms(&self) -> u32 { self.boss_fiend_penetrate_last_used_ms }
    pub(crate) const fn mark_boss_fiend_penetrate_used(&mut self, now_ms: u32) { self.boss_fiend_penetrate_last_used_ms = now_ms; }

    pub(crate) const fn sprite_burn(&self) -> Option<&SpriteBurnExecutionState> {
        self.sprite_burn.as_ref()
    }

    pub(crate) fn begin_sprite_burn(&mut self, mut state: SpriteBurnExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.sprite_burn = Some(state);
    }

    pub(crate) fn sprite_burn_mut(&mut self) -> Option<&mut SpriteBurnExecutionState> {
        self.sprite_burn.as_mut()
    }

    pub(crate) const fn sprite_burn_last_used_ms(&self) -> u32 {
        self.sprite_burn_last_used_ms
    }

    pub(crate) const fn mark_sprite_burn_used(&mut self, now_ms: u32) {
        self.sprite_burn_last_used_ms = now_ms;
    }

    pub(crate) const fn wide_arc_attack(&self) -> Option<&SkillExecutionKernel<PlayerSkillDispatch>> {
        self.wide_arc_attack.as_ref()
    }

    pub(crate) fn begin_wide_arc_attack(&mut self, dispatch: PlayerSkillDispatch, now_ms: u32) {
        let now_ms = self.scheduled_skill_begin.filter(|(queued, _)| *queued == dispatch)
            .map_or(now_ms, |(_, started)| started);
        self.wide_arc_attack = Some(SkillExecutionKernel::begin(dispatch, now_ms));
    }

    pub(crate) fn wide_arc_attack_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.wide_arc_attack.as_mut()
    }

    const fn wide_arc_attack_index(skill_id: u32) -> usize {
        match skill_id {
            0x1a7 => 0,
            0x1f6 => 1,
            _ => unreachable!(),
        }
    }

    pub(crate) const fn wide_arc_attack_last_used_ms(&self, skill_id: u32) -> u32 {
        self.wide_arc_attack_last_used_ms[Self::wide_arc_attack_index(skill_id)]
    }

    pub(crate) const fn mark_wide_arc_attack_used(&mut self, skill_id: u32, now_ms: u32) {
        self.wide_arc_attack_last_used_ms[Self::wide_arc_attack_index(skill_id)] = now_ms;
    }

    pub(crate) const fn lord_fast_attack(&self) -> Option<&LordFastAttackExecutionState> {
        self.lord_fast_attack.as_ref()
    }

    pub(crate) fn begin_lord_fast_attack(&mut self, mut state: LordFastAttackExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.lord_fast_attack = Some(state);
    }

    pub(crate) fn lord_fast_attack_mut(&mut self) -> Option<&mut LordFastAttackExecutionState> {
        self.lord_fast_attack.as_mut()
    }

    pub(crate) const fn fast_attack_last_used_ms(&self, skill_id: u32) -> u32 {
        if skill_id == super::super::skills::monsterfastattack::MONSTER_FAST_ATTACK_SKILL_ID {
            self.monster_fast_attack_last_used_ms
        } else {
            self.lord_fast_attack_last_used_ms
        }
    }

    pub(crate) const fn monster_base_attack(&self) -> Option<&SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_base_attack.as_ref()
    }

    pub(crate) fn monster_base_attack_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_base_attack.as_mut()
    }

    pub(crate) fn begin_monster_base_attack(&mut self, dispatch: PlayerSkillDispatch, now_ms: u32) {
        let now_ms = self.scheduled_skill_begin.filter(|(queued, _)| *queued == dispatch)
            .map_or(now_ms, |(_, started)| started);
        self.monster_base_attack = Some(SkillExecutionKernel::begin(dispatch, now_ms));
    }

    pub(crate) const fn monster_base_attack_last_used_ms(&self) -> u32 {
        self.monster_base_attack_last_used_ms
    }

    pub(crate) const fn mark_monster_base_attack_used(&mut self, now_ms: u32) {
        self.monster_base_attack_last_used_ms = now_ms;
    }

    pub(crate) const fn monster_range_attack(&self) -> Option<&SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_range_attack.as_ref()
    }

    pub(crate) fn monster_range_attack_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_range_attack.as_mut()
    }

    pub(crate) fn begin_monster_range_attack(&mut self, dispatch: PlayerSkillDispatch, now_ms: u32) {
        let now_ms = self.scheduled_skill_begin.filter(|(queued, _)| *queued == dispatch)
            .map_or(now_ms, |(_, started)| started);
        self.monster_range_attack = Some(SkillExecutionKernel::begin(dispatch, now_ms));
    }

    pub(crate) const fn monster_range_attack_last_used_ms(&self) -> u32 { self.monster_range_attack_last_used_ms }

    pub(crate) const fn mark_monster_range_attack_used(&mut self, now_ms: u32) { self.monster_range_attack_last_used_ms = now_ms; }

    pub(crate) const fn mark_fast_attack_used(&mut self, skill_id: u32, now_ms: u32) {
        if skill_id == super::super::skills::monsterfastattack::MONSTER_FAST_ATTACK_SKILL_ID {
            self.monster_fast_attack_last_used_ms = now_ms;
        } else {
            self.lord_fast_attack_last_used_ms = now_ms;
        }
    }

    pub(crate) const fn chaos_sphere(&self) -> Option<&ChaosSphereExecutionState> {
        self.chaos_sphere.as_ref()
    }

    pub(crate) fn begin_chaos_sphere(&mut self, mut state: ChaosSphereExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.chaos_sphere = Some(state);
    }

    pub(crate) fn chaos_sphere_mut(&mut self) -> Option<&mut ChaosSphereExecutionState> {
        self.chaos_sphere.as_mut()
    }

    pub(crate) const fn chaos_sphere_last_used_ms(&self) -> u32 {
        self.chaos_sphere_last_used_ms
    }

    pub(crate) const fn mark_chaos_sphere_used(&mut self, now_ms: u32) {
        self.chaos_sphere_last_used_ms = now_ms;
    }

    pub(crate) const fn lightning(&self) -> Option<LightningExecutionState> {
        self.lightning
    }

    pub(crate) fn begin_lightning(&mut self, mut state: LightningExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.lightning = Some(state);
    }

    pub(crate) fn lightning_mut(&mut self) -> Option<&mut LightningExecutionState> {
        self.lightning.as_mut()
    }

    pub(crate) const fn lightning_last_used_ms(&self) -> u32 {
        self.lightning_last_used_ms
    }

    pub(crate) const fn mark_lightning_used(&mut self, now_ms: u32) {
        self.lightning_last_used_ms = now_ms;
    }

    pub(crate) const fn seal(&self) -> Option<SealExecutionState> {
        self.seal
    }

    pub(crate) fn begin_seal(&mut self, mut state: SealExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.seal = Some(state);
    }

    pub(crate) fn seal_mut(&mut self) -> Option<&mut SealExecutionState> {
        self.seal.as_mut()
    }

    pub(crate) const fn seal_last_used_ms(&self) -> u32 {
        self.seal_last_used_ms
    }

    pub(crate) const fn mark_seal_used(&mut self, now_ms: u32) {
        self.seal_last_used_ms = now_ms;
    }

    pub(crate) const fn yin_yang(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.yin_yang
    }

    pub(crate) fn begin_yin_yang(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.yin_yang = Some(state);
    }

    pub(crate) fn yin_yang_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.yin_yang.as_mut()
    }

    pub(crate) const fn yin_yang_last_used_ms(&self) -> u32 { self.yin_yang_last_used_ms }

    pub(crate) const fn mark_yin_yang_used(&mut self, now_ms: u32) {
        self.yin_yang_last_used_ms = now_ms;
    }
    pub(crate) const fn yin_yang_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.yin_yang_2 }
    pub(crate) fn begin_yin_yang_2(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.yin_yang_2 = Some(state);
    }
    pub(crate) fn yin_yang_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.yin_yang_2.as_mut() }
    pub(crate) const fn yin_yang_2_last_used_ms(&self) -> u32 { self.yin_yang_2_last_used_ms }
    pub(crate) const fn mark_yin_yang_2_used(&mut self, now_ms: u32) { self.yin_yang_2_last_used_ms = now_ms; }
    pub(crate) const fn god_punishment(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_punishment }
    pub(crate) fn begin_god_punishment(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.god_punishment = Some(state);
    }
    pub(crate) fn god_punishment_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_punishment.as_mut() }
    pub(crate) const fn god_punishment_last_used_ms(&self) -> u32 { self.god_punishment_last_used_ms }
    pub(crate) const fn mark_god_punishment_used(&mut self, now: u32) { self.god_punishment_last_used_ms = now; }
    pub(crate) const fn god_thunder(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder }
    pub(crate) fn begin_god_thunder(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.god_thunder = Some(state);
    }
    pub(crate) fn god_thunder_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder.as_mut() }
    pub(crate) const fn god_thunder_last_used_ms(&self) -> u32 { self.god_thunder_last_used_ms }
    pub(crate) const fn mark_god_thunder_used(&mut self, now: u32) { self.god_thunder_last_used_ms = now; }
    pub(crate) const fn god_thunder_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder_2 }
    pub(crate) fn begin_god_thunder_2(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.god_thunder_2 = Some(state);
    }
    pub(crate) fn god_thunder_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder_2.as_mut() }
    pub(crate) const fn god_thunder_2_last_used_ms(&self) -> u32 { self.god_thunder_2_last_used_ms }
    pub(crate) const fn mark_god_thunder_2_used(&mut self, now: u32) { self.god_thunder_2_last_used_ms = now; }
    pub(crate) const fn soul_collect(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_collect }
    pub(crate) fn begin_soul_collect(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.soul_collect = Some(state);
    }
    pub(crate) fn soul_collect_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_collect.as_mut() }
    pub(crate) const fn soul_collect_last_used_ms(&self) -> u32 { self.soul_collect_last_used_ms }
    pub(crate) const fn mark_soul_collect_used(&mut self, now: u32) { self.soul_collect_last_used_ms = now; }
    pub(crate) const fn soul_mirror(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_mirror }
    pub(crate) fn begin_soul_mirror(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.soul_mirror = Some(state);
    }
    pub(crate) fn soul_mirror_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_mirror.as_mut() }
    pub(crate) const fn soul_mirror_last_used_ms(&self) -> u32 { self.soul_mirror_last_used_ms }
    pub(crate) const fn mark_soul_mirror_used(&mut self, now: u32) { self.soul_mirror_last_used_ms = now; }

    pub(crate) const fn callosity(&self) -> Option<CallosityExecutionState> {
        self.callosity
    }

    pub(crate) fn begin_callosity(&mut self, mut state: CallosityExecutionState) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_skill_begin);
        self.callosity = Some(state);
    }

    pub(crate) fn callosity_mut(&mut self) -> Option<&mut CallosityExecutionState> {
        self.callosity.as_mut()
    }

    pub(crate) const fn callosity_last_used_ms(&self, skill_id: u32) -> u32 {
        if skill_id == crate::gameserver::appserver::skills::callosity::CALLOSITY_2_SKILL_ID {
            self.callosity_last_used_ms[1]
        } else {
            self.callosity_last_used_ms[0]
        }
    }

    pub(crate) fn mark_callosity_used(&mut self, skill_id: u32, now_ms: u32) {
        let index = if skill_id
            == crate::gameserver::appserver::skills::callosity::CALLOSITY_2_SKILL_ID
        {
            1
        } else {
            0
        };
        self.callosity_last_used_ms[index] = now_ms;
    }

    pub(crate) const fn hearten(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.hearten
    }
    pub(crate) fn begin_hearten(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.hearten = Some(state);
    }
    pub(crate) fn hearten_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.hearten.as_mut()
    }
    pub(crate) const fn hearten_last_used_ms(&self) -> u32 { self.hearten_last_used_ms }
    pub(crate) const fn mark_hearten_used(&mut self, now_ms: u32) {
        self.hearten_last_used_ms = now_ms;
    }

    pub(crate) const fn promotion(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.promotion
    }

    pub(crate) fn begin_promotion(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.promotion = Some(state);
    }

    pub(crate) fn promotion_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.promotion.as_mut()
    }

    pub(crate) const fn promotion_last_used_ms(&self) -> u32 {
        self.promotion_last_used_ms
    }

    pub(crate) const fn mark_promotion_used(&mut self, now_ms: u32) {
        self.promotion_last_used_ms = now_ms;
    }

    pub(crate) const fn heal_family(
        &self,
        index: usize,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.heal_family[index]
    }

    pub(crate) fn begin_heal_family(
        &mut self,
        index: usize,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.heal_family[index] = Some(state);
    }

    pub(crate) fn heal_family_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.heal_family[index].as_mut()
    }

    pub(crate) const fn heal_family_last_used_ms(&self, index: usize) -> u32 {
        self.heal_family_last_used_ms[index]
    }

    pub(crate) const fn mark_heal_family_used(&mut self, index: usize, now_ms: u32) {
        self.heal_family_last_used_ms[index] = now_ms;
    }

    pub(crate) const fn pets_control(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.pets_control
    }

    pub(crate) fn begin_pets_control(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.pets_control = Some(state);
    }

    pub(crate) fn pets_control_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.pets_control.as_mut()
    }

    pub(crate) const fn pets_control_last_used_ms(&self) -> u32 {
        self.pets_control_last_used_ms
    }

    pub(crate) const fn mark_pets_control_used(&mut self, now_ms: u32) {
        self.pets_control_last_used_ms = now_ms;
    }

    pub(crate) const fn monster_taming(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_taming
    }

    pub(crate) fn begin_monster_taming(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.monster_taming = Some(state);
    }

    pub(crate) fn monster_taming_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_taming.as_mut()
    }

    pub(crate) const fn monster_taming_last_used_ms(&self) -> u32 {
        self.monster_taming_last_used_ms
    }

    pub(crate) const fn mark_monster_taming_used(&mut self, now_ms: u32) {
        self.monster_taming_last_used_ms = now_ms;
    }

    pub(crate) const fn knock_out(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.knock_out
    }

    pub(crate) fn begin_knock_out(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.knock_out = Some(state);
    }

    pub(crate) fn knock_out_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.knock_out.as_mut()
    }

    pub(crate) const fn knock_out_last_used_ms(&self) -> u32 { self.knock_out_last_used_ms }

    pub(crate) const fn mark_knock_out_used(&mut self, now_ms: u32) {
        self.knock_out_last_used_ms = now_ms;
    }

    pub(crate) const fn snow_storm(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.snow_storm
    }

    pub(crate) fn begin_snow_storm(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.snow_storm = Some(state);
    }

    pub(crate) fn snow_storm_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.snow_storm.as_mut()
    }

    pub(crate) const fn snow_storm_last_used_ms(&self) -> u32 { self.snow_storm_last_used_ms }

    pub(crate) const fn mark_snow_storm_used(&mut self, now_ms: u32) {
        self.snow_storm_last_used_ms = now_ms;
    }

    pub(crate) const fn weak(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.weak
    }

    pub(crate) fn begin_weak(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.weak = Some(state);
    }

    pub(crate) fn weak_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.weak.as_mut()
    }

    pub(crate) const fn weak_last_used_ms(&self) -> u32 { self.weak_last_used_ms }

    pub(crate) const fn mark_weak_used(&mut self, now_ms: u32) {
        self.weak_last_used_ms = now_ms;
    }

    pub(crate) const fn god_bless(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_bless }
    pub(crate) fn begin_god_bless(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.god_bless = Some(state);
    }
    pub(crate) fn god_bless_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_bless.as_mut() }
    pub(crate) const fn god_bless_last_used_ms(&self, index: usize) -> u32 { self.god_bless_last_used_ms[index] }
    pub(crate) const fn mark_god_bless_used(&mut self, index: usize, now_ms: u32) { self.god_bless_last_used_ms[index] = now_ms; }
    pub(crate) const fn cure(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.cure }
    pub(crate) fn begin_cure(&mut self, mut state: SkillExecutionKernel<PlayerSkillDispatch>) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.cure = Some(state);
    }
    pub(crate) fn cure_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.cure.as_mut() }
    pub(crate) const fn cure_last_used_ms(&self) -> u32 { self.cure_last_used_ms }
    pub(crate) const fn mark_cure_used(&mut self, now_ms: u32) { self.cure_last_used_ms = now_ms; }

    pub(crate) const fn machine_shield(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.machine_shield
    }

    pub(crate) fn begin_machine_shield(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.machine_shield = Some(state);
    }

    pub(crate) fn machine_shield_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.machine_shield.as_mut()
    }

    pub(crate) const fn machine_shield_last_used_ms(&self) -> u32 {
        self.machine_shield_last_used_ms
    }

    pub(crate) const fn mark_machine_shield_used(&mut self, now_ms: u32) {
        self.machine_shield_last_used_ms = now_ms;
    }

    pub(crate) const fn mana_shield(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.mana_shield
    }

    pub(crate) fn begin_mana_shield(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.mana_shield = Some(state);
    }
    pub(crate) fn mana_shield_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.mana_shield.as_mut()
    }

    pub(crate) const fn mana_shield_last_used_ms(&self) -> u32 {
        self.mana_shield_last_used_ms
    }

    pub(crate) const fn mark_mana_shield_used(&mut self, now_ms: u32) {
        self.mana_shield_last_used_ms = now_ms;
    }

    pub(crate) const fn immediate_state(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.immediate_state
    }
    pub(crate) fn begin_immediate_state(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.immediate_state = Some(state);
    }
    pub(crate) fn immediate_state_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.immediate_state.as_mut()
    }

    pub(crate) fn immediate_state_last_used_ms(&self, skill_id: u32) -> u32 {
        self.immediate_state_last_used_ms
            .get(&skill_id)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn mark_immediate_state_used(&mut self, skill_id: u32, now_ms: u32) {
        self.immediate_state_last_used_ms.insert(skill_id, now_ms);
    }

    pub(crate) const fn non_fun(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.non_fun
    }

    pub(crate) fn begin_non_fun(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.non_fun = Some(state);
    }

    pub(crate) fn non_fun_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.non_fun.as_mut()
    }

    pub(crate) const fn swordship(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.swordship
    }

    pub(crate) fn begin_swordship(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.swordship = Some(state);
    }

    pub(crate) fn swordship_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.swordship.as_mut()
    }

    pub(crate) const fn gibe(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.gibe
    }

    pub(crate) fn begin_gibe(
        &mut self,
        mut state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_skill_begin);
        self.gibe = Some(state);
    }

    pub(crate) fn gibe_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.gibe.as_mut()
    }

    pub(crate) const fn gibe_last_used_ms(&self) -> u32 {
        self.gibe_last_used_ms
    }

    pub(crate) const fn mark_gibe_used(&mut self, now_ms: u32) {
        self.gibe_last_used_ms = now_ms;
    }

    /// Продвигает ровно одну ожидающую команду только после конечного состояния
    /// предыдущей. Новый запрос может заменить ещё не начатый хвост FIFO, но
    /// не уничтожает уже начатый `SkillExecutionKernel`. Запрет нового
    /// расписания у мёртвого владельца не останавливает активную команду.
    pub(crate) fn begin_next_battle_fairy_skill(
        &mut self,
        can_schedule: bool,
    ) -> Option<BattleFairySkillDispatch> {
        if self.current_battle_fairy_skill.is_none() && can_schedule {
            self.current_battle_fairy_skill = self.battle_fairy_skills.pop_front();
            if let Some(dispatch) = self.current_battle_fairy_skill {
                self.selected_battle_fairy_skill_id = dispatch.skill_id();
            }
        }
        self.current_battle_fairy_skill
    }

    /// Исходный игрок сохраняет выбранный навык после его `End`; нулевое
    /// начальное поле Rust кодирует установленную конструктором базовую атаку.
    pub(crate) const fn selected_battle_fairy_skill_id(&self) -> u32 {
        if self.selected_battle_fairy_skill_id == 0 {
            crate::gameserver::appserver::skills::battlefairybasemagic::BATTLE_FAIRY_BASE_MAGIC_SKILL_ID
        } else {
            self.selected_battle_fairy_skill_id
        }
    }

    /// Возвращает часы конкретного выбранного навыка боевого духа. Все ID
    /// диапазона `0x212..0x224` уже имеют отдельное типизированное состояние
    /// исполнения и одну временную отметку.
    pub(crate) const fn selected_battle_fairy_skill_last_used_ms(&self) -> Option<u32> {
        let skill_id = self.selected_battle_fairy_skill_id();
        match skill_id {
            0x212..=0x219 => Some(self.battle_fairy_attribute_last_used_ms(skill_id)),
            crate::gameserver::appserver::skills::tianhuo::TIANHUO_SKILL_ID => {
                Some(self.tianhuo_last_used_ms)
            }
            crate::gameserver::appserver::skills::thunder2::LEIMING2_SKILL_ID => {
                Some(self.leiming2_last_used_ms)
            }
            crate::gameserver::appserver::skills::fatalblow::FATAL_BLOW_SKILL_ID => {
                Some(self.fatal_blow_last_used_ms)
            }
            crate::gameserver::appserver::skills::bloodloss::BLOOD_LOSS_SKILL_ID => {
                Some(self.blood_loss_last_used_ms)
            }
            crate::gameserver::appserver::skills::poisonarrow::POISON_ARROW_SKILL_ID => {
                Some(self.poison_arrow_last_used_ms)
            }
            crate::gameserver::appserver::skills::thunder::THUNDER_SKILL_ID => {
                Some(self.thunder_last_used_ms)
            }
            crate::gameserver::appserver::skills::lifeshield::LIFE_SHIELD_SKILL_ID => {
                Some(self.life_shield_last_used_ms)
            }
            crate::gameserver::appserver::skills::wangsheng::WANGSHENG_SKILL_ID => {
                Some(self.wangsheng_last_used_ms)
            }
            crate::gameserver::appserver::skills::huoxieshu::HUOXIESHU_SKILL_ID => Some(
                self.battle_fairy_transfer_last_used_ms(BattleFairyTransferKind::Health),
            ),
            crate::gameserver::appserver::skills::lingzhishu::LINGZHISHU_SKILL_ID => Some(
                self.battle_fairy_transfer_last_used_ms(BattleFairyTransferKind::Mana),
            ),
            crate::gameserver::appserver::skills::battlefairybasemagic::BATTLE_FAIRY_BASE_MAGIC_SKILL_ID => {
                Some(self.battle_fairy_base_magic_last_used_ms)
            }
            _ => None,
        }
    }

    pub(crate) fn finish_battle_fairy_skill(
        &mut self,
        expected: BattleFairySkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        if self.current_battle_fairy_skill != Some(expected) {
            return false;
        }
        self.current_battle_fairy_skill = None;
        if let Some(mut execution) = self.battle_fairy_base_magic.take() {
            let _ = execution
                .kernel_mut()
                .terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение базовой атаки боевой феи завершено");
        }
        if let Some(mut execution) = self.life_shield.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение щита жизни завершено");
        }
        if let Some(mut execution) = self.battle_fairy_transfer.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение переноса здоровья боевому духу завершено");
        }
        if let Some(mut execution) = self.wangsheng.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение восстановления здоровья игрока завершено");
        }
        if let Some(mut execution) = self.poison_arrow.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ядовитой стрелы завершено");
        }
        if let Some(mut execution) = self.blood_loss.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение потери крови завершено");
        }
        if let Some(mut execution) = self.fatal_blow.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение смертельного удара завершено");
        }
        if let Some(mut execution) = self.thunder.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение грома завершено");
        }
        if let Some(mut execution) = self.leiming2.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение отложенного грома завершено");
        }
        if let Some(mut execution) = self.tianhuo.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение небесного огня завершено");
        }
        if let Some(mut execution) = self.battle_fairy_attribute.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение атрибутного навыка боевого духа завершено");
        }
        true
    }

    /// `true` означает, что выбранная war-soul команда уже прошла concrete
    /// `Begin` и её терминальный путь обязан выполнить унаследованный `End`.
    /// Одна только извлечённая команда не эквивалентна native skill execution.
    pub(crate) const fn battle_fairy_skill_execution_is_materialized(&self) -> bool {
        self.battle_fairy_base_magic.is_some()
            || self.life_shield.is_some()
            || self.battle_fairy_transfer.is_some()
            || self.wangsheng.is_some()
            || self.poison_arrow.is_some()
            || self.blood_loss.is_some()
            || self.fatal_blow.is_some()
            || self.thunder.is_some()
            || self.leiming2.is_some()
            || self.tianhuo.is_some()
            || self.battle_fairy_attribute.is_some()
    }

    /// Общая запись `CSkill::End(true)` после оружейного эффекта.
    /// Диапазон `0x212..=0x224` полностью материализован
    /// типизированными владельцами, поэтому неизвестный ID остаётся без часов.
    pub(crate) fn mark_battle_fairy_skill_used(
        &mut self,
        skill_id: u32,
        now_ms: u32,
    ) -> bool {
        match skill_id {
            0x212..=0x219 => self.mark_battle_fairy_attribute_used(skill_id, now_ms),
            crate::gameserver::appserver::skills::tianhuo::TIANHUO_SKILL_ID => {
                self.mark_tianhuo_used(now_ms)
            }
            crate::gameserver::appserver::skills::thunder2::LEIMING2_SKILL_ID => {
                self.mark_leiming2_used(now_ms)
            }
            crate::gameserver::appserver::skills::fatalblow::FATAL_BLOW_SKILL_ID => {
                self.mark_fatal_blow_used(now_ms)
            }
            crate::gameserver::appserver::skills::bloodloss::BLOOD_LOSS_SKILL_ID => {
                self.mark_blood_loss_used(now_ms)
            }
            crate::gameserver::appserver::skills::poisonarrow::POISON_ARROW_SKILL_ID => {
                self.mark_poison_arrow_used(now_ms)
            }
            crate::gameserver::appserver::skills::thunder::THUNDER_SKILL_ID => {
                self.mark_thunder_used(now_ms)
            }
            crate::gameserver::appserver::skills::lifeshield::LIFE_SHIELD_SKILL_ID => {
                self.mark_life_shield_used(now_ms)
            }
            crate::gameserver::appserver::skills::wangsheng::WANGSHENG_SKILL_ID => {
                self.mark_wangsheng_used(now_ms)
            }
            crate::gameserver::appserver::skills::huoxieshu::HUOXIESHU_SKILL_ID => self
                .mark_battle_fairy_transfer_used(BattleFairyTransferKind::Health, now_ms),
            crate::gameserver::appserver::skills::lingzhishu::LINGZHISHU_SKILL_ID => self
                .mark_battle_fairy_transfer_used(BattleFairyTransferKind::Mana, now_ms),
            crate::gameserver::appserver::skills::battlefairybasemagic::BATTLE_FAIRY_BASE_MAGIC_SKILL_ID => {
                self.mark_battle_fairy_base_magic_used(now_ms)
            }
            _ => return false,
        }
        true
    }

    /// Точный последний side effect `OnChangeSkillWithWarSoul` и
    /// `OnLoseTargetWarSoul`: ID `0x224` назначается только после полного
    /// concrete `End(1)`, включая оружейный эффект и cooldown.
    pub(crate) const fn restore_battle_fairy_base_attack_after_end(&mut self) {
        self.selected_battle_fairy_skill_id = 0;
    }

    pub(crate) const fn battle_fairy_skill_is_active(&self) -> bool {
        self.current_battle_fairy_skill.is_some()
    }

    /// Общий унаследованный `End(false)` для отмены активного war-soul skill:
    /// его вызывают spatial удаление духа и повторный запрос уже подготовленного
    /// навыка. Выбранный skill ID и ещё не начатый FIFO-хвост сохраняются.
    pub(crate) fn cancel_active_battle_fairy_skill(
        &mut self,
    ) -> Option<BattleFairySkillDispatch> {
        let dispatch = self.current_battle_fairy_skill?;
        self.finish_battle_fairy_skill(dispatch, SkillTermination::Cancelled)
            .then_some(dispatch)
    }

    pub(crate) const fn battle_fairy_base_magic(
        &self,
    ) -> Option<BattleFairyBaseMagicExecutionState> {
        self.battle_fairy_base_magic
    }

    pub(crate) const fn battle_fairy_attribute(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_attribute
    }

    pub(crate) fn begin_battle_fairy_attribute(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.battle_fairy_attribute = Some(state);
    }

    pub(crate) fn battle_fairy_attribute_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_attribute.as_mut()
    }

    const fn battle_fairy_attribute_index(skill_id: u32) -> usize {
        match skill_id {
            0x212..=0x219 => (skill_id - 0x212) as usize,
            _ => unreachable!(),
        }
    }

    pub(crate) const fn battle_fairy_attribute_last_used_ms(&self, skill_id: u32) -> u32 {
        self.battle_fairy_attribute_last_used_ms[Self::battle_fairy_attribute_index(skill_id)]
    }

    pub(crate) fn mark_battle_fairy_attribute_used(&mut self, skill_id: u32, now_ms: u32) {
        let index = Self::battle_fairy_attribute_index(skill_id);
        self.battle_fairy_attribute_last_used_ms[index] = now_ms;
    }

    pub(crate) fn begin_battle_fairy_base_magic(
        &mut self,
        mut state: BattleFairyBaseMagicExecutionState,
    ) {
        state.kernel_mut().inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.battle_fairy_base_magic = Some(state);
    }

    pub(crate) fn battle_fairy_base_magic_mut(
        &mut self,
    ) -> Option<&mut BattleFairyBaseMagicExecutionState> {
        self.battle_fairy_base_magic.as_mut()
    }

    pub(crate) const fn battle_fairy_base_magic_last_used_ms(&self) -> u32 {
        self.battle_fairy_base_magic_last_used_ms
    }

    pub(crate) const fn mark_battle_fairy_base_magic_used(&mut self, now_ms: u32) {
        self.battle_fairy_base_magic_last_used_ms = now_ms;
    }

    pub(crate) const fn life_shield(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.life_shield
    }

    pub(crate) fn begin_life_shield(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.life_shield = Some(state);
    }

    pub(crate) fn life_shield_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.life_shield.as_mut()
    }

    pub(crate) const fn life_shield_last_used_ms(&self) -> u32 {
        self.life_shield_last_used_ms
    }

    pub(crate) const fn mark_life_shield_used(&mut self, now_ms: u32) {
        self.life_shield_last_used_ms = now_ms;
    }

    pub(crate) const fn battle_fairy_transfer(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_transfer
    }

    pub(crate) fn begin_battle_fairy_transfer(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.battle_fairy_transfer = Some(state);
    }

    pub(crate) fn battle_fairy_transfer_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_transfer.as_mut()
    }

    const fn battle_fairy_transfer_index(kind: BattleFairyTransferKind) -> usize {
        match kind {
            BattleFairyTransferKind::Health => 0,
            BattleFairyTransferKind::Mana => 1,
        }
    }

    pub(crate) const fn battle_fairy_transfer_last_used_ms(
        &self,
        kind: BattleFairyTransferKind,
    ) -> u32 {
        self.battle_fairy_transfer_last_used_ms[Self::battle_fairy_transfer_index(kind)]
    }

    pub(crate) fn mark_battle_fairy_transfer_used(
        &mut self,
        kind: BattleFairyTransferKind,
        now_ms: u32,
    ) {
        self.battle_fairy_transfer_last_used_ms[Self::battle_fairy_transfer_index(kind)] = now_ms;
    }

    pub(crate) const fn wangsheng(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.wangsheng
    }

    pub(crate) fn begin_wangsheng(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.wangsheng = Some(state);
    }

    pub(crate) fn wangsheng_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.wangsheng.as_mut()
    }

    pub(crate) const fn wangsheng_last_used_ms(&self) -> u32 {
        self.wangsheng_last_used_ms
    }

    pub(crate) const fn mark_wangsheng_used(&mut self, now_ms: u32) {
        self.wangsheng_last_used_ms = now_ms;
    }

    pub(crate) const fn poison_arrow(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.poison_arrow
    }

    pub(crate) fn begin_poison_arrow(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.poison_arrow = Some(state);
    }

    pub(crate) fn poison_arrow_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.poison_arrow.as_mut()
    }

    pub(crate) const fn poison_arrow_last_used_ms(&self) -> u32 {
        self.poison_arrow_last_used_ms
    }

    pub(crate) const fn mark_poison_arrow_used(&mut self, now_ms: u32) {
        self.poison_arrow_last_used_ms = now_ms;
    }

    pub(crate) const fn blood_loss(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.blood_loss
    }

    pub(crate) fn begin_blood_loss(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.blood_loss = Some(state);
    }

    pub(crate) fn blood_loss_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.blood_loss.as_mut()
    }

    pub(crate) const fn blood_loss_last_used_ms(&self) -> u32 {
        self.blood_loss_last_used_ms
    }

    pub(crate) const fn mark_blood_loss_used(&mut self, now_ms: u32) {
        self.blood_loss_last_used_ms = now_ms;
    }

    pub(crate) const fn fatal_blow(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.fatal_blow
    }

    pub(crate) fn begin_fatal_blow(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.fatal_blow = Some(state);
    }

    pub(crate) fn fatal_blow_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.fatal_blow.as_mut()
    }

    pub(crate) const fn fatal_blow_last_used_ms(&self) -> u32 {
        self.fatal_blow_last_used_ms
    }

    pub(crate) const fn mark_fatal_blow_used(&mut self, now_ms: u32) {
        self.fatal_blow_last_used_ms = now_ms;
    }

    pub(crate) const fn thunder(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.thunder
    }

    pub(crate) fn begin_thunder(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.thunder = Some(state);
    }

    pub(crate) fn thunder_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.thunder.as_mut()
    }

    pub(crate) const fn thunder_last_used_ms(&self) -> u32 {
        self.thunder_last_used_ms
    }

    pub(crate) const fn mark_thunder_used(&mut self, now_ms: u32) {
        self.thunder_last_used_ms = now_ms;
    }

    pub(crate) const fn leiming2(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.leiming2
    }

    pub(crate) fn begin_leiming2(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.leiming2 = Some(state);
    }

    pub(crate) fn leiming2_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.leiming2.as_mut()
    }

    pub(crate) const fn leiming2_last_used_ms(&self) -> u32 {
        self.leiming2_last_used_ms
    }

    pub(crate) const fn mark_leiming2_used(&mut self, now_ms: u32) {
        self.leiming2_last_used_ms = now_ms;
    }

    pub(crate) const fn tianhuo(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.tianhuo
    }

    pub(crate) fn begin_tianhuo(
        &mut self,
        mut state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        state.inherit_scheduled_begin(self.scheduled_fairy_skill_begin);
        self.tianhuo = Some(state);
    }

    pub(crate) fn tianhuo_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.tianhuo.as_mut()
    }

    pub(crate) const fn tianhuo_last_used_ms(&self) -> u32 {
        self.tianhuo_last_used_ms
    }

    pub(crate) const fn mark_tianhuo_used(&mut self, now_ms: u32) {
        self.tianhuo_last_used_ms = now_ms;
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn increment_player_progress(
        &mut self,
        player: &mut CPlayer,
        interval_ms: u32,
        auto_exp_1: f32,
        auto_exp_2: f32,
        exp_to_vigour_x: u32,
        exp_to_vigour_y: u32,
        maximum_vigour_once: u32,
        get_tick_ms: &mut dyn FnMut() -> u32,
    ) -> Option<PlayerAutoProgress> {
        if player.is_dead() || player.faction_id() == 0 {
            return None;
        }
        let sampled_at_ms = get_tick_ms();
        if self.auto_inc_last_time_ms >= sampled_at_ms.wrapping_sub(interval_ms) {
            return None;
        }
        self.auto_inc_last_time_ms = get_tick_ms();

        let level = f64::from(player.level());
        let experience_gain = (((f64::from(player.faction_level()) * 0.05 + 1.0)
            * level.powi(3)
            * f64::from(auto_exp_2)
            + f64::from(auto_exp_1))
            * f64::from(0.000_115_740_74_f32))
        .trunc() as u32;
        if experience_gain == 0 {
            return None;
        }
        let vigour_raw = f64::from(exp_to_vigour_x)
            * f64::from(experience_gain.wrapping_add(600)).log10()
            - f64::from(exp_to_vigour_y);
        let vigour_gain = (vigour_raw.trunc() as i32 as u32).min(maximum_vigour_once);
        let previous_experience = player.experience();
        let previous_vigour = player.vigour();
        player.set_experience(previous_experience.wrapping_add(experience_gain));
        player.set_vigour(previous_vigour.wrapping_add(vigour_gain));
        Some(PlayerAutoProgress {
            player_id: player.player_id(),
            sampled_at_ms,
            experience_gain,
            vigour_gain,
            previous_experience,
            current_experience: player.experience(),
            previous_vigour,
            current_vigour: player.vigour(),
        })
    }

    /// Exact energy tail `CPlayerAI::Run`: первый живой tick только заводит
    /// clock; full energy не двигает его дальше. Due comparison намеренно не
    /// wrap-safe (`last < now - interval`) — это наблюдаемая native-семантика.
    pub(crate) fn regenerate_player_energy(
        &mut self,
        player: &mut CPlayer,
        interval_ms: u32,
        get_tick_ms: &mut dyn FnMut() -> u32,
    ) -> Option<PlayerEnergyRegeneration> {
        if player.is_dead() {
            return None;
        }
        if self.auto_inc_energy_last_time_ms == 0 {
            self.auto_inc_energy_last_time_ms = get_tick_ms();
        }
        let previous_energy = player.energy();
        if previous_energy == player.maximum_energy() {
            return None;
        }
        let sampled_at_ms = get_tick_ms();
        if self.auto_inc_energy_last_time_ms >= sampled_at_ms.wrapping_sub(interval_ms) {
            return None;
        }
        self.auto_inc_energy_last_time_ms = get_tick_ms();

        let faction_bonus = if player.faction_id() == 0 {
            0.0
        } else {
            (f64::from(player.level()) * f64::from(0.01_f32))
                .min(1.0)
                .mul_add(f64::from(player.faction_level()) * 0.5, 0.0)
                .max(1.0)
        };
        // MSVC меняет x87 rounding mode на truncation перед `__ftol2`.
        let increment =
            ((f64::from(player.level()) * f64::from(0.1_f32) - 1.0) * 5.0 + faction_bonus + 10.0)
                .trunc() as u32;
        if increment == 0 {
            return None;
        }
        player.set_energy(previous_energy.wrapping_add(increment));
        let current_energy = player.energy();
        (current_energy != previous_energy).then_some(PlayerEnergyRegeneration {
            player_id: player.player_id(),
            sampled_at_ms,
            increment,
            previous_energy,
            current_energy,
        })
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp

// ============================================================================
// FUNCTION: CPlayerAI::Tracing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:414
// RVA: 0x00108DC0
// ADDRESS: 00508dc0
// PROTOTYPE: int __thiscall Tracing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnChangeSkill
// STATUS: PARTIALLY_IMPLEMENTED
// MATERIALIZED: ended-ветвь исполняется отдельным событием ChangeSkill,
// восстанавливает default после End и не продвигает следующую команду в том
// же такте. Вызов 0x0047B150 возвращает константу 1; prepared-ветвь ещё не
// достигнута. RAW сохранён для этой оставшейся зависимости.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:608
// RVA: 0x00108E40
// ADDRESS: 00508e40
// PROTOTYPE: int __thiscall OnChangeSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnMoving
// STATUS: IMPLEMENTED
// MATERIALIZED: первый проход `ASA_MOVE` вызывает владельца точки перехода до
// ожидания задержки; `CGame` сохраняет возможную смену региона и уведомления.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:658
// RVA: 0x00108E90
// ADDRESS: 00508e90
// PROTOTYPE: int __thiscall OnMoving(void)

// ============================================================================
// FUNCTION: CPlayerAI::OnStanding
// STATUS: IMPLEMENTED
// MATERIALIZED: первый проход `ASA_STAND` вызывает владельца точки перехода,
// затем общий FIFO сохраняет исходную задержку до следующего расписания.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:685
// RVA: 0x00108ED0
// ADDRESS: 00508ed0
// PROTOTYPE: int __thiscall OnStanding(void)


// ============================================================================
// FUNCTION: CPlayerAI::OnLoseTarget
// STATUS: PARTIALLY_IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: достигнутый вызов из `CPet::ReleaseReciprocalTarget` завершает
// начатый concrete skill через `End(1)` и только тогда отправляет отказный
// `0xBFE01`; ещё не начатую совпавшую object-команду удаляет без ответа.
// Default attack восстанавливается в обоих случаях, независимая очередь боевой
// феи не затрагивается. Остались иные недостигнутые вызывающие стороны.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:450
// RVA: 0x00109130
// ADDRESS: 00509130
// PROTOTYPE: int __thiscall OnLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnLoseTargetWarSoul
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: scheduler-rejection завершает текущий execution и после `End`
// возвращает выбранный war-soul skill к базовой атаке. Общий `CBaseAI` target
// cleanup для ещё не достигнутых вызывающих сторон сохранён ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:484
// RVA: 0x001091B0
// ADDRESS: 005091b0
// PROTOTYPE: int __thiscall OnLoseTargetWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnFightingWithWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:517
// RVA: 0x00109230
// ADDRESS: 00509230
// PROTOTYPE: int __thiscall OnFightingWithWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnFighting
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:563
// RVA: 0x001092B0
// ADDRESS: 005092b0
// PROTOTYPE: int __thiscall OnFighting(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnChangeSkillWithWarSoul
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// MATERIALIZED: scheduler завершает concrete owner, выполняет общий `End(1)`
// и только затем выбирает базовую атаку `0x224`, не затрагивая ожидающий FIFO.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:629
// RVA: 0x00109310
// ADDRESS: 00509310
// PROTOTYPE: int __thiscall OnChangeSkillWithWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CPlayerAI::Run
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:27
// RVA: 0x001093E0
// ADDRESS: 005093e0
// PROTOTYPE: AI_EXEC_STATE __thiscall Run(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnScheduleAboutWarSoul
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: ожидающая war-soul команда не извлекается у мёртвого владельца;
// уже активное выполнение остаётся отдельной ProcessActiveAction-ветвью.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:143
// RVA: 0x00109730
// ADDRESS: 00509730
// PROTOTYPE: void __thiscall OnScheduleAboutWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnSchedule
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: FIFO назначения и навыков, `ASA_MOVE`, запрет начала у мёртвого
// владельца, удаление ожидающей команды при `RideState` и общий отказ с
// `OnLoseTarget → End(1)` при активном skill. Ветвь разрешения целей ниже ещё
// не достигнута.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:256
// RVA: 0x001098D0
// ADDRESS: 005098d0
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::CPlayerAI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CBaseAI`, очередь назначений и часы автоматического прироста;
// оставшиеся очереди целей и `_last_count_time` сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:19
// RVA: 0x00109B70
// ADDRESS: 00509b70
// PROTOTYPE: undefined __thiscall CPlayerAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: типизированная FIFO-очередь объектных команд, замена ожидающего
// хвоста и отдельное удержание уже начатого навыка; прочие проверки сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:711
// RVA: 0x00109FF0
// ADDRESS: 00509ff0
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: типизированная FIFO-очередь координатных команд, замена ожидающего
// хвоста и отдельное удержание уже начатого навыка; прочие проверки сохранены ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:841
// RVA: 0x0010A230
// ADDRESS: 0050a230
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
