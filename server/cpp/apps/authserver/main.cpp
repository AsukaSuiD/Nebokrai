#include "auth/cgame.h"
#include "dbaccess/authdb/mssql_odbc.h"

#include <spdlog/spdlog.h>

#include <csignal>
#include <cstdlib>
#include <filesystem>

/*
 * Техническая Linux-точка входа AuthServer.
 *
 * Исходные владельцы: AuthServer/Source/Main/authmain.cpp и
 * authserver/src/authserver.cpp из AuthServer/authserver.exe + PDB. Windows
 * WinMain, MFC pump и отдельный game-thread заменены обычным Linux-процессом;
 * доменный lifecycle остаётся в auth/cgame.{h,cpp}. Имена setup.ini,
 * allowed_ls.ini и client_forbid_ip.ini и их расположение относительно
 * runtime-каталога сохранены. MSSQL подключается через Linux ODBC-адаптер.
 */
namespace
{
volatile std::sig_atomic_t g_StopRequested{};

void RequestStop(int)
{
    g_StopRequested = 1;
}

const char* InitializationStatusName(AuthInitializationStatus status) noexcept
{
    switch (status) {
    case AuthInitializationStatus::Ok:
        return "успешно";
    case AuthInitializationStatus::NetworkFailed:
        return "не удалось запустить сеть";
    case AuthInitializationStatus::DatabaseWorkersFailed:
        return "не удалось запустить обработчики БД";
    }
    return "неизвестная ошибка";
}
}

int main(int argc, char* argv[])
{
    std::signal(SIGINT, RequestStop);
    std::signal(SIGTERM, RequestStop);

    const std::filesystem::path runtimeDirectory =
        argc > 1 ? std::filesystem::path(argv[1]) : std::filesystem::current_path();

    spdlog::info("AuthServer: runtime-каталог {}", runtimeDirectory.string());

    CGame game(ConfigReader{}, MakeMssqlOdbcDatabaseFactory());
    const AuthInitializationResult initialized = game.InitializeRuntime(
        runtimeDirectory / "setup.ini",
        runtimeDirectory / "allowed_ls.ini",
        runtimeDirectory / "client_forbid_ip.ini");

    for (const AuthInitializationNotice& notice : initialized.notices) {
        spdlog::warn("AuthServer: предупреждение инициализации, код {}",
                     static_cast<int>(notice.type));
    }

    if (!initialized) {
        spdlog::error("AuthServer: инициализация остановлена: {}",
                      InitializationStatusName(initialized.status));
        static_cast<void>(game.ReleaseRuntime());
        return EXIT_FAILURE;
    }

    bool clean = true;
    while (g_StopRequested == 0) {
        const AuthRuntimeStep step = game.RunMainLoopTurn();
        if (step.status != AuthProcessStatus::Ok) {
            spdlog::error("AuthServer: main-loop остановлен, код {}",
                          static_cast<int>(step.status));
            clean = false;
            break;
        }
    }

    const AuthReleaseResult released = game.ReleaseRuntime();
    if (!released.databaseWorkerErrors.empty() || released.networkTaskFailures != 0) {
        spdlog::error(
            "AuthServer: завершение с ошибками: обработчики БД {}, сетевые задачи {}",
            released.databaseWorkerErrors.size(),
            released.networkTaskFailures);
        clean = false;
    }

    spdlog::info("AuthServer: {}", clean ? "штатно завершён" : "завершён с ошибкой");
    return clean ? EXIT_SUCCESS : EXIT_FAILURE;
}
