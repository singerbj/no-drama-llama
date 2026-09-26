import type { ReactNode, CSSProperties } from 'react';
/** Tone state sequence. */
export interface StateTimelineProps {
  steps: { tone: 'running' | 'loading' | 'paused' | 'off' | 'error' | 'game'; title: string; note?: string }[];
  style?: CSSProperties;
}
export declare function StateTimeline(props: StateTimelineProps): JSX.Element;
