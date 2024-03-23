import "preact/debug";
import hydrate from "preact-iso/hydrate";
import { h } from "preact";

import { App } from "./app";

hydrate(<App />);

export async function prerender(data: {}) {
	const { default: prerender } = await import("preact-iso/prerender");
	return await prerender(<App {...data} />);
}
