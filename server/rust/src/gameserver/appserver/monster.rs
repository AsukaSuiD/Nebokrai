//! Достигнутая часть свойств и жизненного цикла `CMonster`.
//! Runtime-модификаторы принадлежат одной базе CMoveShape::tagProperties
//! (+0x84, 25 signed DWORD по PDB). Общий UpdateProperty обнуляет их и
//! последовательно вызывает состояния; getter не повторяет этот обход.
//! GetMinAtk/MaxAtk 0x004E6620/0x004E66C0, GetDef 0x004E6780 и
//! GetElementResistant 0x004E6880 складывают ресурс и modifier до signed
//! нижней границы и pet-множителя. GetElementModify 0x004E6900 читает
//! только modifier +0xDC, без выдуманного resource-base. GetMaxHP 0x004E65A0
//! сохраняет unsigned wrapping-сумму без clamp. Прежние x87-преобразования
//! pet-факторов остаются у конкретных getters; готовые боевые свойства
//! больше не дорабатываются повторными копиями MonsterPropertyState.
//! Сведения об убийце принадлежат единственной базе CMoveShape: монстр
//! лишь делегирует чтение потребляемой OnDied-проекции type/ID/guild.
//! Native SetKilledMeAttackInfo (0x004CCE50) вызывается после пакета смерти 0xBF60B;
//! поля навыка и результата удара не выдаются за его сохранённый layout.
//! CPet vtable 0x00652D0C: OnFighting (+0x1C) указывает на CBaseAI
//! 0x004C9320. Завершение его атаки ставит ChangeSkill независимо от
//! сохранённого первичного AI. У стационарных лучников SearchEnemy следует
//! после базового ChangeSkill, у StupidArcher заменяет его. Последовательность
//! вычисляется по каноническому binding, а не хранится отдельным action-полем.
//! CBaseAI::OnFighting 0x004C9320 проверяет IsEnded до AI навыка. End
//! сохраняет terminated kernel до следующего active-прохода: только тот
//! ставит completion FIFO и снимает Attack, читая часы каждого события.
//! Установщик cast сохраняет переданную owner-ом фазу: MonsterThorn Begin
//! (0x005416E0) и широкие атаки (0x00531520) не выполняют первый AI.
//! Общая постановка Attack не подменяет этот переход; прежний helper
//! оставляет Check для ещё не перенесённых владельцев.
//! Завершение immediate-cast без reuse не читает часы перед этим FIFO:
//! `CSkill::End(0)` (0x004D84C0) пропускает timeGetTime. Отсутствие часов
//! представлено явно, а не отдельным флагом рядом с неиспользуемым timestamp.
//! OnFighting без GetCurrentSkill не исполняет старый kernel и не ставит
//! базовый ChangeSkill; производный SearchEnemy учитывается независимо.
//! GetCurrentSkill (0x004C9338) определяет и IsEnded, и последующий AI:
//! завершённый cast другого skill ID не завершает выбранный навык. Если
//! выбранный навык не имеет достигнутого исполнения, IsEnded читается из
//! постоянной базы зарегистрированного экземпляра, без создания kernel.
//! CPet::OnAttackingSchedule (0x004E9A20) и OnStayingSchedule (0x004E9650)
//! переходят от Tracing/диапазона к Begin без таймера GetAttackSpeed.
//! Общая точка допуска атаки не проверяет и не обновляет ai_schedule питомца;
//! проверка восстановления конкретного навыка остаётся у его владельца.
//! OnMoving выбирает ровно одного AI-владельца: CPet ищет цель лишь живым
//! и без GetCurrentSkill, не наследуя добавочные SearchEnemy первичного AI.
//! Назначение/сброс цели используют тот же is_tamed (флаг и identity игрока),
//! что и координатор; один оставшийся флаг не переключает действие CPet.
//! Kernel и типизированный ресурс cast принадлежат MonsterSkillExecution
//! конкретного MoveShapeSkill; reuse хранится в том же зарегистрированном
//! экземпляре. GetSkill выбирает первый элемент native-категории, поэтому
//! повторяемые ID не объединяются общей картой. Смена AI и завершение одного
//! навыка сохраняют исполнения остальных. Begin не получает дополнительного
//! сброса ресурсов; жизнь призванного существа остаётся у CMonster.
//! StopAllSkills перед приручением обходит все зарегистрированные экземпляры,
//! не только текущий cast. Общая синхронная граница смерти публикует настоящий
//! регион, а последующий OnDied допускается пассивной очередью выбранного AI.
//! OnBeenHurted (0x004E6EF0, appserver/monster.cpp:920) вызывается только
//! после нелетального BF60A. CGame сначала выполняет Nation-уведомление для
//! прямого игрока, затем разрешает player ID/хозяина приручённого монстра/0.
//! Защита первого удара читает часы для проверки только при ненулевом первом
//! ID, а для принятой записи читает их заново; чужой защищённый удар не
//! обновляет таймер. Цели-pet/carriage не исключаются этим callback.
//! Успех приручения назначает tamed/master после StopAll и снимает цель только
//! auxiliary CPet: прежний primary AI, его команды и hate не очищаются.
//! Повторного IsTamable после увеличения счётчика попыток нет; AddPet и
//! UpgradePetLevel следуют за назначением master в owner-е MonsterTaming.
//! Общий registered End теперь обращается к тому же MonsterSkillExecution:
//! фаза и подтверждённые флаги полёта снимаются без удаления payload,
//! затем owned пути очищаются в порядке SkillOwner::end_policy. Каталог
//! вариантов прогресса задаёт и доступ, и эти hooks; отдельного списка ID нет.
//! Source/target и lifecycle сохраняются до общего base End, выбранный ID
//! и AI-очереди сами эти hooks не меняют. Destination у SpiderMist и
//! YunShengLightning не обнуляется вместе с derived-полётными флагами:
//! End 0x0057B810 пишет только +0x4C/+0x50/+0x54/+0x58 перед GetUser,
//! а координаты базового CState очищаются уже общим хвостом.
//! Native constructor создаёт уже IsEnded-навык без Begin: Inactive хранит
//! единственный SkillLifecycle, а настоящий Begin переносит эту же базу в
//! обязательный kernel Monster-варианта. Source и разрешённый target получают
//! реальные region/type/ID; null-target сохраняет прежнюю сторону базы.
//! Время берётся из достигнутого Begin, без дополнительного чтения часов.
//! Cure снимает только derived-payload с сохранением базы, не имитируя End.
//! Обычное завершение cast/немедленного навыка не стирает выбранный ID:
//! CBaseAI::OnFighting (0x004C9320) лишь ставит ChangeSkill после IsEnded.
//! Выбор меняется отдельным обработчиком очереди; освобождение исполнения
//! и фиксация reuse не подменяют этот переход.
//! Stiffen после завершения Attack выбирает базовый default-навык через
//! CMoveShape::GetDefaultAttackSkillID (0x004CE240). Выбранный ID отделён
//! от исполнения: этот переход не вызывает Begin и не создаёт cast.
//! OnStiffen (0x004C880D) вызывает virtual OnLoseTarget после снятия Attack
//! и до выбора default. Координатор освобождает заимствование CMonster на
//! этой границе, чтобы производный AI мог обратиться к региону и близнецу.
//! CBaseAttack/CMonsterBaseAttack::End (0x005B3010) через 0x005DFBD0 вызывает
//! CSkill::End (0x004D84C0): любой ненулевой аргумент, включая Stiffen=4,
//! обновляет reuse даже у уже завершённого навыка. OnLoseTarget видит прежний
//! выбранный навык; default назначается только после возврата из callback.
//! CMonsterRangeAttack/CMonsterThorn::End (0x00546090) дополнительно вызывает
//! SetMoveable(true) перед общей очисткой и reuse. Этот callback обслуживает
//! обычный End, отмену и Stiffen. У RangeAttack non-player CheckCastCondition
//! (0x00511DE7) пропускает SetMoveable(false), поэтому счётчик может стать
//! отрицательным; искусственная парная блокировка при Begin не добавляется.
//! CMonsterFastAttack/CLordFastAttack::End (0x00512B50) используют тот же
//! SetMoveable(true) и общий End после сброса двухударных флагов. Их ID входят
//! в единую политику завершения; очистку прогресса выполняет существующий
//! MonsterSkillExecution, без отдельных ветвей для отмены и Stiffen.
//! Fury/BossBlueFury/BossBlueQuake разделяют End 0x00546090: их cast также
//! снимает один запрет движения. Это не End наложенных Fury/Cure/Quake-state:
//! их контейнеры, сроки и собственные блокировки остаются у state-владельцев.
//! MachineryStomp/LordWiderangingAttack также используют End 0x00546090.
//! Семейный wide-arc owner оставляет отдельный End(0) до создания cast;
//! завершение существующего исполнения проходит через эту общую политику.
//! SpiderMist::End (0x00540890) вызывает SetMoveable(true) и CSummonSkill::End,
//! обновляющий reuse при ненулевом аргументе. Общая очистка снимает регистрацию
//! curable-навыка; созданная CSpiderMistPhalanx ей не принадлежит и сохраняется.
//! ChuckStone/SkeletonArchery::End (0x0056A330) сбрасывает полётные поля,
//! снимает один запрет движения и вызывает общий End. Прогресс полёта хранится
//! в MonsterSkillExecution; отмена и Stiffen используют ту же очистку.
//! EnergyBolt/SnakeBolt/ZombieClaw::End (0x0053BF50) освобождает массив пути
//! и полётные поля, затем вызывает SetMoveable(true) и общий End. Owned Vec
//! пути освобождается с ресурсом экземпляра; нанесённые попадания и состояния
//! целей не являются ресурсами этого исполнения и при End не откатываются.
//! SpiderPoison/SpriteBurn/CorpsePtomaine/Promotion/KnockOut разделяют
//! End 0x00546090. Общая политика завершает их cast, не снимая наложенные
//! poison/burn/control-состояния; отдельные player-only эффекты сюда не входят.
//! Их runtime читает часы reuse через общий End после очистки cast и возврата
//! движения (`CSkill::End`, 0x004D84C0), отдельно от времени попадания,
//! создания/перезапуска состояния и его публикации. Отказ без reuse часов
//! завершения не читает; сроки наложенных состояний этим End не меняются.
//! SpiderWeb::End (0x0057B810) сбрасывает поля полёта и вызывает
//! SetMoveable(true) перед общим End. SpiderWebProgress сбрасывается в том же экземпляре,
//! а SpiderWebState на цели остаётся у собственного state-owner-а.
//! YunShengLightning разделяет End 0x0057B810: общий путь снимает один запрет
//! движения и очищает полёт, без повторного удара или сообщения эффекта.
//! CorpseCandleBlasting/SporeBlasting разделяют End 0x00582810: снятие
//! запрета движения принадлежит End, а взрыв, удаление источника и сообщение
//! смерти — только AI. Прерывание не исполняет самоубийственный эффект.
//! SummonCorpseCandle/SummonSkeleton/SummonSpore/BossFiendSummon разделяют
//! End 0x005AE7A0: SetMoveable(true), затем CSummonSkill::End. Завершение
//! cast не затрагивает уже созданных существ и не выполняет новый призыв.
//! Archery/BaseMagic/SnowStorm используют тот же End 0x005AE7A0;
//! их самостоятельные phalanx не удаляются вместе с исполнением навыка.
//! YakshaSlash разделяет End 0x0057B810. BossFiendPenetrate::End
//! (0x0052B410) освобождает путь и список поражённых целей до возврата движения;
//! оба завершают CAttackSkill без повторного урона и без пакета End.
//! Ресурсы совпавшего active-cast освобождаются до конкретных побочных
//! эффектов End: в частности, пути EnergyBolt/SnakeBolt/ZombieClaw
//! (0x0053BF50) — до SetMoveable(true). Stiffen чужого текущего навыка
//! не очищает сохранённое исполнение другого owner-а.
//! Неизвестный concrete End не заменяется cancel_base_attack_cast: Stiffen
//! оставляет его Attack и ресурсы нетронутыми до подключения owner-а, вместо
//! фиктивного IsEnded с потерей цели. Это незавершённая ветвь реконструкции.
//! OnStiffen разрешает GetCurrentSkill (0x004C87C8), не сохранённый dispatch.
//! Отсутствующий навык проходит без End/OnLoseTarget; подтверждённый End
//! выбранного навыка вызывается и без kernel. Очистка cast затрагивает только
//! совпадающий ID. Неизвестный End без своего исполнения не считается
//! завершённым и не снимает Attack; его concrete owner остаётся восстановить.
//! Немедленные State/WuXing/Swordship разделяют End (0x005AFA40), который
//! не снимает наложенное состояние. Stiffen=4 отмечает общий навык ended и
//! обновляет reuse независимо от обычного End(0) некоторых AI; фон увидит
//! этот флаг при следующем проходе, без преждевременного удаления его FIFO.
//! CPassiveGladiator::OnBeenHurted (0x00610E40) ставит SearchEnemy после
//! base-handler каждого Defense, до pop в ProcessPassiveAction. Реакции
//! не откладываются на конец пачки: следующий Defense очищает новый
//! SearchEnemy, если перед ним нет сохраняемой границы Attack/Move.
//! У CPet слот OnBeenHurted (+0x34 таблицы 0x00652D0C) указывает на
//! CBaseAI::OnBeenHurted (0x004C8700): приручение исключает добавочный
//! SearchEnemy сохранённого CPassiveGladiator, но сохраняет базовую очистку.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `server/gameserver/appserver/monster.h/.cpp` подтверждают
//! наследование от `CMoveShape`, тип `600`, начальные HP `1`, индекс обновления
//! `-1`, нулевые признаки лидера и десять множителей `1.0`.
//! `CBaseObject::CreateObject(600,id)` записывает ID после конструктора
//! производного класса.
//!
//! Старый `m_pBaseProperty` указывал внутрь общего `CMonsterList` и
//! перепривязывался после селектора `0x02`. Rust хранит точный байтовый ключ
//! исходного имени и разрешает текущий `MonsterProperties` у `CGame`, устраняя
//! висячий указатель и сохраняя обновление свойств. Снимок создания — имя,
//! графика, HP и скорость — остаётся в объекте, как в `AddMonster`.
//! `InitAI` и `GetAI` материализованы типизированным binding-ом из
//! `ai/aifactory.rs`; `InitSkills` использует канонические `CSkillFactory` и
//! `CMoveShape`.
//! `CMonster::GetAI` (0x004E6D80) выбирает единственного текущего владельца:
//! primary, auxiliary CPet и auxiliary CCarriage сохраняют независимые CBaseAI.
//! Три базовых состояния хранятся стандартным массивом с единым индексом
//! owner-а, без копирования target/FIFO/dormancy при смене master identity.
//! Нулевой GetAI представлен None, а не запасным первичным контроллером.
//! Прямые m_pPetAI-команды из pet-list адресуют свой слот независимо от GetAI.
//! Если первичный тип тоже AI12, его action и master-таймеры также остаются
//! независимыми от auxiliary; переключение GetAI не переносит их состояние.
//! Очереди, цель и сон разрешаются одним селектором при постановке событий и
//! их обработке. Повозка наследует `WhenBeenHurted` (0x005DCE00) с Defense и
//! Stiffen и `WhenBeenKilled` (0x004C9460) с Died; события не остаются в
//! неисполняемой FIFO первичного AI. Её `OnMoving` (0x0047B150) возвращает 1
//! без SearchEnemy, а `OnBeenHurted` (0x004C8700) не наследует реакцию
//! сохранённого CPassiveGladiator.
//! InitAI (0x004E6E10) пересоздаёт все AI и их derived-состояния: очередь,
//! target, dormancy, schedule и lifecycle начинаются с constructor defaults.
//! Это не Clear: WarSoul тоже заменяется. Skill End/OnLoseTarget не вызываются,
//! ресурсы/текущий навык CMoveShape и identity хозяина остаются на месте.
//! Сохранённый cast сам по себе не выбирает фазу: OnFighting исполняет его
//! только при достигнутом Attack выбранного AI и совпадении с GetCurrentSkill.
//! Пустая новая FIFO после смены GetAI по-прежнему допускает Schedule/Idle;
//! ресурсный End обращается к самому cast независимо от этих AI-проверок.
//! GameSave игрока проверяет наличие auxiliary m_pCarriageAI напрямую
//! (CPlayer::AddToByteArray, 0x00441291), не тип текущего GetAI.
//! Auto-start очередь при первом AI-проходе исполняет
//! подтверждённые monster-ветви `TaiJi`/`Origin` и три состояния увеличения
//! `601..603`, а также четыре `Swordship`. Пять `WuXing` завершаются без
//! эффекта по исходному player-only gate; неизвестные state ID сохраняются.
//! Сериализация и полный автономный ИИ остаются ниже в исходном материале.
//! Достигнутая цепочка базовой атаки
//! хранит канонические
//! HP, защиту первого нападающего, снимок смертельной атаки и цель боевого ИИ;
//! `CGame` координирует урон и смерть, `Nation/GodsBattle`, награду, добычу,
//! сценарий и окончательное удаление из региона. `Defense` и `Died` проходят
//! через каноническую очередь `passive_actions` владельца `CBaseAI`, причём
//! достигнутые `Defense` и вероятностный `Stiffen` обрабатываются до активного
//! хода монстра; stun сохраняет движение, но прерывает атаку и цель.
//! Специализированный ИИ синего босса с ID `21` хранит здесь восемь
//! одноразовых HP-порогов ярости как часть жизненного цикла конкретного
//! монстра; выбор навыка остаётся в `ai/bossblue.rs`.
//! ИИ демона-босса с ID `23` аналогично хранит восемь порогов призыва и
//! исходный таймер повторного призыва, инициализируемый вместе с созданием
//! конкретного монстра; выбор принадлежит `ai/bossfiend.rs`.
//!
//! Для обычного монстра со списком навыков `0x2bd`, `0x2d1`, `0x2ef`,
//! `0x197`, `0x191`, `0x198`, `0x199`, `0x19a`, `0x19b`, `0x19c`, `0x19d`,
//! `0x19e`, `0x19f`, `0x1a0`, `0x1a1`, `0x1a2`, `0x1a3`, `0x1a4` и `0x1a5`
//! тот же владелец
//! хранит цель, выбранный по исходным `odds` навык, выполнение и задержку
//! повторного применения каждого установленного навыка отдельно; timestamp
//! расписания `CMonsterAI` остаётся независимым. Быстрая атака дополнительно хранит визуальную фазу
//! и первый из двух ударов; `0x19d/0x1a1` используют единое состояние полёта
//! прямого снаряда, `0x1a0/0x1a2` — общий пошаговый путь с разной шириной,
//! `0x1a3` — накапливаемые состояния ярости, а `0x1a4` — длительный линейный
//! путь поражения. `0x1a5` использует тот же пошаговый полёт с собственной
//! начальной позицией и областью уровня 3. Поиск игроков и питомцев,
//! преследование ИИ `0/3` и задержка обходного шага остаются здесь; урон и
//! смерть питомца сохраняют приоритет цели и связь с хозяином. Пассивная либо
//! командная цель питомца доходит через масштабированную атаку до смерти дикого
//! монстра. Расписание питомца хранит ежесекундную проверку хозяина,
//! шестичасовой счётчик возраста и срок одичания, а `CGame` координирует поиск
//! целей активного режима, возврат, уведомления и исчезновение. Случайное
//! перемещение без цели, специальные сторожевые ИИ и списки с ещё не
//! достигнутыми навыками этим не подменяются. Призванный монстр хранит срок в
//! том же владельце и исчезает
//! до обычного хода ИИ, а создание сразу публикует его в `MonsterWorld` без
//! параллельного контейнера. Для достигнутого обычного
//! пути бездействия `CBaseAI` хранит время начала и интервал сна; `CGame`
//! восстанавливает HP и рассылает `OnChangeStates` до возврата ID в список
//! активных объектов области. Владелец повозки хранит режимы следования и
//! ожидания, ежесекундную привязку к хозяину и срок ожидания недействительного
//! хозяина; `CGame` исполняет движение, уведомления, отвязку и пакет удаления.
//! Восстановление питомца при входе и управление клиентом используют
//! принадлежащие владельцу `tagMasterInfo`, признак и ход приручения, раздельные
//! множители опыта и свойств `Globe`, достигнутое повышение уровня от опыта
//! последователя с `0xC0203` и узкое состояние управления питомцем. Асинхронное
//! дерево решений `CPet` этим не подменяется.
//! Достигнутое приручение хранит исходный счётчик попыток в том же владельце:
//! проверка выполняется до увеличения, а установка признака — после него,
//! включая исходную недостижимость успеха на последней разрешённой попытке.
//! `Talk` формирует принадлежащий монстру пакет `0xBF801` и сохраняет строгий
//! прямоугольный предел `AREA_WIDTH/AREA_HEIGHT`; обход игроков и доставка
//! остаются у регионального runtime-владельца.
//! City/country guard refresh восстанавливает HP, очищает существующий
//! `CBaseAI` и оставляет формирование `0xBF60F` координирующему `CGame`.
//! Property-формулы боевых свойств и property-пакеты перенесены в Zone
//! `combat/monsterformula.rs`; методы ниже делегируют туда без изменения
//! сигнатур, включая квоту/поправку опыта и pet attack/speed/timing.
//! Скалярная база монстра — tame/master предикаты, счётчик попыток
//! приручения, pet-progress колонки, бит-хранилище factors, записи
//! script/refresh, защита первого нападающего и Nation-допуск — перенесена
//! второй порцией в Zone `regions/monster.rs`; методы ниже делегируют туда
//! без изменения сигнатур, нематериальные accessor-ы полей остаются здесь.
//! Pet-факторы применяются только при валидной player-owner связи;
//! целочисленные результаты сохраняют x87 truncation. Некоммутативные
//! attack/element property-state обходятся в общем byte-exact порядке
//! `m_vStates`, включая повторяемые Fury и BattleFairy.
//! Два направления virtual `IsAttackAble` разведены явно: этот owner
//! проверяет monster-target относительно player/monster attacker-а, а
//! обратную player-target политику хранит `CPlayer` и координирует `CGame`.
//! Таблица квоты опыта индексируется числом живых участников, а оба результата
//! усекаются к нулю после x87-порядка операций. Состав живой группы,
//! поэтапные масштабы игрока/региона и выдачу координирует `CGame`.
//! Там же разрешается `GetBeneficiary`: при непригодности прямого кандидата
//! используется первый участник его типизированного командного сеанса в том
//! же регионе и в исходном порядке списка подключений.
//! Для ненулевой figure унаследованный `CSkill::GetTargetPath` выбирает
//! ближайшую клетку footprint, а не центральную tile-позицию монстра.

use super::ai::aifactory::{ActiveMonsterAi, MonsterAiBinding, MonsterAiKind};
use super::ai::baseai::{
    AiPhaseState, AiShapeAction, CBaseAI, PassiveDeathAction, PassiveStiffenAction,
};
use super::ai::bossblue::BossBlueAiState;
use super::ai::bossfiend::BossFiendAiState;
use super::ai::carriage::{
    CarriageLifecycleState, CarriageMasterFacts, CarriageMasterOutcome,
};
use super::ai::guardtarget::GuardStationState;
use super::ai::jiumai::JiuMaiAiState;
use super::ai::monsterai::{MonsterAiScheduleState, accepts_hurt_target};
use super::ai::passivegladiator::PassiveGladiatorState;
use super::ai::pet::{PetBehaviorState, PetLifecycleFacts, PetLifecycleOutcome};
use super::ai::smartgladiator::SmartGladiatorState;
use super::masterinfo::MasterInfo;
use super::summonedcreature::{SummonedCreatureLifecycle, SummonedCreatureTick};
use super::moveshape::{CMoveShape, KillingAttackIdentity, MoveShapePositionFacts};
use super::shape::{SHAPE_CHANGE_DELETE, ShapeFigure, ShapeIdentity, ShapeView};
use super::skills::kernel::{SkillStage, SkillTermination};
use super::skills::spidermist::SPIDER_MIST_SKILL_ID;
use super::skills::skillfactory::CSkillFactory;
use crate::nets::netserver::message::CMessage;
use crate::setup::monsterlist::MonsterProperties;
use nebokrai_zone::combat::monsterformula;
use nebokrai_zone::regions::monster;
pub(crate) use nebokrai_zone::combat::monsterformula::{
    MonsterCombatProperties, MonsterExperienceFormula, PetAttackProperties, PetExperienceUpdate,
};

const MONSTER_TYPE: i32 = 600;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CMonster {
    move_shape: CMoveShape,
    original_name: Vec<u8>,
    base_property_key: Option<Vec<u8>>,
    script_file: Vec<u8>,
    hit_points: u32,
    live_time: i32,
    refresh_index: i32,
    sign: u16,
    leader_sign: u16,
    leader_distance: u16,
    leader_type: i32,
    leader_id: i32,
    died_remove: bool,
    factors: [u32; 10],
    master_info: MasterInfo,
    tamed: bool,
    tame_attempt_count: u32,
    pet_level: u32,
    pet_experience: u32,
    pet_behavior: PetBehaviorState,
    carriage_lifecycle: CarriageLifecycleState,
    primary_carriage_lifecycle: CarriageLifecycleState,
    first_attack_player_id: i32,
    last_attack_timer_ms: u32,
    summoned_creature: Option<SummonedCreatureLifecycle>,
    ai_schedule: MonsterAiScheduleState,
    base_attack_owned_tick: bool,
    boss_blue_ai: BossBlueAiState,
    boss_fiend_ai: Option<BossFiendAiState>,
    passive_gladiator_ai: Option<PassiveGladiatorState>,
    smart_gladiator_ai: Option<SmartGladiatorState>,
    guard_station_ai: Option<GuardStationState>,
    jiu_mai_ai: Option<JiuMaiAiState>,
    ai_binding: Option<MonsterAiBinding>,
    base_ai: [CBaseAI; 3],
}

// Тип dispatch активного cast-а монстра перенесён в Zone
// `ai/monsterai.rs` (кластер A1 Monster 0x19x); alias ядра исполнения и все
// hub-методы lifecycle сохраняют прежний контракт.
// Enum-каталог прогресса, обёртка kernel+progress и их End-hooks перенесены
// в Zone `skills/execution/monster.rs` (волна Z-M4); re-export ниже сохраняет
// прежние имена, потребители типов здесь и в `moveshape.rs` не правятся.
pub(crate) use nebokrai_zone::ai::monsterai::MonsterBaseAttackDispatch;
pub(crate) use nebokrai_zone::skills::execution::{
    MonsterBaseAttackCast, MonsterSkillExecution, MonsterSkillProgress,
};
pub(crate) use nebokrai_zone::skills::execution::MonsterSkillProgressAccess as MonsterSkillProgressState;

impl CMonster {
    pub(crate) fn with_constructor_defaults() -> Self {
        let mut move_shape = CMoveShape::default();
        move_shape
            .shape_mut()
            .base_object_mut()
            .set_type(MONSTER_TYPE);
        Self {
            move_shape,
            original_name: Vec::new(),
            base_property_key: None,
            script_file: Vec::new(),
            hit_points: 1,
            live_time: -1,
            refresh_index: -1,
            sign: 0,
            leader_sign: 0,
            leader_distance: 0,
            leader_type: 0,
            leader_id: 0,
            died_remove: false,
            factors: [1.0f32.to_bits(); 10],
            master_info: MasterInfo::default(),
            tamed: false,
            tame_attempt_count: 0,
            pet_level: 0,
            pet_experience: 0,
            pet_behavior: PetBehaviorState::default(),
            carriage_lifecycle: CarriageLifecycleState::default(),
            primary_carriage_lifecycle: CarriageLifecycleState::default(),
            first_attack_player_id: 0,
            last_attack_timer_ms: 0,
            summoned_creature: None,
            ai_schedule: MonsterAiScheduleState::default(),
            base_attack_owned_tick: false,
            boss_blue_ai: BossBlueAiState::default(),
            boss_fiend_ai: None,
            passive_gladiator_ai: None,
            smart_gladiator_ai: None,
            guard_station_ai: None,
            jiu_mai_ai: None,
            ai_binding: None,
            base_ai: Default::default(),
        }
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        &self.move_shape
    }

    pub(crate) const fn move_shape_mut(&mut self) -> &mut CMoveShape {
        &mut self.move_shape
    }

    pub(crate) const fn master_info(&self) -> MasterInfo {
        self.master_info
    }

    pub(crate) const fn set_master_info(&mut self, master_info: MasterInfo) {
        self.master_info = master_info;
    }

    /// Точный fresh-monster `AddToByteArray` tail поверх client-prefix
    /// `CMoveShape` — Zone `regions::monster`; `master_name` уже разрешён
    /// владельцем `CGame`: обычный spawn передаёт локализованный `GS0119`,
    /// pet/carriage — имя игрока.
    pub(crate) fn encode_fresh_client_snapshot(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
    ) -> Option<Vec<u8>> {
        monster::encode_fresh_monster_client_snapshot(&self.client_snapshot_parts(property, master_name))
    }

    pub(crate) fn encode_client_snapshot(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        monster::encode_monster_client_snapshot(
            &self.client_snapshot_parts(property, master_name),
            timed_state_now_milliseconds,
        )
    }

    fn client_snapshot_parts<'a>(
        &'a self,
        property: &'a MonsterProperties,
        master_name: &'a [u8],
    ) -> monster::MonsterClientSnapshotParts<'a> {
        monster::MonsterClientSnapshotParts {
            move_shape: &self.move_shape,
            hit_points: self.hit_points,
            maximum_hp: self.maximum_hp(property),
            tamed: self.tamed,
            master_info: self.master_info,
            pet_level: self.pet_level,
            pet_experience: self.pet_experience,
            property,
            master_name,
        }
    }

    /// Формирует exact `CServerRegion::AddMonster` envelope `0xBF502` для
    /// только что созданного monster. Выбор around-получателей остаётся у
    /// owning region/CGame и не дублируется в объекте.
    pub(crate) fn build_fresh_enter_message(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
    ) -> Option<CMessage> {
        monster::build_fresh_monster_enter_message(&self.client_snapshot_parts(property, master_name))
    }

    pub(crate) fn build_enter_message(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<CMessage> {
        monster::build_monster_enter_message(
            &self.client_snapshot_parts(property, master_name),
            timed_state_now_milliseconds,
        )
    }

    pub(crate) const fn set_summoned_creature_lifecycle(
        &mut self,
        lifecycle: Option<SummonedCreatureLifecycle>,
    ) {
        self.summoned_creature = lifecycle;
    }

    pub(crate) const fn is_summoned_creature(&self) -> bool {
        self.summoned_creature.is_some()
    }

    pub(crate) fn tick_summoned_creature(&self, now_ms: u32) -> Option<SummonedCreatureTick> {
        self.summoned_creature
            .map(|lifecycle| lifecycle.tick(now_ms, CMoveShape::is_died(self.hit_points)))
    }

    pub(crate) const fn set_tamed(&mut self, tamed: bool) {
        monster::set_tamed(&mut self.tamed, tamed);
    }

    /// Точный `CMonster::DoesCreatureBeenTamed` (RVA `0x000E6460`): одного
    /// внутреннего знака недостаточно, требуется живой identity хозяина-игрока.
    pub(crate) const fn is_tamed(&self) -> bool {
        self.has_player_pet_master()
    }

    pub(crate) const fn is_tamable(&self, property: &MonsterProperties) -> bool {
        monster::is_tamable(self.tame_attempt_count, property)
    }

    pub(crate) const fn increase_tame_attempt_count(&mut self) {
        monster::increase_tame_attempt_count(&mut self.tame_attempt_count);
    }

    pub(crate) fn try_become_tamed(
        &mut self,
        master: MasterInfo,
        pet_mode: i32,
    ) -> bool {
        if self.is_tamed() {
            return false;
        }
        self.tamed = true;
        self.master_info = master;
        if self.has_pet_ai() {
            self.set_pet_mode(pet_mode);
            self.release_pet_ai_target();
        }
        true
    }

    /// Соответствует `dynamic_cast<CCarriage *>(GetAI())`: до назначения
    /// player-master учитывается первичный AI12, после назначения — отдельный
    /// auxiliary `CCarriage`, созданный для tamable-свойства с нулём попыток.
    pub(crate) fn is_carriage(&self, _property: &MonsterProperties) -> bool {
        matches!(
            self.active_ai(),
            Some(
                ActiveMonsterAi::Carriage
                    | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)
            )
        )
    }

    pub(crate) const fn active_ai(&self) -> Option<ActiveMonsterAi> {
        match self.ai_binding {
            Some(binding) => binding.active(self.master_info),
            None => None,
        }
    }

    pub(crate) const fn selected_base_ai(&self) -> Option<&CBaseAI> {
        match self.active_ai() {
            Some(active) => Some(&self.base_ai[active.storage_index()]),
            None => None,
        }
    }

    /// Номер первичного owner-а допустим для virtual dispatch только пока
    /// GetAI не выбрал отдельный pet/carriage-контроллер.
    pub(crate) const fn active_primary_ai_type(&self) -> Option<u32> {
        if matches!(self.active_ai(), Some(ActiveMonsterAi::Primary(_))) {
            match self.ai_binding {
                Some(binding) => Some(binding.ai_type()),
                None => None,
            }
        } else {
            None
        }
    }

    pub(crate) const fn selected_base_ai_mut(&mut self) -> Option<&mut CBaseAI> {
        match self.active_ai() {
            Some(active) => Some(&mut self.base_ai[active.storage_index()]),
            None => None,
        }
    }

    /// AutoStart выбирает AI на границе входа в область. Последующая смена
    /// хозяина не переносит уже зарегистрированный фон в другой слот.
    pub(crate) fn auto_start_passive_skills(&mut self) -> usize {
        let Some(active) = self.active_ai() else { return 0; };
        self.move_shape.auto_start_passive_skills(&mut self.base_ai[active.storage_index()])
    }

    pub(crate) fn prepare_back_stage_skill_pass(&mut self) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.prepare_back_stage_skill_pass();
        }
    }

    /// Отложенный AutoStart передаёт Begin тому же экземпляру навыка до AI.
    pub(crate) fn begin_pending_back_stage_skill_ids(&mut self) -> Vec<u32> {
        self.selected_base_ai_mut().map(CBaseAI::begin_pending_back_stage_skill_ids)
            .unwrap_or_default()
    }

    pub(crate) fn back_stage_skill_id(&self, index: usize) -> Option<u32> {
        self.selected_base_ai()?.back_stage_skill_id(index)
    }

    pub(crate) fn mark_ended_back_stage_skill(&mut self, index: usize, expected: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.mark_ended_back_stage_skill(index, expected);
        }
    }

    const fn pet_base_ai(&self) -> Option<&CBaseAI> {
        match self.ai_binding {
            Some(binding) if binding.has_pet() => Some(&self.base_ai[ActiveMonsterAi::Pet.storage_index()]),
            _ => None,
        }
    }

    fn pet_base_ai_mut(&mut self) -> Option<&mut CBaseAI> {
        if self.has_pet_ai() {
            Some(&mut self.base_ai[ActiveMonsterAi::Pet.storage_index()])
        } else {
            None
        }
    }

    const fn pet_ai_target(&self) -> Option<ShapeIdentity> {
        match self.pet_base_ai() {
            Some(ai) if ai.has_object_target() => ai.object_target(),
            _ => None,
        }
    }

    pub(crate) fn has_pet_ai(&self) -> bool {
        self.ai_binding.is_some_and(MonsterAiBinding::has_pet)
    }

    pub(crate) fn has_carriage_ai(&self) -> bool {
        self.ai_binding.is_some_and(MonsterAiBinding::has_carriage)
    }

    /// Exact `CMonster::InitSkills`: базовая защита добавляется первой,
    /// затем исходный бессодержательный `random(skill_count)` расходует RNG,
    /// после чего property skills проходят в wire-порядке. `odds` на этой
    /// границе не читается; fallback base attack в точном owner-е отсутствует.
    pub(crate) fn initialize_skills(
        &mut self,
        property: &MonsterProperties,
        factory: &CSkillFactory,
        random: &mut impl FnMut(i32) -> i32,
    ) {
        self.move_shape.clear_skills(factory);
        self.move_shape.add_base_defense_skill(factory);
        let _discarded_roll = random(property.skills.len() as i32);
        for skill in &property.skills {
            let _loaded =
                self.move_shape
                    .add_skill(u32::from(skill.id), i32::from(skill.level), factory);
        }
    }

    pub(crate) const fn carriage_action(&self) -> i32 {
        self.selected_carriage_lifecycle().action()
    }

    pub(crate) const fn set_carriage_action(&mut self, action: i32) {
        self.selected_carriage_lifecycle_mut().set_action(action);
    }

    const fn selected_carriage_lifecycle(&self) -> &CarriageLifecycleState {
        if matches!(self.active_ai(), Some(ActiveMonsterAi::Primary(MonsterAiKind::Carriage))) {
            &self.primary_carriage_lifecycle
        } else {
            &self.carriage_lifecycle
        }
    }

    const fn selected_carriage_lifecycle_mut(&mut self) -> &mut CarriageLifecycleState {
        if matches!(self.active_ai(), Some(ActiveMonsterAi::Primary(MonsterAiKind::Carriage))) {
            &mut self.primary_carriage_lifecycle
        } else {
            &mut self.carriage_lifecycle
        }
    }

    pub(crate) fn begin_carriage_master_check(
        &mut self,
        facts: CarriageMasterFacts,
        now: impl FnMut() -> u32,
        rebind: impl FnMut(),
    ) -> CarriageMasterOutcome {
        self.selected_carriage_lifecycle_mut().begin_master_check(facts, now, rebind)
    }

    pub(crate) fn carriage_master_timed_out(
        &mut self,
        facts: CarriageMasterFacts,
        now: impl FnMut() -> u32,
    ) -> bool {
        self.selected_carriage_lifecycle_mut().master_timed_out(facts, now)
    }

    pub(crate) const fn is_owned_pet(&self, player_id: i32) -> bool {
        monster::is_owned_pet(self.master_info, player_id)
    }

    pub(crate) const fn set_pet_progress(&mut self, level: u32, experience: u32) {
        monster::set_pet_progress(&mut self.pet_level, &mut self.pet_experience, level, experience);
    }

    pub(crate) const fn pet_progress(&self) -> (u32, u32) {
        monster::pet_progress(self.pet_level, self.pet_experience)
    }

    pub(crate) fn increase_pet_experience(
        &mut self,
        experience: u32,
        property: &MonsterProperties,
        experience_factor: f32,
        current_factors: Option<[f32; 10]>,
        next_factors: Option<[f32; 10]>,
    ) -> Option<PetExperienceUpdate> {
        self.pet_experience = self.pet_experience.wrapping_add(experience);
        if self.pet_level < 10 {
            let threshold = self.maximum_hp(property) as f32 * experience_factor;
            if self.pet_experience as f32 <= threshold {
                if self.pet_level == 0 && self.pet_experience == 0 {
                    if let Some(factors) = current_factors {
                        self.adjust_pet_factors(factors);
                    }
                }
            } else if self.pet_level + 1 < 10 {
                self.pet_level += 1;
                self.pet_experience = 0;
                if let Some(factors) = next_factors {
                    self.adjust_pet_factors(factors);
                }
                self.hit_points = self.maximum_hp(property);
            }
        }
        (experience != 0).then(|| PetExperienceUpdate {
            level: self.pet_level,
            experience: self.pet_experience,
            maximum_hp: self.maximum_hp(property),
            hit_points: self.hit_points,
        })
    }

    pub(crate) fn adjust_pet_factors(&mut self, factors: [f32; 10]) {
        monster::adjust_pet_factors(&mut self.factors, factors);
    }

    pub(crate) fn maximum_hp(&self, property: &MonsterProperties) -> u32 {
        monsterformula::maximum_hp(
            property,
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    pub(crate) fn roll_stiffen(
        &mut self,
        damage: u32,
        property: &MonsterProperties,
        setup: crate::setup::globesetup::GlobeStiffenSetup,
        now_ms: impl FnMut() -> u32,
        random: impl FnMut(i32) -> i32,
    ) -> u32 {
        let maximum_hp = self.maximum_hp(property);
        self.move_shape.stiffen(
            damage as u16,
            maximum_hp,
            property.re_ank as u16,
            setup,
            now_ms,
            random,
        )
    }

    pub(crate) fn set_pet_mode(&mut self, mode: i32) {
        if !self.has_pet_ai() {
            return;
        }
        if self.pet_behavior.mode_change_releases_target(mode) {
            self.release_pet_ai_target();
        }
        self.pet_behavior.set_mode(mode);
    }

    pub(crate) const fn pet_mode(&self) -> i32 {
        self.pet_behavior.mode()
    }

    pub(crate) fn set_pet_action(&mut self, action: i32) {
        if !self.has_pet_ai() {
            return;
        }
        if action == 1 && self.pet_ai_target().is_some() {
            self.release_pet_ai_target();
        }
        self.pet_behavior.set_action(action);
    }

    pub(crate) const fn pet_action(&self) -> i32 {
        self.pet_behavior.action()
    }

    /// Состояние точного `CPet::OnSchedule`; поиск хозяина, региона и навыка,
    /// а также наблюдаемые сетевые эффекты и удаление остаются у `CGame`.
    pub(crate) fn tick_pet_lifecycle(&mut self, facts: PetLifecycleFacts) -> PetLifecycleOutcome {
        if !self.has_pet_ai() {
            return PetLifecycleOutcome::default();
        }
        let outcome = self.pet_behavior.tick(facts, self.pet_ai_target().is_some());
        if outcome.clear_target {
            self.release_pet_ai_target();
        }
        outcome
    }

    pub(crate) fn set_pet_target(&mut self, target: ShapeIdentity) {
        if !self.has_pet_ai() {
            return;
        }
        self.pet_behavior.begin_target();
        if let Some(ai) = self.pet_base_ai_mut() {
            ai.set_object_target(target);
        }
    }

    pub(crate) fn retarget_passive_pet(&mut self, target: ShapeIdentity) -> bool {
        if !self.has_pet_ai() {
            return false;
        }
        if !self
            .pet_behavior
            .retarget_passive(self.tamed, self.pet_ai_target().is_some())
        {
            return false;
        }
        if let Some(ai) = self.pet_base_ai_mut() {
            ai.set_object_target(target);
        }
        true
    }

    pub(crate) fn evanish_pet(&mut self) {
        self.stage_for_delete();
    }

    /// Назначает exact поля, которые `AddMonster` пишет до virtual `Init`.
    pub(crate) fn bind_spawn_property(&mut self, property: &MonsterProperties) {
        let shape = self.move_shape.shape_mut();
        shape.base_object_mut().set_name(&property.name);
        shape
            .base_object_mut()
            .set_graphics_id(property.picture_id as i32);
        self.original_name = property.original_name.clone();
        self.base_property_key = Some(property.original_name.clone());
        self.hit_points = property.maximum_hp;
    }

    /// Скорость исходный spawn назначает только после `Init` и позиции.
    pub(crate) fn set_spawn_speed(&mut self, property: &MonsterProperties) {
        self.move_shape
            .shape_mut()
            .set_speed(property.move_speed as f32);
    }

    pub(crate) fn base_property_key(&self) -> Option<&[u8]> {
        self.base_property_key.as_deref()
    }

    pub(crate) fn original_name(&self) -> &[u8] {
        &self.original_name
    }

    pub(crate) fn script_file(&self) -> &[u8] {
        &self.script_file
    }

    pub(crate) fn display_name(&self) -> &[u8] {
        monster::display_name(self.move_shape.shape().base_object().get_name(), &self.original_name)
    }

    /// Формирует точный кадр `CMonster::Talk`; завершающие нули строк остаются
    /// частью `CBaseMessage::Add(char const*)` wire-контракта.
    pub(crate) fn build_talk_message(&self, text: &[u8]) -> CMessage {
        let identity = self.move_shape.shape().identity();
        let mut message = CMessage::new(0x000b_f801);
        message.add_long(0);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add(self.display_name());
        message.add_byte(0);
        message.base_mut().add(text);
        message.add_byte(0);
        message
    }

    /// Сохраняет две независимые строгие проверки расстояния из `Talk`.
    pub(crate) fn talk_reaches(
        &self,
        target_x: i32,
        target_y: i32,
        area_width: i32,
        area_height: i32,
    ) -> bool {
        let shape = self.move_shape.shape();
        let (Ok(source_x), Ok(source_y)) = (shape.get_tile_x(), shape.get_tile_y()) else {
            return false;
        };
        i64::from(target_x).abs_diff(i64::from(source_x)) < area_width as u64
            && i64::from(target_y).abs_diff(i64::from(source_y)) < area_height as u64
    }

    pub(crate) const fn hit_points(&self) -> u32 {
        self.hit_points
    }

    pub(crate) const fn set_hit_points(&mut self, hit_points: u32) {
        self.hit_points = hit_points;
    }

    /// Выполняет достигнутый `SetHP(GetMaxHP) -> GetAI()->Clear()` war guard-
    /// путь. `CBaseAI::Clear` сбрасывает собственную цель напрямую и не
    /// запускает расширенную потерю цели питомца или отмену текущего skill.
    pub(crate) fn refresh_war_guard(&mut self, maximum_hp: u32) {
        self.hit_points = maximum_hp;
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.clear_guard_refresh_state();
        }
    }

    pub(crate) fn hibernate_ai(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.hibernate(now_ms);
        }
    }

    pub(crate) const fn is_ai_hibernated(&self) -> bool {
        match self.selected_base_ai() {
            Some(ai) => ai.is_hibernated(),
            None => false,
        }
    }

    /// `CMonsterAI::WakeUp` сначала завершает сон, затем восстанавливает
    /// HP с точной DWORD-арифметикой и вызывает `OnChangeStates`, если до
    /// пробуждения HP отличался от максимума. Сам круговой пакет шлёт `CGame`.
    pub(crate) fn wake_ai(
        &mut self,
        now_ms: u32,
        resume_timer_ms: u32,
        property: &MonsterProperties,
    ) -> bool {
        let Some(ai) = self.selected_base_ai_mut() else {
            return false;
        };
        let dormancy_interval_ms = ai.wake_up(now_ms);
        let maximum_hp = self.maximum_hp(property);
        let publish_states = self.hit_points != maximum_hp;
        if publish_states {
            if resume_timer_ms == 0 {
                tracing::warn!(
                    monster_id = self.move_shape.shape().identity().id,
                    "нулевой интервал восстановления монстра не допускает деление"
                );
            } else {
                let recovery_steps = dormancy_interval_ms / resume_timer_ms;
                let recovery_speed = u32::from(self.hp_recovery_speed(property));
                self.hit_points = self
                    .hit_points
                    .wrapping_add(recovery_speed.wrapping_mul(recovery_steps))
                    .min(maximum_hp);
            }
        }
        if matches!(
            self.active_ai(),
            Some(ActiveMonsterAi::Primary(MonsterAiKind::BossBlue))
        ) {
            self.boss_blue_ai.wake(self.hit_points, maximum_hp);
        }
        publish_states
    }

    pub(crate) fn boss_blue_ai_mut(&mut self) -> &mut BossBlueAiState {
        &mut self.boss_blue_ai
    }

    /// Материализует полный `CMonster::InitAI`: фабричный выбор первичного,
    /// pet- и carriage-владельцев выполняется до инициализации их concrete
    /// состояния. Повторный вызов заменяет прежний binding, как delete/create
    /// в исходном owner-е.
    pub(crate) fn initialize_ai(&mut self, property: &MonsterProperties, now_ms: u32) {
        let binding = MonsterAiBinding::create(property, self.tame_attempt_count);
        let primary = binding.primary();
        self.ai_binding = Some(binding);
        self.base_ai = Default::default();
        self.ai_schedule = MonsterAiScheduleState::default();
        self.pet_behavior = PetBehaviorState::default();
        self.carriage_lifecycle = CarriageLifecycleState::default();
        self.primary_carriage_lifecycle = CarriageLifecycleState::default();
        self.boss_blue_ai = BossBlueAiState::default();
        self.boss_fiend_ai =
            matches!(primary, MonsterAiKind::BossFiend).then(|| BossFiendAiState::new(now_ms));
        self.passive_gladiator_ai =
            matches!(primary, MonsterAiKind::PassiveGladiator).then(PassiveGladiatorState::default);
        self.smart_gladiator_ai =
            matches!(primary, MonsterAiKind::SmartGladiator).then(SmartGladiatorState::default);
        self.guard_station_ai = primary.has_guard_station().then(GuardStationState::default);
        self.jiu_mai_ai = matches!(primary, MonsterAiKind::JiuMai).then(JiuMaiAiState::default);
    }

    pub(crate) fn boss_fiend_ai_mut(&mut self) -> Option<&mut BossFiendAiState> {
        self.boss_fiend_ai.as_mut()
    }

    pub(crate) const fn boss_fiend_ai(&self) -> Option<&BossFiendAiState> {
        self.boss_fiend_ai.as_ref()
    }

    pub(crate) fn passive_gladiator_ai_mut(&mut self) -> Option<&mut PassiveGladiatorState> {
        self.passive_gladiator_ai.as_mut()
    }

    pub(crate) fn take_passive_gladiator_ai(&mut self) -> Option<PassiveGladiatorState> {
        self.passive_gladiator_ai.take()
    }

    pub(crate) fn restore_passive_gladiator_ai(&mut self, state: PassiveGladiatorState) {
        self.passive_gladiator_ai = Some(state);
    }

    pub(crate) fn smart_gladiator_ai_mut(&mut self) -> Option<&mut SmartGladiatorState> {
        self.smart_gladiator_ai.as_mut()
    }

    pub(crate) fn smart_gladiator_ai(&self) -> Option<&SmartGladiatorState> {
        self.smart_gladiator_ai.as_ref()
    }

    pub(crate) fn guard_station_ai_mut(&mut self) -> Option<&mut GuardStationState> {
        self.guard_station_ai.as_mut()
    }

    pub(crate) const fn jiu_mai_ai(&self) -> Option<&JiuMaiAiState> {
        self.jiu_mai_ai.as_ref()
    }

    pub(crate) fn jiu_mai_ai_mut(&mut self) -> Option<&mut JiuMaiAiState> {
        self.jiu_mai_ai.as_mut()
    }

    pub(crate) fn combat_properties(
        &self,
        property: &MonsterProperties,
    ) -> MonsterCombatProperties {
        MonsterCombatProperties {
            level: property.level as u8,
            defense: self.defense(property),
            dodge: u32::from(self.dodge(property)),
            element_resistance: self.element_resistance(property),
            soul_resistance: self.soul_resistance(property),
            attack_avoid: self.attack_avoid(property),
            element_avoid: self.element_avoid(property),
            promotion_magic_attack_factor: self.move_shape.promotion_magic_attack_factor(),
        }
    }

    pub(crate) fn dodge(&self, property: &MonsterProperties) -> u16 {
        monsterformula::dodge(
            property,
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    pub(crate) fn hit(&self, property: &MonsterProperties) -> u16 {
        monsterformula::hit(property, self.move_shape.property_modifiers())
    }

    pub(crate) fn attack_avoid(&self, property: &MonsterProperties) -> u16 {
        monsterformula::attack_avoid(property, self.move_shape.property_modifiers())
    }

    pub(crate) fn element_avoid(&self, property: &MonsterProperties) -> u16 {
        monsterformula::element_avoid(property, self.move_shape.property_modifiers())
    }

    pub(crate) fn defense(&self, property: &MonsterProperties) -> u32 {
        monsterformula::defense(
            property,
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    pub(crate) fn element_resistance(&self, property: &MonsterProperties) -> u32 {
        monsterformula::element_resistance(
            property,
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    pub(crate) fn soul_resistance(&self, property: &MonsterProperties) -> u16 {
        monsterformula::soul_resistance(property, self.move_shape.property_modifiers())
    }

    pub(crate) fn hp_recovery_speed(&self, property: &MonsterProperties) -> u16 {
        monsterformula::hp_recovery_speed(property, self.move_shape.property_modifiers())
    }

    pub(crate) fn soul_attack(&self, property: &MonsterProperties) -> u16 {
        monsterformula::soul_attack(property, self.move_shape.property_modifiers())
    }

    pub(crate) fn stop_frame(&self, property: &MonsterProperties) -> u32 {
        monsterformula::stop_frame(
            property,
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    const fn has_player_pet_master(&self) -> bool {
        monster::has_player_pet_master(self.tamed, self.master_info)
    }

    pub(crate) fn attack_interval(&self, property: &MonsterProperties) -> u32 {
        monsterformula::attack_interval(
            property,
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    pub(crate) fn speed(&self) -> f32 {
        monsterformula::speed(
            self.move_shape.shape().get_speed(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    pub(crate) fn pet_attack_properties(
        &self,
        property: &MonsterProperties,
    ) -> PetAttackProperties {
        monsterformula::pet_attack_properties(
            property,
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
            self.move_shape.shape().get_speed(),
        )
    }

    pub(crate) fn state_attack_bounds(
        &self,
        minimum: u32,
        maximum: u32,
    ) -> (u32, u32) {
        monsterformula::state_attack_bounds(
            minimum,
            maximum,
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    pub(crate) fn element_modifier(&self) -> u32 {
        monsterformula::element_modifier(
            self.move_shape.property_modifiers(),
            self.has_player_pet_master(),
            self.factors.map(f32::from_bits),
        )
    }

    /// Точная завершающая часть владельца защиты `CMonster::OnBeenHurted`.
    /// Уведомление Nation остаётся в `CGame` перед этой мутацией, как в EXE.
    pub(crate) fn register_attacking_player(
        &mut self,
        attacker_player_id: i32,
        protection_ms: u32,
        clock: impl FnMut() -> u32,
    ) -> bool {
        monster::register_attacking_player(
            &mut self.first_attack_player_id,
            &mut self.last_attack_timer_ms,
            attacker_player_id,
            protection_ms,
            clock,
        )
    }

    pub(crate) const fn first_attack_player_id(&self) -> i32 {
        self.first_attack_player_id
    }

    pub(crate) const fn refresh_index(&self) -> i32 {
        self.refresh_index
    }

    pub(crate) const fn killed_by(&self) -> Option<KillingAttackIdentity> {
        self.move_shape.killed_by()
    }

    pub(crate) fn when_been_hurted_by(
        &mut self,
        attacker: ShapeIdentity,
        attacker_is_tamed: bool,
        now_ms: u32,
    ) {
        let Some(ai) = self.selected_base_ai_mut() else { return; };
        ai.when_been_hurted(now_ms);
        let current_target = if ai.has_object_target() { ai.object_target() } else { None };
        if accepts_hurt_target(current_target, attacker, attacker_is_tamed) {
            ai.set_object_target(attacker);
        }
    }

    /// Общая часть `CBaseAI::WhenBeenHurted` без политики выбора цели
    /// конкретного производного ИИ.
    pub(crate) fn when_been_hurted(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.when_been_hurted(now_ms);
        }
    }

    pub(crate) fn when_been_stiffened(&mut self, delay_ms: u32, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.when_been_stiffened(delay_ms, now_ms);
        }
    }

    pub(crate) fn when_passive_gladiator_hurted_by(
        &mut self,
        attacker: ShapeIdentity,
        now_ms: u32,
        attacker_is_owned_creature: bool,
    ) {
        if self.selected_base_ai().is_none() { return; }
        self.when_been_hurted(now_ms);
        let already_fighting = self.ai_target().is_some();
        let selected = self.passive_gladiator_ai.as_mut().and_then(|state| {
            state.on_hurt(attacker, already_fighting, attacker_is_owned_creature)
        });
        if let Some(selected) = selected
            && let Some(ai) = self.selected_base_ai_mut()
        {
            ai.set_object_target(selected);
        }
    }

    pub(crate) fn when_pet_been_hurted_by(&mut self, attacker: ShapeIdentity, now_ms: u32) {
        let Some(ai) = self.pet_base_ai_mut() else {
            return;
        };
        ai.when_been_hurted(now_ms);
        let target = if ai.has_object_target() { ai.object_target() } else { None };
        if self.pet_behavior.on_hurt(target, attacker)
            && let Some(ai) = self.pet_base_ai_mut()
        {
            ai.set_object_target(attacker);
        }
    }

    pub(crate) fn when_been_killed(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.when_been_killed(now_ms);
        }
    }

    pub(crate) fn process_reached_defense_actions(
        &mut self,
        mut now_ms: impl FnMut() -> u32,
    ) -> usize {
        let passive_gladiator = matches!(
            self.active_ai(), Some(ActiveMonsterAi::Primary(MonsterAiKind::PassiveGladiator))
        ) && self.passive_gladiator_ai.is_some();
        let Some(ai) = self.selected_base_ai_mut() else { return 0; };
        ai.process_reached_defense_actions(|ai| {
            if passive_gladiator {
                ai.begin_active_search_enemy(now_ms());
            }
        })
    }

    pub(crate) fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction {
        self.selected_base_ai_mut().map_or(PassiveStiffenAction::None,
            CBaseAI::begin_reached_stiffen_action)
    }

    /// Очистка concrete End до доставки эффекта; Attack пока остаётся в FIFO.
    pub(crate) fn prepare_stiffen_attack(&mut self, factory: &CSkillFactory) -> Option<(bool, Option<u32>)> {
        let ai = self.selected_base_ai()?;
        if !ai.stiffen_attack_pending() {
            return None;
        }
        let current_skill = self.move_shape.current_skill(factory).map(|skill| skill.id());
        let release_target = ai.stiffen_attack_needs_end()
            && current_skill.is_some();
        let mut ended_skill = None;
        if release_target {
            let skill_id = current_skill.expect("разрешённый текущий навык Stiffen");
            if matches!(skill_id,
                super::skills::baseattack::BASE_ATTACK_SKILL_ID
                | super::skills::monsterbaseattack::MONSTER_BASE_ATTACK_SKILL_ID)
                || Self::attack_end_restores_movement(skill_id)
                || skill_id == super::skills::littlestar::LITTLE_STAR_SKILL_ID
            {
                self.clear_skill_progress(skill_id, factory);
                if skill_id == super::skills::littlestar::LITTLE_STAR_SKILL_ID {
                    self.prepare_little_star_end(factory);
                } else {
                    self.finish_attack_skill_resources(skill_id);
                }
                self.move_shape.finish_skill_base(skill_id, factory, SkillTermination::Completed);
                ended_skill = Some(skill_id);
            } else {
                return None;
            }
        } else {
            let default_skill = self.move_shape.default_attack_skill_id();
            self.move_shape.set_current_skill_id(Some(default_skill));
        }
        Some((release_target, ended_skill))
    }

    pub(crate) fn finish_stiffen_attack(&mut self, ended_skill: Option<u32>, factory: &CSkillFactory, now: impl FnOnce() -> u32) {
        if self.selected_base_ai().is_none() { return; }
        if let Some(skill_id) = ended_skill {
            self.move_shape.mark_skill_used(skill_id, now(), factory);
        }
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.finish_stiffen_attack(false);
        }
    }

    pub(crate) fn resume_stiffen_after_target_release(&mut self, released: bool) {
        if self.selected_base_ai().is_none() { return; }
        if released {
            let default_skill = self.move_shape.default_attack_skill_id();
            self.move_shape.set_current_skill_id(Some(default_skill));
        }
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.discard_active_prefix();
        }
    }

    pub(crate) fn finish_reached_stiffen_action(
        &mut self,
        action: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction {
        self.selected_base_ai_mut().map_or(PassiveStiffenAction::None,
            |ai| ai.finish_reached_stiffen_action(action, now))
    }

    pub(crate) fn begin_reached_death_action(&mut self) -> bool {
        self.selected_base_ai_mut().is_some_and(CBaseAI::begin_reached_death_action)
    }

    /// Общий `CMonsterAI::OnLoseTarget` смерти очищает только target-поля и
    /// не отменяет сохранённый `ASA_MOVE`; расширенный `clear_ai_target`
    /// намеренно остаётся для обычных schedule/interruption путей.
    pub(crate) fn release_ai_target_for_death(&mut self) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.lose_target();
        }
    }

    pub(crate) fn release_pet_ai_target(&mut self) {
        let Some(ai) = self.pet_base_ai_mut() else {
            return;
        };
        ai.lose_target();
        self.pet_behavior.target_cleared(true);
    }

    pub(crate) fn reached_death_action_state(&self) -> PassiveDeathAction {
        self.selected_base_ai().map_or(PassiveDeathAction::None,
            CBaseAI::reached_death_action_state)
    }

    pub(crate) fn finish_reached_death_action(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.finish_reached_death_action(now_ms);
        }
    }

    pub(crate) fn advance_active_ai_stand(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.selected_base_ai_mut().is_some_and(|ai| ai.advance_active_stand(now))
    }

    pub(crate) fn advance_handled_active_ai_action(&mut self, now: impl FnOnce() -> u32) -> Option<AiPhaseState> {
        self.selected_base_ai_mut()?.advance_handled_active_action(now)
    }

    pub(crate) fn advance_handled_passive_ai_action(&mut self, now: impl FnOnce() -> u32) -> Option<bool> {
        self.selected_base_ai_mut()?.advance_handled_passive_action(now)
    }

    pub(crate) fn active_ai_change_skill_pending(&self) -> bool {
        self.selected_base_ai().is_some_and(CBaseAI::active_change_skill_pending)
    }

    pub(crate) fn active_ai_search_enemy_pending(&self) -> bool {
        self.selected_base_ai().is_some_and(CBaseAI::active_search_enemy_pending)
    }

    pub(crate) fn active_ai_attack_pending(&self) -> bool {
        self.selected_base_ai().is_some_and(CBaseAI::active_attack_pending)
    }

    pub(crate) fn queue_search_after_active_move(&mut self, ai_type: u32, factory: &CSkillFactory, now: impl FnOnce() -> u32) {
        if matches!(self.active_ai(), Some(ActiveMonsterAi::Carriage
            | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)))
        {
            return;
        }
        if !self.selected_base_ai().is_some_and(CBaseAI::active_move_unhandled) {
            return;
        }
        let alive = !CMoveShape::is_died(self.hit_points);
        let passive_gladiator_search = ai_type == 1
            && alive
            && self
                .passive_gladiator_ai
                .as_ref()
                .is_some_and(PassiveGladiatorState::has_enemy_players);
        // `CGuardWithSword::OnMoving` RVA `0x0020E260` добавляет SearchEnemy
        // после успешного общего OnMoving и наследуется AI10/15/19; базовый
        // factory type AI9 обязан проходить тот же путь. Отдельный
        // `CGuardCountry::OnMoving` RVA `0x0020C4F0` делает то же для живых
        // factory-типов AI13/20, но не для самостоятельного AI14.
        // `CPassiveGladiator::OnMoving` RVA `0x00210E70` дополнительно требует
        // непустой `m_vEnemy`, которой соответствует owned IndexSet AI1.
        let search = if matches!(self.active_ai(), Some(ActiveMonsterAi::Pet)) {
            alive && self.move_shape.current_skill(factory).is_none()
        } else {
            (alive && matches!(ai_type, 4 | 13 | 20))
                || matches!(ai_type, 9 | 10 | 15 | 19)
                || passive_gladiator_search
        };
        if search && let Some(ai) = self.selected_base_ai_mut() {
            ai.begin_active_search_enemy(now());
        }
    }

    pub(crate) fn begin_active_ai_move(&mut self, delay_ms: u32, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.begin_active_move(delay_ms, now_ms);
        }
    }

    pub(crate) fn begin_active_ai_stand(&mut self, delay_ms: u32, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.begin_active_stand(delay_ms, now_ms);
        }
    }

    pub(crate) fn begin_active_ai_search_enemy(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.begin_active_search_enemy(now_ms);
        }
    }

    pub(crate) fn begin_active_ai_change_skill(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.add_ai_event(AiShapeAction::ChangeSkill, 0, 0, now_ms);
        }
    }

    pub(crate) fn advance_active_ai_move(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.selected_base_ai_mut().is_some_and(|ai| ai.advance_active_move(now))
    }

    pub(crate) fn active_ai_attack_ended(&self, factory: &CSkillFactory) -> bool {
        if self.selected_base_ai().is_none() { return false; }
        let Some(skill) = self.move_shape.current_skill(factory) else { return false };
        self.move_shape.skill_lifecycle(skill.id(), factory).is_some_and(|lifecycle| lifecycle.is_ended())
    }

    /// Проекция исполнения для OnFighting, не общий доступ к ресурсу CSkill.
    /// Наличие cast другого AI не превращает Schedule/Idle в вызов CSkill::AI.
    pub(crate) fn current_active_attack_cast(&self, factory: &CSkillFactory) -> Option<MonsterBaseAttackCast> {
        if !self.active_ai_attack_pending() { return None; }
        let skill_id = self.move_shape.current_skill(factory)?.id();
        self.base_attack_cast(skill_id, factory)
    }

    pub(crate) fn active_ai_attack_can_execute(&self, factory: &CSkillFactory) -> bool {
        if self.selected_base_ai().is_none() { return false; }
        if self.active_ai_attack_ended(factory) {
            return false;
        }
        self.current_active_attack_cast(factory).is_some()
    }

    pub(crate) fn finish_active_ai_attack(&mut self, factory: &CSkillFactory, mut now: impl FnMut() -> u32) -> bool {
        if self.selected_base_ai().is_none() { return false; }
        let has_skill = self.move_shape.current_skill(factory).is_some();
        let skill_ended = self.active_ai_attack_ended(factory);
        if has_skill && !skill_ended {
            return false;
        }
        let completion_ai_type = self.active_primary_ai_type().unwrap_or(0);
        let alive = !CMoveShape::is_died(self.hit_points);
        let Some(ai) = self.selected_base_ai_mut() else { return false; };
        for &action in crate::gameserver::appserver::ai::fixedpositionarcher::attack_completion_actions(
            completion_ai_type, alive, skill_ended,
        ) {
            ai.add_ai_event(action, 0, 0, now());
        }
        ai.finish_active_attack(now());
        true
    }

    pub(crate) fn finish_active_ai_change_skill(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.finish_active_change_skill(now_ms);
        }
    }

    pub(crate) fn finish_active_ai_search_enemy(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.finish_active_search_enemy(now_ms);
        }
    }

    pub(crate) fn primary_ai_queues_idle(&self) -> bool {
        self.selected_base_ai().is_some_and(CBaseAI::primary_queues_idle)
    }

    pub(crate) const fn ai_target(&self) -> Option<ShapeIdentity> {
        let Some(ai) = self.selected_base_ai() else { return None; };
        if !ai.has_object_target() {
            return None;
        }
        ai.object_target()
    }

    pub(crate) fn set_ai_target(&mut self, target: ShapeIdentity) {
        if self.selected_base_ai().is_none() { return; }
        if matches!(self.active_ai(), Some(ActiveMonsterAi::Pet)) {
            self.pet_behavior.begin_ai_target(true);
        }
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.set_object_target(target);
        }
    }

    pub(crate) fn base_attack_cast(&self, skill_id: u32, factory: &CSkillFactory) -> Option<MonsterBaseAttackCast> {
        Some(self.move_shape.monster_skill_execution(skill_id, factory)?.kernel)
    }

    pub(crate) fn begin_base_attack_cast(
        &mut self,
        target: ShapeIdentity,
        skill_id: u32,
        skill_level: u16,
        now_ms: u32,
        target_object: Option<(i32, ShapeIdentity)>,
        factory: &CSkillFactory,
    ) {
        let mut execution = MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
            target,
            skill_id,
            skill_level,
        }, now_ms);
        let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        self.install_base_attack_cast(execution, target_object, factory);
    }

    pub(crate) fn install_base_attack_cast(
        &mut self,
        execution: MonsterBaseAttackCast,
        target_object: Option<(i32, ShapeIdentity)>,
        factory: &CSkillFactory,
    ) {
        let now_ms = execution.started_at_ms();
        if self.install_base_attack_execution(execution, target_object, factory) {
            self.enqueue_base_attack_cast(now_ms);
        }
    }

    /// Регистрирует ресурс до проверки Begin, не ставя Attack в очередь.
    /// При отказе тот же экземпляр должен получить End(0).
    pub(crate) fn prepare_base_attack_cast(
        &mut self,
        target: ShapeIdentity,
        skill_id: u32,
        skill_level: u16,
        now_ms: u32,
        target_object: Option<(i32, ShapeIdentity)>,
        factory: &CSkillFactory,
    ) -> bool {
        let execution = MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
            target,
            skill_id,
            skill_level,
        }, now_ms);
        self.install_base_attack_execution(execution, target_object, factory)
    }

    fn install_base_attack_execution(
        &mut self,
        execution: MonsterBaseAttackCast,
        target_object: Option<(i32, ShapeIdentity)>,
        factory: &CSkillFactory,
    ) -> bool {
        if self.selected_base_ai().is_none() { return false; }
        let skill_id = execution.dispatch().skill_id;
        let now_ms = execution.started_at_ms();
        let source = self.move_shape.shape();
        let source_object = (source.get_region_id(), source.identity());
        let Some(lifecycle) = self.move_shape.skill_lifecycle_mut(skill_id, factory) else { return false; };
        lifecycle.begin_objects(Some(source_object), target_object, || now_ms);
        let _ = lifecycle.finish_begin(true);
        let progress = self.move_shape.monster_skill_execution_mut(skill_id, factory)
            .and_then(|stored| stored.progress.take());
        self.move_shape.install_monster_execution(MonsterSkillExecution {
            kernel: execution,
            progress,
        }, factory)
    }

    pub(crate) fn enqueue_base_attack_cast(&mut self, now_ms: u32) {
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.add_ai_event(AiShapeAction::Attack, 0, 0, now_ms);
        }
    }

    pub(crate) fn skill_progress<T: MonsterSkillProgressState>(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&T> {
        let progress = self.move_shape.monster_skill_execution(skill_id, factory)?.progress.as_ref()?;
        T::from_progress(progress)
    }

    pub(crate) fn skill_progress_mut<T: MonsterSkillProgressState>(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut T> {
        let progress = self.move_shape.monster_skill_execution_mut(skill_id, factory)?.progress.as_mut()?;
        T::from_progress_mut(progress)
    }

    pub(crate) fn set_skill_progress(
        &mut self,
        skill_id: u32,
        progress: impl Into<MonsterSkillProgress>,
        factory: &CSkillFactory,
    ) {
        if let Some(execution) = self.move_shape.monster_skill_execution_mut(skill_id, factory) {
            execution.progress = Some(progress.into());
        }
    }

    pub(crate) fn clear_skill_progress(&mut self, skill_id: u32, factory: &CSkillFactory) {
        if let Some(execution) = self.move_shape.monster_skill_execution_mut(skill_id, factory) {
            execution.progress = None;
        }
    }

    pub(crate) fn prepare_little_star_end(&mut self, factory: &CSkillFactory) {
        self.clear_skill_progress(super::skills::littlestar::LITTLE_STAR_SKILL_ID, factory);
        self.move_shape.set_moveable(true);
    }

    /// Общий `CSkill::End` (0x004D84C0) читает reuse после derived-cleanup.
    /// Готовое время AI/попадания сюда не передаётся: каждый owner предоставляет
    /// чтение runtime, которое вызывается только для живого завершения с reuse.
    pub(crate) fn finish_base_attack_cast_with_clock(&mut self, skill_id: u32, factory: &CSkillFactory, now: impl FnOnce() -> u32) -> Option<MonsterBaseAttackCast> {
        self.finish_base_attack_cast_with_reuse(skill_id, factory, Some(now))
    }

    /// `CSkill::End(false)` завершает самостоятельное AI-действие, но не
    /// запускает reuse навыка. Это отличается от внешней отмены cast-а:
    /// derived AI всё равно получает своё обычное completion action.
    pub(crate) fn finish_base_attack_cast_without_reuse(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<MonsterBaseAttackCast> {
        self.finish_base_attack_cast_with_reuse(skill_id, factory, None::<fn() -> u32>)
    }

    const fn attack_end_restores_movement(skill_id: u32) -> bool {
        matches!(skill_id,
            super::skills::monsterrangeattack::MONSTER_RANGE_ATTACK_SKILL_ID
            | super::skills::monsterthorn::MONSTER_THORN_SKILL_ID
            | super::skills::monsterfastattack::MONSTER_FAST_ATTACK_SKILL_ID
            | super::skills::lordfastattack::LORD_FAST_ATTACK_SKILL_ID
            | super::skills::fury::FURY_SKILL_ID
            | super::skills::bossbluefury::BOSS_BLUE_FURY_SKILL_ID
            | super::skills::bossbluequake::BOSS_BLUE_QUAKE_SKILL_ID
            | super::skills::machinerystomp::MACHINERY_STOMP_SKILL_ID
            | super::skills::lordwiderangingattack::LORD_WIDERANGING_ATTACK_SKILL_ID
            | SPIDER_MIST_SKILL_ID
            | super::skills::chuckstone::CHUCK_STONE_SKILL_ID
            | super::skills::skeletonarchery::SKELETON_ARCHERY_SKILL_ID
            | super::skills::energybolt::ENERGY_BOLT_SKILL_ID
            | super::skills::snakebolt::SNAKE_BOLT_SKILL_ID
            | super::skills::zombieclaw::ZOMBIE_CLAW_SKILL_ID
            | super::skills::spiderpoison::SPIDER_POISON_SKILL_ID
            | super::skills::spriteburn::SPRITE_BURN_SKILL_ID
            | super::skills::corpseptomaine::CORPSE_PTOMAINE_SKILL_ID
            | super::skills::promotion::PROMOTION_SKILL_ID
            | super::skills::knockoutruntime::KNOCK_OUT_SKILL_ID
            | super::skills::spiderweb::SPIDER_WEB_SKILL_ID
            | super::skills::yunshenglightning::YUNSHENG_LIGHTNING_SKILL_ID
            | super::skills::corpsecandleblasting::CORPSE_CANDLE_BLASTING_SKILL_ID
            | super::skills::sporeblasting::SPORE_BLASTING_SKILL_ID
            | super::skills::summoncorpsecandle::SUMMON_CORPSE_CANDLE_SKILL_ID
            | super::skills::summonskeleton::SUMMON_SKELETON_SKILL_ID
            | super::skills::summonspore::SUMMON_SPORE_SKILL_ID
            | super::skills::bossfiendsummon::BOSS_FIEND_SUMMON_SKILL_ID
            | super::skills::archery::ARCHERY_SKILL_ID
            | super::skills::basemagic::BASE_MAGIC_SKILL_ID
            | super::skills::snowstorm::SNOW_STORM_SKILL_ID
            | super::skills::bossfiendpenetrate::BOSS_FIEND_PENETRATE_SKILL_ID)
    }

    fn finish_attack_skill_resources(&mut self, skill_id: u32) {
        if Self::attack_end_restores_movement(skill_id) {
            self.move_shape.set_moveable(true);
        }
    }

    fn finish_base_attack_cast_with_reuse(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
        reuse_clock: Option<impl FnOnce() -> u32>,
    ) -> Option<MonsterBaseAttackCast> {
        if self.move_shape.monster_skill_execution(skill_id, factory)?.kernel.termination().is_some() {
            return None;
        }
        self.clear_skill_progress(skill_id, factory);
        self.finish_attack_skill_resources(skill_id);
        let slot = self.move_shape.skill_slot(skill_id, factory)?;
        let skill = self.move_shape.skill_at_mut(slot)?;
        skill.clear_base_end_context();
        if let Some(now) = reuse_clock {
            skill.mark_used(now());
        }
        skill.finish_cleared_base_end(SkillTermination::Completed);
        self.base_attack_cast(skill_id, factory)
    }

    pub(crate) fn advance_base_attack_cast(
        &mut self,
        skill_id: u32,
        expected: SkillStage,
        next: SkillStage,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.monster_skill_execution_mut(skill_id, factory)
            .is_some_and(|execution| execution.kernel.advance(expected, next))
    }

    pub(crate) fn clear_ai_target(&mut self, factory: &CSkillFactory) {
        let Some(ai) = self.selected_base_ai_mut() else { return; };
        ai.lose_target();
        self.cancel_base_attack_cast(factory);
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.cancel_active_move();
        }
        if matches!(self.active_ai(), Some(ActiveMonsterAi::Pet)) {
            self.pet_behavior.target_cleared(true);
        }
    }

    pub(crate) fn lose_ai_target_and_search(&mut self, now_ms: u32, factory: &CSkillFactory) {
        self.clear_ai_target(factory);
        if let Some(ai) = self.selected_base_ai_mut() {
            ai.begin_active_search_enemy(now_ms);
        }
    }

    pub(crate) fn cancel_base_attack_cast(&mut self, factory: &CSkillFactory) {
        let Some(skill_id) = self.move_shape.current_skill(factory).map(|skill| skill.id()) else { return; };
        self.clear_skill_progress(skill_id, factory);
        let Some(termination) = self.move_shape.monster_skill_execution(skill_id, factory)
            .map(|execution| execution.kernel.termination()) else { return; };
        if termination.is_none() {
            self.finish_attack_skill_resources(skill_id);
        }
        self.move_shape.set_current_skill_id(None);
        self.move_shape.finish_skill_base(skill_id, factory, termination.unwrap_or(SkillTermination::Cancelled));
    }

    pub(crate) fn skill_last_used_ms(&self, skill_id: u32, factory: &CSkillFactory) -> u32 {
        self.move_shape.skill_last_used_ms(skill_id, factory)
    }

    pub(crate) const fn begin_ai_attack_attempt(
        &mut self,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool {
        match self.active_ai() {
            Some(ActiveMonsterAi::Pet) => true,
            Some(ActiveMonsterAi::Primary(kind)) if !matches!(kind, MonsterAiKind::Carriage) => {
                self.ai_schedule.begin_attack_attempt(now_ms, interval_ms)
            }
            _ => false,
        }
    }

    pub(crate) fn begin_ai_attack_attempt_with_clock(
        &mut self,
        interval_ms: u32,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        match self.active_ai() {
            Some(ActiveMonsterAi::Pet) => true,
            Some(ActiveMonsterAi::Primary(kind)) if !matches!(kind, MonsterAiKind::Carriage) => {
                self.ai_schedule.begin_attack_attempt_with_clock(interval_ms, now)
            }
            _ => false,
        }
    }

    pub(crate) const fn set_base_attack_owned_tick(&mut self, owned: bool) {
        self.base_attack_owned_tick = owned;
    }

    pub(crate) fn take_base_attack_owned_tick(&mut self) -> bool {
        std::mem::take(&mut self.base_attack_owned_tick)
    }

    pub(crate) const fn movement_position_facts(
        &self,
        figure: ShapeFigure,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        MoveShapePositionFacts {
            current_hit_points: self.hit_points,
            figure,
            current_area: None,
            area_width,
            area_height,
        }
    }

    /// Guards reached from `CMonster::OnBeenHurted` before Nation first-hit
    /// dispatch: action `ACT_DIED` and health-based death are independent.
    pub(crate) fn can_trigger_nation_damage(&self) -> bool {
        monster::can_trigger_nation_damage(self.move_shape.shape().get_action(), self.hit_points)
    }

    /// Exact `OnClearWar` predicate использует ту же пару virtual action/HP,
    /// но остаётся отдельным gameplay-контрактом phase cleanup.
    pub(crate) fn can_clear_from_nation_war(&self) -> bool {
        self.move_shape.shape().get_action() != 6 && !CMoveShape::is_died(self.hit_points)
    }

    pub(crate) const fn staged_for_delete(&self) -> bool {
        self.move_shape.shape().change_state() == SHAPE_CHANGE_DELETE
    }

    pub(crate) fn stage_for_delete(&mut self) {
        self.move_shape
            .shape_mut()
            .set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn set_script_file(&mut self, script_file: &[u8]) {
        monster::set_script_file(&mut self.script_file, script_file);
    }

    pub(crate) const fn set_refresh_data(
        &mut self,
        sign: u16,
        leader_sign: u16,
        leader_distance: u16,
        refresh_index: i32,
    ) {
        monster::set_refresh_data(
            &mut self.sign,
            &mut self.leader_sign,
            &mut self.leader_distance,
            &mut self.refresh_index,
            monster::MonsterRefreshData {
                sign,
                leader_sign,
                leader_distance,
                refresh_index,
            },
        );
    }

    pub(crate) fn figure(property: &MonsterProperties) -> ShapeFigure {
        let figure = property.figure as u8;
        ShapeFigure::from_directions([figure; 4])
    }

    pub(crate) fn shape_view(&self, property: &MonsterProperties) -> Option<ShapeView> {
        let shape = self.move_shape.shape();
        Some(ShapeView {
            identity: shape.identity(),
            tile_x: shape.get_tile_x().ok()?,
            tile_y: shape.get_tile_y().ok()?,
            pos_x_bits: shape.get_pos_x().to_bits(),
            pos_y_bits: shape.get_pos_y().to_bits(),
            figure: Self::figure(property),
        })
    }

    /// Exact `CMonster::GetBeAttackedPoint` (RVA `0x000E6AA0`). Нулевая
    /// figure сохраняет базовую центральную точку `CMoveShape`.
    pub(crate) fn be_attacked_point(
        &self,
        property: &MonsterProperties,
        attacker_x: i32,
        attacker_y: i32,
    ) -> Option<(i32, i32)> {
        let view = self.shape_view(property)?;
        if property.figure as u8 == 0 {
            return Some((view.tile_x, view.tile_y));
        }
        Some(CMoveShape::nearest_figure_attack_point(
            view.tile_x,
            view.tile_y,
            view.figure,
            attacker_x,
            attacker_y,
        ))
    }
}
