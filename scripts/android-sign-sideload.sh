#!/usr/bin/env bash
# 生成侧载/其他渠道分发的 APK：
# tauri 构建出的 APK 用「上传密钥」签名（用于上传 Play 的 AAB 同源），
# 这里用「签名密钥」重签，使侧载版本与 Play 分发的版本签名一致，可互相覆盖升级。
# 支持 Windows(git-bash) 与 Linux(CI)。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [ -z "$ANDROID_HOME" ]; then
	echo "ANDROID_HOME 未设置"
	exit 1
fi

# 找到最新 build-tools 里的 apksigner / zipalign（Windows 带 .bat/.exe 后缀）
BT_DIR="$(ls -d "$ANDROID_HOME/build-tools"/*/ 2>/dev/null | sort -V | tail -1 | sed 's:/*$::')"
if [ -z "$BT_DIR" ]; then
	echo "未找到 build-tools"
	exit 1
fi
if [ -f "$BT_DIR/apksigner.bat" ]; then
	APKSIGNER="$BT_DIR/apksigner.bat"
	ZIPALIGN="$BT_DIR/zipalign.exe"
else
	APKSIGNER="$BT_DIR/apksigner"
	ZIPALIGN="$BT_DIR/zipalign"
fi

KS_PROPS="$ROOT/src-tauri/keystore/keystore.properties"
KS="$ROOT/src-tauri/keystore/paintbox-signing.jks"
OUT_DIR="$ROOT/dist"

if [ ! -f "$KS_PROPS" ]; then
	echo "缺少 $KS_PROPS（签名配置）"
	exit 1
fi
SIGN_PASS=$(grep '^signing.storePassword=' "$KS_PROPS" | cut -d= -f2)
KEY_PASS=$(grep '^signing.keyPassword=' "$KS_PROPS" | cut -d= -f2)
KEY_ALIAS=$(grep '^signing.keyAlias=' "$KS_PROPS" | cut -d= -f2)

BUILT_APK="$(find "$ROOT/src-tauri/gen/android/app/build/outputs/apk" -name "*universal*release*.apk" 2>/dev/null | head -1 || true)"
if [ -z "$BUILT_APK" ]; then
	echo "未找到 universal release APK，请先执行 tauri android build"
	exit 1
fi

mkdir -p "$OUT_DIR"
ALIGNED="$OUT_DIR/_aligned.apk"
"$ZIPALIGN" -f 4 "$BUILT_APK" "$ALIGNED"
"$APKSIGNER" sign \
	--ks "$KS" \
	--ks-pass pass:"$SIGN_PASS" \
	--ks-key-alias "$KEY_ALIAS" \
	--key-pass pass:"$KEY_PASS" \
	--out "$OUT_DIR/paintbox-sideload.apk" \
	"$ALIGNED"
rm -f "$ALIGNED"
echo "侧载版已生成：$OUT_DIR/paintbox-sideload.apk"
"$APKSIGNER" verify "$OUT_DIR/paintbox-sideload.apk"
