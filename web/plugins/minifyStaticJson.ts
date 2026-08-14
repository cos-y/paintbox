import { type Plugin } from 'vite';
import fs from 'fs';
import path from 'path';

/** 递归收集目录下所有 .json 文件 */
function collectJsonFiles(dir: string, out: string[] = []): string[] {
	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		const full = path.join(dir, entry.name);
		if (entry.isDirectory()) collectJsonFiles(full, out);
		else if (entry.name.endsWith('.json')) out.push(full);
	}
	return out;
}

export function minifyStaticJson(): Plugin {
	return {
		name: 'minify-static-json',
		closeBundle() {
			const distDir = path.resolve(import.meta.dirname, '../build'); // 对应构建后的 static 目录
			if (!fs.existsSync(distDir)) return;

			for (const filePath of collectJsonFiles(distDir)) {
				try {
					// 解析并重新序列化以去除空格和换行
					const minified = JSON.stringify(JSON.parse(fs.readFileSync(filePath, 'utf-8')));
					fs.writeFileSync(filePath, minified);
					console.log(`[minify-static-json] ${path.relative(distDir, filePath)}`);
				} catch (e) {
					console.error(`Failed to minify ${filePath}:`, e);
				}
			}
		}
	};
}
