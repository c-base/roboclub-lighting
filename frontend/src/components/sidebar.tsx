import clsx from "clsx";
import { ALL_STATES, STATES } from "../state/state";
import styles from "./sidebar.module.css";
import { Zap } from "preact-feather";
import { prettyName } from "../util/pretty-names";
import {
	DisplayState,
	DisplayStateEffect,
	Effect,
	EffectConfig,
	Effects,
	Presets,
} from "../state/api.ts";

export function Sidebar({
	state,
	displayState,
	effects,
	presets,
	loadPreset,
	setEffectConfig,
}: {
	state: ALL_STATES;
	displayState: DisplayState;
	effects: Effects;
	presets: Presets;
	loadPreset: (name: string) => void;
	setEffectConfig: (idx: number, effect: string, config: EffectConfig) => void;
}) {
	return (
		<nav class={styles.sidebar}>
			<h1>RoboClub Lighting</h1>
			{state === STATES.LOADING ? (
				<p>loading...</p>
			) : (
				<ul>
					<li>Effects</li>
					{Object.entries(effects)
						.sort(([, a], [, b]) => (a.name > b.name ? 1 : a.name < b.name ? -1 : 0))
						.map(([id, e]) => (
							<li class={clsx({ [styles.active]: displayState.effects[0]?.effect === id })}>
								<button onClick={() => setEffectConfig(0, id, e.defaultConfig)}>
									<Zap size={20} /> &nbsp; {" " + prettyName(e.name)}
								</button>
							</li>
						))}
					<li>Presets</li>
					{Object.entries(presets)
						.sort(([a], [b]) => (a > b ? 1 : a < b ? -1 : 0))
						.map(([name]) => (
							<li>
								<button onClick={() => loadPreset(name)}>
									<Zap size={20} /> &nbsp; {" " + prettyName(name)}
								</button>
							</li>
						))}
				</ul>
			)}
		</nav>
	);
}
