import { JSONSchema7 } from "json-schema";
import { useMemo } from "preact/hooks";
import clsx from "clsx";
import { Sliders } from "preact-feather";
import { dset } from "dset";

import { EffectData } from "../state/api";
import { STATES } from "../state/state";

import styles from "./effect-settings.module.css";
import { prettyName } from "../util/pretty-names";
import { ColorPicker, MultiColorPicker } from "./color-picker";

export function EffectSettings({
	effectData,
	setEffectConfig,
	state,
}: {
	effectData: EffectData;
	setEffectConfig: (config: Record<string, any>) => void;
	state: string;
}) {
	if (state === STATES.LOADING) {
		return <p>loading...</p>;
	}

	if (state === STATES.ERROR) {
		return <p>something went wrong</p>;
	}

	function patchAndUpdateConfig(field, value) {
		let config: { [name: string]: any } = JSON.parse(JSON.stringify(effectData.config));
		dset(config, field, value);
		setEffectConfig(config);
	}

	return (
		<>
			<h2 class={styles.title}>
				<Sliders /> &nbsp; {prettyName(effectData.name)}
			</h2>
			<Settings
				config={effectData.config}
				schema={effectData.schema}
				update={patchAndUpdateConfig}
			/>
		</>
	);
}

type Field = {
	name: string;
	field: string;
	value: any;
	schema: JSONSchema7 | null;
	custom: "color" | "color_gradient" | null;
};

function Settings({
	config,
	schema,
	update,
}: {
	config: Record<string, any>;
	schema: JSONSchema7;
	update: (field: string, value: any) => void;
}) {
	let fields: Field[] = useMemo(() => {
		if (schema.type !== "object") {
			return [];
		}

		return createFields(config, schema.properties as any, schema.definitions as any);
	}, [config, schema]);

	if (fields.length === 0) {
		return <p>No config options for this effect.</p>;
	}

	return (
		<form onSubmit={(e) => e.preventDefault} class={styles.form}>
			{fields.map((f) => (
				<Setting field={f} onChange={(value) => update(f.field, value)} />
			))}
		</form>
	);
}

type DefinitionMap = {
	[k: string]: JSONSchema7;
};

function createFields(
	config: Record<string, any>,
	properties: DefinitionMap,
	definitions: DefinitionMap,
	prefix = []
): Field[] {
	return Object.keys(config).flatMap((name) => {
		let propertySchema = properties[name];
		if (typeof propertySchema === "boolean") {
			propertySchema = null;
		}

		if (typeof config[name] === "object") {
			if (typeof propertySchema["$ref"] === "string") {
				let ref = propertySchema["$ref"];
				let definition = ref.substring("#/definitions/".length);

				if (definition === "Color") {
					return {
						name,
						field: [...prefix, name].join("."),
						value: config[name],
						schema: null,
						custom: "color",
					};
				}

				if (definition === "ColorGradient") {
					return {
						name,
						field: [...prefix, name].join("."),
						value: config[name],
						schema: null,
						custom: "color_gradient",
					};
				}

				return createFields(
					config[name],
					definitions[ref.substring("#/definitions/".length)].properties as any,
					definitions,
					prefix.concat(name)
				);
			}
		}

		return {
			name: name,
			field: [...prefix, name].join("."),
			value: config[name],
			schema: propertySchema as JSONSchema7 | null,
			custom: null,
		};
	});
}

function getInputType(schema: JSONSchema7): HTMLInputElement["type"] {
	switch (schema.type) {
		case "number":
		case "integer":
			return "number";

		case "boolean":
			return "checkbox";

		default:
			return "text";
	}
}

function getValue(schema: JSONSchema7, el: HTMLInputElement) {
	switch (schema.type) {
		case "number":
		case "integer":
			return el.valueAsNumber;

		case "boolean":
			return el.checked;

		default:
			return el.value;
	}
}

function readableValue(schema: JSONSchema7, value: any) {
	switch (schema.type) {
		case "number":
		case "integer":
			return Math.round(value * 1000) / 1000;

		default:
			return value;
	}
}

function Setting({ field, onChange }: { field: Field; onChange: (value: any) => void }) {
	let id = `input__${field.name}`;

	let label = prettyName(field.name);
	if (field.custom !== null) {
		return <CustomSetting field={field} onChange={onChange} />;
	}

	if (field.schema === null) {
		label = label + " (invalid schema)";
	}

	let inputType = getInputType(field.schema);
	let value = readableValue(field.schema, field.value);

	return (
		<fieldset class={clsx({ error: field.schema === null })}>
			<label htmlFor={id}>{label}</label>
			<input
				type={inputType}
				disabled={field.schema === null}
				value={value}
				checked={inputType === "checkbox" && value}
				onChange={(e) => onChange(getValue(field.schema, e.currentTarget))}
			/>
		</fieldset>
	);
}

function CustomSetting({ field, onChange }: { field: Field; onChange: (value: any) => void }) {
	if (field.custom === "color") {
		return <ColorPicker value={field.value} onChange={onChange} />;
	}

	if (field.custom === "color_gradient") {
		return (
			<MultiColorPicker
				values={[field.value.from, field.value.to]}
				onChange={([from, to]) => onChange({ from, to })}
			/>
		);
	}

	throw new Error("invalid custom input: " + field.custom);
}
