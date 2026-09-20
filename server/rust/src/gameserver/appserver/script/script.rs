//! Реестр сценарных ресурсов GameServer.
//!
//! `CScript::LoadFunction(nullptr, data)` из точной пары EXE/PDB читает
//! непрерывный `FunctionList`, преобразует подпись через `atoi` и сохраняет
//! соответствие текста числовому идентификатору в упорядоченном `std::map`.
//! `BTreeMap` сохраняет наблюдаемые правила поиска и порядка. Принадлежащий
//! среде экземпляр `ActiveScript` хранит исходный текст, позицию, контекст
//! игрока, NPC и региона, а также переменные между стадиями главного цикла.
//! `call` создаёт отдельный экземпляр, а `TalkBox` возобновляется ответом
//! клиента через тот же идентификатор сценария. Поиск и удаление по пути или
//! игроку учитывают и текущий экземпляр, временно вынутый из карты. Асинхронная
//! функция внутри выражения сохраняет позицию и при повторном проходе один раз
//! потребляет результат продолжения.
//!
//! Проход аргументов хранит 32 позиции — подтверждённую границу достигнутого
//! `AddTimeGoods`; первые 12 также покрывают полный `CreateNpc`. Поэтому регион,
//! видимость и срок жизни вычисляет тот же вычислитель выражений до входа в
//! диспетчер, а десять пар отверстий `AddTimeGoods` вычисляются позднее заново
//! для каждого подходящего созданного предмета. `MonsterTalk 3304` сохраняет
//! отдельный порядок выражений `text -> name` и намеренно не вычисляет хвост
//! команды. `PlayerMessage 3308` вычисляет тип сообщения только после успешно
//! вычисленного явного цвета, сохраняя раннее прекращение исходного владельца.
//! `GetMonsterRefeashTime 8101` вычисляет только пару региона и обновления,
//! получает время после успешного поиска региона и читает действующую
//! настройку того же `CServerRegion`, который обновляет периодический ИИ
//! монстров. Группа `5411/5412/5414/5420` читает единый загруженный
//! `CPlayerList`, текущий опыт игрока и начальные идентификаторы; виртуальная
//! машина не вычисляет аргументы, которых не касается точный selector.
//! GodsBattle `11121/11126` проверяют соответственно live player и пару
//! live player/GodsBattle region до вычисления аргумента; `11130` проверяет
//! наличие script NPC до вычисления секунд, после чего общий dispatcher
//! связывает вызов с текущим регионом игрока.
//!
//! `wait 6` и `RunTime 22` хранят срок ожидания в том же `ActiveScript`.
//! Первый продолжает выполнение с сохранённой позиции, второй раз в секунду
//! отправляет `0xBF80E`, а после нуля запускает дочерний сценарий с исходным
//! контекстом игрока, NPC и региона. Остальные неподтверждённые семейства
//! приостановки остаются в RAW ниже. `RegisterBuffSkillFunctions` дополняет
//! загруженный русский `FunctionList` потерянным `AddJingJieBuff = 11131`;
//! последующий поиск ведёт в общий `CScript::RunFunction`. Строковые результаты
//! `GetStringByID` и `GetTeamerName` проходят типизированный диспетчер,
//! `TalkBoxSmall` сохраняет экземпляр до ответа клиента, а `random` расходует
//! общий поток MSVCRT. Сброс боевой феи не создаёт отдельное теневое состояние
//! виртуальной машины.

use std::collections::BTreeMap;

use super::buffskillfunc::{
    SCRIPT_FUNCTION_ADD_JING_JIE_BUFF, SCRIPT_FUNCTION_ADD_JING_JIE_BUFF_NAME,
};
use super::function::{
    SCRIPT_FUNCTION_ADD_APPELLATION_STATE, SCRIPT_FUNCTION_ADD_GEM_EXCHANGE_LOG,
    SCRIPT_FUNCTION_ADD_INCREMENT_LOG, SCRIPT_FUNCTION_ADD_JEWELRY_MADE_LOG,
    SCRIPT_FUNCTION_ADD_TIME_GOODS, SCRIPT_FUNCTION_APPLY_FOR_VILLAGE_WAR,
    SCRIPT_FUNCTION_ARGUMENT_CAPACITY, SCRIPT_FUNCTION_CITY_WAR_DECLARE,
    SCRIPT_FUNCTION_DEL_APPELLATION_STATE, SCRIPT_FUNCTION_GET_APPELLATION_STATE,
    SCRIPT_FUNCTION_ENTER_GODS_BATTLE_CONTEND, SCRIPT_FUNCTION_GET_COPY_NUMBER,
    SCRIPT_FUNCTION_GET_GODS_BATTLE_FACTION_XYD, SCRIPT_FUNCTION_GET_LEVEL_EXPERIENCE,
    SCRIPT_FUNCTION_GET_NAME, SCRIPT_FUNCTION_GET_OWNED_REGION_FACTION_ID,
    SCRIPT_FUNCTION_GET_OWNED_REGION_UNION_ID, SCRIPT_FUNCTION_GET_STRING_BY_ID,
    SCRIPT_FUNCTION_GET_PLAYER_GODS_BATTLE_FACTION,
    SCRIPT_FUNCTION_GET_TEAMER_NAME, SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME,
    SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME, SCRIPT_FUNCTION_IS_CITY_WAR_DECLARE_TIME,
    SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME, SCRIPT_FUNCTION_LIST_BANNED_PLAYER,
    SCRIPT_FUNCTION_MONSTER_TALK, SCRIPT_FUNCTION_PLAY_EFFECT, SCRIPT_FUNCTION_PLAY_SOUND,
    SCRIPT_FUNCTION_PLAYER_MESSAGE, SCRIPT_FUNCTION_PLAYER_TALK,
    SCRIPT_FUNCTION_REQUEST_PLAYER_RANKS, ScriptFunctionDispatchOutcome,
    ScriptFunctionParameterKind, ScriptFunctionRuntime, ScriptStringFunctionDispatchOutcome,
    dispatch_script_function, dispatch_script_string_function,
    gods_battle_region_script_caller_is_live, owned_region_script_caller_is_live,
    run_add_time_goods_script_function, script_function_parameter_kind,
    script_player_npc_caller_exists, village_war_script_caller_is_live,
};
use super::jjcfunc::{JjcScriptFunctionOutcome, dispatch_jjc_script_function};
use super::parser;
use super::variablelist::section_records;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CScriptFunctionRegistry {
    functions: BTreeMap<Vec<u8>, i32>,
}

impl CScriptFunctionRegistry {
    pub(crate) fn load(&mut self, source: &[u8]) {
        self.functions.clear();
        let mut declared_functions = 0usize;
        let mut replaced_names = 0usize;
        for (caption, name) in section_records(source, b"FunctionList") {
            let id = legacy_atoi(caption);
            if self.functions.insert(name.to_vec(), id).is_some() {
                replaced_names += 1;
            }
            declared_functions += 1;
        }
        if self
            .functions
            .insert(
                SCRIPT_FUNCTION_ADD_JING_JIE_BUFF_NAME.to_vec(),
                SCRIPT_FUNCTION_ADD_JING_JIE_BUFF,
            )
            .is_some()
        {
            replaced_names += 1;
        }
        tracing::debug!(
            declared_functions,
            replaced_names,
            registered_functions = self.functions.len(),
            "загружен реестр сценарных функций"
        );
    }

    pub(crate) fn query(&self, name: &[u8]) -> Option<i32> {
        self.functions.get(visible_c_string(name)).copied()
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.functions.len();
        self.functions.clear();
        count
    }
}

fn visible_c_string(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

pub(crate) fn legacy_atoi(value: &[u8]) -> i32 {
    let value = visible_c_string(value);
    let value = value
        .get(
            value
                .iter()
                .position(|byte| !byte.is_ascii_whitespace())
                .unwrap_or(value.len())..,
        )
        .unwrap_or_default();
    let (negative, digits) = match value.first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    let magnitude =
        digits
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .fold(0_i64, |current, byte| {
                current
                    .saturating_mul(10)
                    .saturating_add(i64::from(*byte - b'0'))
            });
    let signed = if negative { -magnitude } else { magnitude };
    signed.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

const SCRIPT_INT_PARAMETER_ERROR: i32 = 0x09ff_fff9;
const SCRIPT_FUNCTION_WAIT: i32 = 6;
const SCRIPT_FUNCTION_RUN_TIME: i32 = 22;

/// Контекст исполнения `stRunScript`, заполняемый конкретными вызывающими
/// владельцами. Смерть монстра передаёт его базовый индекс, игрока, регион и
/// точку выпадения предметов, поэтому отложенные сценарии сохраняют полный
/// контекст, а не только имя файла.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ScriptExecutionContext {
    pub(crate) player_id: Option<i32>,
    pub(crate) npc_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) used_item_id: Option<CGuid>,
    pub(crate) died_monster_index: Option<u32>,
    pub(crate) drop_goods_position: Option<(i32, i32)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptCommandOutcome {
    Handled {
        function_id: i32,
        legacy_return: i32,
    },
    Yielded {
        function_id: i32,
        legacy_return: i32,
    },
    Terminated {
        function_id: i32,
        legacy_return: i32,
    },
    UnknownFunction,
    InvalidExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScriptStepDisposition {
    Ended,
    YieldedCall {
        path: Vec<u8>,
    },
    WaitingFunction {
        function_id: i32,
        replay_command: bool,
    },
    WaitingFunctionTimedOut {
        function_id: i32,
        legacy_return: i32,
    },
    WaitingRuntime {
        countdown_seconds: Option<i32>,
        expired_path: Option<Vec<u8>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveScript {
    pub(crate) id: i32,
    pub(crate) path: Vec<u8>,
    source: Vec<u8>,
    point: usize,
    context: ScriptExecutionContext,
    integer_variables: BTreeMap<Vec<u8>, i32>,
    string_variables: BTreeMap<Vec<u8>, Vec<u8>>,
    waiting_function: Option<i32>,
    waiting_replay: bool,
    resumed_function: Option<(i32, i32)>,
    runtime_wait: Option<ScriptRuntimeWait>,
    waiting_started_ms: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScriptRuntimeWait {
    deadline_ms: u32,
    last_update_ms: u32,
    completion: ScriptRuntimeCompletion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ScriptRuntimeCompletion {
    Continue,
    RunScript(Vec<u8>),
}

impl ActiveScript {
    pub(crate) fn new(
        id: i32,
        path: Vec<u8>,
        source: Vec<u8>,
        context: ScriptExecutionContext,
    ) -> Self {
        Self {
            id,
            path,
            source,
            point: 0,
            context,
            integer_variables: BTreeMap::new(),
            string_variables: BTreeMap::new(),
            waiting_function: None,
            waiting_replay: false,
            resumed_function: None,
            runtime_wait: None,
            waiting_started_ms: None,
        }
    }

    pub(crate) const fn player_id(&self) -> Option<i32> {
        self.context.player_id
    }

    pub(crate) const fn context(&self) -> ScriptExecutionContext {
        self.context
    }

    pub(crate) const fn waiting_function(&self) -> Option<i32> {
        self.waiting_function
    }

    pub(crate) fn is_countdown_runtime_waiting(&self) -> bool {
        matches!(
            self.runtime_wait.as_ref(),
            Some(ScriptRuntimeWait {
                completion: ScriptRuntimeCompletion::RunScript(_),
                ..
            })
        )
    }

    pub(crate) fn continue_with(&mut self, value: i32) -> bool {
        let Some(function_id) = self.waiting_function.take() else {
            return false;
        };
        self.waiting_started_ms = None;
        self.integer_variables
            .insert(normalize_name(b"$m_TalkRet"), value);
        if self.waiting_replay {
            self.resumed_function = Some((function_id, value));
        }
        self.waiting_replay = false;
        true
    }

    pub(crate) fn run_step<Runtime: ScriptFunctionRuntime>(
        &mut self,
        game: &mut CGame,
        runtime: &mut Runtime,
    ) -> ScriptStepDisposition {
        if let Some(wait) = self.runtime_wait.as_mut() {
            let now = runtime.now_milliseconds();
            if matches!(&wait.completion, ScriptRuntimeCompletion::Continue) {
                if wait.deadline_ms < now {
                    self.runtime_wait = None;
                }
                return ScriptStepDisposition::WaitingRuntime {
                    countdown_seconds: None,
                    expired_path: None,
                };
            }
            if now.wrapping_sub(wait.last_update_ms) <= 999 {
                return ScriptStepDisposition::WaitingRuntime {
                    countdown_seconds: None,
                    expired_path: None,
                };
            }
            wait.last_update_ms = now;
            let remaining_ms = if wait.deadline_ms > now {
                wait.deadline_ms - now
            } else {
                0
            };
            let expired_path = (remaining_ms < 1).then(|| match &wait.completion {
                ScriptRuntimeCompletion::RunScript(path) => path.clone(),
                ScriptRuntimeCompletion::Continue => unreachable!("continue обработан выше"),
            });
            if expired_path.is_some() {
                self.runtime_wait = None;
            }
            return ScriptStepDisposition::WaitingRuntime {
                countdown_seconds: Some((remaining_ms / 1000) as i32),
                expired_path,
            };
        }
        if let Some(function_id) = self.waiting_function {
            if matches!(
                function_id,
                SCRIPT_FUNCTION_GET_COPY_NUMBER | SCRIPT_FUNCTION_LIST_BANNED_PLAYER
            ) && self
                .waiting_started_ms
                .is_some_and(|started| runtime.now_milliseconds().wrapping_sub(started) >= 10_000)
            {
                self.waiting_function = None;
                self.waiting_replay = false;
                self.waiting_started_ms = None;
                let legacy_return = if function_id == SCRIPT_FUNCTION_LIST_BANNED_PLAYER {
                    -1
                } else {
                    0
                };
                self.integer_variables
                    .insert(normalize_name(b"$m_TalkRet"), legacy_return);
                tracing::warn!(
                    script_id = self.id,
                    function_id,
                    legacy_return,
                    "истекло ожидание результата сценарной функции"
                );
                return ScriptStepDisposition::WaitingFunctionTimedOut {
                    function_id,
                    legacy_return,
                };
            }
            return ScriptStepDisposition::WaitingFunction {
                function_id,
                replay_command: self.waiting_replay,
            };
        }
        let mut script = CScript {
            source: &self.source,
            path: &self.path,
            point: self.point,
            context: self.context,
            integer_variables: std::mem::take(&mut self.integer_variables),
            string_variables: std::mem::take(&mut self.string_variables),
            script_id: self.id,
            resumed_function: self.resumed_function.take(),
            pending_yield: None,
            runtime_wait: &mut self.runtime_wait,
        };
        let disposition = script.run_step(game, runtime);
        self.point = script.point;
        self.integer_variables = script.integer_variables;
        self.string_variables = script.string_variables;
        self.resumed_function = script.resumed_function;
        if let ScriptStepDisposition::WaitingFunction {
            function_id,
            replay_command,
        } = &disposition
        {
            self.waiting_function = Some(*function_id);
            self.waiting_replay = *replay_command;
            self.waiting_started_ms = Some(runtime.now_milliseconds());
        }
        disposition
    }
}

/// Достигнутый владелец исполнения исходного `CScript`: экземпляр читает одну
/// непрерывную группу команд и передаёт вычисленные параметры в числовой
/// диспетчер `RunFunction`. Команды `call` и приостановка диалога возвращают
/// управление планировщику, который сохраняет позицию и контекст. Поддержанная
/// часть языка включает `if/else`, `goto`, `call` и локальные присваивания;
/// неизвестная команда завершает экземпляр до побочных эффектов следующей
/// строки.
pub(crate) struct CScript<'a> {
    source: &'a [u8],
    path: &'a [u8],
    point: usize,
    context: ScriptExecutionContext,
    integer_variables: BTreeMap<Vec<u8>, i32>,
    string_variables: BTreeMap<Vec<u8>, Vec<u8>>,
    script_id: i32,
    resumed_function: Option<(i32, i32)>,
    pending_yield: Option<i32>,
    runtime_wait: &'a mut Option<ScriptRuntimeWait>,
}

impl<'a> CScript<'a> {
    pub(crate) fn run_step<Runtime: ScriptFunctionRuntime>(
        &mut self,
        game: &mut CGame,
        runtime: &mut Runtime,
    ) -> ScriptStepDisposition {
        loop {
            let command_point = self.point;
            let Some(command) = self.read_command() else {
                break;
            };
            let command = trim_ascii(&command);
            if command.is_empty() || command == b"{" || command == b"}" {
                continue;
            }
            let name = command_name(command);
            if name.eq_ignore_ascii_case(b"return") {
                let expression = trim_ascii(&command[6..]);
                let legacy_return = if expression.is_empty() {
                    0
                } else {
                    self
                        .evaluate_integer(game, runtime, expression)
                        .unwrap_or(SCRIPT_INT_PARAMETER_ERROR)
                };
                tracing::trace!(
                    script_id = self.script_id,
                    command_point,
                    legacy_return,
                    "сценарий завершён командой return"
                );
                return ScriptStepDisposition::Ended;
            }
            if name.eq_ignore_ascii_case(b"<begin>")
                || name.eq_ignore_ascii_case(b"<end>")
                || parser::label(command).is_some()
            {
                continue;
            }
            if name.eq_ignore_ascii_case(b"if") {
                let condition = function_parameters(command)
                    .and_then(|values| values.first().copied())
                    .or_else(|| {
                        command
                            .get(name.len()..)
                            .map(trim_ascii)
                            .filter(|value| !value.is_empty())
                    });
                let Some(condition) = condition else {
                    self.trace_invalid_command(command_point, "отсутствует условие if");
                    break;
                };
                let Some(condition) = self.evaluate_integer(game, runtime, condition) else {
                    if let Some(function_id) = self.pending_yield.take() {
                        self.point = command_point;
                        return ScriptStepDisposition::WaitingFunction {
                            function_id,
                            replay_command: true,
                        };
                    }
                    self.trace_invalid_command(command_point, "не вычислено условие if");
                    break;
                };
                if condition == 0 && !self.skip_next_block(true) {
                    self.trace_invalid_command(command_point, "не найден блок после if");
                    break;
                }
                continue;
            }
            if name.eq_ignore_ascii_case(b"else") {
                if !self.skip_next_block(false) {
                    self.trace_invalid_command(command_point, "не найден блок после else");
                    break;
                }
                continue;
            }
            if name.eq_ignore_ascii_case(b"goto") {
                let Some(target) = function_parameters(command)
                    .and_then(|values| values.first().copied())
                    .map(unquote)
                    .filter(|value| !value.is_empty())
                else {
                    self.trace_invalid_command(command_point, "отсутствует цель goto");
                    break;
                };
                if !self.jump_to(target) {
                    self.trace_invalid_command(command_point, "не найдена цель goto");
                    break;
                }
                continue;
            }
            if name.eq_ignore_ascii_case(b"call") {
                let Some(path) = function_parameters(command)
                    .and_then(|values| values.first().copied())
                    .and_then(|value| self.evaluate_string(game, runtime, value))
                else {
                    self.trace_invalid_command(command_point, "не вычислен путь call");
                    break;
                };
                if game.script_file_data(&path).is_none() {
                    self.trace_invalid_command(command_point, "не найден файл call");
                    break;
                }
                return ScriptStepDisposition::YieldedCall { path };
            }
            if let Some(assigned) = self.run_assignment(game, runtime, command) {
                if assigned {
                    continue;
                }
                if let Some(function_id) = self.pending_yield.take() {
                    self.point = command_point;
                    return ScriptStepDisposition::WaitingFunction {
                        function_id,
                        replay_command: true,
                    };
                }
                self.trace_invalid_command(command_point, "не вычислено присваивание");
                break;
            }
            match self.run_function(game, runtime, command) {
                ScriptCommandOutcome::Handled {
                    function_id,
                    legacy_return,
                } => {
                    tracing::trace!(
                        script_id = self.script_id,
                        command_point,
                        function_id,
                        legacy_return,
                        "сценарная функция выполнена"
                    );
                    if self.runtime_wait.is_some() {
                        return ScriptStepDisposition::WaitingRuntime {
                            countdown_seconds: None,
                            expired_path: None,
                        };
                    }
                }
                ScriptCommandOutcome::Yielded {
                    function_id,
                    legacy_return,
                } => {
                    tracing::trace!(
                        script_id = self.script_id,
                        command_point,
                        function_id,
                        legacy_return,
                        "сценарная функция ожидает продолжения"
                    );
                    return ScriptStepDisposition::WaitingFunction {
                        function_id,
                        replay_command: false,
                    };
                }
                ScriptCommandOutcome::Terminated {
                    function_id,
                    legacy_return,
                } => {
                    tracing::trace!(
                        script_id = self.script_id,
                        command_point,
                        function_id,
                        legacy_return,
                        "сценарная функция завершила сценарий"
                    );
                    return ScriptStepDisposition::Ended;
                }
                ScriptCommandOutcome::UnknownFunction => {
                    self.trace_invalid_command(command_point, "неизвестная сценарная функция");
                    break;
                }
                ScriptCommandOutcome::InvalidExpression => {
                    self.trace_invalid_command(command_point, "не вычислены аргументы функции");
                    break;
                }
            }
        }
        ScriptStepDisposition::Ended
    }

    fn trace_invalid_command(&self, command_point: usize, reason: &'static str) {
        tracing::debug!(
            script_id = self.script_id,
            command_point,
            reason,
            "исполнение сценария остановлено на некорректной команде"
        );
    }

    fn run_function<Runtime: ScriptFunctionRuntime>(
        &mut self,
        game: &mut CGame,
        runtime: &mut Runtime,
        expression: &[u8],
    ) -> ScriptCommandOutcome {
        let Some((name, parameters)) = split_function(expression) else {
            return ScriptCommandOutcome::UnknownFunction;
        };
        let Some(function_id) = game.script_function_id(name) else {
            return ScriptCommandOutcome::UnknownFunction;
        };
        if self
            .resumed_function
            .is_some_and(|(resumed_id, _)| resumed_id == function_id)
        {
            let (_, legacy_return) = self
                .resumed_function
                .take()
                .expect("resumed function проверена перед consume");
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return,
            };
        }
        if matches!(function_id, SCRIPT_FUNCTION_WAIT | SCRIPT_FUNCTION_RUN_TIME) {
            let Some(wait_ms) = parameters
                .first()
                .and_then(|parameter| self.evaluate_integer(game, runtime, parameter))
            else {
                return ScriptCommandOutcome::InvalidExpression;
            };
            let completion = if function_id == SCRIPT_FUNCTION_RUN_TIME {
                let Some(path) = parameters
                    .get(1)
                    .map(|parameter| self.evaluate_string(game, runtime, parameter))
                    .unwrap_or_else(|| Some(Vec::new()))
                else {
                    return ScriptCommandOutcome::InvalidExpression;
                };
                ScriptRuntimeCompletion::RunScript(path)
            } else {
                ScriptRuntimeCompletion::Continue
            };
            let now = runtime.now_milliseconds();
            *self.runtime_wait = Some(ScriptRuntimeWait {
                deadline_ms: now.wrapping_add(wait_ms.max(0) as u32),
                last_update_ms: now,
                completion,
            });
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if function_id == SCRIPT_FUNCTION_ENTER_GODS_BATTLE_CONTEND
            && self.context.npc_id.is_none()
        {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if function_id == SCRIPT_FUNCTION_GET_PLAYER_GODS_BATTLE_FACTION
            && !self
                .context
                .player_id
                .is_some_and(|player_id| game.find_player(player_id).is_some())
        {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if function_id == SCRIPT_FUNCTION_GET_GODS_BATTLE_FACTION_XYD
            && !gods_battle_region_script_caller_is_live(
                game,
                self.context.player_id,
                self.context.region_id,
            )
        {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if matches!(
            function_id,
            SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME
                | SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME
                | SCRIPT_FUNCTION_APPLY_FOR_VILLAGE_WAR
                | SCRIPT_FUNCTION_CITY_WAR_DECLARE
                | SCRIPT_FUNCTION_IS_CITY_WAR_DECLARE_TIME
                | SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME
        ) && !village_war_script_caller_is_live(
            game,
            self.context.player_id,
            self.context.npc_id,
        ) {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if matches!(
            function_id,
            SCRIPT_FUNCTION_GET_OWNED_REGION_FACTION_ID | SCRIPT_FUNCTION_GET_OWNED_REGION_UNION_ID
        ) && !owned_region_script_caller_is_live(
            game,
            self.context.player_id,
            self.context.region_id,
        ) {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if function_id == SCRIPT_FUNCTION_PLAY_EFFECT
            && !owned_region_script_caller_is_live(
                game,
                self.context.player_id,
                self.context.region_id,
            )
        {
            return ScriptCommandOutcome::InvalidExpression;
        }
        if function_id == SCRIPT_FUNCTION_PLAY_SOUND
            && !owned_region_script_caller_is_live(
                game,
                self.context.player_id,
                self.context.region_id,
            )
        {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if function_id == SCRIPT_FUNCTION_PLAYER_TALK
            && !self
                .context
                .player_id
                .is_some_and(|player_id| game.find_player(player_id).is_some())
        {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if (function_id == SCRIPT_FUNCTION_ADD_GEM_EXCHANGE_LOG
            && !game.log_system().goods_gem_exchange_log_enabled())
            || (function_id == SCRIPT_FUNCTION_ADD_JEWELRY_MADE_LOG
                && !game.log_system().goods_jewelry_made_log_enabled())
        {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if function_id == SCRIPT_FUNCTION_ADD_JING_JIE_BUFF
            && !self
                .context
                .player_id
                .is_some_and(|player_id| game.find_player(player_id).is_some())
        {
            return ScriptCommandOutcome::InvalidExpression;
        }
        if function_id == SCRIPT_FUNCTION_REQUEST_PLAYER_RANKS
            && !script_player_npc_caller_exists(game, self.context.player_id, self.context.npc_id)
        {
            return ScriptCommandOutcome::Handled {
                function_id,
                legacy_return: 0,
            };
        }
        if function_id == SCRIPT_FUNCTION_GET_LEVEL_EXPERIENCE
            && !self
                .context
                .player_id
                .is_some_and(|player_id| game.find_player(player_id).is_some())
        {
            return ScriptCommandOutcome::InvalidExpression;
        }
        if matches!(
            function_id,
            SCRIPT_FUNCTION_ADD_APPELLATION_STATE
                | SCRIPT_FUNCTION_DEL_APPELLATION_STATE
                | SCRIPT_FUNCTION_GET_APPELLATION_STATE
        ) && !self
            .context
            .player_id
            .is_some_and(|player_id| game.find_player(player_id).is_some())
        {
            return ScriptCommandOutcome::InvalidExpression;
        }
        let mut integer_arguments = [None; SCRIPT_FUNCTION_ARGUMENT_CAPACITY];
        let mut string_arguments: [Option<Vec<u8>>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY] =
            std::array::from_fn(|_| None);
        if function_id == SCRIPT_FUNCTION_MONSTER_TALK {
            // В точной реализации EXE функция `3304` вычисляет текст с индексом
            // `1` раньше имени и не трогает хвост; выражения с побочными
            // эффектами наблюдают тот же порядок.
            for index in [1_usize, 0] {
                if let Some(parameter) = parameters.get(index) {
                    string_arguments[index] = self.evaluate_string(game, runtime, parameter);
                }
            }
        } else {
            for (index, parameter) in parameters
                .iter()
                .take(SCRIPT_FUNCTION_ARGUMENT_CAPACITY)
                .enumerate()
            {
                if function_id == SCRIPT_FUNCTION_PLAYER_MESSAGE && index == 3 {
                    let color = integer_arguments[2].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
                    if color == SCRIPT_INT_PARAMETER_ERROR {
                        break;
                    }
                }
                if function_id == SCRIPT_FUNCTION_ADD_INCREMENT_LOG && index >= 3 {
                    let parsed_type = integer_arguments[2].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
                    let log_type = if parsed_type == SCRIPT_INT_PARAMETER_ERROR {
                        1
                    } else {
                        parsed_type
                    };
                    if log_type != 0 {
                        break;
                    }
                }
                if function_id == SCRIPT_FUNCTION_ADD_TIME_GOODS && index == 12 {
                    // Пары отверстий исходный владелец вычисляет позднее,
                    // внутри прохода каждого фактически созданного предмета.
                    break;
                }
                match script_function_parameter_kind(function_id, index) {
                    ScriptFunctionParameterKind::Integer => {
                        integer_arguments[index] = Some(
                            self.evaluate_integer(game, runtime, parameter)
                                .unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
                        );
                    }
                    ScriptFunctionParameterKind::String if index < string_arguments.len() => {
                        string_arguments[index] = self.evaluate_string(game, runtime, parameter);
                    }
                    ScriptFunctionParameterKind::String | ScriptFunctionParameterKind::Unused => {}
                }
            }
        }
        let dispatch_outcome = match dispatch_jjc_script_function(
            game,
            self.context.player_id,
            self.context.region_id,
            function_id,
            integer_arguments,
            || runtime.now_milliseconds(),
        ) {
            JjcScriptFunctionOutcome::Handled { legacy_return } => {
                ScriptFunctionDispatchOutcome::Handled { legacy_return }
            }
            JjcScriptFunctionOutcome::DifferentFunction => {
                if function_id == SCRIPT_FUNCTION_ADD_TIME_GOODS {
                    let player_id = self.context.player_id;
                    run_add_time_goods_script_function(
                        game,
                        runtime,
                        player_id,
                        string_arguments[0].as_deref(),
                        &integer_arguments,
                        |game, runtime| {
                            let mut sockets = [(0, 0); 10];
                            for (socket, pair) in sockets.iter_mut().enumerate() {
                                let color = parameters
                                    .get(12 + socket * 2)
                                    .and_then(|parameter| {
                                        self.evaluate_integer(game, runtime, parameter)
                                    })
                                    .unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
                                let gem_index = parameters
                                    .get(13 + socket * 2)
                                    .and_then(|parameter| {
                                        self.evaluate_integer(game, runtime, parameter)
                                    })
                                    .unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
                                if matches!(color, SCRIPT_INT_PARAMETER_ERROR | -1) {
                                    return None;
                                }
                                *pair = (color, gem_index);
                            }
                            Some(sockets)
                        },
                    )
                } else {
                    dispatch_script_function(
                        game,
                        runtime,
                        self.context.player_id,
                        self.context.npc_id,
                        self.context.region_id,
                        self.context.used_item_id,
                        self.context.died_monster_index,
                        self.context.drop_goods_position,
                        self.script_id,
                        self.path,
                        function_id,
                        parameters.len(),
                        integer_arguments,
                        std::array::from_fn(|index| string_arguments[index].as_deref()),
                    )
                }
            }
        };
        match dispatch_outcome {
            ScriptFunctionDispatchOutcome::Invalid => ScriptCommandOutcome::InvalidExpression,
            ScriptFunctionDispatchOutcome::DifferentFunction => {
                ScriptCommandOutcome::UnknownFunction
            }
            ScriptFunctionDispatchOutcome::Yielded { legacy_return } => {
                self.pending_yield = Some(function_id);
                ScriptCommandOutcome::Yielded {
                    function_id,
                    legacy_return,
                }
            }
            ScriptFunctionDispatchOutcome::Terminated { legacy_return } => {
                ScriptCommandOutcome::Terminated {
                    function_id,
                    legacy_return,
                }
            }
            ScriptFunctionDispatchOutcome::Handled { legacy_return } => {
                if function_id == SCRIPT_FUNCTION_LIST_BANNED_PLAYER && legacy_return == -1 {
                    self.integer_variables
                        .insert(normalize_name(b"$m_TalkRet"), -1);
                }
                ScriptCommandOutcome::Handled {
                    function_id,
                    legacy_return,
                }
            }
        }
    }

    fn evaluate_integer<Runtime: ScriptFunctionRuntime>(
        &mut self,
        game: &mut CGame,
        runtime: &mut Runtime,
        expression: &[u8],
    ) -> Option<i32> {
        let expression = strip_wrapped_parentheses(expression);
        if expression.is_empty() {
            return None;
        }
        for operation in [b"||".as_slice(), b"&&".as_slice()] {
            if let Some(position) = find_top_level(expression, operation, false) {
                let left = self.evaluate_integer(game, runtime, &expression[..position])?;
                let right = self.evaluate_integer(
                    game,
                    runtime,
                    &expression[position + operation.len()..],
                )?;
                return Some(i32::from(if operation == b"||" {
                    left != 0 || right != 0
                } else {
                    left != 0 && right != 0
                }));
            }
        }
        for operation in [
            b"==".as_slice(),
            b"!=".as_slice(),
            b">=".as_slice(),
            b"<=".as_slice(),
            b">".as_slice(),
            b"<".as_slice(),
        ] {
            if let Some(position) = find_top_level(expression, operation, true) {
                if (operation == b"==" || operation == b"!=")
                    && (is_string_expression(&expression[..position])
                        || is_string_expression(&expression[position + operation.len()..]))
                {
                    let left = self.evaluate_string(game, runtime, &expression[..position])?;
                    let right = self.evaluate_string(
                        game,
                        runtime,
                        &expression[position + operation.len()..],
                    )?;
                    return Some(i32::from(if operation == b"==" {
                        left == right
                    } else {
                        left != right
                    }));
                }
                let left = self.evaluate_integer(game, runtime, &expression[..position])?;
                let right = self.evaluate_integer(
                    game,
                    runtime,
                    &expression[position + operation.len()..],
                )?;
                let result = match operation {
                    b"==" => left == right,
                    b"!=" => left != right,
                    b">=" => left >= right,
                    b"<=" => left <= right,
                    b">" => left > right,
                    _ => left < right,
                };
                return Some(i32::from(result));
            }
        }
        if let Some(position) = find_top_level_chars_reverse(expression, b"&|", false) {
            let left = self.evaluate_integer(game, runtime, &expression[..position])?;
            let right = self.evaluate_integer(game, runtime, &expression[position + 1..])?;
            return Some(if expression[position] == b'&' {
                left & right
            } else {
                left | right
            });
        }
        if let Some(position) = find_top_level_chars_reverse(expression, b"+-", true) {
            let left = self.evaluate_integer(game, runtime, &expression[..position])?;
            let right = self.evaluate_integer(game, runtime, &expression[position + 1..])?;
            return if expression[position] == b'+' {
                left.checked_add(right)
            } else {
                left.checked_sub(right)
            };
        }
        if let Some(position) = find_top_level_chars_reverse(expression, b"*/%", false) {
            let left = self.evaluate_integer(game, runtime, &expression[..position])?;
            let right = self.evaluate_integer(game, runtime, &expression[position + 1..])?;
            return match expression[position] {
                b'*' => left.checked_mul(right),
                b'/' => left.checked_div(right),
                _ => left.checked_rem(right),
            };
        }
        if let Ok(text) = std::str::from_utf8(expression) {
            if let Ok(value) = text.parse::<i32>() {
                return Some(value);
            }
        }
        if expression.starts_with(b"$") {
            let (name, index) = split_variable_reference(expression)?;
            let index = match index {
                Some(index) => {
                    usize::try_from(self.evaluate_integer(game, runtime, index)?).ok()?
                }
                None => 0,
            };
            if index == 0 {
                if let Some(value) = self.integer_variables.get(&normalize_name(name)) {
                    return Some(*value);
                }
            }
            if let Some(value) = self
                .context
                .player_id
                .and_then(|player_id| game.find_player(player_id))
                .and_then(|player| player.variable_list().integer(name, index))
            {
                return Some(value);
            }
            return game.general_variables().integer(name, index);
        }
        match self.run_function(game, runtime, expression) {
            ScriptCommandOutcome::Handled { legacy_return, .. } => Some(legacy_return),
            ScriptCommandOutcome::UnknownFunction
            | ScriptCommandOutcome::InvalidExpression
            | ScriptCommandOutcome::Yielded { .. }
            | ScriptCommandOutcome::Terminated { .. } => None,
        }
    }

    fn evaluate_string<Runtime: ScriptFunctionRuntime>(
        &mut self,
        game: &mut CGame,
        runtime: &mut Runtime,
        expression: &[u8],
    ) -> Option<Vec<u8>> {
        let expression = trim_ascii(expression);
        if let Some(position) = find_top_level_chars_reverse(expression, b"+", false) {
            let mut left = self.evaluate_string(game, runtime, &expression[..position])?;
            let right = self.evaluate_string(game, runtime, &expression[position + 1..])?;
            left.extend_from_slice(&right);
            return Some(left);
        }
        if expression.len() >= 2
            && matches!(expression[0], b'"' | b'\'')
            && expression.last() == Some(&expression[0])
        {
            return Some(expression[1..expression.len() - 1].to_vec());
        }
        if expression.starts_with(b"#") {
            if let Some(value) = self.string_variables.get(&normalize_name(expression)) {
                return Some(value.clone());
            }
            if let Some(value) = self
                .context
                .player_id
                .and_then(|player_id| game.find_player(player_id))
                .and_then(|player| player.variable_list().string(expression))
            {
                return Some(value.to_vec());
            }
            return game
                .general_variables()
                .string(expression)
                .map(<[u8]>::to_vec);
        }
        if let Some((name, parameters)) = split_function(expression) {
            let function_id = game.script_function_id(name)?;
            let first = parameters.first().copied();
            let evaluated_integer = if matches!(
                function_id,
                SCRIPT_FUNCTION_GET_NAME | SCRIPT_FUNCTION_GET_TEAMER_NAME
            ) {
                first.and_then(|parameter| self.evaluate_integer(game, runtime, parameter))
            } else {
                None
            };
            let evaluated_string = if function_id == SCRIPT_FUNCTION_GET_STRING_BY_ID {
                first.and_then(|parameter| self.evaluate_string(game, runtime, parameter))
            } else {
                None
            };
            match dispatch_script_string_function(
                game,
                self.context.player_id,
                self.context.died_monster_index,
                function_id,
                evaluated_integer,
                evaluated_string.as_deref(),
            ) {
                ScriptStringFunctionDispatchOutcome::Handled(value) => return Some(value),
                ScriptStringFunctionDispatchOutcome::Invalid => return None,
                ScriptStringFunctionDispatchOutcome::DifferentFunction => {}
            }
        }
        self.evaluate_integer(game, runtime, expression)
            .map(|value| value.to_string().into_bytes())
    }

    fn read_command(&mut self) -> Option<Vec<u8>> {
        match parser::next_command(self.source, self.point) {
            Ok(Some(command)) => {
                self.point = command.next_point;
                Some(command.bytes)
            }
            Ok(None) => {
                self.point = self.source.len();
                None
            }
            Err(error) => {
                tracing::debug!(
                    script_id = self.script_id,
                    parse_offset = error.offset,
                    parse_kind = ?error.kind,
                    "структурный разбор сценарной команды остановлен"
                );
                self.point = self.source.len();
                None
            }
        }
    }

    fn run_assignment<Runtime: ScriptFunctionRuntime>(
        &mut self,
        game: &mut CGame,
        runtime: &mut Runtime,
        command: &[u8],
    ) -> Option<bool> {
        let Some(position) = find_assignment(command) else {
            return None;
        };
        let name = trim_ascii(&command[..position]);
        let expression = trim_ascii(&command[position + 1..]);
        if name.starts_with(b"$") {
            let Some(value) = self.evaluate_integer(game, runtime, expression) else {
                return Some(false);
            };
            let normalized = normalize_name(name);
            if let Some(current) = self.integer_variables.get_mut(&normalized) {
                *current = value;
                return Some(true);
            }
            if let Some(player) = self
                .context
                .player_id
                .and_then(|player_id| game.find_player_mut(player_id))
            {
                let outcome = player.set_integer_variable(name, 0, value);
                if !matches!(
                    outcome,
                    super::variablelist::GameVariableMutationOutcome::NameNotFound
                ) {
                    return Some(true);
                }
            }
            game.set_general_variable_integer(name, value);
            return Some(true);
        }
        if name.starts_with(b"#") {
            let Some(value) = self.evaluate_string(game, runtime, expression) else {
                return Some(false);
            };
            let normalized = normalize_name(name);
            if let Some(current) = self.string_variables.get_mut(&normalized) {
                *current = value;
                return Some(true);
            }
            if let Some(player) = self
                .context
                .player_id
                .and_then(|player_id| game.find_player_mut(player_id))
            {
                let outcome = player.set_string_variable(name, &value);
                if !matches!(
                    outcome,
                    super::variablelist::GameVariableMutationOutcome::NameNotFound
                ) {
                    return Some(true);
                }
            }
            game.set_general_variable_string(name, &value);
            return Some(true);
        }
        None
    }

    fn skip_next_block(&mut self, preserve_else: bool) -> bool {
        let saved = self.point;
        let Some(command) = self.read_command() else {
            return false;
        };
        if trim_ascii(&command) != b"{" {
            self.point = saved;
            return false;
        }
        let mut depth = 1_i32;
        while let Some(command) = self.read_command() {
            match trim_ascii(&command) {
                b"{" => depth += 1,
                b"}" => {
                    depth -= 1;
                    if depth == 0 {
                        if preserve_else {
                            let after_block = self.point;
                            let Some(next) = self.read_command() else {
                                return true;
                            };
                            if !command_name(trim_ascii(&next)).eq_ignore_ascii_case(b"else") {
                                self.point = after_block;
                            }
                        }
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn jump_to(&mut self, target: &[u8]) -> bool {
        let saved = self.point;
        self.point = 0;
        while let Some(command) = self.read_command() {
            let command = trim_ascii(&command);
            if parser::label(command).is_some_and(|label| label == target) {
                return true;
            }
        }
        self.point = saved;
        false
    }
}

fn is_string_expression(expression: &[u8]) -> bool {
    let expression = trim_ascii(expression);
    expression.starts_with(b"#")
        || matches!(expression.first(), Some(b'"' | b'\''))
        || expression.contains(&b'#')
}

fn trim_ascii(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

fn strip_wrapped_parentheses(value: &[u8]) -> &[u8] {
    let mut value = trim_ascii(value);
    loop {
        if value.len() < 2 || value[0] != b'(' || value[value.len() - 1] != b')' {
            return value;
        }
        let mut depth = 0_i32;
        let mut quoted = false;
        let mut encloses_all = true;
        for (index, byte) in value.iter().copied().enumerate() {
            if byte == b'"' {
                quoted = !quoted;
            } else if !quoted && byte == b'(' {
                depth += 1;
            } else if !quoted && byte == b')' {
                depth -= 1;
                if depth == 0 && index + 1 != value.len() {
                    encloses_all = false;
                    break;
                }
            }
        }
        if !encloses_all || depth != 0 {
            return value;
        }
        value = trim_ascii(&value[1..value.len() - 1]);
    }
}

fn split_function(expression: &[u8]) -> Option<(&[u8], Vec<&[u8]>)> {
    parser::function(expression).ok()
}

fn function_parameters(expression: &[u8]) -> Option<Vec<&[u8]>> {
    split_function(expression).map(|(_, parameters)| parameters)
}

fn command_name(command: &[u8]) -> &[u8] {
    parser::command_name(command).unwrap_or_default()
}

fn unquote(value: &[u8]) -> &[u8] {
    let value = trim_ascii(value);
    if value.len() >= 2 && matches!(value[0], b'"' | b'\'') && value.last() == Some(&value[0]) {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn normalize_name(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}

fn find_assignment(value: &[u8]) -> Option<usize> {
    let mut depth = 0_i32;
    let mut quoted = false;
    for (position, byte) in value.iter().copied().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b'(' if !quoted => depth += 1,
            b')' if !quoted => depth -= 1,
            b'=' if !quoted && depth == 0 => {
                let previous = position.checked_sub(1).and_then(|index| value.get(index));
                let next = value.get(position + 1);
                if !matches!(previous, Some(b'=' | b'!' | b'<' | b'>')) && next != Some(&b'=') {
                    return Some(position);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_top_level(value: &[u8], needle: &[u8], reverse: bool) -> Option<usize> {
    let mut found = None;
    let mut depth = 0_i32;
    let mut quoted = false;
    let mut position = 0;
    while position + needle.len() <= value.len() {
        match value[position] {
            b'"' => quoted = !quoted,
            b'(' if !quoted => depth += 1,
            b')' if !quoted => depth -= 1,
            _ => {}
        }
        if !quoted && depth == 0 && &value[position..position + needle.len()] == needle {
            if !reverse {
                return Some(position);
            }
            found = Some(position);
        }
        position += 1;
    }
    found
}

fn find_top_level_chars_reverse(
    value: &[u8],
    operations: &[u8],
    allow_unary: bool,
) -> Option<usize> {
    let mut depth = 0_i32;
    let mut quoted = false;
    for position in (0..value.len()).rev() {
        let byte = value[position];
        match byte {
            b'"' => {
                quoted = !quoted;
                continue;
            }
            b')' if !quoted => {
                depth += 1;
                continue;
            }
            b'(' if !quoted => {
                depth -= 1;
                continue;
            }
            _ => {}
        }
        if quoted || depth != 0 || !operations.contains(&byte) {
            continue;
        }
        if allow_unary && matches!(byte, b'+' | b'-') {
            let previous = trim_ascii(&value[..position]).last().copied();
            if previous.is_none_or(|previous| b"(=+-*/%&|".contains(&previous)) {
                continue;
            }
        }
        return Some(position);
    }
    None
}

fn split_variable_reference(value: &[u8]) -> Option<(&[u8], Option<&[u8]>)> {
    let open = value.iter().position(|byte| *byte == b'[');
    match open {
        None => Some((value, None)),
        Some(open) if value.last() == Some(&b']') => Some((
            &value[..open],
            Some(trim_ascii(&value[open + 1..value.len() - 1])),
        )),
        Some(_) => None,
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp

// ============================================================================
// FUNCTION: stRunScript::stRunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.h:267
// RVA: 0x00024AB0
// ADDRESS: 00424ab0
// PROTOTYPE: undefined __thiscall stRunScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::ReleaseGeneralVariable` замкнут в `CGame::release` через owned `CVariableList`.

// ============================================================================
// FUNCTION: CScript::SetVariableList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:391
// RVA: 0x00024AF0
// ADDRESS: 00424af0
// PROTOTYPE: void __thiscall SetVariableList(CVariableList * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::UpdateVariableList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:399
// RVA: 0x00024B10
// ADDRESS: 00424b10
// PROTOTYPE: void __thiscall UpdateVariableList(CVariableList * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GotoNextLine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:410
// RVA: 0x00024B30
// ADDRESS: 00424b30
// PROTOTYPE: int __thiscall GotoNextLine(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::LoadScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:521
// RVA: 0x00024B90
// ADDRESS: 00424b90
// PROTOTYPE: bool __thiscall LoadScript(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::ReadCmd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:575
// RVA: 0x00024CB0
// ADDRESS: 00424cb0
// PROTOTYPE: bool __thiscall ReadCmd(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::IsOperation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1061
// RVA: 0x00024F00
// ADDRESS: 00424f00
// PROTOTYPE: bool __thiscall IsOperation(char param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::OperationNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1073
// RVA: 0x00024F30
// ADDRESS: 00424f30
// PROTOTYPE: int __thiscall OperationNum(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::Prew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1111
// RVA: 0x00024FA0
// ADDRESS: 00424fa0
// PROTOTYPE: int __thiscall Prew(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::PrewString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1124
// RVA: 0x00024FD0
// ADDRESS: 00424fd0
// PROTOTYPE: int __thiscall PrewString(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GetFunctionName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1521
// RVA: 0x00025000
// ADDRESS: 00425000
// PROTOTYPE: char * __thiscall GetFunctionName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::DumpString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2174
// RVA: 0x00025080
// ADDRESS: 00425080
// PROTOTYPE: int __cdecl DumpString(char * * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::UpdateToWorldServer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2192
// RVA: 0x000250E0
// ADDRESS: 004250e0
// PROTOTYPE: bool __cdecl UpdateToWorldServer(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::UpdateToWorldServer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2202
// RVA: 0x00025170
// ADDRESS: 00425170
// PROTOTYPE: bool __cdecl UpdateToWorldServer(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::DispatchCommand
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2213
// RVA: 0x00025200
// ADDRESS: 00425200
// PROTOTYPE: long __thiscall DispatchCommand(int param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::LoadGeneralVariable` замкнут в startup decoder-е owned `CVariableList`.

// ============================================================================
// FUNCTION: CScript::SetPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:384
// RVA: 0x000252B0
// ADDRESS: 004252b0
// PROTOTYPE: void __thiscall SetPlayer(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::JumpTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:433
// RVA: 0x000252D0
// ADDRESS: 004252d0
// PROTOTYPE: int __thiscall JumpTo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::JumpToNextBlock
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:467
// RVA: 0x00025420
// ADDRESS: 00425420
// PROTOTYPE: int __thiscall JumpToNextBlock(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::Count
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1139
// RVA: 0x000255E0
// ADDRESS: 004255e0
// PROTOTYPE: int __thiscall Count(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DelectAllScripByPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:96
// RVA: 0x00025FA0
// ADDRESS: 00425fa0
// PROTOTYPE: long __cdecl DelectAllScripByPlayer(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DelPlayerTalkBoxScrip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:116
// RVA: 0x00026090
// ADDRESS: 00426090
// PROTOTYPE: long __cdecl DelPlayerTalkBoxScrip(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
// IMPLEMENTED: player/path lookup и безопасное удаление текущего/чужих instances
// принадлежат `CGame::player_script_is_running` / `remove_player_scripts`;
// ID-based delete/continue замкнуты `delete_player_script` / `continue_player_script`.
// Состояние ожидания функции `0x16` принадлежит `ActiveScript`, а обратный
// отсчёт, дочерний запуск и закрытие при удалении — `CGame`.

// ============================================================================
// FUNCTION: CScript::~CScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:317
// RVA: 0x000264E0
// ADDRESS: 004264e0
// PROTOTYPE: void __thiscall ~CScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::CScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:268
// RVA: 0x00026620
// ADDRESS: 00426620
// PROTOTYPE: undefined __thiscall CScript(CPlayer * param_1, CRegion * param_2, CNpc * param_3, CGUID * param_4, ulong param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::Check
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:724
// RVA: 0x00026780
// ADDRESS: 00426780
// PROTOTYPE: void __thiscall Check(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::ComputeVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:915
// RVA: 0x00026D20
// ADDRESS: 00426d20
// PROTOTYPE: bool __thiscall ComputeVar(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::RunLine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:678
// RVA: 0x00027AA0
// ADDRESS: 00427aa0
// PROTOTYPE: void __thiscall RunLine(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GetIntParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1546
// RVA: 0x00027BF0
// ADDRESS: 00427bf0
// PROTOTYPE: int __thiscall GetIntParam(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::GetStringParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:1705
// RVA: 0x00027E80
// ADDRESS: 00427e80
// PROTOTYPE: char * __thiscall GetStringParam(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: RunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:29
// RVA: 0x00028840
// ADDRESS: 00428840
// PROTOTYPE: long __cdecl RunScript(stRunScript * param_1, char * param_2, tagPOINT * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::ReleaseFunction` замкнут в `CGame::release` через owned function registry.

// ============================================================================
// FUNCTION: CScript::RunStep
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:2030
// RVA: 0x00028D80
// ADDRESS: 00428d80
// PROTOTYPE: SCRIPTRETURN __thiscall RunStep(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ScriptLoop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp:63
// RVA: 0x00029000
// ADDRESS: 00429000
// PROTOTYPE: long __cdecl ScriptLoop(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CScript::LoadFunction` материализован выше как `CScriptFunctionRegistry::load`.

// ============================================================================
// FUNCTION: CVariableList::`scalar_deleting_destructor'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\script.cpp
// RVA: 0x000AE4E0
// ADDRESS: 004ae4e0
// PROTOTYPE: void * __thiscall `scalar_deleting_destructor'(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
