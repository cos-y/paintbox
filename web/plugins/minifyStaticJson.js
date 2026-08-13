// vite.config.ts
import fs from 'fs';
import path from 'path';

export function minifyStaticJson() {
	return {
		name: 'minify-static-json',
		closeBundle() {
			const distDir = path.resolve(__dirname, 'dist/static'); // 对应构建后的 static 目录
			if (!fs.existsSync(distDir)) return;

			const files = fs.readdirSync(distDir);
			files.forEach((file) => {
				if (file.endsWith('.json')) {
					const filePath = path.join(distDir, file);
					const content = fs.readFileSync(filePath, 'utf-8');
					try {
						// 解析并重新序列化以去除空格和换行
						const minified = JSON.stringify(JSON.parse(content));
						fs.writeFileSync(filePath, minified);
					} catch (e) {
						console.error(`Failed to minify ${file}:`, e);
					}
				}
			});
		}
	};
}
