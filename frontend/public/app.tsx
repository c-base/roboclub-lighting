import { LocationProvider } from "preact-iso";
import { useCallback, useMemo } from "preact/hooks";

import { useMachine } from "./util/state-machine";

import { actions, machine, MESSAGES, STATES } from "./state/state";
import { EffectData } from "./state/api";

import { EffectSettings } from "./components/effect-settings";
import { Sidebar } from "./components/sidebar";

import styles from "./app.module.css";

export function App() {
	const [
		{
			value: state,
			context: { activeEffect, effects },
		},
		send,
	] = useMachine(machine, actions);

	let setActiveEffect = useCallback(
		(activeEffect: string) => {
			send({
				type: MESSAGES.SET_ACTIVE_EFFECT,
				activeEffect,
			});
		},
		[send]
	);

	let setEffectConfig = useCallback(
		(config: Record<string, any>) => {
			send({
				type: MESSAGES.SET_EFFECT_CONFIG,
				name: activeEffect,
				config,
			});
		},
		[activeEffect, effects, send]
	);

	let activeEffectData: EffectData | null = useMemo(() => {
		if (!activeEffect) return null;
		let data = effects[activeEffect];
		return data == null ? null : data;
	}, [activeEffect, effects]);

	return (
		<LocationProvider>
			<Sidebar
				state={state}
				activeEffect={activeEffect}
				effects={effects}
				setActiveEffect={setActiveEffect}
			/>
			<main class={styles.main}>
				{state === STATES.ERROR && (
					<div class="error">
						<p>Something went wrong</p>
						<button onClick={() => send({ type: MESSAGES.RETRY })}>Retry</button>
					</div>
				)}
				{activeEffectData != null && (
					<EffectSettings
						state={state}
						effectData={activeEffectData}
						setEffectConfig={setEffectConfig}
					/>
				)}
			</main>
		</LocationProvider>
	);
}
