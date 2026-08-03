#!/usr/bin/env node
// 图标生成脚本：
//   web/src/lib/assets/favicon.svg（网页 favicon，你维护的源图）
//     → web/src/lib/assets/app-icon.svg（1024×1024 方形 App 图标源图）
//     → src-tauri/icons/（tauri 全套图标：安卓/桌面/ico/icns）
// 用法：项目根目录执行  pnpm icons  （等价 node scripts/generate-icons.mjs）
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const faviconPath = resolve(root, 'web/src/lib/assets/favicon.svg');
const appIconPath = resolve(root, 'web/src/lib/assets/app-icon.svg');
const SIZE = 1024; // 输出方形尺寸
const BG_COLOR = '#101828'; // 图标底色（adaptive 背景层 + 传统图标源图背景矩形）

// 1. 读取 favicon
if (!existsSync(faviconPath)) {
	console.error(`找不到源图标: ${faviconPath}`);
	process.exit(1);
}
const src = readFileSync(faviconPath, 'utf8');
const vb = src.match(/viewBox="([\d.]+) ([\d.]+) ([\d.]+) ([\d.]+)"/);
if (!vb) {
	console.error('favicon.svg 缺少 viewBox，请检查 SVG 格式');
	process.exit(1);
}
const vw = Number(vb[3]); // width
const vh = Number(vb[4]); // height

// 2. 提取 defs（样式/渐变等）和绘制元素（polygon/path/... 都兼容）
const body = src
	.replace(/<\?xml[\s\S]*?\?>/s, '')
	.replace(/<svg[^>]*>/s, '')
	.replace(/<\/svg>\s*$/s, '')
	.trim();
const defs = body.match(/<defs>[\s\S]*?<\/defs>/s)?.[0] ?? '';
const shapes = body.replace(/<defs>[\s\S]*?<\/defs>/s, '').trim();
if (!shapes) {
	console.error('favicon.svg 里没有绘制元素');
	process.exit(1);
}

// 3. 包装成正方形（内容等比缩放居中）
// 内容与圆形遮罩内接：内容矩形对角线 = 画布边长 → 四角恰好贴遮罩圆，不被裁剪
// 用嵌套 <svg> + viewBox 映射代替 transform：resvg 对 transform 的组合解析不可靠
const diag = Math.hypot(vw, vh);
const scale = SIZE / diag * 0.7;
const contentW = vw * scale;
const contentH = vh * scale;
const tx = (SIZE - contentW) / 2;
const ty = (SIZE - contentH) / 2;
const out = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${SIZE}" height="${SIZE}" viewBox="0 0 ${SIZE} ${SIZE}">
	<!-- 底色：传统图标（<8.0）/桌面图标直接可见；adaptive 前景矩形与背景层同色，遮罩裁切无缝 -->
	<rect width="${SIZE}" height="${SIZE}" fill="${BG_COLOR}" />
	${defs}
	<svg
		x="${tx.toFixed(2)}"
		y="${ty.toFixed(2)}"
		width="${contentW.toFixed(2)}"
		height="${contentH.toFixed(2)}"
		viewBox="0 0 ${vw} ${vh}"
		overflow="visible"
	>
${shapes}
	</svg>
</svg>
`;
writeFileSync(appIconPath, out, 'utf8');
console.log(`✓ 方形源图: web/src/lib/assets/app-icon.svg (${SIZE}×${SIZE}, 内容 ${Math.round(contentW)}×${Math.round(contentH)})`);

// 4. 生成全套 App 图标（依赖根目录 node_modules 里的 @tauri-apps/cli）
const tauriBin = resolve(
	root,
	'node_modules/.bin/tauri' + (process.platform === 'win32' ? '.cmd' : '')
);
if (!existsSync(tauriBin)) {
	console.error(`未找到 tauri CLI: ${tauriBin}（请先在根目录安装依赖）`);
	process.exit(1);
}
console.log('生成 App 图标（src-tauri/icons/）…');
execSync(`"${tauriBin}" icon "${appIconPath}"`, { cwd: root, stdio: 'inherit' });
console.log('✓ 完成：src-tauri/icons/ 已更新，重新构建 APK 即可看到新图标');

// 5. Adaptive 图标背景色（Android 8.0+）：tauri 默认 #fff，改为图标底色
//（tauri icon 每次都会重置这个文件，所以生成后立即修补）
const bgRes = resolve(
	root,
	'src-tauri/gen/android/app/src/main/res/values/ic_launcher_background.xml'
);
if (existsSync(bgRes)) {
	const bg = readFileSync(bgRes, 'utf8').replace(
		/(<color name="ic_launcher_background">)[^<]*(<\/color>)/,
		`$1${BG_COLOR}$2`
	);
	writeFileSync(bgRes, bg, 'utf8');
	console.log(`✓ Adaptive 背景色已设为 ${BG_COLOR}`);
}
