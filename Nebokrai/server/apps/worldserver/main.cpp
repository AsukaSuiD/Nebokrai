#include "world/worldserver.h"

#include <spdlog/spdlog.h>

#include <chrono>
#include <csignal>
#include <cstdlib>
#include <filesystem>
#include <thread>

/*
 * Техническая Linux-точка входа WorldServer. WinMain, глобальный CGame и
 * Windows message pump заменены обычным процессом; доменный lifecycle остаётся
 * в world/worldserver.{h,cpp}. Runtime-каталог по-прежнему содержит setup.ini
 * и оригинальные ресурсы World.
 */
namespace
{
volatile std::sig_atomic_t g_StopRequested{};
void RequestStop(int) { g_StopRequested = 1; }
}

int main(int argc, char* argv[])
{
    std::signal(SIGINT, RequestStop);
    std::signal(SIGTERM, RequestStop);
    const std::filesystem::path runtimeDirectory =
        argc > 1 ? std::filesystem::path(argv[1]) : std::filesystem::current_path();

    spdlog::info("WorldServer: runtime-каталог {}", runtimeDirectory.string());
    CWorldServer server;
    const WorldInitializationResult initialized = server.Initialize(runtimeDirectory);
    if (!initialized) {
        spdlog::error("WorldServer: инициализация остановлена: {}", initialized.error);
        return EXIT_FAILURE;
    }

    bool clean = true;
    while (g_StopRequested == 0) {
        const WorldLoopResult turn = server.RunOne();
        if (!turn.success) {
            spdlog::error("WorldServer: main-loop остановлен: {}", turn.error);
            clean = false;
            break;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(1));
    }
    server.RequestStop();
    clean = server.Shutdown() && clean;
    spdlog::info("WorldServer: {}", clean ? "штатно завершён" : "завершён с ошибкой");
    return clean ? EXIT_SUCCESS : EXIT_FAILURE;
}
