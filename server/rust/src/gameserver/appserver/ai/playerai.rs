//! Достигнутая часть очередей и исполнения `CPlayerAI` GameServer.
//! OnSchedule (0x0050993E..0x0050999D) извлекает обычный запрос до Begin;
//! End/OnLoseTarget не подменяют текущую команду головой pending FIFO.
//! Повторное завершение отсутствующего исполнения не удаляет будущий запрос
//! с тем же dispatch. Выбор, исполнение и очередь остаются разными состояниями.
//! При свободных очередях действий новый запрос заменяет текущую цель, даже
//! если прежняя команда ещё сохранена. Riding-отказ вызывает OnLoseTarget
//! до pop и не устанавливает цель отклоняемого запроса (0x0050998E..0x0050999D).
//! HasTarget (0x004C7DD0) не допускает Begin при нулевой координате или
//! неположительном type/id. Такой запрос уже извлечён, но следующий запрос
//! может заменить цель при свободной очереди действий, в том числе WarSoul.
//! Встречный OnLoseTarget питомца (0x004E96DC..0x004E970E) сравнивает
//! установленные OnSchedule type/id цели, не ожидающий запрос и не GUID.
//! После проверки цели общий OnLoseTarget очищает команду до End;
//! pending FIFO не затрагивается, специального cleanup для питомца нет.
//! Attack (0x00509FF0/0x0050A230) заменяет ожидающую команду независимо от
//! текущего исполнения. Для WarSoul point-ветвь 0x0050A334..0x0050A3BF
//! сравнивает голову, удаляет старые запросы и добавляет новый без Reject;
//! обычная очередь отдельно отправляет Reject для каждой замены (0x0050A43C).
//! Общая Rust-операция сохраняет два отдельных FIFO и не отменяет текущий ID.
//! У подключённых WarSoul 0x212..0x224 нет записи prepared (см. battlefairyskill),
//! поэтому проверка повторного prepared Attack не даёт им дополнительный End.
//! Исполнение и reuse принадлежат зарегистрированному CMoveShape::skill,
//! а не отдельным картам AI. Один экземпляр хранит типизированное исполнение
//! игрока, боевого духа либо монстра; очереди и выбранные команды независимы.
//! Единственная SkillLifecycle существует и без concrete-данных, в Inactive;
//! при установке исполнения она перемещается в его kernel, а при удалении
//! данных возвращается обратно. Общий Begin пишет source/target, время и
//! ended до OnBeginSkill; установка kernel не заменяет эту базу поздним
//! отсчётом. Отдельных timestamp-маркеров расписания в AI нет.
//! Достигнутый хвост End сверяет полный dispatch, сбрасывает базу перед
//! удалением собственных concrete-данных и сохраняет reuse; фон не снимает
//! текущую или ожидающую команду. Полный registered End, включая отказ
//! Begin без concrete-данных и визуальные ресурсы, ещё требует подключения.
//! Наличие payload не заменяет native IsEnded: отказ после общего Begin
//! оставляет уже изменённую базу независимо от установки исполнения.
//! Реестр выбирает первый экземпляр по native-категории, а не глобальную
//! запись по ID. Удаление/повторная регистрация не наследует прежний cooldown.
//! Attack обеих перегрузок (0x00509FF0/0x0050A230) до изменения FIFO
//! требует существующий GetSkill: отсутствие не создаёт отказ и не снимает
//! старые запросы. OnSchedule заново разрешает текущий owner; при null
//! выбирает virtual default игрока с прежней целью (0x00509A34..0x00509A60).
//! WarSoul не выбирает default при null GetSkill (0x00509804): сохраняет
//! только что извлечённую цель и ID до следующего допущенного расписания.
//! Оно очищает цель до проверки пустоты pending FIFO (0x00509780..0x00509789),
//! поэтому отсутствующий owner не превращается в повторяющийся Begin.
//! Отсчёт CState::Begin фиксируется в базе зарегистрированного навыка до
//! OnBeginSkill. Обычный и WarSoul Begin меняют каждый свой экземпляр;
//! два исполнения одного такта не используют общий временный контекст.
//! Отказ не откатывает записанную базу, а активный AI не повторяет Begin.
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
//! Четырёхаргументный `MoveTo` использует общий `CBaseAI::Slip`: каждый
//! одноклеточный проход пробует `_slip_order` и figure-specific move-check
//! клетки до единственного `0xBF605`. Задержка сохраняет скорость игрока,
//! направление между исходной и конечной клетками и нулевой stop-frame.
//! Слот +0x48 таблицы CPlayerAI указывает на пустой RET
//! (0x00485540), поэтому idle игрока не ставит базовый Stand на 1000 мс.
//! CBaseAI::Run (0x004C7D10) вызывает OnSchedule, затем background, passive и
//! active; только AES_HUNG_UP запрещает следующую основную фазу. WarSoul
//! обрабатывается после них независимо от результата passive. В Rust Begin
//! выполняется до фона, а WarSoul вызывается отдельным хвостом даже
//! при Defense/Stiffen. Прерывание concrete навыка завершается до этого
//! хвоста. Attack снимается лишь после подтверждённого завершения concrete
//! навыка; неоконченный End(4) сохраняет событие. До Move очистка доходит без
//! прерывания навыка. Расписание проверяет пустоту обеих основных очередей
//! до background/passive/active; новое расписание после их очистки ждёт
//! следующего Run. WarSoul ставит собственный Attack после Begin. Stiffen передаёт
//! ненулевой End отдельно от отказного End(0); удерживаемая HeartLessArrow
//! выпускается без снятия execution и Attack до последующего AI.
//! Обработанный Defense разрешает active в том же Run; Stiffen запрещает
//! active лишь до deadline. Его прерывание при уже истёкшем сроке сохраняется
//! отдельно от ожидания, не останавливая собственный auto-inc
//! хвост `CPlayerAI::Run`. Достигнутые reciprocal/death `OnLoseTarget` всегда
//! возвращают player-а к вычисленному default attack; death-tail делает это и
//! без активного skill, а разорванный concrete owner больше не оставляет
//! current-skill и запрет движения.
//! `WhenBeenKilled` использует ту же passive FIFO `CBaseAI`: достигнутый
//! Died сохраняет только первый Move, вызывает OnLoseTarget и ждёт его
//! завершения перед OnDied. Координатор публикует настоящий CPlayerAI на
//! время обоих callback; отдельной очереди смерти или копии снимка удара нет.
//! Отказный `0xBFE01` при `OnLoseTarget` следует только за `End(1)` реально
//! прерванного навыка; одна ожидающая object-команда удаляется без ответа.
//! Завершённый либо prepared экземпляр не получает End при потере цели
//! (OnLoseTarget, 0x00509130); выбранный ID сам по себе не означает исполнение.
//! `OnSchedule` (0x005098D0) извлекает команду до допуска и Begin: текущая
//! команда хранится в Option, ожидающая m_qTarget — в VecDeque. Attack
//! (0x00509FF0/0x0050A230) заменяет только ожидающую команду, не execution.
//! Общий хвост завершения проверяет именно выбранную команду, поэтому End
//! concrete owner-а не позволяет повторно снять следующую совпавшую команду.
//! Освобождение execution также ограничено этой командой: CSkill::End
//! (0x004D84C0) очищает свой экземпляр, не соседние навыки. Удаление по ID
//! с проверкой dispatch сохраняет остальные kernel, включая варианты лечения.
//! Фоновый обход завершает базу и concrete-исполнение, не активную команду; автонавыки
//! AddObject получают тот же kernel при Begin. Очередь повторно разрешает ID
//! и помечает уже завершённый экземпляр, не применяя дубликат второй раз.
//! Автоматические состояния выполняются тем же concrete AI, что активные:
//! у них нет второго упрощённого применения или отдельного хвоста End.
//! OnFighting (0x005092B0) сохраняет Attack до отдельного такта после End;
//! затем ChangeSkill (0x00508E40) возвращает вычисленный после End default.
//! Допуск OnSchedule не повторяется внутри Attack, а завершивший AI не
//! получает лишний Reject от общего координатора. Используется существующая
//! FIFO CBaseAI, включая её очистку Defense/Stiffen и отдельный такт смены.
//! Базовые атака, стрельба, магия и достигнутые длительные снаряды возвращают
//! Begun до первого AI: Attack исполняет его в том же Run, без искусственного
//! дополнительного такта. У SkeletonArchery Begin также отделён от AI,
//! хотя сам полёт не передаётся в фон.
//! Владельцы WarSoul также возвращают Begun отдельно от первого AI; общий
//! диспетчер не повторяет допуск при продолжении в том же Run. Очередь
//! WarSoul Attack ставится после Begin и переживает собственный End навыка:
//! следующий Run снимает событие без нового расписания и без ChangeSkill
//! (OnFightingWithWarSoul, 0x00509230). Выбранный ID сохраняется после End.
//! Prepared-ветвь вызывает общий WhenAddBackStageSkill (слот +0x88,
//! 0x004C94B0): ID добавляется в ту же FIFO, без отдельного фона WarSoul.
//! Фоновый End удаляет только concrete-данные, сохраняя зарегистрированный
//! экземпляр, команды и выбранный ID.
//! OnChangeSkillWithWarSoul (0x00509310, исходный playerai.cpp:629) выполняет
//! End(1) выбранного навыка и выбирает 0x224, но источник этого события в
//! данной паре EXE/PDB не найден. Из 82 прямых вызовов AddAIEvent
//! (0x004C8F90) только 0x00509877 передаёт WarSoul=1, с действием Attack.
//! Все 13 постановок ChangeSkill используют WarSoul=0. Прямые вставки
//! через 0x004C8E30 вне AddAIEvent адресуют основные active/passive очереди;
//! абсолютных ссылок на обе функции в образе нет. Поэтому синтетический
//! ChangeSkill после End не добавляется. Текущие 19 concrete WarSoul AI
//! не выставляют prepared; их отдельные Summon также завершаются End(1),
//! поэтому длительность созданной области не продлевает исполнение навыка.
//! ID подготовленного навыка добавляется в общую фоновую очередь до ChangeSkill,
//! без End, переноса kernel и повторного Begin. Достигнутые длительные prepared-навыки
//! устанавливают общий флаг в своих подтверждённых точках выпуска. Сам по себе
//! локальный fired не означает prepared: SkeletonArchery сохраняет активный полёт.
//! Begin подключённых player/WarSoul адаптеров отделён от первого AI;
//! дополнительный tick между этими фазами не добавляется.
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
//! сохраняет выбранный ID, а потеря цели возвращает его к базовой атаке
//! `0x224`. Обычная очередь действий игрока и
//! очередь боевой феи исполняются независимо: движение игрока не
//! приостанавливает стадии феи.

use std::collections::VecDeque;

use super::baseai::{AiShapeAction, CBaseAI, PassiveDeathAction, PassiveStiffenAction};
use crate::gameserver::appserver::player::{
    BattleFairySkillDispatch, CPlayer, PlayerSkillDispatch,
};
use crate::gameserver::appserver::skills::kernel::SkillExecutionKernel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAiDestination {
    pub(crate) direction: i32,
    pub(crate) is_run: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillQueueOutcome {
    PendingUnchanged,
    Queued { replaced: usize },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CPlayerAI {
    base_ai: CBaseAI,
    destinations: VecDeque<PlayerAiDestination>,
    player_skills: VecDeque<PlayerSkillDispatch>,
    current_player_skill: Option<PlayerSkillDispatch>,
    selected_battle_fairy_skill_id: u32,
    current_battle_fairy_skill: Option<BattleFairySkillDispatch>,
    battle_fairy_skills: VecDeque<BattleFairySkillDispatch>,
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
    pub(crate) const fn base_ai(&self) -> &CBaseAI {
        &self.base_ai
    }

    pub(crate) const fn base_ai_mut(&mut self) -> &mut CBaseAI {
        &mut self.base_ai
    }

    pub(crate) fn when_been_hurted(&mut self, now_ms: u32) {
        self.base_ai.when_been_hurted(now_ms);
    }

    pub(crate) fn when_been_stiffened(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.when_been_stiffened(delay_ms, now_ms);
    }

    pub(crate) fn when_been_killed(&mut self, now_ms: u32) {
        self.base_ai.when_been_killed(now_ms);
    }

    pub(crate) fn process_reached_defense_actions(&mut self) -> usize {
        self.base_ai.process_reached_defense_actions(|_| {})
    }

    pub(crate) fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction {
        self.base_ai.begin_reached_stiffen_action()
    }

    pub(crate) fn finish_reached_stiffen_action(
        &mut self,
        begun: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction {
        self.base_ai.finish_reached_stiffen_action(begun, now)
    }

    pub(crate) fn begin_reached_death_action(&mut self) -> bool {
        self.base_ai.begin_reached_death_action()
    }

    pub(crate) fn reached_death_action_state(&self) -> PassiveDeathAction {
        self.base_ai.reached_death_action_state()
    }

    pub(crate) fn finish_reached_death_action(&mut self, now_ms: u32) {
        self.base_ai.finish_reached_death_action(now_ms);
    }

    pub(crate) fn stiffen_attack_needs_end(&self) -> bool {
        self.base_ai.stiffen_attack_needs_end()
    }

    pub(crate) fn stiffen_attack_pending(&self) -> bool {
        self.base_ai.stiffen_attack_pending()
    }

    pub(crate) fn current_active_action(&self) -> Option<AiShapeAction> {
        self.base_ai.current_active_action()
    }

    pub(crate) fn advance_handled_active_action(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_handled_active_action(now).is_some()
    }

    pub(crate) fn advance_handled_passive_action(&mut self, now: impl FnOnce() -> u32) -> Option<bool> {
        self.base_ai.advance_handled_passive_action(now)
    }

    pub(crate) fn advance_handled_war_soul_action(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_handled_war_soul_action(now).is_some()
    }

    pub(crate) fn finish_stiffen_attack(&mut self, release_target: bool) {
        self.base_ai.finish_stiffen_attack(release_target);
    }

    pub(crate) fn discard_active_prefix(&mut self) {
        self.base_ai.discard_active_prefix();
    }

    pub(crate) fn queue_client_destination(&mut self, direction: i32, is_run: bool) {
        while 3 < self.destinations.len() {
            self.destinations.pop_front();
        }
        self.destinations
            .push_back(PlayerAiDestination { direction, is_run });
    }

    /// Выполняет достигнутую `ASA_MOVE`-границу после текущего `OnSchedule`.
    /// Даже снятое в этом вызове событие удерживает расписание до следующего
    /// такта, как `CBaseAI::ProcessActiveAction`.
    pub(crate) fn advance_active_move(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_active_move(now)
    }

    pub(crate) fn active_move_unhandled(&self) -> bool {
        self.base_ai.active_move_unhandled()
    }

    pub(crate) fn advance_active_stand(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_active_stand(now)
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

    pub(crate) fn primary_queues_idle(&self) -> bool {
        self.base_ai.primary_queues_idle()
    }

    pub(crate) fn begin_player_fighting(&mut self, now_ms: u32) {
        if !self.base_ai.active_actions().iter().any(|event| event.action == AiShapeAction::Attack) {
            self.base_ai.add_ai_event(AiShapeAction::Attack, 0, 0, now_ms);
        }
    }

    /// OnFighting проверяет IsEnded и IsPrepared до AI. Перенос в фон
    /// выполняется до ChangeSkill, но после уже прошедшего фонового обхода.
    pub(crate) fn finish_player_attack(
        &mut self,
        selected_skill_id: Option<u32>,
        execution: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
        mut now: impl FnMut() -> u32,
    ) -> bool {
        if !self.base_ai.active_attack_pending() {
            return false;
        }
        if let Some(skill_id) = selected_skill_id
            && let Some(execution) = execution
        {
            if !execution.is_prepared() {
                return false;
            }
            self.base_ai.add_started_back_stage_skill(skill_id);
        }
        self.current_player_skill = None;
        if selected_skill_id.is_some() {
            self.base_ai.add_ai_event(AiShapeAction::ChangeSkill, 0, 0, now());
        }
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

    pub(crate) fn next_destination(&self) -> Option<PlayerAiDestination> {
        self.destinations.front().copied()
    }

    /// `CPlayerAI::OnSchedule` удаляет назначение после попытки `MoveTo`,
    /// независимо от результата region cast и самого движения; поэтому
    /// изъятие принадлежит самому FIFO-owner-у, но его порядок относительно
    /// `OnMove`/`OnCannotMove` сохраняет caller.
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
        Self::replace_pending_skill(&mut self.player_skills, dispatch, |pending, requested| {
            pending.same_pending_request(*requested)
        }).unwrap_or(0)
    }

    pub(crate) fn queue_battle_fairy_skill(
        &mut self,
        dispatch: BattleFairySkillDispatch,
    ) -> BattleFairySkillQueueOutcome {
        match Self::replace_pending_skill(&mut self.battle_fairy_skills, dispatch, |pending, requested| {
            pending.same_pending_request(*requested)
        }) {
            None => BattleFairySkillQueueOutcome::PendingUnchanged,
            Some(replaced) => BattleFairySkillQueueOutcome::Queued { replaced },
        }
    }

    /// Общая механика двух независимых очередей: точный повтор головы ничего
    /// не меняет, другой запрос заменяет ожидающие. Текущее исполнение не входит
    /// в эту операцию; разницу wire-отказов применяет CGame по числу замен.
    fn replace_pending_skill<Dispatch>(
        queue: &mut VecDeque<Dispatch>,
        dispatch: Dispatch,
        same_request: impl FnOnce(&Dispatch, &Dispatch) -> bool,
    ) -> Option<usize> {
        if queue.front().is_some_and(|pending| same_request(pending, &dispatch)) {
            return None;
        }
        let replaced = queue.len();
        queue.clear();
        queue.push_back(dispatch);
        Some(replaced)
    }

    pub(crate) fn player_skills(&self) -> &VecDeque<PlayerSkillDispatch> {
        &self.player_skills
    }

    pub(crate) const fn current_player_skill(&self) -> Option<PlayerSkillDispatch> {
        self.current_player_skill
    }

    /// Pop не меняет текущую цель: riding-отказ выполняется до этой границы.
    pub(crate) fn take_pending_player_skill(&mut self) -> Option<PlayerSkillDispatch> {
        self.player_skills.pop_front()
    }

    pub(crate) fn select_player_skill(&mut self, dispatch: PlayerSkillDispatch) {
        self.current_player_skill = Some(dispatch);
    }

    pub(crate) fn has_current_object_target(&self, target: super::super::shape::ShapeIdentity) -> bool {
        self.current_player_skill.and_then(PlayerSkillDispatch::object_target)
            .is_some_and(|current| current.object_type == target.object_type && current.id == target.id)
    }

    pub(crate) fn release_current_player_command(&mut self) {
        self.current_player_skill = None;
    }

    /// Свободная WarSoul-очередь очищает прежнюю цель до проверки нового FIFO;
    /// живой kernel не уничтожается этим выбором.
    /// Запрет расписания у мёртвого владельца не останавливает активный AI.
    pub(crate) fn begin_next_battle_fairy_skill(
        &mut self,
        can_schedule: bool,
        has_execution: impl Fn(u32) -> bool,
    ) -> Option<BattleFairySkillDispatch> {
        if self.base_ai.active_war_soul_actions().is_empty() {
            if !can_schedule {
                return None;
            }
            self.current_battle_fairy_skill = None;
            if let Some(dispatch) = self.battle_fairy_skills.pop_front() {
                self.current_battle_fairy_skill = Some(dispatch);
                self.selected_battle_fairy_skill_id = dispatch.skill_id();
                // OnScheduleAboutWarSoul извлёк запрос, но IsEnded запрещает
                // повторный Begin уже работающего фонового экземпляра;
                // выбранная цель при этом сохраняется до нового расписания.
                if has_execution(dispatch.skill_id()) {
                    return None;
                }
            }
        }
        self.current_battle_fairy_skill
    }

    pub(crate) fn begin_battle_fairy_fighting(&mut self, now_ms: u32) {
        self.base_ai.add_ai_event(AiShapeAction::Attack, 0, 1, now_ms);
    }

    /// OnFightingWithWarSoul (0x00509230) проверяет IsEnded до вызова AI.
    /// End внутри AI оставляет Attack до следующего Run, без ChangeSkill.
    pub(crate) fn finish_battle_fairy_attack(
        &mut self,
        execution: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
        now_ms: u32,
    ) -> bool {
        let Some(handling) = self.base_ai.active_war_soul_actions().front()
            .filter(|event| event.action == AiShapeAction::Attack && matches!(event.handling, 0 | 1))
            .map(|event| event.handling)
        else {
            return false;
        };
        let skill_id = self.selected_battle_fairy_skill_id();
        if handling == 0 && let Some(execution) = execution {
            if !execution.is_prepared() {
                return false;
            }
            self.base_ai.add_started_back_stage_skill(skill_id);
        }
        self.current_battle_fairy_skill = None;
        self.base_ai.finish_war_soul_attack(now_ms);
        true
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

    pub(crate) const fn current_battle_fairy_skill(&self) -> Option<BattleFairySkillDispatch> {
        self.current_battle_fairy_skill
    }

    pub(crate) fn release_current_battle_fairy_command(&mut self) {
        self.current_battle_fairy_skill = None;
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
// STATUS: IMPLEMENTED
// MATERIALIZED: событие ChangeSkill вызывает общий OnLoseTarget до выбора
// default и снятия события; End(1) нужен только живому неподготовленному
// экземпляру. Prepared остаётся в фоне, следующая команда ждёт нового Run.
// Вызов 0x0047B150 возвращает константу 1. Native повторно назначает default
// после OnLoseTarget; Rust использует его общий завершающий шаг без side effects.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:608
// RVA: 0x00108E40
// ADDRESS: 00508e40
// PROTOTYPE: int __thiscall OnChangeSkill(void)

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
// IMPLEMENTED: встречный вызов из `CPet::OnStayingSchedule` завершает
// начатый concrete skill через `End(1)` и только тогда отправляет отказный
// `0xBFE01`; текущую команду без живого исполнения снимает без ответа.
// Ожидающая команда сама по себе не проходит встречную проверку цели.
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
