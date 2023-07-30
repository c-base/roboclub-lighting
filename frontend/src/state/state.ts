import { ActionFunc, asyncAction, MESSAGES as STD_MESSAGES } from "../util/state-machine";
import {
	EffectData,
	getEffects,
	GetEffectsResponse,
	setActiveEffect,
	SetActiveEffectResponse,
	setEffectConfig,
} from "./api";
import { assign, createMachine } from "@xstate/fsm";
import { EventObject } from "@xstate/fsm/lib/types";

export enum STATES {
	LOADING = "loading",
	LOADED = "loaded",
	SET_ACTIVE_EFFECT = "setActiveEffect",
	SET_EFFECT_CONFIG = "setEffectConfig",
	ERROR = "error",
}

export enum CUSTOM_MESSAGES {
	SET_ACTIVE_EFFECT = "SET_ACTIVE_EFFECT",
	SET_EFFECT_CONFIG = "SET_EFFECT_CONFIG",
	RETRY = "RETRY",
}

export const MESSAGES = { ...STD_MESSAGES, ...CUSTOM_MESSAGES };
type MESSAGE_TYPES = typeof MESSAGES;

type Context = {
	effects: Record<string, EffectData>;
	activeEffect: string;
};

type LoadEffectsSuccessEvent = {
	type: MESSAGE_TYPES["SUCCESS"];
	effects: Record<string, EffectData>;
	activeEffect: string;
};

type SetActiveEffectSuccessEvent = {
	type: string;
	activeEffect: EffectData;
};

type SetActiveEffectEvent = {
	type: string;
	activeEffect: string;
};

type SetEffectConfigSuccessEvent = {
	type: string;
	effect: EffectData;
};

export const machine = createMachine<Context>({
	id: "dropdown",
	initial: STATES.LOADING,
	context: {
		effects: {},
		activeEffect: "",
	},
	states: {
		[STATES.LOADING]: {
			entry: "loadEffects",
			on: {
				[MESSAGES.SUCCESS]: {
					target: STATES.LOADED,
					actions: assign<Context, LoadEffectsSuccessEvent>({
						effects: (_, event) => event.effects,
						activeEffect: (_, event) => event.activeEffect,
					}),
				},
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.LOADED]: {
			on: {
				[MESSAGES.SET_ACTIVE_EFFECT]: {
					target: STATES.SET_ACTIVE_EFFECT,
					actions: assign<Context, SetActiveEffectEvent>({
						activeEffect: (_, event) => event.activeEffect,
					}),
				},
				[MESSAGES.SET_EFFECT_CONFIG]: {
					target: STATES.SET_EFFECT_CONFIG,
					actions: assign<Context, SetEffectConfigEvent>({
						effects: (ctx, event) => {
							let effects = { ...ctx.effects };
							effects[event.name].config = event.config;
							return effects;
						},
					}),
				},
			},
		},
		[STATES.SET_ACTIVE_EFFECT]: {
			entry: ["setActiveEffect"],
			on: {
				[MESSAGES.SUCCESS]: STATES.LOADED,
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.SET_EFFECT_CONFIG]: {
			entry: ["setEffectConfig"],
			on: {
				[MESSAGES.SUCCESS]: {
					target: STATES.LOADED,
					actions: assign<Context, SetEffectConfigSuccessEvent>({
						effects: (ctx, { effect }) => {
							let effects = { ...ctx.effects };
							effects[effect.name] = effect;
							return effects;
						},
					}),
				},
				[MESSAGES.FAILURE]: STATES.ERROR,
			},
		},
		[STATES.ERROR]: {
			entry: (_, b) => console.error(b),
			on: {
				[MESSAGES.RETRY]: STATES.LOADING,
			},
		},
	},
});

type SetEffectConfigEvent = {
	type: string;
	name: string;
	config: { [key: string]: any };
};

export const actions: { [key: string]: ActionFunc<Context, any> } = {
	loadEffects: asyncAction<Context, EventObject, GetEffectsResponse, LoadEffectsSuccessEvent>({
		promise: () => getEffects(),
		cb: ({ effects, active_effect: activeEffect }) => ({
			effects,
			activeEffect,
		}),
	}),
	setActiveEffect: asyncAction<
		Context,
		SetActiveEffectEvent,
		SetActiveEffectResponse,
		SetActiveEffectSuccessEvent
	>({
		promise: (_, event) => setActiveEffect(event.activeEffect),
		cb: ({ active_effect: activeEffect }) => ({ activeEffect }),
	}),
	setEffectConfig: asyncAction<
		Context,
		SetEffectConfigEvent,
		EffectData,
		SetEffectConfigSuccessEvent
	>({
		promise: (_, event) => setEffectConfig(event.name, event.config),
		cb: (effect) => ({ effect }),
	}),
};
