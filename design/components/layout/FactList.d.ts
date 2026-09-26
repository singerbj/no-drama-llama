import type { ReactNode, CSSProperties } from 'react';
/** Key/value facts panel (GPU, llama.cpp build, context in use...). */
export interface FactListProps { items: [ReactNode, ReactNode][]; style?: CSSProperties; }
export declare function FactList(props: FactListProps): JSX.Element;
