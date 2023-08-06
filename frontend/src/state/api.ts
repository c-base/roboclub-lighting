import { JSONSchema7 } from "json-schema";
import { ControllerClient } from "../proto/ControlServiceClientPb.ts";
import { Empty } from "google-protobuf/google/protobuf/empty_pb";
import { Struct } from "google-protobuf/google/protobuf/struct_pb";
import * as proto from "../proto/control_pb";

const HOSTNAME = import.meta.hot ? "localhost:4445" : ":4445";

const client = new ControllerClient(HOSTNAME);

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
	strips: number[][];
};

export type DisplayStateEffect = {
	effect: string;
	config: EffectConfig;
};

function serializeDisplayState(state: DisplayState): proto.DisplayState {
	const strips = state.strips.map((strip) => new proto.DisplayStateStrip().setSegmentsList(strip));
	const effects = state.effects.map((effect) =>
		new proto.DisplayStateEffect()
			.setEffect(effect.effect)
			.setConfig(Struct.fromJavaScript(effect.config))
	);

	return new proto.DisplayState().setStripsList(strips).setEffectsList(effects);
}

function deserializeDisplayState(state: proto.DisplayState): DisplayState {
	return {
		effects: state.getEffectsList().map((effect) => ({
			effect: effect.getEffect(),
			config: effect.getConfig().toJavaScript(),
		})),
		strips: state.getStripsList().map((strip) => strip.getSegmentsList()),
	};
}

const EMPTY = new Empty();

export async function getConfig(): Promise<Config> {
	const config = await client.getConfig(EMPTY, null);

	return config.toObject();
}

export async function setConfig(config: Config): Promise<Config> {
	const req = new proto.Config().setBrightness(config.brightness).setSrgb(config.srgb);

	const serverConfig = await client.setConfig(req, null);

	return serverConfig.toObject();
}

export async function listEffects(): Promise<Effects> {
	const list = await client.listEffects(EMPTY, null);

	const effects: Effects = {};
	for (const [name, effect] of list.getEffectsMap().entries()) {
		effects[name] = {
			name: effect.getName(),
			schema: effect.getSchema().toJavaScript(),
			defaultConfig: effect.getDefaultConfig().toJavaScript(),
		};
	}

	return effects;
}

export async function listSegments(): Promise<Strip[]> {
	const list = await client.listSegments(EMPTY, null);

	return list.getStripsList().map((strip) => ({
		offset: strip.getOffset(),
		segments: strip.getSegmentsList().map((segment) => segment.toObject()),
	}));
}

export async function setSegments(segments: Strip[]): Promise<Strip[]> {
	const strips = segments.map((strip) =>
		new proto.Strip()
			.setOffset(strip.offset)
			.setSegmentsList(
				strip.segments.map((segment) =>
					new proto.Segment()
						.setName(segment.name)
						.setLength(segment.length)
						.setReversed(segment.reversed)
				)
			)
	);

	const req = new proto.SetSegmentsRequest().setSegments(
		new proto.Segments().setStripsList(strips)
	);

	const list = await client.setSegments(req, null);

	return list.getStripsList().map((strip) => ({
		offset: strip.getOffset(),
		segments: strip.getSegmentsList().map((segment) => segment.toObject()),
	}));
}

export async function listPresets(): Promise<Record<string, DisplayState>> {
	const list = await client.listPresets(EMPTY, null);

	const presets: Record<string, DisplayState> = {};
	for (const [name, state] of list.getPresetsMap().entries()) {
		presets[name] = deserializeDisplayState(state);
	}

	return presets;
}

export async function setPreset(name: string, state: DisplayState): Promise<DisplayState> {
	const req = new proto.SetPresetRequest().setName(name).setData(serializeDisplayState(state));

	const serverState = await client.setPreset(req, null);

	return deserializeDisplayState(serverState);
}

export async function loadPreset(name: string): Promise<DisplayState> {
	const req = new proto.LoadPresetRequest().setName(name);

	const state = await client.loadPreset(req, null);

	return deserializeDisplayState(state);
}

export async function savePreset(name: string): Promise<DisplayState> {
	const req = new proto.SavePresetRequest().setName(name);

	const state = await client.savePreset(req, null);

	return deserializeDisplayState(state);
}

export async function getState(): Promise<DisplayState> {
	const state = await client.getState(EMPTY, null);

	return deserializeDisplayState(state);
}

export async function setState(state: DisplayState): Promise<DisplayState> {
	const req = new proto.SetStateRequest().setState(serializeDisplayState(state));

	const serverState = await client.setState(req, null);

	return deserializeDisplayState(serverState);
}

export async function streamState(): Promise<DisplayState> {
	const _ = client.streamState(EMPTY, null);

	throw "unimplemented";
}
