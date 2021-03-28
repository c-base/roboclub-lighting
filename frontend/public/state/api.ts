import { JSONSchema7 } from "json-schema";

const BASE_URL = "http://10.0.0.57:4444";

function request(method: string, url: string, body?: any): Promise<Response> {
	let config: RequestInit = {
		method,
		headers: {
			Accept: "application/json",
			"Content-Type": "application/json",
		},
	};

	if (body !== undefined) {
		config.body = JSON.stringify(body);
	}

	return fetch(BASE_URL + url, config);
}

const req = {
	async get<RES extends object>(url: string): Promise<RES> {
		let req = await request("GET", url);
		return req.json();
	},
	async post<RES extends object>(url: string, body: any): Promise<RES> {
		let req = await request("POST", url, body);
		return req.json();
	},
	async put<RES extends object>(url: string, body: any): Promise<RES> {
		let req = await request("PUT", url, body);
		return req.json();
	},
};

// export type RootSchema = {
// 	$schema: "http://json-schema.org/draft-07/schema#",
// 	title: string,
// 	"type": "object",
// 	"required": string[],
// 	"properties": {
// 		[name: string]: {
// 			type: string,
// 			//"uint" | "uint8" | "uint64",
// 			format?: string,
// 			minimum?: number,
// 			maximum?: number,
// 		}
// 	}
// };

export type EffectData = {
	name: string;
	schema: JSONSchema7;
	config: { [name: string]: any };
};

export type GetEffectsResponse = {
	effects: EffectData[];
	active_effect: EffectData;
};

export async function getEffects(): Promise<GetEffectsResponse> {
	return req.get<GetEffectsResponse>("/api/effects");
}

export type SetActiveEffectResponse = {
	active_effect: EffectData;
};

export async function setActiveEffect(activeEffect: string): Promise<SetActiveEffectResponse> {
	return req.post<SetActiveEffectResponse>(`/api/active_effect`, { active_effect: activeEffect });
}

// type SetEffectConfigResponse = {
// 	active_effect: EffectData;
// };

export async function setEffectConfig(name: string, config: any): Promise<EffectData> {
	return req.put<EffectData>(`/api/effects/${name}`, config);
}
