#!/usr/bin/env bash
# 生成侧载/其他渠道分发的 APK：
# Play 构建出的 APK 用「上传密钥」签名（用于上传 AAB），
# 这里用「签名密钥」重签，使侧载版本与 Play 分发的版本签名一致，可互相覆盖升级。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APKSIGNER="$ANDROID_HOME/build-tools/36.1.0/apksigner.bat"
ZIPALIGN="$ANDROID_HOME/build-tools/36.1.0/zipalign.exe"
BUILT_APK="$ROOT/src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk"
KS_PROPS="$ROOT/src-tauri/keystore/keystore.properties"
KS="$ROOT/src-tauri/keystore/paintbox-signing.jks"
OUT_DIR="$ROOT/dist"

if [ ! -f "$KS_PROPS" ]; then
	echo "缺少 $KS_PROPS（签名配置），请先按 src-tauri/keystore/README 生成密钥"
	exit 1
fi
SIGN_PASS=$(grep '^signing.storePassword=' "$KS_PROPS" | cut -d= -f2)
KEY_PASS=$(grep '^signing.keyPassword=' "$KS_PROPS" | cut -d= -f2)
KEY_ALIAS=$(grep '^signing.keyAlias=' "$KS_PROPS" | cut -d= -f2)

if [ ! -f "$BUILT_APK" ]; then
	echo "未找到构建产物：$BUILT_APK"
	echo "请先构建：cd src-tauri/gen/android && ./gradlew assembleRelease"
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
