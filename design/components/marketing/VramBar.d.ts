import type { ReactNode, CSSProperties } from 'react';
/** VRAM usage bars (who owns the GPU). */
export interface VramBarProps {
  rows: { label: string; segments: { label: string; tone: 'running' | 'loading' | 'paused' | 'off' | 'error' | 'game'; pct: number }[] }[];
  style?: CSSProperties;
}
export declare function VramBar(props: VramBarProps): JSX.Element;
