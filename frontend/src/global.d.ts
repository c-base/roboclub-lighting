declare module "*.css" {
	const mapping: Record<string, string>;
	export default mapping;
}

interface ImportMeta {
	hot: any;
}
