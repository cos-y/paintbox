// 库存页运行时状态：导航层级（品牌→系列→型号）与页面配置（排序/搜索/筛选）。
// 模块级 $state 跨路由切换存活——从其他页面切回 /stock/ 时组件重建，但导航路径
// 和筛选/搜索/排序配置保持原样，不再只依赖 URL 查询参数（切换路由会清掉 URL）。
//
// URL 仍保留为同步通道：首次打开/刷新时从 ?brand=&serie=&code= 初始化（支持分享链接），
// 之后 navigateTo 写回 URL 保持地址栏与分享能力；刷新（整页重载）则 store 重置为初值。

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
	baseSel: []
});
