const STORAGE_KEY = 'paintbox:stock';

const loadFromStorage = (): string[] => {
	if (typeof localStorage === 'undefined') return [];
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? JSON.parse(raw) : [];
	} catch {
		return [];
	}
};

class StockStore {
	owned = $state<Set<string>>(new Set(loadFromStorage()));

	has(id: string): boolean {
		return this.owned.has(id);
	}

	set(id: string, owned: boolean) {
		const next = new Set(this.owned);
		if (owned) next.add(id);
		else next.delete(id);
		this.owned = next;
		this.persist();
	}

	toggle(id: string) {
		this.set(id, !this.has(id));
	}

	private persist() {
		localStorage.setItem(STORAGE_KEY, JSON.stringify([...this.owned]));
	}
}

export const stock = new StockStore();
