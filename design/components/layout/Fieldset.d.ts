import type { ReactNode, CSSProperties } from 'react';
/** Bordered settings group with an optional legend. */
export interface FieldsetProps { legend?: string; disabled?: boolean; style?: CSSProperties; children?: ReactNode; }
export declare function Fieldset(props: FieldsetProps): JSX.Element;
