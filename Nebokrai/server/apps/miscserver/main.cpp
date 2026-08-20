#include "misc/miscserver.h"

#include <spdlog/spdlog.h>

#include <chrono>
#include <csignal>
#include <cstdlib>
#include <filesystem>
#include <thread>

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
        std::filesystem::absolute(argc > 1 ? std::filesystem::path(argv[1])
                                           : std::filesystem::current_path());

    Misc::MiscServer server;
    while (g_StopRequested == 0) {
        const Misc::MiscGameResult result = server.Initialize(runtimeDirectory);
        if (result) break;
        if (result.status != Misc::MiscGameStatus::NetworkError) {
            spdlog::error("MiscServer: инициализация остановлена: {}", result.detail);
            static_cast<void>(server.Release());
            return EXIT_FAILURE;
        }
        spdlog::warn("MiscServer: WorldServer недоступен, повтор через 8 секунд: {}",
                     result.detail);
        std::this_thread::sleep_for(std::chrono::seconds(8));
    }
    if (g_StopRequested != 0) {
        static_cast<void>(server.Release());
        return EXIT_SUCCESS;
    }

    bool clean = true;
    while (g_StopRequested == 0) {
        const Misc::MiscGameResult turn = server.RunTurn();
        if (!turn) {
            spdlog::error("MiscServer: main-loop остановлен: {}", turn.detail);
            clean = false;
            break;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }
    static_cast<void>(server.Release());
    std::this_thread::sleep_for(std::chrono::milliseconds(2'005));
    return clean ? EXIT_SUCCESS : EXIT_FAILURE;
}
