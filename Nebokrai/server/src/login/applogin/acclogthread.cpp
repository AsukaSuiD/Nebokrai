#include "acclogthread.h"

#include <utility>

namespace Login
{
namespace
{
constexpr std::size_t kLegacySqlBufferSize = 0x800U;
constexpr char kAccLogErrorLabel[] = "Acc Log Err";
}

AccLogThread::AccLogThread(AccLogQueue& logs) noexcept : m_Logs(logs) {}

void AccLogThread::Run()
{
    // Direct EXE CoInitialize/CoUninitialize are omitted: CMyAdoBase now uses
    // unixODBC and has no COM apartment state.
    CMyAdoBase::Connection connection;

    for (;;) {
        std::string sql = m_Logs.Pop();

        // VERIFIED_ASSEMBLY 0x00420D98..0x00420DB0: strlen==0 is the sole
        // normal exit condition, including stale semaphore-token after clear().
        if (sql.empty()) {
            CMyAdoBase::ReleaseCn(connection);
            return;
        }

        // Old caller copied into zeroed char[2048] without a length guard.
        // Oversize input is an explicit technical boundary rather than stack OOB;
        // the popped SQL is not requeued because direct exception paths also
        // continue with the next queue item.
        if (sql.size() >= kLegacySqlBufferSize) {
            RecordError("SQL exceeds legacy AccLogThread char[2048]");
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
        if (!CMyAdoBase::ExecuteCn(sql.c_str(), connection)) {
            RecordError(connection.lastError);
            CMyAdoBase::ReleaseCn(connection);
            continue;
        }

        // Run ignores the returned bool. ADO Close failure would throw into the
        // same "Acc Log Err" handler; unixODBC exposes it explicitly instead.
        if (!CMyAdoBase::CloseCn(connection)) {
            RecordError(connection.lastError);
        }
        CMyAdoBase::ReleaseCn(connection);
    }
}

std::optional<AccLogThreadError> AccLogThread::PopTechnicalError()
{
    std::lock_guard guard(m_ErrorMutex);
    if (m_Errors.empty()) {
        return std::nullopt;
    }
    AccLogThreadError error = std::move(m_Errors.front());
    m_Errors.pop_front();
    return error;
}

void AccLogThread::RecordError(std::string detail)
{
    std::lock_guard guard(m_ErrorMutex);
    m_Errors.push_back(AccLogThreadError{
        .label = kAccLogErrorLabel,
        .detail = std::move(detail),
    });
}
}
