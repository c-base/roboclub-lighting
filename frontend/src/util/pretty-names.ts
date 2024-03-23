export function prettyName(name: string): string {
	return name
		.split("_")
		.map((segment) => (segment[0] ?? "").toUpperCase() + segment.slice(1))
		.join(" ");
}
