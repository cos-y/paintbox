import sharp from 'sharp';
import { readFileSync } from 'fs';

const svg = readFileSync('src/lib/assets/favicon.svg');

const sizes = [192, 512];

for (const size of sizes) {
	sharp(svg)
		.resize(size, size)
		.png()
		.toFile(`static/pwa-${size}x${size}.png`)
		.then(() => console.log(`✓ pwa-${size}x${size}.png`))
		.catch((e) => console.error(`✗ ${size}`, e));
}
