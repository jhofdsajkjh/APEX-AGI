@echo off
:: =============================================================================
:: ΩMEGA AGI SUPREMACY — 一键安装脚本 (Windows)
:: =============================================================================
:: 使用方法:
::   .\install.bat
:: =============================================================================

chcp 65001 >nul

echo ╔══════════════════════════════════════════════════════════════╗
echo ║     ΩMEGA AGI SUPREMACY — 一键安装 (Windows)              ║
echo ║     11-Layer Autonomous AGI System                         ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

:: ── 检查 Rust ──
echo 🔍 检查 Rust 工具链...
where rustc >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo 📦 未检测到 Rust，正在下载安装程序...
    echo    请访问 https://rustup.rs 或运行:
    echo    curl -sSf https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe -o rustup-init.exe
    echo    .\rustup-init.exe -y
    pause
    exit /b 1
)
echo ✅ Rust: 
rustc --version

:: ── 检查 Git ──
where git >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo 📦 未检测到 Git，请从 https://git-scm.com/download/win 安装
    pause
    exit /b 1
)
echo ✅ Git: 
git --version

:: ── 克隆仓库 ──
if exist "omega-agi" (
    echo 📂 目录已存在，更新...
    cd omega-agi
    git pull --ff-only
) else (
    echo 📦 克隆仓库...
    git clone --depth 1 https://github.com/omega-agi/omega-agi.git
    cd omega-agi
)

:: ── 配置环境变量 ──
if not exist ".env" (
    copy .env.example .env
    echo ⚠️  请编辑 .env 文件填入你的 API Key
    echo    notepad .env
)

:: ── 构建 ──
echo 🔧 构建项目 (release)...
cargo build --release

:: ── 复制二进制到系统路径 ──
echo 📦 安装到系统...
if not exist "%LOCALAPPDATA%\omega-agi" mkdir "%LOCALAPPDATA%\omega-agi"
if not exist "%LOCALAPPDATA%\omega-agi\data" mkdir "%LOCALAPPDATA%\omega-agi\data"
copy target\release\omega-agi.exe "%LOCALAPPDATA%\omega-agi\" >nul

:: ── 添加到 PATH ──
echo 🔧 添加到 PATH...
setx PATH "%PATH%;%LOCALAPPDATA%\omega-agi" >nul 2>&1

echo.
echo ✅ ΩMEGA AGI 安装完成！
echo.
echo   二进制: %LOCALAPPDATA%\omega-agi\omega-agi.exe
echo   数据:   %LOCALAPPDATA%\omega-agi\data
echo.
echo   ⚡ 快速启动 (重新打开终端后):
echo      omega-agi check          系统健康检查
echo      omega-agi interactive     交互模式
echo      omega-agi evolve          自我进化
echo      omega-agi apex --verbose   Φ_APEX*∞ 进化
echo.
echo  ⚠️  请编辑 .env 文件配置 API Key
echo     notepad .env
echo.

:: 验证构建
echo 🔍 运行验证...
cargo run -- check

pause
