export interface BrandMeta {
	slug: string;
	name: string;
	desc: string;
	url: string;
}

export interface SerieMeta {
	brand: string;
	serie: string;
	name: string;
	desc: string;
}

let brandMap = new Map<string, BrandMeta>();
let serieMap = new Map<string, SerieMeta>();

export const loadMeta = async (fetchFn: typeof fetch) => {
	const [brands, series] = await Promise.all([
		fetchFn('/brands.json').then((r) => r.json() as Promise<BrandMeta[]>),
		fetchFn('/series.json').then((r) => r.json() as Promise<SerieMeta[]>)
	]);
	brandMap = new Map(brands.map((b) => [b.slug.toLowerCase(), b]));
	serieMap = new Map(series.map((s) => [`${s.brand}::${s.serie}`, s]));
};

export const getBrandMeta = (brand: string): BrandMeta | undefined => brandMap.get(brand.toLowerCase());

export const getSerieMeta = (brand: string, serie: string): SerieMeta | undefined =>
	serieMap.get(`${brand.toLowerCase()}::${serie}`);

export const serieThumb = (brand: string, serie: string) =>
	`/series-thumbs/${brand.toLowerCase()}-${serie}.jpg`;
