import { EffectData } from "../state/api";
import clsx from "clsx";
import { STATES } from "../state/state";
import styles from "./sidebar.module.css";
import { Zap } from "preact-feather";
import { prettyName } from "../util/pretty-names";

export function Sidebar({
	state,
	activeEffect,
	effects,
	setActiveEffect,
}: {
	state: string;
	activeEffect: string;
	effects: Record<string, EffectData>;
	setActiveEffect: (activeEffect: string) => void;
}) {
	return (
		<nav class={styles.sidebar}>
			<h1>RoboClub Lighting</h1>
			{state === STATES.LOADING ? (
				<p>loading...</p>
			) : (
				<ul>
					{Object.values(effects)
						.sort((a, b) => (a.name > b.name ? 1 : a.name < b.name ? -1 : 0))
						.map((e) => (
							<li class={clsx({ [styles.active]: activeEffect === e.name })}>
								<button onClick={() => setActiveEffect(e.name)}>
									<Zap size={20} /> &nbsp; {" " + prettyName(e.name)}
								</button>
							</li>
						))}
				</ul>
			)}
		</nav>
	);
}
