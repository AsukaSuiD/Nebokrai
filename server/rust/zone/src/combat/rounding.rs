//! Числовые преобразования исходного GameServer для боевых формул.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/fightdefense.cpp` и `thunder.cpp` с соседними `.h`.

/// Усечение через FISTP DWORD: вне signed-диапазона получается indefinite INT_MIN.
pub fn truncate_original(value: f64) -> i32 {
    let value = value.trunc();
    if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
        i32::MIN
    } else {
        value as i32
    }
}

/// Младший DWORD результата `__ftol2`, включая случай выхода за пределы i64.
pub fn truncate_original_i64_low(value: f64) -> i32 {
    if !value.is_finite()
        || value < -9_223_372_036_854_775_808.0
        || value >= 9_223_372_036_854_775_808.0
    {
        i64::MIN as i32
    } else {
        (value as i64) as i32
    }
}
