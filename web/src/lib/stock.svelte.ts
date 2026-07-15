const STORAGE_KEY = 'paintbox:stock';

const loadFromStorage = (): Record<string, number> => {
	if (typeof localStorage === 'undefined') return {};
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? JSON.parse(raw) : {};
	} catch {
		return {};
	}
};

class StockStore {
	quantities = $state<Record<string, number>>(loadFromStorage());

	get(id: string): number {
		return this.quantities[id] ?? 0;
	}

	set(id: string, qty: number) {
		if (qty <= 0) {
			delete this.quantities[id];
		} else {
			this.quantities[id] = qty;
		}
		this.persist();
	}

	add(id: string, delta: number) {
		this.set(id, this.get(id) + delta);
	}

	private persist() {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(this.quantities));
	}
}

export const stock = new StockStore();
