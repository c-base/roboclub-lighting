import { MutableRef, useEffect, useRef } from "preact/hooks";
import iro from "@jaames/iro";
import styles from "./color-picker.module.css";

type HsvColor = {
	hue: number;
	saturation: number;
	value: number;
};

type ColorInputProps = {
	value: HsvColor;
	onChange: (color: HsvColor) => void;
};

export function ColorPicker({ value, onChange }: ColorInputProps) {
	let el: MutableRef<HTMLDivElement | null> = useRef(null);
	let colorPicker: MutableRef<iro.ColorPicker | null> = useRef(null);

	useEffect(() => {
		if (el.current == null) return;
		if (colorPicker.current) return;

		/// @ts-ignore
		colorPicker.current = new iro.ColorPicker(el.current, {
			padding: 8,
			color: {
				h: value.hue,
				s: value.saturation * 100.0,
				v: value.value * 100.0,
			},
		});
	}, [el.current]);

	useEffect(() => {
		if (!colorPicker.current) return;
		const pickerRef = colorPicker.current;

		function callback(color: iro.Color) {
			let { h, s, v } = color.hsv;
			onChange({
				hue: h!,
				saturation: s! / 100.0,
				value: v! / 100.0,
			});
		}

		pickerRef.on("color:change", callback);
		return () => pickerRef.off("color:change", callback);
	}, [colorPicker.current, onChange]);

	useEffect(() => {
		if (el.current == null || colorPicker.current == null || !value) return;

		colorPicker.current.color.hsv = {
			h: value.hue,
			s: value.saturation * 100.0,
			v: value.value * 100.0,
		};
	}, [colorPicker.current, value]);

	return <div ref={el} class={styles.picker} />;
}

type MultiColorInputProps = {
	values: HsvColor[];
	onChange: (colors: HsvColor[]) => void;
};

export function MultiColorPicker({ values, onChange }: MultiColorInputProps) {
	let el: MutableRef<HTMLDivElement | null> = useRef(null);
	let colorPicker: MutableRef<iro.ColorPicker | null> = useRef(null);

	useEffect(() => {
		if (el.current == null) return;
		if (colorPicker.current) return;

		/// @ts-ignore
		colorPicker.current = new iro.ColorPicker(el.current, {
			padding: 8,
			colors: values.map((value) => ({
				h: value.hue,
				s: value.saturation * 100.0,
				v: value.value * 100.0,
			})),
		});
	}, [el.current]);

	useEffect(() => {
		if (!colorPicker.current) return;
		const pickerRef = colorPicker.current;

		function callback() {
			let colors = pickerRef.colors.map(({ hsv: { h, s, v } }) => ({
				hue: h!,
				saturation: s! / 100.0,
				value: v! / 100.0,
			}));
			onChange(colors);
		}

		pickerRef.on("color:change", callback);
		return () => pickerRef.off("color:change", callback);
	}, [colorPicker.current, onChange]);

	useEffect(() => {
		if (el.current == null || colorPicker.current == null || !values || !values.length) return;

		colorPicker.current.colors.map(
			(color, i) =>
				(color.hsv = {
					h: values[i]?.hue,
					s: (values[i]?.saturation ?? 0) * 100.0,
					v: (values[i]?.value ?? 0) * 100.0,
				}),
		);
	}, [colorPicker.current, values]);

	return <div ref={el} class={styles.picker} />;
}
