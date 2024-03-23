import { ActionFunc, asyncAction, MESSAGES as STD_MESSAGES } from "../util/state-machine";
import {
	Config,
	DisplayState,
	DisplayStateEffect,
	Effect,
	getConfig,
	getState,
	listEffects,
	listPresets,
	listSegments,
	loadPreset,
	savePreset,
	setConfig,
	setPreset,
	setSegments,
	setState,
	Strip,
} from "./api";
import { assign, createMachine } from "@xstate/fsm";
import { EventObject } from "@xstate/fsm/lib/types";

export enum STATES {
	LOADING = "loading",
	LOADED = "loaded",
	SET_CONFIG = "setConfig",
	SET_EFFECT_CONFIG = "setEffectConfig",
	SET_SEGMENTS = "setSegments",
	SET_PRESET = "setPreset",
	LOAD_PRESET = "loadPreset",
	SAVE_PRESET = "savePreset",
	SET_STATE = "setState",
	ERROR = "error",
}

export type ALL_STATES = (typeof STATES)[keyof typeof STATES];

export enum CUSTOM_MESSAGES {
	SET_CONFIG = "SET_CONFIG",
	SET_EFFECT_CONFIG = "SET_EFFECT_CONFIG",
	SET_SEGMENTS = "SET_SEGMENTS",
	SET_PRESET = "SET_PRESET",
	LOAD_PRESET = "LOAD_PRESET",
	SAVE_PRESET = "SAVE_PRESET",
	SET_STATE = "SET_STATE",
	RETRY = "RETRY",
}

export const MESSAGES = { ...STD_MESSAGES, ...CUSTOM_MESSAGES };

type Context = {
	config: Config;
	effects: Record<string, Effect>;
	segments: Strip[];
	presets: Record<string, DisplayState>;
	state: DisplayState;
};

type FailureEvent = {
	type: typeof MESSAGES.FAILURE;
	error: any;
};

type SuccessEvent<DATA extends {} = {}> = { type: typeof MESSAGES.SUCCESS } & Partial<Context> &
	DATA;

type LoadAllSuccessEvent = SuccessEvent<LoadAllResponse>;

type LoadAllResponse = {
	config: Config;
	effects: Record<string, Effect>;
	segments: Strip[];
	presets: Record<string, DisplayState>;
	state: DisplayState;
};

type SetConfigEvent = {
	type: typeof MESSAGES.SET_CONFIG;
	config: Config;
};

type SetEffectConfigEvent = {
	type: typeof MESSAGES.SET_EFFECT_CONFIG;
	idx: number;
} & Pick<DisplayStateEffect, "effectId" | "config">;

type ConfigSuccessEvent = SuccessEvent<{ config: Config }>;

type SetSegmentsEvent = {
	type: typeof MESSAGES.SET_SEGMENTS;
	segments: Strip[];
};

type SetSegmentsSuccessEvent = SuccessEvent<{ segments: Strip[] }>;

type SetPresetEvent = {
	type: typeof MESSAGES.SET_PRESET;
	name: string;
	state: DisplayState;
};

type DisplayStateSuccessEvent = SuccessEvent<{ state: DisplayState }>;

type LoadPresetEvent = {
	type: typeof MESSAGES.LOAD_PRESET;
	name: string;
};

type SavePresetEvent = {
	type: typeof MESSAGES.SAVE_PRESET;
	name: string;
};

type SetStateEvent = {
	type: typeof MESSAGES.SET_STATE;
	state: DisplayState;
};

type RetryEvent = {
	type: typeof MESSAGES.RETRY;
};

type Events =
	| SetConfigEvent
	| SetEffectConfigEvent
	| SetSegmentsEvent
	| SetPresetEvent
	| LoadPresetEvent
	| SavePresetEvent
	| SetStateEvent
	| SuccessEvent
	| FailureEvent
	| RetryEvent;

export const machine = createMachine<
	Context,
	Events,
	{
		value: ALL_STATES;
		context: Context;
	}
>({
	id: "control",
	initial: STATES.LOADING,
	context: {
		config: {
			brightness: 1.0,
			srgb: false,
		},
		effects: {},
		segments: [],
		presets: {},
		state: {
			effects: [],
		},
	},
	states: {
		[STATES.LOADING]: {
			entry: ["loadAll"],
			on: {
				[MESSAGES.SUCCESS]: {
					target: STATES.LOADED,
					actions: assign((_, { type, ...rest }) => rest),
				},
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.LOADED]: {
			on: {
				[MESSAGES.SET_CONFIG]: {
					target: STATES.SET_CONFIG,
					actions: assign({
						config: (_, event) => event.config,
					}),
				},
				[MESSAGES.SET_EFFECT_CONFIG]: {
					target: STATES.SET_EFFECT_CONFIG,
					actions: assign({
						state: (ctx, { idx, effectId, config }) => {
							let state = {
								effects: [...ctx.state.effects],
							} satisfies DisplayState;

							state.effects[idx] = { ...state.effects[idx], effectId, config };

							return state;
						},
					}),
				},
				[MESSAGES.SET_SEGMENTS]: {
					target: STATES.SET_SEGMENTS,
					actions: assign({
						segments: (_, event) => event.segments,
					}),
				},
				[MESSAGES.SET_PRESET]: {
					target: STATES.SET_PRESET,
					actions: assign({
						presets: (ctx, event) => ({
							...ctx.presets,
							[event.name]: event.state,
						}),
					}),
				},
				[MESSAGES.LOAD_PRESET]: {
					target: STATES.LOAD_PRESET,
					actions: assign({
						state: (ctx, event) => ctx.presets[event.name] ?? ctx.state,
					}),
				},
				[MESSAGES.SAVE_PRESET]: {
					target: STATES.SAVE_PRESET,
					actions: assign({
						presets: (ctx, event) => ({
							...ctx.presets,
							[event.name]: ctx.state,
						}),
					}),
				},
				[MESSAGES.SET_STATE]: {
					target: STATES.SET_STATE,
					actions: assign({
						state: (_, event) => event.state,
					}),
				},
			},
		},
		[STATES.SET_CONFIG]: {
			entry: ["setConfig"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.SET_EFFECT_CONFIG]: {
			entry: ["setEffectConfig"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.SET_SEGMENTS]: {
			entry: ["setSegments"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.SET_PRESET]: {
			entry: ["setPreset"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.SAVE_PRESET]: {
			entry: ["savePreset"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.LOAD_PRESET]: {
			entry: ["loadPreset"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.SET_STATE]: {
			entry: ["setState"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.ERROR]: {
			entry: (_, b) => (b.type == "FAILURE" ? console.error(b.error) : console.error(b)),
			on: {
				[MESSAGES.RETRY]: STATES.LOADING,
			},
		},
	},
});

export const actions = {
	loadAll: asyncAction<Context, EventObject, LoadAllSuccessEvent>({
		promise: async () => {
			let [config, effects, segments, presets, state] = await Promise.all([
				getConfig(),
				listEffects(),
				listSegments(),
				listPresets(),
				getState(),
			]);

			return { config, effects, segments, presets, state };
		},
		cb: (d) => d,
	}),
	setConfig: asyncAction<Context, SetConfigEvent, ConfigSuccessEvent, Config>({
		promise: (_, { config }) => setConfig(config),
		cb: (config) => ({ config }),
	}),
	setEffectConfig: asyncAction<
		Context,
		SetEffectConfigEvent,
		DisplayStateSuccessEvent,
		DisplayState
	>({
		promise: (ctx, { idx, effectId, config }) => {
			let state = {
				effects: [...ctx.state.effects],
			} satisfies DisplayState;

			state.effects[idx] = { ...state.effects[idx], effectId, config };

			return setState(state);
		},
		cb: (state) => ({ state }),
	}),
	setSegments: asyncAction<Context, SetSegmentsEvent, SetSegmentsSuccessEvent, Strip[]>({
		promise: (_, { segments }) => setSegments(segments),
		cb: (segments) => ({ segments }),
	}),
	setPreset: asyncAction<Context, SetPresetEvent, DisplayStateSuccessEvent, DisplayState>({
		promise: (_, { name, state }) => setPreset(name, state),
		cb: (state) => ({ state }),
	}),
	loadPreset: asyncAction<Context, LoadPresetEvent, DisplayStateSuccessEvent, DisplayState>({
		promise: (_, { name }) => loadPreset(name),
		cb: (state) => ({ state }),
	}),
	savePreset: asyncAction<Context, SavePresetEvent, DisplayStateSuccessEvent, DisplayState>({
		promise: (_, { name }) => savePreset(name),
		cb: (state) => ({ state }),
	}),
	setState: asyncAction<Context, SetStateEvent, DisplayStateSuccessEvent, DisplayState>({
		promise: (_, { state }) => setState(state),
		cb: (state) => ({ state }),
	}),
} satisfies { [key: string]: ActionFunc<Context, any, any> };
