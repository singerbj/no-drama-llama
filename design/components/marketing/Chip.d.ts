import type { ReactNode, CSSProperties } from 'react';
/** Outline name pill. */
export interface ChipProps {
  tone?: 'running' | 'loading' | 'paused' | 'off' | 'error' | 'game';
  children?: ReactNode;
  style?: CSSProperties;
}
export declare function Chip(props: ChipProps): JSX.Element;
