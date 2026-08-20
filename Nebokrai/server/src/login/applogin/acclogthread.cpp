#include "acclogthread.h"

#include <spdlog/spdlog.h>

#include <utility>

namespace Login
{
namespace
{
constexpr std::size_t kLegacySqlBufferSize = 0x800U;
constexpr char kAccLogErrorLabel[] = "Ошибка журнала входа";
}

AccLogThread::AccLogThread(AccLogQueue& logs) noexcept : m_Logs(logs) {}

void AccLogThread::Run()
{
    // Прямые вызовы CoInitialize/CoUninitialize из EXE опущены: CMyAdoBase
    // теперь использует unixODBC и не имеет состояния COM apartment.
    CMyAdoBase::Connection connection;

    for (;;) {
        std::optional<AccLogRecord> record = m_Logs.Pop();

        // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x00420D98..0x00420DB0: strlen==0 —
        // единственное штатное условие выхода, включая устаревший токен
        // семафора после clear().
        if (!record) {
            CMyAdoBase::ReleaseCn(connection);
            return;
        }

        // Старый вызывающий код копировал данные в обнулённый char[2048] без
        // проверки длины. Слишком большой ввод считается явной технической
        // границей вместо выхода за стек; извлечённый SQL не возвращается в
        // очередь, поскольку прямые пути исключений тоже переходили к следующему.
        constexpr std::size_t kLegacySqlLiteralSize =
            sizeof("INSERT INTO LogInfo(Account,AccountEnterTime,IP) VALUES('','','')") - 1U;
        const std::size_t legacySqlSize = kLegacySqlLiteralSize +
                                          record->account.size() +
                                          record->enteredAt.size() +
                                          record->ip.size();
        if (legacySqlSize >= kLegacySqlBufferSize) {
            RecordError("SQL не помещается в старый буфер AccLogThread char[2048]");
            CMyAdoBase::ReleaseCn(connection);
            continue;
        }

        if (!CMyAdoBase::CreateCn(connection)) {
            RecordError(connection.lastError);
            CMyAdoBase::ReleaseCn(connection);
            continue;
        }
        if (!CMyAdoBase::OpenCn(connection)) {
            RecordError(connection.lastError);
            CMyAdoBase::ReleaseCn(connection);
            continue;
        }
        if (!CMyAdoBase::ExecuteAccountEnterLog(record->account,
                                                record->enteredAt,
                                                record->ip,
                                                connection)) {
            RecordError(connection.lastError);
            CMyAdoBase::ReleaseCn(connection);
            continue;
        }

        // Run игнорирует возвращаемый bool. Ошибка закрытия ADO попадала бы в тот
        // же обработчик «Acc Log Err»; unixODBC вместо этого сообщает её явно.
        if (!CMyAdoBase::CloseCn(connection)) {
            RecordError(connection.lastError);
        }
        CMyAdoBase::ReleaseCn(connection);
    }
}

void AccLogThread::RecordError(std::string detail)
{
    spdlog::error("{}: {}", kAccLogErrorLabel, detail);
}
}
