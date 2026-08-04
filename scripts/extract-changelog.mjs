#!/usr/bin/env node
// 从 CHANGELOG.md 提取指定版本的 release notes（Keep a Changelog 格式）。
// 用法：node scripts/extract-changelog.mjs <version>   # 输出到 stdout
// 逐行状态机解析，无第三方依赖：
//   - "## [x.y.z]" 开始收集
//   - 遇到下一个 "## "（下一版本）或底部链接定义 "[x]: url" 时停止

import { readFileSync } from 'node:fs';

const version = process.argv[2];
if (!version) {
	console.error('usage: node extract-changelog.mjs <version>');
	process.exit(1);
}

const lines = readFileSync(new URL('../CHANGELOG.md', import.meta.url), 'utf8').split('\n');

const section = [];
let inSection = false;
for (const line of lines) {
	const isHeading = /^## /.test(line);
	const isLinkDef = /^\[[^\]]+\]: /.test(line);

	if (isHeading) {
		if (inSection) break; // 已过目标版本，遇到下一个版本标题即结束
		if (new RegExp(`^## \\[${version.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\]`).test(line)) {
			inSection = true;
		}
		continue;
	}
	if (inSection) {
		if (isLinkDef) break; // 底部链接定义区，结束
		section.push(line);
	}
}

const out = section.join('\n').trim();
console.log(out || 'No changelog entry for this version.');
