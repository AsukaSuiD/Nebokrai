#pragma once

#include "../acclogqueue.h"
#include "../../dbaccess/myadobase.h"

#include <string>

/*
 * Исходный владелец: loginserver/applogin/acclogthread.cpp / acclogthread.h.
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Подтверждённые RVA: конструктор 0x00420C30, деструктор 0x00420C90,
 * Run 0x00420D30.
 * PDB size 12 = старый Thread base + CMyAdoBase base.
 *
 * Thread/COM — только обвязка Windows. Владелец Linux сохраняет тот же
 * блокирующий Run, но не наследует отсутствующий Win32 Thread; AccLogQueue
 * передаётся ссылкой вместо прямого глобального поиска GetGame()->_acc_logs.
 *
 * Исходный Run делал Pop в обнулённый char[2048], и пустая строка завершала
 * рабочий поток. Текущий nullopt сохраняет ту же границу. Для каждой непустой
 * записи создаётся НОВОЕ подключение, затем OpenCn -> запись соответствующего
 * account/role/leave-события -> CloseCn -> ReleaseCn. Ошибка Create/Open/Execute в EXE превращалась в
 * _com_error, обработчик делал PrintErr("Acc Log Err"), ReleaseCn и продолжал
 * со СЛЕДУЮЩЕЙ записью очереди. Повтор той же записи и ожидание отсутствуют.
 *
 * Побочный эффект PrintErr передаётся общему техническому журналу `Nebokrai` и
 * не влияет на жизненный цикл записи и очереди.
 */
namespace Login
{
class AccLogThread : public CMyAdoBase
{
public:
    explicit AccLogThread(AccLogQueue& logs) noexcept;

    AccLogThread(const AccLogThread&) = delete;
    AccLogThread& operator=(const AccLogThread&) = delete;

    void Run();

private:
    void RecordError(std::string detail);

    AccLogQueue& m_Logs;
};
}
