#pragma once

#include "../acclogqueue.h"
#include "../../dbaccess/myadobase.h"

#include <deque>
#include <mutex>
#include <optional>
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
 * Run: Pop в обнулённый char[2048]; пустая строка завершает рабочий поток. Для
 * каждой непустой строки создаётся НОВОЕ подключение, затем OpenCn -> ExecuteCn ->
 * CloseCn -> ReleaseCn. Ошибка Create/Open/Execute в EXE превращалась в
 * _com_error, обработчик делал PrintErr("Acc Log Err"), ReleaseCn и продолжал
 * со СЛЕДУЮЩЕЙ записью очереди. Повтор той же SQL и ожидание отсутствуют.
 *
 * Пока общий владелец AddLogText не материализован, побочный эффект PrintErr
 * хранится только для наблюдаемости как AccLogThreadError. Он не влияет на
 * жизненный цикл SQL и очереди.
 */
namespace Login
{
struct AccLogThreadError
{
    std::string label;
    std::string detail;
};

class AccLogThread : public CMyAdoBase
{
public:
    explicit AccLogThread(AccLogQueue& logs) noexcept;

    AccLogThread(const AccLogThread&) = delete;
    AccLogThread& operator=(const AccLogThread&) = delete;

    void Run();

    [[nodiscard]] std::optional<AccLogThreadError> PopTechnicalError();

private:
    void RecordError(std::string detail);

    AccLogQueue& m_Logs;
    std::mutex m_ErrorMutex;
    std::deque<AccLogThreadError> m_Errors;
};
}
