import type { CSSProperties } from 'react';
export type Tone = 'running' | 'loading' | 'paused' | 'off' | 'error' | 'game';
/** Status tone dot — header (lg 14px), inline (md 10px) or tiny (sm 8px). */
export interface StatusDotProps {
  tone?: Tone;
  size?: 'sm' | 'md' | 'lg';
  style?: CSSProperties;
}
export declare function StatusDot(props: StatusDotProps): JSX.Element;
