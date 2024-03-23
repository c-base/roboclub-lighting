import { EventObject, Typestate } from "@xstate/fsm/lib/types";
import { StateMachine } from "@xstate/fsm";
import { useCallback, useEffect, useMemo, useState } from "preact/hooks";

export type SendFunc<TEvent extends EventObject> = (event: TEvent) => void;

export enum MESSAGES {
	INIT = "INIT",
	SUCCESS = "SUCCESS",
	FAILURE = "FAILURE",
}

type ActionEvent<TEvent extends EventObject> =
	| ({
			type: MESSAGES.SUCCESS;
	  } & Omit<TEvent, "type">)
	| {
			type: MESSAGES.FAILURE;
			error: any;
	  };

export type ActionFunc<
	TContext extends object,
	TEvent extends EventObject,
	TEventOut extends EventObject,
> = (context: TContext, event: TEvent, send: SendFunc<TEventOut>) => void;

export function asyncAction<
	TContext extends object,
	TEvent extends EventObject,
	TEventOut extends EventObject,
	TResult extends any = Omit<TEventOut, "type">,
>({
	promise,
	cb = (d) => d as any,
}: {
	promise: (context: TContext, event: TEvent) => Promise<TResult>;
	cb?: (result: TResult) => Omit<TEventOut, "type">;
}): ActionFunc<TContext, TEvent, ActionEvent<TEventOut>> {
	return (context, event, send) => {
		promise(context, event)
			.then((data) => {
				send({ type: MESSAGES.SUCCESS, ...cb(data) });
			})
			.catch((error) => {
				send({ type: MESSAGES.FAILURE, error });
			});
	};
}

export function useMachine<
	TContext extends object,
	TEvent extends EventObject,
	TState extends Typestate<TContext>,
>(
	machineInit: StateMachine.Machine<TContext, TEvent, TState>,
	actions: {
		[key: string]: (context: TContext, event: TEvent, send: SendFunc<TEvent>) => void;
	},
): [StateMachine.State<TContext, TEvent, TState>, SendFunc<TEvent>] {
	const machine = useMemo(() => machineInit, []);

	const [state, setState] = useState(() => machine.initialState);

	function executeActions(
		state: StateMachine.State<TContext, TEvent, TState>,
		event: TEvent,
		send: SendFunc<TEvent>,
	) {
		state.actions.forEach(({ type, exec }) => {
			if (typeof exec === "function") {
				exec(state.context, event);
				return;
			}
			let action = actions[type];
			if (action == null) {
				throw new Error(`action '${type}' is not defined`);
			}
			action(state.context, event, send);
		});
	}

	let send = useCallback(
		(event: TEvent) => {
			console.log("[fsm] event", event);
			setState((state) => {
				let nextState = machine.transition(state, event);
				if (nextState.changed == null || nextState.changed) {
					console.group("[fsm] state changed");
					console.log("state", nextState.value);
					console.log("context", nextState.context);
					console.log("actions", nextState.actions);
					console.groupEnd();
				}
				executeActions(nextState, event, send);
				return nextState;
			});
		},
		[setState],
	);

	useEffect(() => {
		executeActions(state, { type: MESSAGES.INIT } as any, send);
	}, []);

	return [state, send];
}
