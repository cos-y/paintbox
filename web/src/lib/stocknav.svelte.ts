// 库存页运行时状态：导航层级（品牌→系列→型号）与页面配置（排序/搜索/筛选）。
// 模块级 $state 跨路由切换存活——从其他页面切回 /stock/ 时组件重建，但导航路径
// 和筛选/搜索/排序配置保持原样。纯内存状态，不写 URL、不进 history；
// 刷新（整页重载）时 store 重置为初值（品牌根层）。

interface StockNavState {
	brand: string;
	serie: string;
	code: string;
	sortKey: number;
	searchOpen: boolean;
	query: string;
	filterOpen: boolean;
	surfSel: string[];
	baseSel: string[];
	mediumSel: string[];
}

export const stockNav = $state<StockNavState>({
	brand: '',
	serie: '',
	code: '',
	sortKey: 0,
	searchOpen: false,
	query: '',
	filterOpen: false,
	surfSel: [],
	baseSel: [],
	mediumSel: []
});

/**
 * 返回手势/返回按钮：直接离开当前品牌，回到品牌列表。
 * UI 上 serie/code 都只是品牌内的高亮切换（非独立层级）：
 * 详情面板是预览（桌面右侧栏 / 移动端抽屉），返回 = 退出品牌，
 * 避免 fallback 系列造成的假层级（一次返回即回到品牌根页）。
 * 系列内回退可用标题栏面包屑（goToLevel1）。
 */
export function goBackOneLevel(): boolean {
	if (stockNav.brand) {
		stockNav.brand = '';
		stockNav.serie = '';
		stockNav.code = '';
		return true;
	}
	return false;
}
