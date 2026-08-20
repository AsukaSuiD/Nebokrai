#include "login/loginserver.h"

#include <spdlog/spdlog.h>

#include <csignal>
#include <cstdlib>
#include <filesystem>

/*
 * Техническая Linux-точка входа отдельного LoginServer-процесса. Исходные
 * WinMain/game-thread заменены обычным main и сигналами POSIX; весь сервисный
 * lifecycle находится в login/loginserver.{h,cpp}.
 */
namespace
{
volatile std::sig_atomic_t g_StopRequested{};

void RequestStop(int)
{
    g_StopRequested = 1;
}
}

int main(int argc, char* argv[])
{
    std::signal(SIGINT, RequestStop);
    std::signal(SIGTERM, RequestStop);

    const std::filesystem::path runtimeDirectory =
        std::filesystem::absolute(argc > 1 ? std::filesystem::path(argv[1])
                                           : std::filesystem::current_path());
    std::error_code pathError;
    std::filesystem::current_path(runtimeDirectory, pathError);
    if (pathError) {
        spdlog::error("LoginServer: не выбран runtime-каталог {}: {}",
                      runtimeDirectory.string(), pathError.message());
        return EXIT_FAILURE;
    }

    Login::LoginServer server;
    if (const Login::LoginServerResult initialized = server.Initialize(runtimeDirectory);
        !initialized) {
        spdlog::error("LoginServer: инициализация остановлена: {}", initialized.detail);
        static_cast<void>(server.Release());
        return EXIT_FAILURE;
    }

    bool clean = true;
    while (g_StopRequested == 0) {
        const Login::LoginServerResult turn = server.RunTurn();
        if (!turn) {
            spdlog::error("LoginServer: main-loop остановлен: {}", turn.detail);
            clean = false;
            break;
        }
    }

    const Login::LoginServerResult released = server.Release();
    if (!released) {
        spdlog::error("LoginServer: ошибка освобождения: {}", released.detail);
        clean = false;
    }
    return clean ? EXIT_SUCCESS : EXIT_FAILURE;
}
