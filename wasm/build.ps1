# wasm-pack release 构建 + wasm-opt 优化，产物直接进 web/src/wasm-pkg
# 用法：在 wasm/ 目录下运行（pwsh ./build.ps1）
# 依赖：
#   - wasm-pack（cargo install wasm-pack）
#   - binaryen wasm-opt（D:\binaryen-version_130\bin 已在 PATH）
# 注意：wasm 是预编译提交，改动 wasm/src/ 后必须重跑本脚本再提交
# wasm-pack 0.15 无法指定 wasm-opt 级别（只有 --no-opt），所以手动 -O4 一次到位

wasm-pack build --release --target web --out-dir ../web/src/wasm-pkg --no-opt
wasm-opt -O4 -o ../web/src/wasm-pkg/paintbox_wasm_bg.wasm ../web/src/wasm-pkg/paintbox_wasm_bg.wasm
