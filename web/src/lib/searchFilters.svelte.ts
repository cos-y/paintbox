const STORAGE_KEY = 'paintbox:searchFilters';

interface Serialized {
	selectedSeries: string[];
	surfaceTypes: string[];
	baseTypes: string[];
	searchScope: number;
	mixingLimit: number;
	model: number;
}

function load(): Serialized {
	if (typeof localStorage === 'undefined')
		return { selectedSeries: [], surfaceTypes: [], baseTypes: [], searchScope: 0, mixingLimit: 0, model: 0 };
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) return JSON.parse(raw);
	} catch {}
	return { selectedSeries: [], surfaceTypes: [], baseTypes: [], searchScope: 0, mixingLimit: 0, model: 0 };
}

class SearchFilterStore {
	selectedSeries = $state<Set<string>>(new Set(load().selectedSeries));
	surfaceTypes = $state<string[]>(load().surfaceTypes);
	baseTypes = $state<string[]>(load().baseTypes);
	searchScope = $state(load().searchScope);
	mixingLimit = $state(load().mixingLimit);
	model = $state(load().model);

	persist() {
		const data: Serialized = {
			selectedSeries: [...this.selectedSeries],
			surfaceTypes: this.surfaceTypes,
			baseTypes: this.baseTypes,
			searchScope: this.searchScope,
			mixingLimit: this.mixingLimit,
			model: this.model
		};
		localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
	}

	reset() {
		this.selectedSeries = new Set();
		this.surfaceTypes = [];
		this.baseTypes = [];
		this.searchScope = 0;
		this.mixingLimit = 0;
		this.persist();
	}
}

export const searchFilters = new SearchFilterStore();
