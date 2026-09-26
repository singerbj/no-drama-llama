import type { CSSProperties } from 'react';
/** Installed-models radio group. */
export interface ModelPickerProps { models: { name: string; size: number }[]; value?: string; onChange?: (name: string) => void; style?: CSSProperties; }
export declare function ModelPicker(props: ModelPickerProps): JSX.Element;
