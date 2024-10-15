import { JSONSchema7 } from "json-schema";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";

import { Empty } from "../proto/google/protobuf/empty";
import { Struct } from "../proto/google/protobuf/struct";
import { ControllerClient } from "../proto/control.client";
import * as proto from "../proto/control";
import { SegmentId } from "../proto/control";

const HOSTNAME = import.meta.hot ? "localhost:4445" : `${window.location.hostname}:4445`;

const transport = new GrpcWebFetchTransport({
	format: "binary",
	baseUrl: `http://${HOSTNAME}`,
});
const client = new ControllerClient(transport);

export type Effects = Record<string, Effect>;

export type Config = {
	brightness: number;
	srgb: boolean;
};

export type Effect = {
	name: string;
	schema: JSONSchema7;
	defaultConfig: EffectConfig;
};

export type Segments = Strip[];

export type Strip = {
	offset: number;
	segments: Segment[];
};

export type Segment = {
	name: string;
	length: number;
	reversed: boolean;
};

export type EffectConfig = Record<string, any>;

export type Presets = Record<string, DisplayState>;

export type DisplayState = {
	effects: DisplayStateEffect[];
};

export type DisplayStateEffect = {
	effectId: string;
	config: EffectConfig;
	segmentIds: proto.SegmentId[];
	groupIds: string[];
};

function serializeDisplayState(state: DisplayState): proto.DisplayState {
	const effects = state.effects.map(
		(effect) =>
			({
				effectId: effect.effectId,
				config: Struct.fromJson(effect.config),
				segmentIds: effect.segmentIds,
				groupIds: effect.groupIds,
			}) satisfies proto.DisplayStateEffect,
	);

	return {
		effects,
	};
}

function deserializeDisplayState(state: proto.DisplayState): DisplayState {
	return {
		effects: state.effects.map((effect) => ({
			effectId: effect.effectId,
			config: Struct.toJson(effect.config!) as any,
			segmentIds: effect.segmentIds,
			groupIds: effect.groupIds,
		})),
	};
}

const EMPTY: Empty = {};

export async function getConfig(): Promise<Config> {
	const { response } = await client.getConfig(EMPTY);

	return response;
}

export async function setConfig(config: Config): Promise<Config> {
	const { response } = await client.setConfig(config);

	return response;
}

export async function listEffects(): Promise<Effects> {
	const { response } = await client.listEffects(EMPTY);

	const effects: Effects = {};
	for (const [name, effect] of Object.entries(response.effects)) {
		effects[name] = {
			name: effect.name,
			schema: Struct.toJson(effect.schema!) as any,
			defaultConfig: Struct.toJson(effect.defaultConfig!) as any,
		};
	}

	return effects;
}

export async function listSegments(): Promise<Strip[]> {
	const { response } = await client.listSegments(EMPTY);

	return response.strips.map((strip) => ({
		offset: strip.offset,
		segments: strip.segments.map((segment) => segment),
	}));
}

export async function setSegments(strips: Strip[]): Promise<Strip[]> {
	const { response } = await client.setSegments({ strips });

	return response.strips;
}

export async function listPresets(): Promise<Record<string, DisplayState>> {
	const { response } = await client.listPresets(EMPTY);

	const presets: Record<string, DisplayState> = {};
	for (const [name, state] of Object.entries(response.presets)) {
		presets[name] = deserializeDisplayState(state);
	}

	return presets;
}

export async function setPreset(name: string, state: DisplayState): Promise<DisplayState> {
	const { response } = await client.setPreset({
		name,
		data: serializeDisplayState(state),
	});

	return deserializeDisplayState(response);
}

export async function loadPreset(name: string): Promise<DisplayState> {
	const { response } = await client.loadPreset({ name });

	return deserializeDisplayState(response);
}

export async function savePreset(name: string): Promise<DisplayState> {
	const { response } = await client.savePreset({ name });

	return deserializeDisplayState(response);
}

export async function getState(): Promise<DisplayState> {
	const { response } = await client.getState(EMPTY);

	return deserializeDisplayState(response);
}

export async function setState(state: DisplayState): Promise<DisplayState> {
	const { response } = await client.setState({
		state: serializeDisplayState(state),
	});

	return deserializeDisplayState(response);
}

export async function streamState(): Promise<DisplayState> {
	const _ = client.streamState(EMPTY);

	throw "unimplemented";
}
