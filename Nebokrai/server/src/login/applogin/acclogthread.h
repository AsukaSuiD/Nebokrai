#pragma once

#include "../acclogqueue.h"
#include "../../dbaccess/myadobase.h"

#include <deque>
#include <mutex>
#include <optional>
#include <string>

/*
 * Owner: loginserver/applogin/acclogthread.cpp / acclogthread.h.
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * RVA: ctor 0x00420C30, dtor 0x00420C90, Run 0x00420D30.
 * PDB size 12 = старый Thread base + CMyAdoBase base.
 *
 * Thread/COM — только Windows plumbing. Linux owner сохраняет тот же blocking
 * Run, но не наследует отсутствующий Win32 Thread; AccLogQueue передаётся
 * ссылкой вместо direct GetGame()->_acc_logs global lookup.
 *
 * Run: Pop в обнулённый char[2048]; пустая строка завершает worker. Для каждой
 * непустой строки создаётся НОВАЯ connection, затем OpenCn -> ExecuteCn ->
 * CloseCn -> ReleaseCn. Ошибка Create/Open/Execute в EXE превращалась в
 * _com_error, handler делал PrintErr("Acc Log Err"), ReleaseCn и продолжал со
 * СЛЕДУЮЩЕЙ queue-записью. Retry той же SQL и sleep отсутствуют.
 *
 * Пока общий AddLogText owner не материализован, PrintErr side effect хранится
 * как observability-only AccLogThreadError. Он не влияет на SQL/queue lifecycle.
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
